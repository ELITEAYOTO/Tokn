use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use tokn_domain::{DiagnosticObservations, ToolObservation};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum CapPolicyStatus {
    Pass,
    Fail,
    #[default]
    NoEvidence,
    IncompleteEvidence,
}

impl CapPolicyStatus {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Pass => "PASS",
            Self::Fail => "FAIL",
            Self::NoEvidence => "NO_EVIDENCE",
            Self::IncompleteEvidence => "INCOMPLETE_EVIDENCE",
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CapPolicyCategory {
    pub label: String,
    pub required_max: u64,
    pub tool_count: u64,
    pub cap_known: u64,
    pub compliant: u64,
    pub violations: u64,
    pub unknown: u64,
    pub min_observed_cap: Option<u64>,
    pub max_observed_cap: Option<u64>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CapPolicyReport {
    pub status: CapPolicyStatus,
    pub targeted_tools: u64,
    pub cap_known: u64,
    pub compliant: u64,
    pub violations: u64,
    pub unknown: u64,
    pub parse_failures: u64,
    pub all_known_caps_compliant: bool,
    pub complete_evidence: bool,
    pub categories: Vec<CapPolicyCategory>,
}

pub fn check_cap_policy(
    observations: &DiagnosticObservations,
    policy: &BTreeMap<String, u64>,
) -> CapPolicyReport {
    check_cap_policy_tools(&observations.tools, 0, policy)
}

pub fn check_cap_policy_tools(
    tools: &[ToolObservation],
    parse_failures: u64,
    policy: &BTreeMap<String, u64>,
) -> CapPolicyReport {
    let mut categories = Vec::new();

    for (label, required_max) in policy {
        let matching = tools
            .iter()
            .filter(|tool| tool.category == *label)
            .collect::<Vec<_>>();

        let mut category = CapPolicyCategory {
            label: label.clone(),
            required_max: *required_max,
            tool_count: matching.len() as u64,
            ..Default::default()
        };

        for tool in matching {
            match tool.max_output_tokens {
                Some(cap) => {
                    category.cap_known += 1;
                    category.min_observed_cap = Some(
                        category
                            .min_observed_cap
                            .map_or(cap, |value| value.min(cap)),
                    );
                    category.max_observed_cap = Some(
                        category
                            .max_observed_cap
                            .map_or(cap, |value| value.max(cap)),
                    );
                    if cap <= *required_max {
                        category.compliant += 1;
                    } else {
                        category.violations += 1;
                    }
                }
                None => category.unknown += 1,
            }
        }

        categories.push(category);
    }

    let targeted_tools = categories.iter().map(|item| item.tool_count).sum();
    let cap_known = categories.iter().map(|item| item.cap_known).sum();
    let compliant = categories.iter().map(|item| item.compliant).sum();
    let violations = categories.iter().map(|item| item.violations).sum();
    let unknown = categories.iter().map(|item| item.unknown).sum();

    let status = if violations > 0 {
        CapPolicyStatus::Fail
    } else if targeted_tools == 0 && parse_failures == 0 {
        CapPolicyStatus::NoEvidence
    } else if unknown > 0 || parse_failures > 0 {
        CapPolicyStatus::IncompleteEvidence
    } else {
        CapPolicyStatus::Pass
    };

    CapPolicyReport {
        status,
        targeted_tools,
        cap_known,
        compliant,
        violations,
        unknown,
        parse_failures,
        all_known_caps_compliant: targeted_tools > 0 && cap_known > 0 && violations == 0,
        complete_evidence: targeted_tools > 0 && unknown == 0 && parse_failures == 0,
        categories,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tool(category: &str, cap: Option<u64>) -> ToolObservation {
        ToolObservation {
            tool_call_id: format!("{category}-{cap:?}"),
            category: category.to_string(),
            max_output_tokens: cap,
            ..Default::default()
        }
    }

    #[test]
    fn explicit_violation_is_fail() {
        let tools = vec![
            tool("file_read", Some(5_000)),
            tool("file_read", Some(7_000)),
            tool("file_read", None),
        ];
        let policy = BTreeMap::from([("file_read".to_string(), 5_000)]);
        let report = check_cap_policy_tools(&tools, 0, &policy);

        assert_eq!(report.status, CapPolicyStatus::Fail);
        assert_eq!(report.targeted_tools, 3);
        assert_eq!(report.compliant, 1);
        assert_eq!(report.violations, 1);
        assert_eq!(report.unknown, 1);
        assert!(!report.complete_evidence);
    }

    #[test]
    fn zero_targets_is_no_evidence() {
        let policy = BTreeMap::from([("file_read".to_string(), 5_000)]);
        let report = check_cap_policy_tools(&[], 0, &policy);

        assert_eq!(report.status, CapPolicyStatus::NoEvidence);
        assert!(!report.all_known_caps_compliant);
        assert!(!report.complete_evidence);
    }

    #[test]
    fn unknown_cap_is_incomplete_evidence() {
        let policy = BTreeMap::from([("search".to_string(), 3_000)]);
        let report = check_cap_policy_tools(&[tool("search", None)], 0, &policy);

        assert_eq!(report.status, CapPolicyStatus::IncompleteEvidence);
        assert_eq!(report.unknown, 1);
    }

    #[test]
    fn parse_failure_prevents_pass() {
        let policy = BTreeMap::from([("search".to_string(), 3_000)]);
        let report = check_cap_policy_tools(&[], 1, &policy);

        assert_eq!(report.status, CapPolicyStatus::IncompleteEvidence);
        assert_eq!(report.parse_failures, 1);
    }

    #[test]
    fn fully_observed_compliance_is_pass() {
        let policy = BTreeMap::from([
            ("file_read".to_string(), 5_000),
            ("search".to_string(), 3_000),
        ]);
        let tools = vec![tool("file_read", Some(5_000)), tool("search", Some(3_000))];
        let report = check_cap_policy_tools(&tools, 0, &policy);

        assert_eq!(report.status, CapPolicyStatus::Pass);
        assert!(report.complete_evidence);
    }
}
