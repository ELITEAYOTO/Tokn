use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use anyhow::Context;
use tokn_domain::{
    AgentEvidence, MeasurementContractManifest, ModelRuntimeProfile, RunGroup, RunnerResult,
    RunnerSourceReport, SourceKind,
};
use tokn_storage::{
    Database, MeasurementStoreInput, RateLimitStoreInput, ToolActivityStoreInput,
    fingerprint_bytes, private_id,
};

use super::common::{db_path, open_db};

pub fn run(
    evidence_dir: &Path,
    project_key: &str,
    workspace_key: &str,
    parent_workspace_key: Option<&str>,
    runtime_profile_path: Option<&Path>,
    db_override: Option<&Path>,
) -> anyhow::Result<()> {
    if project_key.trim().is_empty() {
        anyhow::bail!("project key cannot be empty");
    }
    if workspace_key.trim().is_empty() {
        anyhow::bail!("workspace key cannot be empty");
    }
    if parent_workspace_key.is_some_and(|value| value.trim().is_empty()) {
        anyhow::bail!("parent workspace key cannot be empty when provided");
    }
    if !evidence_dir.is_dir() {
        anyhow::bail!("evidence directory not found: {}", evidence_dir.display());
    }

    let contract: MeasurementContractManifest =
        read_json(&evidence_dir.join("measurement-contract.json"))?;
    let result: RunnerResult = read_json(&evidence_dir.join("runner-result.json"))?;
    let group: RunGroup = read_json(&evidence_dir.join("run-group.json"))?;
    let source: RunnerSourceReport = read_json(&evidence_dir.join("source-health.json"))?;

    let session_evidence_path = evidence_dir.join("session-evidence.json");
    let session_evidence = std::fs::read(&session_evidence_path)
        .with_context(|| format!("read {}", session_evidence_path.display()))?;
    let session_members: Vec<AgentEvidence> = serde_json::from_slice(
        session_evidence
            .strip_prefix(&[0xEF, 0xBB, 0xBF])
            .unwrap_or(&session_evidence),
    )
    .with_context(|| format!("parse {}", session_evidence_path.display()))?;
    let source_fingerprint = fingerprint_bytes(&session_evidence);

    let workspace_snapshot_path = evidence_dir.join("workspace-after-snapshot.json");
    let workspace_snapshot_fingerprint = if workspace_snapshot_path.is_file() {
        let bytes = std::fs::read(&workspace_snapshot_path)
            .with_context(|| format!("read {}", workspace_snapshot_path.display()))?;
        Some(fingerprint_bytes(&bytes))
    } else {
        None
    };

    let profile = runtime_profile_path
        .map(read_json::<ModelRuntimeProfile>)
        .transpose()?;

    let project_id = private_id("prj", project_key);
    let tool_activities = build_tool_activities(&project_id, &session_members);
    let rate_limit_snapshots = build_rate_limit_snapshots(&session_members);
    let workspace_id = private_id("wsp", &format!("{project_key}:{workspace_key}"));
    let parent_workspace_id =
        parent_workspace_key.map(|key| private_id("wsp", &format!("{project_key}:{key}")));

    let input = MeasurementStoreInput {
        project_id,
        workspace_id,
        parent_workspace_id,
        workspace_snapshot_fingerprint,
        source_kind: source_kind_label(&source.requested_kind).into(),
        source_fingerprint,
        source_snapshot_bytes: Some(u64::try_from(session_evidence.len())?),
        adapter_name: Some("tokn-runner-evidence".into()),
        adapter_version: Some(contract.session_evidence_schema_version.to_string()),
        contract,
        result,
        group,
        profile,
        tool_activities,
        rate_limit_snapshots,
    };

    let db = match db_override {
        Some(path) => Database::open(path)?,
        None => open_db()?,
    };
    let summary = db.save_measurement(&input)?;

    println!("TOKN STORE EVIDENCE");
    println!("  run_id: {}", summary.run_id);
    println!("  project_id: {}", summary.project_id);
    println!("  workspace_id: {}", summary.workspace_id);
    println!("  agents: {}", summary.agent_count);
    println!("  tool_activities: {}", summary.tool_activity_count);
    println!(
        "  rate_limit_snapshots: {}",
        summary.rate_limit_snapshot_count
    );
    println!("  source_id: {}", summary.source_id);
    println!(
        "  profile_id: {}",
        summary.profile_id.as_deref().unwrap_or("<none>")
    );
    println!(
        "  store_schema: {}",
        db.measurement_store_schema_version()?
            .as_deref()
            .unwrap_or("<unknown>")
    );
    println!(
        "  db: {}",
        db_override
            .map(PathBuf::from)
            .unwrap_or(db_path()?)
            .display()
    );

    Ok(())
}

fn build_tool_activities(
    project_id: &str,
    members: &[AgentEvidence],
) -> Vec<ToolActivityStoreInput> {
    let mut out = Vec::new();

    for member in members {
        for (ordinal, tool) in member.tools.iter().enumerate() {
            let operation_fingerprint = if repeat_relevant_category(&tool.category) {
                tool.command.as_deref().and_then(|command| {
                    let command = command.trim();
                    let workdir = tool
                        .workdir
                        .as_deref()
                        .map(str::trim)
                        .unwrap_or("<unknown>");
                    (!command.is_empty()).then(|| {
                        private_id(
                            "op",
                            &format!("{project_id}:{}:{workdir}:{command}", tool.category),
                        )
                    })
                })
            } else {
                None
            };
            let workdir_fingerprint = tool.workdir.as_deref().and_then(|workdir| {
                let workdir = workdir.trim();
                (!workdir.is_empty()).then(|| private_id("cwd", &format!("{project_id}:{workdir}")))
            });

            out.push(ToolActivityStoreInput {
                source_call_key: tool.tool_call_id.clone(),
                thread_id: member.thread_id.clone(),
                agent_ordinal: ordinal as u64,
                kind: tool.kind.clone(),
                tool_name: tool.tool_name.clone(),
                category: tool.category.clone(),
                surface: tool.surface.clone(),
                requester_type: tool.requester_type.clone(),
                status: tool.status.clone(),
                started_seq: tool.started_seq,
                ended_seq: tool.ended_seq,
                invocation_payload_bytes: tool.invocation_payload_bytes,
                result_payload_bytes: tool.result_payload_bytes,
                result_output_chars: tool.result_output_chars,
                max_output_tokens: tool.max_output_tokens,
                original_token_count: tool.original_token_count,
                operation_fingerprint,
                workdir_fingerprint,
                parse_error_present: tool.parse_error.is_some(),
            });
        }
    }

    out
}

fn build_rate_limit_snapshots(members: &[AgentEvidence]) -> Vec<RateLimitStoreInput> {
    let mut out = Vec::new();
    let mut seen = BTreeSet::new();

    for member in members {
        for snapshot in &member.rate_limit_snapshots {
            let item = RateLimitStoreInput {
                observed_at: snapshot.observed_at.clone(),
                limit_id: snapshot.limit_id.clone(),
                primary_used_percent: snapshot
                    .primary
                    .as_ref()
                    .map(|window| window.used_percent.clone()),
                primary_window_minutes: snapshot
                    .primary
                    .as_ref()
                    .and_then(|window| window.window_minutes),
                primary_resets_at: snapshot
                    .primary
                    .as_ref()
                    .and_then(|window| window.resets_at.clone()),
                secondary_used_percent: snapshot
                    .secondary
                    .as_ref()
                    .map(|window| window.used_percent.clone()),
                secondary_window_minutes: snapshot
                    .secondary
                    .as_ref()
                    .and_then(|window| window.window_minutes),
                secondary_resets_at: snapshot
                    .secondary
                    .as_ref()
                    .and_then(|window| window.resets_at.clone()),
                rate_limit_reached_type: snapshot.rate_limit_reached_type.clone(),
            };
            if seen.insert(item.clone()) {
                out.push(item);
            }
        }
    }

    out
}

fn repeat_relevant_category(category: &str) -> bool {
    matches!(category, "file_read" | "search" | "directory_list" | "git")
}

fn read_json<T>(path: &Path) -> anyhow::Result<T>
where
    T: serde::de::DeserializeOwned,
{
    let bytes = std::fs::read(path).with_context(|| format!("read {}", path.display()))?;
    let bytes = bytes.strip_prefix(&[0xEF, 0xBB, 0xBF]).unwrap_or(&bytes);
    serde_json::from_slice(bytes).with_context(|| format!("parse {}", path.display()))
}

fn source_kind_label(kind: &SourceKind) -> &'static str {
    match kind {
        SourceKind::CodexSession => "codex-session",
        SourceKind::CodexDiagnosticTrace => "codex-diagnostic-trace",
        SourceKind::Unknown => "unknown",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokn_domain::{RateLimitSnapshotObservation, RateLimitWindowObservation, ToolObservation};

    #[test]
    fn source_kind_labels_match_serialized_contract() {
        assert_eq!(
            source_kind_label(&SourceKind::CodexSession),
            "codex-session"
        );
        assert_eq!(
            source_kind_label(&SourceKind::CodexDiagnosticTrace),
            "codex-diagnostic-trace"
        );
        assert_eq!(source_kind_label(&SourceKind::Unknown), "unknown");
    }
    #[test]
    fn activity_fingerprints_are_project_scoped_and_minimized() {
        let private_workdir = format!(r"C:\{}\private-user\Secret Project", "Users");
        let private_read = format!(r"Get-Content {private_workdir}\secret.txt");
        let private_write = format!(r"Set-Content {private_workdir}\secret.txt x");
        let members = vec![AgentEvidence {
            thread_id: "thread-root".into(),
            tools: vec![
                ToolObservation {
                    tool_call_id: "call-1".into(),
                    kind: "exec_command".into(),
                    tool_name: Some("exec_command".into()),
                    category: "file_read".into(),
                    surface: "session_rollout".into(),
                    status: "completed".into(),
                    command: Some(private_read.clone()),
                    workdir: Some(private_workdir.clone()),
                    ..Default::default()
                },
                ToolObservation {
                    tool_call_id: "call-2".into(),
                    kind: "exec_command".into(),
                    tool_name: Some("exec_command".into()),
                    category: "write_mutation".into(),
                    surface: "session_rollout".into(),
                    status: "completed".into(),
                    command: Some(private_write.clone()),
                    workdir: Some(private_workdir.clone()),
                    ..Default::default()
                },
            ],
            ..Default::default()
        }];

        let project_id = "prj-0123456789abcdef01234567";
        let activities = build_tool_activities(project_id, &members);

        assert_eq!(activities.len(), 2);
        assert!(
            activities[0]
                .operation_fingerprint
                .as_deref()
                .is_some_and(|value| value.starts_with("op-"))
        );
        assert!(activities[1].operation_fingerprint.is_none());
        assert!(activities.iter().all(|item| {
            item.workdir_fingerprint
                .as_deref()
                .is_some_and(|value| value.starts_with("cwd-"))
        }));

        let debug = format!("{activities:?}");
        assert!(!debug.contains(&private_read));
        assert!(!debug.contains(&private_write));
        assert!(!debug.contains(&private_workdir));
        assert!(!debug.contains("secret.txt"));
    }

    #[test]
    fn only_repeat_relevant_categories_get_operation_fingerprints() {
        for category in ["file_read", "search", "directory_list", "git"] {
            assert!(repeat_relevant_category(category));
        }
        for category in [
            "write_mutation",
            "test_build",
            "process_control",
            "command_other",
            "command_unknown",
        ] {
            assert!(!repeat_relevant_category(category));
        }
    }

    #[test]
    fn rate_limit_storage_projection_deduplicates_agent_copies() {
        let snapshot = RateLimitSnapshotObservation {
            observed_at: "2026-10-03T07:15:00Z".into(),
            limit_id: Some("codex".into()),
            primary: Some(RateLimitWindowObservation {
                used_percent: "12.5".into(),
                window_minutes: Some(300),
                resets_at: Some("1791018000".into()),
            }),
            secondary: None,
            rate_limit_reached_type: None,
        };
        let members = vec![
            AgentEvidence {
                thread_id: "thread-root".into(),
                rate_limit_snapshots: vec![snapshot.clone()],
                ..Default::default()
            },
            AgentEvidence {
                thread_id: "thread-child".into(),
                rate_limit_snapshots: vec![snapshot],
                ..Default::default()
            },
        ];

        let snapshots = build_rate_limit_snapshots(&members);
        assert_eq!(snapshots.len(), 1);
        assert_eq!(snapshots[0].limit_id.as_deref(), Some("codex"));
        assert_eq!(snapshots[0].primary_used_percent.as_deref(), Some("12.5"));
    }
}
