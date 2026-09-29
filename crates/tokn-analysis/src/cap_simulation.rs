use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use tokn_domain::{DiagnosticObservations, ToolObservation};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CapSimulationCategory {
    pub label: String,
    pub count: u64,
    pub raw_coverage: u64,
    pub raw_tokens: u64,
    pub baseline_cap_coverage: u64,
    pub baseline_upper: u64,
    pub candidate_cap_coverage: u64,
    pub candidate_upper: u64,
    pub delta_upper: i64,
    pub override_cap: Option<u64>,
    pub baseline_raw_exceeds_cap: u64,
    pub candidate_raw_exceeds_cap: u64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CapSimulationReport {
    pub tools_total: u64,
    pub raw_coverage: u64,
    pub raw_tokens: u64,
    pub baseline_cap_coverage: u64,
    pub baseline_upper: u64,
    pub candidate_cap_coverage: u64,
    pub candidate_upper: u64,
    pub delta_upper: i64,
    pub categories: Vec<CapSimulationCategory>,
}

pub fn simulate_caps(
    observations: &DiagnosticObservations,
    overrides: &BTreeMap<String, u64>,
) -> CapSimulationReport {
    let mut grouped = BTreeMap::<String, Vec<&ToolObservation>>::new();
    for tool in &observations.tools {
        let label = if tool.category.is_empty() {
            "unknown".to_string()
        } else {
            tool.category.clone()
        };
        grouped.entry(label).or_default().push(tool);
    }

    let mut report = CapSimulationReport {
        tools_total: observations.tools.len() as u64,
        ..Default::default()
    };

    for (label, tools) in grouped {
        let override_cap = overrides.get(&label).copied();
        let mut category = CapSimulationCategory {
            label,
            count: tools.len() as u64,
            override_cap,
            ..Default::default()
        };

        for tool in tools {
            if let Some(raw) = tool.original_token_count {
                report.raw_coverage += 1;
                report.raw_tokens += raw;
                category.raw_coverage += 1;
                category.raw_tokens += raw;

                if let Some(cap) = tool.max_output_tokens {
                    report.baseline_cap_coverage += 1;
                    report.baseline_upper += raw.min(cap);
                    category.baseline_cap_coverage += 1;
                    category.baseline_upper += raw.min(cap);
                    if raw > cap {
                        category.baseline_raw_exceeds_cap += 1;
                    }
                }

                let candidate_cap = override_cap.or(tool.max_output_tokens);
                if let Some(cap) = candidate_cap {
                    report.candidate_cap_coverage += 1;
                    report.candidate_upper += raw.min(cap);
                    category.candidate_cap_coverage += 1;
                    category.candidate_upper += raw.min(cap);
                    if raw > cap {
                        category.candidate_raw_exceeds_cap += 1;
                    }
                }
            }
        }

        category.delta_upper = signed(category.candidate_upper, category.baseline_upper);
        report.categories.push(category);
    }

    report.delta_upper = signed(report.candidate_upper, report.baseline_upper);
    report.categories.sort_by(|left, right| {
        left.delta_upper
            .cmp(&right.delta_upper)
            .then_with(|| left.label.cmp(&right.label))
    });
    report
}

fn signed(current: u64, baseline: u64) -> i64 {
    let delta = i128::from(current) - i128::from(baseline);
    delta.clamp(i128::from(i64::MIN), i128::from(i64::MAX)) as i64
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokn_domain::ToolObservation;

    #[test]
    fn lower_override_reduces_candidate_upper_bound() {
        let observations = DiagnosticObservations {
            tools: vec![ToolObservation {
                tool_call_id: "tool-1".into(),
                category: "file_read".into(),
                original_token_count: Some(10_000),
                max_output_tokens: Some(6_000),
                ..Default::default()
            }],
            ..Default::default()
        };
        let overrides = BTreeMap::from([("file_read".into(), 3_000)]);
        let report = simulate_caps(&observations, &overrides);

        assert_eq!(report.baseline_upper, 6_000);
        assert_eq!(report.candidate_upper, 3_000);
        assert_eq!(report.delta_upper, -3_000);
        assert_eq!(report.categories[0].candidate_raw_exceeds_cap, 1);
    }
}
