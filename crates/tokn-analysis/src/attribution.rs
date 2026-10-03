use std::collections::{BTreeMap, HashSet};

use serde::{Deserialize, Serialize};
use tokn_domain::{DiagnosticObservations, InferenceObservation, ToolObservation};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ToolKindCount {
    pub kind: String,
    pub count: u64,
    pub original_tokens: u64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ToolAggregate {
    pub label: String,
    pub count: u64,
    pub token_count_known: u64,
    pub original_tokens: u64,
    pub cap_count_known: u64,
    pub cap_adjusted_upper_tokens: u64,
    pub raw_exceeds_cap_count: u64,
    pub result_payload_bytes: u64,
    pub result_output_chars: u64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AttributionWindow {
    pub index: u64,
    pub inference_call_id: String,
    pub model: Option<String>,
    pub input_tokens: u64,
    pub cached_input_tokens: u64,
    pub uncached_input_tokens: Option<u64>,
    pub output_tokens: u64,
    pub reasoning_output_tokens: u64,

    pub input_growth: Option<i64>,
    pub request_payload_bytes: Option<u64>,
    pub request_input_items: Option<u64>,
    pub tools_since_previous: u64,
    pub tool_original_tokens: u64,
    pub tool_cap_adjusted_upper_tokens: u64,
    pub tool_result_payload_bytes: u64,
    pub tool_kinds: Vec<ToolKindCount>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AttributionReport {
    pub inference_started: u64,
    pub inference_completed: u64,
    pub inference_cancelled: u64,
    pub inference_failed: u64,
    pub usage_records: u64,
    pub tools_total: u64,
    pub tools_with_original_token_count: u64,
    pub total_tool_original_tokens: u64,
    pub tools_with_output_cap: u64,
    pub cap_adjusted_tool_token_upper_bound: u64,
    pub raw_exceeds_cap_count: u64,
    pub unattributed_tool_calls: u64,
    pub unattributed_tools_with_original_token_count: u64,
    pub unattributed_tool_original_tokens: u64,
    pub tool_surfaces: Vec<ToolAggregate>,
    pub tool_categories: Vec<ToolAggregate>,
    pub unattributed_categories: Vec<ToolAggregate>,
    pub code_cells_started: u64,
    pub compactions: u64,
    pub windows: Vec<AttributionWindow>,
}

pub fn build_attribution(observations: &DiagnosticObservations) -> AttributionReport {
    let mut report = AttributionReport {
        inference_started: observations.inferences.len() as u64,
        inference_completed: count_status(&observations.inferences, "completed"),
        inference_cancelled: count_status(&observations.inferences, "cancelled"),
        inference_failed: count_status(&observations.inferences, "failed"),
        tools_total: observations.tools.len() as u64,
        tools_with_original_token_count: observations
            .tools
            .iter()
            .filter(|tool| tool.original_token_count.is_some())
            .count() as u64,
        total_tool_original_tokens: observations
            .tools
            .iter()
            .map(|tool| tool.original_token_count.unwrap_or(0))
            .sum(),
        tools_with_output_cap: observations
            .tools
            .iter()
            .filter(|tool| cap_adjusted_upper(tool).is_some())
            .count() as u64,
        cap_adjusted_tool_token_upper_bound: observations
            .tools
            .iter()
            .filter_map(cap_adjusted_upper)
            .sum(),
        raw_exceeds_cap_count: observations
            .tools
            .iter()
            .filter(|tool| raw_exceeds_cap(tool))
            .count() as u64,
        tool_surfaces: aggregate_tools(observations.tools.iter(), |tool| &tool.surface),
        tool_categories: aggregate_tools(observations.tools.iter(), |tool| &tool.category),
        code_cells_started: observations.code_cells_started,
        compactions: observations.compactions,
        ..Default::default()
    };

    let mut completed = observations
        .inferences
        .iter()
        .filter(|item| item.status == "completed" && item.usage.is_some())
        .collect::<Vec<_>>();
    completed.sort_by_key(|item| item.started_seq.unwrap_or(u64::MAX));
    report.usage_records = completed.len() as u64;

    let mut assigned_tools = HashSet::<String>::new();
    let mut previous: Option<&InferenceObservation> = None;

    for (offset, current) in completed.iter().enumerate() {
        let usage = current.usage.as_ref().expect("filtered usage");
        let mut selected = Vec::<&ToolObservation>::new();

        if let Some(previous) = previous {
            let lower = previous.ended_seq.unwrap_or(0);
            let upper = current.started_seq.unwrap_or(u64::MAX);
            selected.extend(
                observations
                    .tools
                    .iter()
                    .filter(|tool| tool.ended_seq.is_some_and(|seq| seq > lower && seq < upper)),
            );
        }

        for tool in &selected {
            assigned_tools.insert(tool.tool_call_id.clone());
        }

        let tool_original_tokens = selected
            .iter()
            .map(|tool| tool.original_token_count.unwrap_or(0))
            .sum();
        let tool_cap_adjusted_upper_tokens = selected
            .iter()
            .filter_map(|tool| cap_adjusted_upper(tool))
            .sum();
        let tool_result_payload_bytes = selected
            .iter()
            .map(|tool| tool.result_payload_bytes.unwrap_or(0))
            .sum();

        let mut kinds = BTreeMap::<String, (u64, u64)>::new();
        for tool in &selected {
            let label = if tool.category.is_empty() {
                tool.kind.clone()
            } else {
                tool.category.clone()
            };
            let entry = kinds.entry(label).or_default();
            entry.0 += 1;
            entry.1 += tool.original_token_count.unwrap_or(0);
        }

        let input = usage.input_tokens.unwrap_or(0);
        let cached = usage.cached_input_tokens.unwrap_or(0);
        let write = usage.cache_write_input_tokens.unwrap_or(0);
        let uncached = input
            .checked_sub(cached)
            .and_then(|value| value.checked_sub(write));

        report.windows.push(AttributionWindow {
            index: (offset + 1) as u64,
            inference_call_id: current.inference_call_id.clone(),
            model: current.model.clone(),
            input_tokens: input,
            cached_input_tokens: cached,
            uncached_input_tokens: uncached,
            output_tokens: usage.output_tokens.unwrap_or(0),
            reasoning_output_tokens: usage.reasoning_output_tokens.unwrap_or(0),
            input_growth: previous.and_then(|prev| {
                prev.usage
                    .as_ref()
                    .and_then(|usage| usage.input_tokens.map(|value| signed_delta(input, value)))
            }),

            request_payload_bytes: current.request_payload_bytes,
            request_input_items: current.request_input_items,
            tools_since_previous: selected.len() as u64,
            tool_original_tokens,
            tool_cap_adjusted_upper_tokens,
            tool_result_payload_bytes,
            tool_kinds: kinds
                .into_iter()
                .map(|(kind, (count, original_tokens))| ToolKindCount {
                    kind,
                    count,
                    original_tokens,
                })
                .collect(),
        });

        previous = Some(current);
    }

    let unassigned = observations
        .tools
        .iter()
        .filter(|tool| !assigned_tools.contains(&tool.tool_call_id))
        .collect::<Vec<_>>();
    report.unattributed_tool_calls = unassigned.len() as u64;
    report.unattributed_tools_with_original_token_count = unassigned
        .iter()
        .filter(|tool| tool.original_token_count.is_some())
        .count() as u64;
    report.unattributed_tool_original_tokens = unassigned
        .iter()
        .map(|tool| tool.original_token_count.unwrap_or(0))
        .sum();
    report.unattributed_categories =
        aggregate_tools(unassigned.iter().copied(), |tool| &tool.category);

    report
}

fn aggregate_tools<'a>(
    tools: impl Iterator<Item = &'a ToolObservation>,
    label: impl Fn(&ToolObservation) -> &str,
) -> Vec<ToolAggregate> {
    let mut grouped = BTreeMap::<String, ToolAggregate>::new();
    for tool in tools {
        let key = {
            let value = label(tool);
            if value.is_empty() {
                "unknown".to_string()
            } else {
                value.to_string()
            }
        };
        let entry = grouped.entry(key.clone()).or_insert_with(|| ToolAggregate {
            label: key,
            ..Default::default()
        });
        entry.count += 1;
        if let Some(tokens) = tool.original_token_count {
            entry.token_count_known += 1;
            entry.original_tokens += tokens;
        }
        if let Some(upper) = cap_adjusted_upper(tool) {
            entry.cap_count_known += 1;
            entry.cap_adjusted_upper_tokens += upper;
        }
        if raw_exceeds_cap(tool) {
            entry.raw_exceeds_cap_count += 1;
        }
        entry.result_payload_bytes += tool.result_payload_bytes.unwrap_or(0);
        entry.result_output_chars += tool.result_output_chars.unwrap_or(0);
    }
    grouped.into_values().collect()
}

fn cap_adjusted_upper(tool: &ToolObservation) -> Option<u64> {
    match (tool.original_token_count, tool.max_output_tokens) {
        (Some(raw), Some(cap)) => Some(raw.min(cap)),
        _ => None,
    }
}

fn raw_exceeds_cap(tool: &ToolObservation) -> bool {
    matches!(
        (tool.original_token_count, tool.max_output_tokens),
        (Some(raw), Some(cap)) if raw > cap
    )
}

fn count_status(items: &[InferenceObservation], status: &str) -> u64 {
    items.iter().filter(|item| item.status == status).count() as u64
}

fn signed_delta(current: u64, previous: u64) -> i64 {
    let delta = i128::from(current) - i128::from(previous);
    delta.clamp(i128::from(i64::MIN), i128::from(i64::MAX)) as i64
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokn_domain::{TokenUsage, ToolObservation};

    #[test]
    fn attributes_tool_between_completed_calls() {
        let observations = DiagnosticObservations {
            inferences: vec![
                inference("a", 10, 20, 100, 90),
                inference("b", 30, 40, 125, 100),
            ],
            tools: vec![ToolObservation {
                tool_call_id: "tool-1".into(),
                kind: "exec_command".into(),
                tool_name: Some("exec_command".into()),
                category: "file_read".into(),
                surface: "runtime_mediated".into(),
                requester_type: Some("code_cell".into()),
                status: "completed".into(),
                observed_at: None,
                started_seq: Some(21),
                ended_seq: Some(25),
                invocation_payload_bytes: Some(50),
                result_payload_bytes: Some(100),
                result_output_chars: Some(80),
                max_output_tokens: Some(15),
                original_token_count: Some(20),
                command: None,
                workdir: None,
                parse_error: None,
            }],
            ..Default::default()
        };

        let report = build_attribution(&observations);
        assert_eq!(report.windows.len(), 2);
        assert_eq!(report.windows[1].tools_since_previous, 1);
        assert_eq!(report.windows[1].tool_original_tokens, 20);
        assert_eq!(report.windows[1].tool_cap_adjusted_upper_tokens, 15);
        assert_eq!(report.raw_exceeds_cap_count, 1);
        assert_eq!(report.windows[1].input_growth, Some(25));
        assert_eq!(report.unattributed_tool_calls, 0);
    }

    fn inference(id: &str, start: u64, end: u64, input: u64, cached: u64) -> InferenceObservation {
        InferenceObservation {
            inference_call_id: id.into(),
            status: "completed".into(),
            started_seq: Some(start),
            ended_seq: Some(end),
            usage: Some(TokenUsage {
                input_tokens: Some(input),
                cached_input_tokens: Some(cached),
                cache_write_input_tokens: Some(0),
                output_tokens: Some(1),
                reasoning_output_tokens: Some(0),
                ..Default::default()
            }),
            ..Default::default()
        }
    }
}
