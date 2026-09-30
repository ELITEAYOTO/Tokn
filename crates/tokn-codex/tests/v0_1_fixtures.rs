use std::fs;
use std::path::{Path, PathBuf};

use serde_json::Value;
use tokn_codex::session::extract_usage_record;

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("..").join("..")
}

fn read_jsonl(path: &Path) -> Vec<Value> {
    let text = fs::read_to_string(path).expect("fixture readable");
    text.lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| serde_json::from_str(line).expect("valid jsonl record"))
        .collect()
}

#[test]
fn parent_fixture_matches_required_shapes() {
    let path = repo_root().join("fixtures/codex/session/v0_1/parent/rollout.jsonl");
    let records = read_jsonl(&path);

    let meta = records
        .iter()
        .find(|record| record.get("type").and_then(Value::as_str) == Some("session_meta"))
        .expect("session meta");
    let payload = meta.get("payload").expect("meta payload");
    assert_eq!(
        payload.get("thread_source").and_then(Value::as_str),
        Some("user")
    );

    let usage = records
        .iter()
        .find_map(extract_usage_record)
        .expect("usage record");
    assert_eq!(usage.usage.input_tokens, Some(10_000));
    assert_eq!(usage.usage.cached_input_tokens, Some(9_000));
    assert_eq!(usage.usage.logical_total(), Some(10_600));

    let terminal = records
        .iter()
        .filter_map(|record| record.get("payload"))
        .find(|payload| payload.get("type").and_then(Value::as_str) == Some("task_complete"))
        .expect("task complete");
    assert_eq!(
        terminal
            .get("error")
            .and_then(|value| value.get("codex_error_info"))
            .and_then(Value::as_str),
        Some("usage_limit_exceeded")
    );
}

#[test]
fn subagent_fixture_preserves_relationship_shape() {
    let path = repo_root().join("fixtures/codex/session/v0_1/subagent/rollout.jsonl");
    let records = read_jsonl(&path);
    let meta = records
        .iter()
        .find(|record| record.get("type").and_then(Value::as_str) == Some("session_meta"))
        .expect("session meta");
    let payload = meta.get("payload").expect("meta payload");

    assert_eq!(
        payload.get("parent_thread_id").and_then(Value::as_str),
        Some("thread-fixture-parent")
    );
    assert_eq!(
        payload.get("forked_from_id").and_then(Value::as_str),
        Some("thread-fixture-parent")
    );
    assert_eq!(
        payload.get("thread_source").and_then(Value::as_str),
        Some("subagent")
    );
    assert!(payload.pointer("/source/subagent/thread_spawn").is_some());
}

#[test]
fn tool_cap_fixture_has_capped_and_uncapped_exec_shapes() {
    let path = repo_root().join("fixtures/codex/session/v0_1/tool-caps/rollout.jsonl");
    let records = read_jsonl(&path);
    let inputs = records
        .iter()
        .filter_map(|record| record.get("payload"))
        .filter(|payload| payload.get("type").and_then(Value::as_str) == Some("custom_tool_call"))
        .filter(|payload| payload.get("name").and_then(Value::as_str) == Some("exec"))
        .filter_map(|payload| payload.get("input").and_then(Value::as_str))
        .collect::<Vec<_>>();

    assert_eq!(inputs.len(), 2);
    assert!(
        inputs
            .iter()
            .any(|input| input.contains("max_output_tokens:5000"))
    );
    assert!(
        inputs
            .iter()
            .any(|input| !input.contains("max_output_tokens"))
    );
}

#[test]
fn partial_diagnostic_fixture_contains_no_usage_surface() {
    let path = repo_root().join("fixtures/codex/diagnostic/partial-protocol-only/trace.jsonl");
    let records = read_jsonl(&path);

    assert_eq!(records.len(), 4);
    assert!(records.iter().all(|record| {
        let payload_type = record
            .get("payload")
            .and_then(|payload| payload.get("type"))
            .and_then(Value::as_str);
        !matches!(
            payload_type,
            Some("inference_completed" | "tool_call_started" | "tool_call_ended")
        )
    }));
}

#[test]
fn v0_1_fixtures_are_sanitized() {
    let roots = [
        repo_root().join("fixtures/codex/session/v0_1"),
        repo_root().join("fixtures/codex/diagnostic/partial-protocol-only"),
        repo_root().join("fixtures/experiments/001-runner-golden"),
    ];
    let forbidden = [
        "timot",
        "renou",
        "blockbench-plugin",
        "jem_ultimate",
        "01a0e3ed",
        "18eff7c4",
        "@gmail",
        "c:\\users\\",
    ];

    for root in roots {
        scan_sanitized(&root, &forbidden);
    }
}

#[test]
fn exp001_runner_golden_sessions_contain_only_minimal_evidence_records() {
    let root = repo_root().join("fixtures/experiments/001-runner-golden/sessions");
    for entry in fs::read_dir(root).expect("golden session directory readable") {
        let path = entry.expect("golden session entry").path();
        if path.extension().and_then(|value| value.to_str()) != Some("jsonl") {
            continue;
        }

        for record in read_jsonl(&path) {
            let record_type = record.get("type").and_then(Value::as_str);
            assert!(
                matches!(
                    record_type,
                    Some("session_meta" | "token_usage_record" | "response_item" | "event_msg")
                ),
                "golden fixture {} contains disallowed top-level record type {:?}",
                path.display(),
                record_type
            );

            if record_type == Some("response_item") {
                let payload = record.get("payload").expect("response item payload");
                assert_eq!(
                    payload.get("type").and_then(Value::as_str),
                    Some("custom_tool_call")
                );
                assert_eq!(payload.get("name").and_then(Value::as_str), Some("exec"));
            }

            if record_type == Some("event_msg") {
                let payload = record.get("payload").expect("event payload");
                assert_eq!(
                    payload.get("type").and_then(Value::as_str),
                    Some("task_complete")
                );
            }
        }
    }
}

fn scan_sanitized(root: &Path, forbidden: &[&str]) {
    for entry in fs::read_dir(root).expect("fixture directory readable") {
        let entry = entry.expect("fixture entry");
        let path = entry.path();
        if path.is_dir() {
            scan_sanitized(&path, forbidden);
            continue;
        }

        let text = fs::read_to_string(&path).expect("fixture is text");
        let lower = text.to_ascii_lowercase();
        for needle in forbidden {
            assert!(
                !lower.contains(needle),
                "fixture {} contains forbidden marker {needle}",
                path.display()
            );
        }
    }
}
