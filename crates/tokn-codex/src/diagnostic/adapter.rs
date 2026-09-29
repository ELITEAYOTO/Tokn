use std::fs::File;
use std::path::{Component, Path};

use serde_json::Value;
use tokn_domain::TokenUsage;

#[derive(Debug, Clone)]
pub struct TraceUsageObservation {
    pub usage: TokenUsage,
    pub inference_call_id: Option<String>,
    pub response_id: Option<String>,
    pub thread_id: Option<String>,
    pub turn_id: Option<String>,
    pub provider: Option<String>,
}

pub fn extract_trace_usage(value: &Value) -> Option<TokenUsage> {
    extract_trace_usage_record(value).map(|record| record.usage)
}

pub fn extract_trace_usage_record(value: &Value) -> Option<TraceUsageObservation> {
    find_inline_usage(value)
}

pub fn extract_trace_usage_record_from_bundle(
    value: &Value,
    bundle_root: &Path,
) -> Option<TraceUsageObservation> {
    if let Some(observation) = extract_trace_usage_record(value) {
        return Some(observation);
    }

    let envelope = value.as_object()?;
    let payload = envelope.get("payload")?.as_object()?;
    if payload.get("type").and_then(Value::as_str) != Some("inference_completed") {
        return None;
    }

    let response_ref = payload.get("response_payload")?.as_object()?;
    let relative = response_ref.get("path")?.as_str()?;
    let payload_path = safe_bundle_path(bundle_root, relative)?;
    let response: Value = serde_json::from_reader(File::open(payload_path).ok()?).ok()?;
    let usage = parse_usage(response.get("token_usage")?)?;

    Some(TraceUsageObservation {
        usage,
        inference_call_id: string(payload, "inference_call_id"),
        response_id: string(payload, "response_id").or_else(|| {
            response
                .get("response_id")
                .and_then(Value::as_str)
                .map(str::to_string)
        }),
        thread_id: string(envelope, "thread_id"),
        turn_id: string(envelope, "codex_turn_id"),
        provider: None,
    })
}

fn safe_bundle_path(root: &Path, relative: &str) -> Option<std::path::PathBuf> {
    let rel = Path::new(relative);
    if rel
        .components()
        .any(|component| !matches!(component, Component::Normal(_) | Component::CurDir))
    {
        return None;
    }
    Some(root.join(rel))
}

fn find_inline_usage(value: &Value) -> Option<TraceUsageObservation> {
    match value {
        Value::Object(map) => {
            if let Some(usage) = map.get("usage")
                && let Some(parsed) = parse_usage(usage)
            {
                return Some(TraceUsageObservation {
                    usage: parsed,
                    inference_call_id: string(map, "inference_call_id"),
                    response_id: string(map, "response_id"),
                    thread_id: string(map, "thread_id"),
                    turn_id: string(map, "turn_id"),
                    provider: string(map, "provider").or_else(|| string(map, "provider_name")),
                });
            }

            map.values().find_map(find_inline_usage)
        }
        Value::Array(items) => items.iter().find_map(find_inline_usage),
        _ => None,
    }
}

fn parse_usage(value: &Value) -> Option<TokenUsage> {
    let input = number(value, "input_tokens");
    let output = number(value, "output_tokens");
    if input.is_none() && output.is_none() {
        return None;
    }

    Some(TokenUsage {
        input_tokens: input,
        cached_input_tokens: number(value, "cached_input_tokens"),
        cache_write_input_tokens: number(value, "cache_write_input_tokens"),
        output_tokens: output,
        reasoning_output_tokens: number(value, "reasoning_output_tokens"),
        reported_total_tokens: number(value, "total_tokens"),
    })
}

fn number(value: &Value, key: &str) -> Option<u64> {
    value.get(key).and_then(Value::as_u64)
}

fn string(map: &serde_json::Map<String, Value>, key: &str) -> Option<String> {
    map.get(key).and_then(Value::as_str).map(str::to_string)
}
