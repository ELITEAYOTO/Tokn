use serde_json::json;
use tokn_codex::session::{extract_rate_limit_snapshot, extract_token_usage};

#[test]
fn extracts_last_token_usage() {
    let value = json!({
        "type": "event_msg",
        "payload": {
            "type": "token_count",
            "info": {
                "last_token_usage": {
                    "input_tokens": 1200,
                    "cached_input_tokens": 800,
                    "output_tokens": 160,
                    "reasoning_output_tokens": 40,
                    "total_tokens": 1360
                }
            }
        }
    });

    let usage = extract_token_usage(&value).expect("usage");
    assert_eq!(usage.input_tokens, Some(1200));
    assert_eq!(usage.cached_input_tokens, Some(800));
    assert_eq!(usage.logical_total(), Some(1360));
}
#[test]
fn extracts_rate_limit_snapshot() {
    let value = json!({
        "timestamp": "2026-10-03T07:15:00Z",
        "type": "event_msg",
        "payload": {
            "type": "token_count",
            "rate_limits": {
                "limit_id": "codex",
                "limit_name": "PRIVATE-LIMIT-NAME-SHOULD-NOT-PERSIST",
                "primary": {"used_percent": 12.5, "window_minutes": 300, "resets_at": 1791018000},
                "secondary": {"used_percent": 42, "window_minutes": 10080, "resets_at": 1791622800},
                "credits": {"unlimited": false, "balance": "PRIVATE-NOT-STORED"},
                "rate_limit_reached_type": "primary"
            }
        }
    });

    let snapshot = extract_rate_limit_snapshot(&value).expect("rate limit snapshot");
    assert_eq!(snapshot.observed_at, "2026-10-03T07:15:00Z");
    assert_eq!(snapshot.limit_id.as_deref(), Some("codex"));
    assert_eq!(
        snapshot.primary.as_ref().map(|w| w.used_percent.as_str()),
        Some("12.5")
    );
    assert_eq!(
        snapshot.secondary.as_ref().and_then(|w| w.window_minutes),
        Some(10080)
    );
    assert_eq!(snapshot.rate_limit_reached_type.as_deref(), Some("primary"));
    let serialized = serde_json::to_string(&snapshot).expect("serialize snapshot");
    assert!(!serialized.contains("PRIVATE-NOT-STORED"));
    assert!(!serialized.contains("PRIVATE-LIMIT-NAME-SHOULD-NOT-PERSIST"));
    assert!(!serialized.contains("credits"));
}

#[test]
fn ignores_rate_limit_payload_without_diagnostic_windows() {
    let value = json!({
        "timestamp": "2026-10-03T07:15:00Z",
        "type": "event_msg",
        "payload": {"type": "token_count", "rate_limits": {"limit_id": "codex"}}
    });
    assert!(extract_rate_limit_snapshot(&value).is_none());
}
