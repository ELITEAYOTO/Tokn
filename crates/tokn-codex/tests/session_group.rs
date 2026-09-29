use std::path::{Path, PathBuf};

use tokn_codex::session::{
    collect_session_group_from_files, read_session_evidence, read_session_metadata,
};
use tokn_domain::TerminalStatus;

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("..").join("..")
}

#[test]
fn reads_parent_evidence_and_usage_limit_terminal() {
    let path = repo_root().join("fixtures/codex/session/v0_1/parent/rollout.jsonl");
    let evidence = read_session_evidence(&path).expect("session evidence");

    assert_eq!(evidence.thread_id, "thread-fixture-parent");
    assert_eq!(evidence.usages.len(), 1);
    assert_eq!(
        evidence.terminal.status,
        TerminalStatus::IncompleteUsageLimit
    );
    assert_eq!(evidence.terminal.duration_ms, Some(5_000));
}

#[test]
fn metadata_reader_identifies_subagent_without_full_rollout_scan() {
    let path = repo_root().join("fixtures/codex/session/v0_1/subagent/rollout.jsonl");
    let metadata = read_session_metadata(&path)
        .expect("metadata read")
        .expect("session meta");

    assert!(metadata.is_subagent());
    assert!(!metadata.is_root_candidate());
    assert_eq!(metadata.thread_id.as_deref(), Some("thread-fixture-child"));
    assert_eq!(
        metadata.parent_thread_id.as_deref(),
        Some("thread-fixture-parent")
    );
}

#[test]
fn session_evidence_keeps_first_subagent_identity_when_parent_meta_is_replayed() {
    let path = repo_root().join("fixtures/codex/session/v0_1/subagent/rollout.jsonl");
    let evidence = read_session_evidence(&path).expect("session evidence");

    assert_eq!(evidence.thread_id, "thread-fixture-child");
    assert_eq!(
        evidence.parent_thread_id.as_deref(),
        Some("thread-fixture-parent")
    );
    assert_eq!(evidence.agent_nickname.as_deref(), Some("fixture-audit"));
}

#[test]
fn groups_fixture_parent_and_subagent_by_session_id() {
    let root = repo_root().join("fixtures/codex/session/v0_1/parent/rollout.jsonl");
    let child = repo_root().join("fixtures/codex/session/v0_1/subagent/rollout.jsonl");
    let unrelated = repo_root().join("fixtures/codex/session/v0_1/tool-caps/rollout.jsonl");

    let group =
        collect_session_group_from_files(&root, &[child, unrelated]).expect("session group");

    assert_eq!(group.root_thread_id, "thread-fixture-parent");
    assert_eq!(group.session_id.as_deref(), Some("session-fixture-root"));
    assert_eq!(group.members.len(), 2);
    assert!(
        group
            .members
            .iter()
            .any(|member| member.thread_id == "thread-fixture-child")
    );
}
