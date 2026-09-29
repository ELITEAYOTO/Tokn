use serde_json::Value;
use tokn_domain::TokenUsage;

#[derive(Debug, Clone)]
pub struct SessionUsageObservation {
    pub usage: TokenUsage,
    pub response_id: Option<String>,
    pub thread_id: Option<String>,
    pub turn_id: Option<String>,
    pub root_turn_id: Option<String>,
}

pub fn extract_usage_record(value: &Value) -> Option<SessionUsageObservation> {
    if value.get("type").and_then(Value::as_str) != Some("token_usage_record") {
        return None;
    }

    let payload = value.get("payload")?;
    let usage = parse_usage(payload.get("usage")?)?;

    Some(SessionUsageObservation {
        usage,
        response_id: string(payload, "response_id"),
        thread_id: string(payload, "thread_id"),
        turn_id: string(payload, "turn_id"),
        root_turn_id: string(payload, "root_turn_id"),
    })
}
pub fn extract_token_usage(value: &Value) -> Option<TokenUsage> {
    let payload = value.get("payload")?;
    if payload.get("type").and_then(Value::as_str) != Some("token_count") {
        return None;
    }

    let info = payload.get("info")?;
    let usage = info
        .get("last_token_usage")
        .or_else(|| info.get("token_usage"))?;

    parse_usage(usage)
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

fn string(value: &Value, key: &str) -> Option<String> {
    value.get(key).and_then(Value::as_str).map(str::to_string)
}
