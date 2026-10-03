use serde_json::Value;
use tokn_domain::{RateLimitSnapshotObservation, RateLimitWindowObservation, TokenUsage};

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
    let payload = token_count_payload(value)?;
    let info = payload.get("info")?;
    let usage = info
        .get("last_token_usage")
        .or_else(|| info.get("token_usage"))?;

    parse_usage(usage)
}

pub fn extract_rate_limit_snapshot(value: &Value) -> Option<RateLimitSnapshotObservation> {
    let payload = token_count_payload(value)?;
    let snapshot = payload.get("rate_limits")?;
    if !snapshot.is_object() {
        return None;
    }

    let observed_at = string(value, "timestamp")?;
    let observed_at = observed_at.trim();
    if observed_at.is_empty() {
        return None;
    }

    let primary = snapshot.get("primary").and_then(parse_rate_limit_window);
    let secondary = snapshot.get("secondary").and_then(parse_rate_limit_window);
    let rate_limit_reached_type =
        string(snapshot, "rate_limit_reached_type").filter(|value| !value.trim().is_empty());

    if primary.is_none() && secondary.is_none() && rate_limit_reached_type.is_none() {
        return None;
    }

    Some(RateLimitSnapshotObservation {
        observed_at: observed_at.to_string(),
        limit_id: string(snapshot, "limit_id").filter(|value| !value.trim().is_empty()),
        primary,
        secondary,
        rate_limit_reached_type,
    })
}

fn token_count_payload(value: &Value) -> Option<&Value> {
    let payload = value.get("payload")?;
    (payload.get("type").and_then(Value::as_str) == Some("token_count")).then_some(payload)
}

fn parse_rate_limit_window(value: &Value) -> Option<RateLimitWindowObservation> {
    let used_percent = numeric_text(value.get("used_percent")?)?;
    Some(RateLimitWindowObservation {
        used_percent,
        window_minutes: signed_integer(value.get("window_minutes")),
        resets_at: integer_text(value.get("resets_at")),
    })
}

fn numeric_text(value: &Value) -> Option<String> {
    match value {
        Value::Number(number) => number
            .as_f64()
            .filter(|value| value.is_finite())
            .map(|_| number.to_string()),
        Value::String(raw) => raw
            .trim()
            .parse::<f64>()
            .ok()
            .filter(|value| value.is_finite())
            .map(|_| raw.trim().to_string()),
        _ => None,
    }
}

fn signed_integer(value: Option<&Value>) -> Option<i64> {
    match value? {
        Value::Number(number) => number.as_i64(),
        Value::String(raw) => raw.trim().parse::<i64>().ok(),
        _ => None,
    }
}

fn integer_text(value: Option<&Value>) -> Option<String> {
    signed_integer(value).map(|value| value.to_string())
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
