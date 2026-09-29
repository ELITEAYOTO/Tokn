use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};
use tokn_domain::DiagnosticObservations;

use crate::{ToolAggregate, build_attribution};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RunMetrics {
    pub inference_started: u64,
    pub inference_completed: u64,
    pub inference_cancelled: u64,
    pub inference_failed: u64,
    pub usage_records: u64,
    pub input_tokens: u64,
    pub cached_input_tokens: u64,
    pub cache_write_input_tokens: u64,
    pub uncached_input_tokens: Option<u64>,
    pub output_tokens: u64,
    pub reasoning_output_tokens: u64,
    pub logical_total_tokens: u64,
    pub observed_span_ms: Option<u64>,
    pub tools_total: u64,
    pub tools_with_token_count: u64,
    pub observed_tool_tokens: u64,
    pub tools_with_output_cap: u64,
    pub cap_adjusted_tool_token_upper_bound: u64,
    pub raw_exceeds_cap_count: u64,
    pub tool_categories: Vec<ToolAggregate>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CategoryComparison {
    pub label: String,
    pub baseline_count: u64,
    pub candidate_count: u64,
    pub baseline_token_coverage: u64,
    pub candidate_token_coverage: u64,
    pub baseline_observed_tokens: u64,
    pub candidate_observed_tokens: u64,
    pub delta_observed_tokens: i64,
    pub baseline_cap_coverage: u64,
    pub candidate_cap_coverage: u64,
    pub baseline_cap_upper: u64,
    pub candidate_cap_upper: u64,
    pub delta_cap_upper: i64,
    pub baseline_raw_exceeds_cap: u64,
    pub candidate_raw_exceeds_cap: u64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RunComparison {
    pub baseline: RunMetrics,
    pub candidate: RunMetrics,
    pub delta_input_tokens: i64,
    pub delta_cached_input_tokens: i64,
    pub delta_uncached_input_tokens: Option<i64>,
    pub delta_output_tokens: i64,
    pub delta_reasoning_output_tokens: i64,
    pub delta_logical_total_tokens: i64,
    pub delta_completed_inferences: i64,
    pub delta_observed_tool_tokens: i64,
    pub delta_cap_adjusted_tool_token_upper_bound: i64,
    pub delta_raw_exceeds_cap_count: i64,
    pub category_deltas: Vec<CategoryComparison>,
}

pub fn summarize_run(observations: &DiagnosticObservations) -> RunMetrics {
    let attribution = build_attribution(observations);
    let completed = observations
        .inferences
        .iter()
        .filter(|item| item.status == "completed" && item.usage.is_some())
        .collect::<Vec<_>>();

    let mut input = 0_u64;
    let mut cached = 0_u64;
    let mut cache_write = 0_u64;
    let mut output = 0_u64;
    let mut reasoning = 0_u64;
    let mut uncached = Some(0_u64);

    for item in &completed {
        let usage = item.usage.as_ref().expect("filtered usage");
        input += usage.input_tokens.unwrap_or(0);
        cached += usage.cached_input_tokens.unwrap_or(0);
        cache_write += usage.cache_write_input_tokens.unwrap_or(0);
        output += usage.output_tokens.unwrap_or(0);
        reasoning += usage.reasoning_output_tokens.unwrap_or(0);
        uncached = match (uncached, usage.ordinary_uncached()) {
            (Some(total), Some(value)) => total.checked_add(value),
            _ => None,
        };
    }

    let starts = observations
        .inferences
        .iter()
        .filter_map(|item| item.started_at_unix_ms)
        .collect::<Vec<_>>();
    let ends = observations
        .inferences
        .iter()
        .filter_map(|item| item.ended_at_unix_ms)
        .collect::<Vec<_>>();
    let observed_span_ms = match (starts.iter().min(), ends.iter().max()) {
        (Some(start), Some(end)) if end >= start => u64::try_from(end - start).ok(),
        _ => None,
    };

    RunMetrics {
        inference_started: attribution.inference_started,
        inference_completed: attribution.inference_completed,
        inference_cancelled: attribution.inference_cancelled,
        inference_failed: attribution.inference_failed,
        usage_records: completed.len() as u64,
        input_tokens: input,
        cached_input_tokens: cached,
        cache_write_input_tokens: cache_write,
        uncached_input_tokens: uncached,
        output_tokens: output,
        reasoning_output_tokens: reasoning,
        logical_total_tokens: input.saturating_add(output),
        observed_span_ms,
        tools_total: attribution.tools_total,
        tools_with_token_count: attribution.tools_with_original_token_count,
        observed_tool_tokens: attribution.total_tool_original_tokens,
        tools_with_output_cap: attribution.tools_with_output_cap,
        cap_adjusted_tool_token_upper_bound: attribution.cap_adjusted_tool_token_upper_bound,
        raw_exceeds_cap_count: attribution.raw_exceeds_cap_count,
        tool_categories: attribution.tool_categories,
    }
}

pub fn compare_runs(
    baseline_observations: &DiagnosticObservations,
    candidate_observations: &DiagnosticObservations,
) -> RunComparison {
    let baseline = summarize_run(baseline_observations);
    let candidate = summarize_run(candidate_observations);
    let category_deltas = compare_categories(&baseline.tool_categories, &candidate.tool_categories);

    RunComparison {
        delta_input_tokens: signed(candidate.input_tokens, baseline.input_tokens),
        delta_cached_input_tokens: signed(
            candidate.cached_input_tokens,
            baseline.cached_input_tokens,
        ),
        delta_uncached_input_tokens: match (
            candidate.uncached_input_tokens,
            baseline.uncached_input_tokens,
        ) {
            (Some(candidate), Some(baseline)) => Some(signed(candidate, baseline)),
            _ => None,
        },
        delta_output_tokens: signed(candidate.output_tokens, baseline.output_tokens),
        delta_reasoning_output_tokens: signed(
            candidate.reasoning_output_tokens,
            baseline.reasoning_output_tokens,
        ),
        delta_logical_total_tokens: signed(
            candidate.logical_total_tokens,
            baseline.logical_total_tokens,
        ),
        delta_completed_inferences: signed(
            candidate.inference_completed,
            baseline.inference_completed,
        ),
        delta_observed_tool_tokens: signed(
            candidate.observed_tool_tokens,
            baseline.observed_tool_tokens,
        ),
        delta_cap_adjusted_tool_token_upper_bound: signed(
            candidate.cap_adjusted_tool_token_upper_bound,
            baseline.cap_adjusted_tool_token_upper_bound,
        ),
        delta_raw_exceeds_cap_count: signed(
            candidate.raw_exceeds_cap_count,
            baseline.raw_exceeds_cap_count,
        ),
        baseline,
        candidate,
        category_deltas,
    }
}

fn compare_categories(
    baseline: &[ToolAggregate],
    candidate: &[ToolAggregate],
) -> Vec<CategoryComparison> {
    let baseline_map = baseline
        .iter()
        .map(|item| (item.label.clone(), item))
        .collect::<BTreeMap<_, _>>();
    let candidate_map = candidate
        .iter()
        .map(|item| (item.label.clone(), item))
        .collect::<BTreeMap<_, _>>();
    let labels = baseline_map
        .keys()
        .chain(candidate_map.keys())
        .cloned()
        .collect::<BTreeSet<_>>();

    labels
        .into_iter()
        .map(|label| {
            let before = baseline_map.get(&label);
            let after = candidate_map.get(&label);
            let before_tokens = before.map_or(0, |item| item.original_tokens);
            let after_tokens = after.map_or(0, |item| item.original_tokens);
            CategoryComparison {
                label,
                baseline_count: before.map_or(0, |item| item.count),
                candidate_count: after.map_or(0, |item| item.count),
                baseline_token_coverage: before.map_or(0, |item| item.token_count_known),
                candidate_token_coverage: after.map_or(0, |item| item.token_count_known),
                baseline_observed_tokens: before_tokens,
                candidate_observed_tokens: after_tokens,
                delta_observed_tokens: signed(after_tokens, before_tokens),
                baseline_cap_coverage: before.map_or(0, |item| item.cap_count_known),
                candidate_cap_coverage: after.map_or(0, |item| item.cap_count_known),
                baseline_cap_upper: before.map_or(0, |item| item.cap_adjusted_upper_tokens),
                candidate_cap_upper: after.map_or(0, |item| item.cap_adjusted_upper_tokens),
                delta_cap_upper: signed(
                    after.map_or(0, |item| item.cap_adjusted_upper_tokens),
                    before.map_or(0, |item| item.cap_adjusted_upper_tokens),
                ),
                baseline_raw_exceeds_cap: before.map_or(0, |item| item.raw_exceeds_cap_count),
                candidate_raw_exceeds_cap: after.map_or(0, |item| item.raw_exceeds_cap_count),
            }
        })
        .collect()
}

fn signed(current: u64, baseline: u64) -> i64 {
    let delta = i128::from(current) - i128::from(baseline);
    delta.clamp(i128::from(i64::MIN), i128::from(i64::MAX)) as i64
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokn_domain::{InferenceObservation, TokenUsage};

    #[test]
    fn identical_runs_have_zero_token_deltas() {
        let observations = DiagnosticObservations {
            inferences: vec![InferenceObservation {
                inference_call_id: "a".into(),
                status: "completed".into(),
                started_at_unix_ms: Some(100),
                ended_at_unix_ms: Some(200),
                usage: Some(TokenUsage {
                    input_tokens: Some(100),
                    cached_input_tokens: Some(80),
                    cache_write_input_tokens: Some(0),
                    output_tokens: Some(10),
                    reasoning_output_tokens: Some(2),
                    ..Default::default()
                }),
                ..Default::default()
            }],
            ..Default::default()
        };

        let comparison = compare_runs(&observations, &observations);
        assert_eq!(comparison.delta_input_tokens, 0);
        assert_eq!(comparison.delta_logical_total_tokens, 0);
        assert_eq!(comparison.delta_uncached_input_tokens, Some(0));
        assert_eq!(comparison.delta_cap_adjusted_tool_token_upper_bound, 0);
        assert_eq!(comparison.delta_raw_exceeds_cap_count, 0);
    }
}
