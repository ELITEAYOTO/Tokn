use serde_json::json;
use tokn_codex::session::extract_token_usage;

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
