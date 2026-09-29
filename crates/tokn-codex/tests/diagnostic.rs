use serde_json::json;
use tokn_codex::diagnostic::extract_trace_usage_record;

#[test]
fn extracts_trace_identity_and_usage() {
    let value = json!({
        "kind": "inference_call",
        "inference_call_id": "inf-1",
        "thread_id": "thread-root",
        "turn_id": "turn-1",
        "response_id": "resp-1",
        "provider_name": "fixture",
        "usage": {
            "input_tokens": 1000,
            "cached_input_tokens": 800,
            "output_tokens": 200,
            "reasoning_output_tokens": 60
        }
    });

    let observation = extract_trace_usage_record(&value).expect("trace usage");
    assert_eq!(observation.inference_call_id.as_deref(), Some("inf-1"));
    assert_eq!(observation.response_id.as_deref(), Some("resp-1"));
    assert_eq!(observation.thread_id.as_deref(), Some("thread-root"));
    assert_eq!(observation.turn_id.as_deref(), Some("turn-1"));
    assert_eq!(observation.provider.as_deref(), Some("fixture"));
    assert_eq!(observation.usage.logical_total(), Some(1200));
}

#[test]
fn resolves_real_style_response_payload_reference() {
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};
    use tokn_codex::diagnostic::extract_trace_usage_record_from_bundle;

    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let root = std::env::temp_dir().join(format!("tokn-diag-{stamp}"));
    fs::create_dir_all(root.join("payloads")).unwrap();
    fs::write(
        root.join("payloads").join("7.json"),
        r#"{"response_id":"resp-real","token_usage":{"input_tokens":149287,"cached_input_tokens":145792,"cache_write_input_tokens":0,"output_tokens":149,"reasoning_output_tokens":18,"total_tokens":149436}}"#,
    )
    .unwrap();

    let value = json!({
        "thread_id": "thread-real",
        "codex_turn_id": "turn-real",
        "payload": {
            "type": "inference_completed",
            "inference_call_id": "inf-real",
            "response_id": "resp-real",
            "response_payload": {
                "raw_payload_id": "raw_payload:7",
                "kind": { "type": "inference_response" },
                "path": "payloads/7.json"
            }
        }
    });

    let observation =
        extract_trace_usage_record_from_bundle(&value, &root).expect("resolved usage");
    assert_eq!(observation.inference_call_id.as_deref(), Some("inf-real"));
    assert_eq!(observation.response_id.as_deref(), Some("resp-real"));
    assert_eq!(observation.thread_id.as_deref(), Some("thread-real"));
    assert_eq!(observation.turn_id.as_deref(), Some("turn-real"));
    assert_eq!(observation.usage.input_tokens, Some(149_287));
    assert_eq!(observation.usage.cached_input_tokens, Some(145_792));
    assert_eq!(observation.usage.output_tokens, Some(149));
    let _ = fs::remove_dir_all(root);
}
