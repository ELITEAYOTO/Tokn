use serde_json::Value;
use tokn_domain::{TerminalObservation, TerminalStatus};

pub fn extract_terminal_observation(value: &Value) -> Option<TerminalObservation> {
    if value.get("type").and_then(Value::as_str) != Some("event_msg") {
        return None;
    }
    let payload = value.get("payload")?;
    if payload.get("type").and_then(Value::as_str) != Some("task_complete") {
        return None;
    }

    let error = payload.get("error");
    let error_code = error
        .and_then(|value| value.get("codex_error_info"))
        .and_then(Value::as_str)
        .map(str::to_string);
    let message = error
        .and_then(|value| value.get("message"))
        .and_then(Value::as_str);

    let status = classify_terminal(error_code.as_deref(), message, error);
    Some(TerminalObservation {
        status,
        error_code,
        duration_ms: payload.get("duration_ms").and_then(Value::as_u64),
        turn_id: payload
            .get("turn_id")
            .and_then(Value::as_str)
            .map(str::to_string),
    })
}

fn classify_terminal(
    error_code: Option<&str>,
    message: Option<&str>,
    error: Option<&Value>,
) -> TerminalStatus {
    if error.is_none() || error.is_some_and(Value::is_null) {
        return TerminalStatus::Completed;
    }

    match error_code {
        Some("usage_limit_exceeded") => TerminalStatus::IncompleteUsageLimit,
        Some("cancelled" | "canceled") => TerminalStatus::IncompleteCancelled,
        Some("user_aborted" | "user_cancelled" | "user_canceled") => {
            TerminalStatus::IncompleteUserAbort
        }
        Some(_) => TerminalStatus::IncompleteError,
        None => {
            let message = message.unwrap_or_default().to_ascii_lowercase();
            if message.contains("usage") && message.contains("limit") {
                TerminalStatus::IncompleteUsageLimit
            } else if message.contains("cancel") {
                TerminalStatus::IncompleteCancelled
            } else {
                TerminalStatus::IncompleteError
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn usage_limit_is_distinct_terminal_state() {
        let value = json!({
            "type": "event_msg",
            "payload": {
                "type": "task_complete",
                "turn_id": "turn-1",
                "duration_ms": 123,
                "error": {
                    "codex_error_info": "usage_limit_exceeded",
                    "message": "fixture"
                }
            }
        });
        let terminal = extract_terminal_observation(&value).expect("terminal");
        assert_eq!(terminal.status, TerminalStatus::IncompleteUsageLimit);
    }

    #[test]
    fn null_error_is_completed() {
        let value = json!({
            "type": "event_msg",
            "payload": {
                "type": "task_complete",
                "turn_id": "turn-1",
                "duration_ms": 123,
                "error": null
            }
        });
        let terminal = extract_terminal_observation(&value).expect("terminal");
        assert_eq!(terminal.status, TerminalStatus::Completed);
    }
}
