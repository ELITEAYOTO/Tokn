use std::path::{Path, PathBuf};

use tokn_codex::diagnostic::{DiagnosticBundle, assess_diagnostic_health};
use tokn_codex::session::assess_session_health;
use tokn_domain::SourceHealthStatus;

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("..").join("..")
}

#[test]
fn protocol_only_diagnostic_is_partial() {
    let root = repo_root().join("fixtures/codex/diagnostic/partial-protocol-only");
    let bundle = DiagnosticBundle::detect(&root).expect("diagnostic fixture");
    let health = assess_diagnostic_health(&bundle).expect("health");

    assert_eq!(health.status, SourceHealthStatus::Partial);
    assert_eq!(health.coverage.records_seen, 4);
    assert_eq!(health.coverage.inference_events, 0);
    assert_eq!(health.coverage.usage_records, 0);
    assert!(!health.is_usable_for_token_accounting());
}

#[test]
fn diagnostic_with_inference_usage_is_healthy() {
    let root = repo_root().join("fixtures/codex/diagnostic/v0_1-healthy");
    let bundle = DiagnosticBundle::detect(&root).expect("diagnostic fixture");
    let health = assess_diagnostic_health(&bundle).expect("health");

    assert_eq!(health.status, SourceHealthStatus::Healthy);
    assert_eq!(health.coverage.inference_events, 2);
    assert_eq!(health.coverage.usage_records, 1);
    assert!(health.is_usable_for_token_accounting());
}

#[test]
fn session_with_meta_and_usage_is_healthy() {
    let path = repo_root().join("fixtures/codex/session/v0_1/parent/rollout.jsonl");
    let health = assess_session_health(&path).expect("health");

    assert_eq!(health.status, SourceHealthStatus::Healthy);
    assert_eq!(health.coverage.usage_records, 1);
    assert_eq!(health.coverage.terminal_events, 1);
    assert!(health.is_usable_for_token_accounting());
}

#[test]
fn tool_only_session_is_partial() {
    let path = repo_root().join("fixtures/codex/session/v0_1/tool-caps/rollout.jsonl");
    let health = assess_session_health(&path).expect("health");

    assert_eq!(health.status, SourceHealthStatus::Partial);
    assert_eq!(health.coverage.usage_records, 0);
    assert!(!health.is_usable_for_token_accounting());
}
