use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use anyhow::Context;
use tokn_analysis::{ProjectSnapshot, ProjectSnapshotGitStatus};
use tokn_codex::session::{
    extract_file_source_locator, file_source_locator_from_relative_path,
    read_session_tool_result_outputs,
};
use tokn_domain::{
    AgentEvidence, EvidenceIdentityCoverage, MeasurementContractManifest, ModelRuntimeProfile,
    PROJECT_SNAPSHOT_SCHEMA_VERSION, RunGroup, RunnerResult, RunnerSourceReport, SourceKind,
    SourceVersionBoundary, WorkspaceGitProvenanceCoverage,
};
use tokn_storage::{
    Database, MeasurementStoreInput, RateLimitStoreInput, SourceVersionStoreInput,
    ToolActivityStoreInput, WorkspaceGitProvenanceStoreInput, fingerprint_bytes, private_id,
    scoped_fingerprint_bytes, scoped_git_head_bytes, scoped_source_id_bytes,
    scoped_source_version_bytes,
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
    let tool_activities = build_tool_activities(
        &project_id,
        project_key,
        result.selected_workspace.as_deref(),
        &session_members,
    );
    let rate_limit_snapshots = build_rate_limit_snapshots(&session_members);
    let source_versions = build_source_versions(
        evidence_dir,
        project_key,
        result.selected_workspace.as_deref(),
    )?;
    let workspace_git_provenance = build_workspace_git_provenance(evidence_dir, project_key)?;
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
        source_versions,
        workspace_git_provenance,
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
    println!("  source_versions: {}", summary.source_version_count);
    println!(
        "  workspace_git_provenance: {}",
        summary.workspace_git_provenance_count
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
    project_key: &str,
    selected_workspace: Option<&str>,
    members: &[AgentEvidence],
) -> Vec<ToolActivityStoreInput> {
    let mut out = Vec::new();

    for member in members {
        let result_outputs = read_session_tool_result_outputs(Path::new(&member.source_path));
        let mut per_call_operation_count = BTreeMap::<String, usize>::new();
        for tool in &member.tools {
            *per_call_operation_count
                .entry(base_source_call_id(&tool.tool_call_id).to_string())
                .or_default() += 1;
        }

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
            let source_locator =
                extract_file_source_locator(tool, member.cwd.as_deref(), selected_workspace);
            let source_stable_id = source_locator
                .locator
                .as_deref()
                .map(|locator| scoped_source_id_bytes(project_key, locator.as_bytes()));
            let source_identity_coverage = if source_stable_id.is_some() {
                EvidenceIdentityCoverage::Observed
            } else {
                source_locator.coverage
            };
            let base_call_id = base_source_call_id(&tool.tool_call_id);
            let (content_fingerprint, content_identity_coverage) = match &result_outputs {
                Ok(outputs)
                    if per_call_operation_count.get(base_call_id) == Some(&1)
                        && outputs
                            .get(base_call_id)
                            .is_some_and(|items| items.len() == 1) =>
                {
                    let output = &outputs[base_call_id][0];
                    (
                        Some(scoped_fingerprint_bytes(
                            "cnt",
                            project_key,
                            "tool-result-output-v1",
                            output.as_bytes(),
                        )),
                        EvidenceIdentityCoverage::Observed,
                    )
                }
                Ok(_) => (None, EvidenceIdentityCoverage::NotCaptured),
                Err(_) => (None, EvidenceIdentityCoverage::Unknown),
            };

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
                observed_at: tool.observed_at.clone(),
                started_seq: tool.started_seq,
                ended_seq: tool.ended_seq,
                invocation_payload_bytes: tool.invocation_payload_bytes,
                result_payload_bytes: tool.result_payload_bytes,
                result_output_chars: tool.result_output_chars,
                max_output_tokens: tool.max_output_tokens,
                original_token_count: tool.original_token_count,
                operation_fingerprint,
                workdir_fingerprint,
                source_stable_id,
                source_identity_coverage,
                content_fingerprint,
                content_identity_coverage,
                parse_error_present: tool.parse_error.is_some(),
            });
        }
    }

    out
}

fn build_source_versions(
    evidence_dir: &Path,
    project_key: &str,
    selected_workspace: Option<&str>,
) -> anyhow::Result<Vec<SourceVersionStoreInput>> {
    let mut out = Vec::new();
    let mut seen = BTreeSet::new();

    for (boundary, name) in [
        (
            SourceVersionBoundary::Before,
            "workspace-before-snapshot.json",
        ),
        (
            SourceVersionBoundary::After,
            "workspace-after-snapshot.json",
        ),
    ] {
        let path = evidence_dir.join(name);
        if !path.is_file() {
            continue;
        }
        let snapshot: ProjectSnapshot = read_json(&path)?;
        if snapshot.schema_version != PROJECT_SNAPSHOT_SCHEMA_VERSION {
            anyhow::bail!(
                "unsupported project snapshot schema_version {} in {}; expected {}",
                snapshot.schema_version,
                path.display(),
                PROJECT_SNAPSHOT_SCHEMA_VERSION
            );
        }
        for file in snapshot.files {
            if file.sha256.trim().is_empty() {
                anyhow::bail!("empty file sha256 in {}", path.display());
            }
            let source = file_source_locator_from_relative_path(&file.path, selected_workspace);
            let Some(locator) = source.locator.as_deref() else {
                continue;
            };
            let source_stable_id = scoped_source_id_bytes(project_key, locator.as_bytes());
            if !seen.insert((boundary.as_str(), source_stable_id.clone())) {
                anyhow::bail!(
                    "duplicate source version boundary {} for {}",
                    boundary.as_str(),
                    source_stable_id
                );
            }
            out.push(SourceVersionStoreInput {
                source_stable_id,
                boundary,
                version_fingerprint: scoped_source_version_bytes(
                    project_key,
                    file.sha256.as_bytes(),
                ),
                snapshot_observed_at: snapshot.created_at.clone(),
                bytes: file.bytes,
            });
        }
    }

    Ok(out)
}

fn build_workspace_git_provenance(
    evidence_dir: &Path,
    project_key: &str,
) -> anyhow::Result<Vec<WorkspaceGitProvenanceStoreInput>> {
    let mut out = Vec::new();
    for (boundary, name) in [
        (SourceVersionBoundary::Before, "workspace-before-snapshot.json"),
        (SourceVersionBoundary::After, "workspace-after-snapshot.json"),
    ] {
        let path = evidence_dir.join(name);
        if !path.is_file() {
            continue;
        }
        let snapshot: ProjectSnapshot = read_json(&path)?;
        if snapshot.schema_version != PROJECT_SNAPSHOT_SCHEMA_VERSION {
            anyhow::bail!(
                "unsupported project snapshot schema_version {} in {}; expected {}",
                snapshot.schema_version,
                path.display(),
                PROJECT_SNAPSHOT_SCHEMA_VERSION
            );
        }
        let (coverage, head_fingerprint, dirty) = match snapshot.git.as_ref() {
            None => (WorkspaceGitProvenanceCoverage::NotCaptured, None, None),
            Some(git) if git.status == ProjectSnapshotGitStatus::Unknown => {
                (WorkspaceGitProvenanceCoverage::Unknown, None, None)
            }
            Some(git) => {
                let head = git
                    .head
                    .as_deref()
                    .map(str::trim)
                    .filter(|value| !value.is_empty())
                    .ok_or_else(|| anyhow::anyhow!("OBSERVED Git snapshot is missing HEAD in {}", path.display()))?;
                if !matches!(head.len(), 40 | 64)
                    || !head.bytes().all(|byte| byte.is_ascii_hexdigit())
                {
                    anyhow::bail!("invalid Git HEAD in {}", path.display());
                }
                let dirty = git.dirty.ok_or_else(|| {
                    anyhow::anyhow!("OBSERVED Git snapshot is missing dirty state in {}", path.display())
                })?;
                (
                    WorkspaceGitProvenanceCoverage::Observed,
                    Some(scoped_git_head_bytes(
                        project_key,
                        head.to_ascii_lowercase().as_bytes(),
                    )),
                    Some(dirty),
                )
            }
        };
        out.push(WorkspaceGitProvenanceStoreInput {
            boundary,
            coverage,
            head_fingerprint,
            dirty,
            snapshot_observed_at: snapshot.created_at,
        });
    }
    Ok(out)
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

fn base_source_call_id(tool_call_id: &str) -> &str {
    if let Some((base, suffix)) = tool_call_id.rsplit_once('#')
        && suffix.parse::<usize>().is_ok()
    {
        return base;
    }
    tool_call_id
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
        let activities = build_tool_activities(project_id, "fixture-project-key", None, &members);

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
    fn source_identity_is_workspace_relative_stable_and_project_scoped() {
        let missing_source = std::env::temp_dir()
            .join("tokn-source-id-no-result.jsonl")
            .to_string_lossy()
            .to_string();
        let member_a = AgentEvidence {
            source_path: missing_source.clone(),
            thread_id: "thread-a".into(),
            cwd: Some(r"E:\clone-a\PROJECT".into()),
            tools: vec![ToolObservation {
                tool_call_id: "call-source-a#0".into(),
                kind: "exec_command".into(),
                category: "file_read".into(),
                surface: "session_rollout".into(),
                status: "completed".into(),
                command: Some("Get-Content README.md".into()),
                ..Default::default()
            }],
            ..Default::default()
        };
        let mut member_b = member_a.clone();
        member_b.thread_id = "thread-b".into();
        member_b.cwd = Some(r"E:\clone-b\PROJECT".into());
        member_b.tools[0].tool_call_id = "call-source-b#0".into();

        let project_id = "prj-0123456789abcdef01234567";
        let left = build_tool_activities(
            project_id,
            "project-key-a",
            Some(r"E:\clone-a\PROJECT"),
            &[member_a],
        );
        let right = build_tool_activities(
            project_id,
            "project-key-a",
            Some(r"E:\clone-b\PROJECT"),
            &[member_b.clone()],
        );
        let other_project = build_tool_activities(
            project_id,
            "project-key-b",
            Some(r"E:\clone-b\PROJECT"),
            &[member_b],
        );

        assert_eq!(
            left[0].source_identity_coverage,
            EvidenceIdentityCoverage::Observed
        );
        assert_eq!(left[0].source_stable_id, right[0].source_stable_id);
        assert_ne!(left[0].source_stable_id, other_project[0].source_stable_id);
        assert!(
            left[0]
                .source_stable_id
                .as_deref()
                .is_some_and(|value| value.starts_with("src-v1-"))
        );
        let debug = format!("{left:?}{right:?}");
        assert!(!debug.contains("clone-a"));
        assert!(!debug.contains("clone-b"));
        assert!(!debug.contains("README.md"));
    }

    #[test]
    fn result_identity_is_project_scoped_and_raw_output_is_not_projected() {
        let path =
            std::env::temp_dir().join(format!("tokn-result-identity-{}.jsonl", std::process::id()));
        let raw_output = "PRIVATE FIXTURE RESULT alpha-123";
        let record = serde_json::json!({
            "type": "response_item",
            "payload": {
                "type": "custom_tool_call_output",
                "call_id": "call-identity",
                "output": raw_output
            }
        });
        std::fs::write(
            &path,
            format!("{}\n", serde_json::to_string(&record).unwrap()),
        )
        .unwrap();

        let member = AgentEvidence {
            source_path: path.to_string_lossy().to_string(),
            thread_id: "thread-root".into(),
            tools: vec![ToolObservation {
                tool_call_id: "call-identity#0".into(),
                kind: "exec_command".into(),
                category: "file_read".into(),
                surface: "session_rollout".into(),
                status: "completed".into(),
                ..Default::default()
            }],
            ..Default::default()
        };
        let project_id = "prj-0123456789abcdef01234567";
        let first = build_tool_activities(
            project_id,
            "project-key-a",
            None,
            std::slice::from_ref(&member),
        );
        let second = build_tool_activities(
            project_id,
            "project-key-a",
            None,
            std::slice::from_ref(&member),
        );
        let other_project = build_tool_activities(project_id, "project-key-b", None, &[member]);

        assert_eq!(
            first[0].content_identity_coverage,
            EvidenceIdentityCoverage::Observed
        );
        assert_eq!(first[0].content_fingerprint, second[0].content_fingerprint);
        assert_ne!(
            first[0].content_fingerprint,
            other_project[0].content_fingerprint
        );
        assert!(
            first[0]
                .content_fingerprint
                .as_deref()
                .is_some_and(|value| value.starts_with("cnt-v1-"))
        );
        assert_eq!(
            first[0].source_identity_coverage,
            EvidenceIdentityCoverage::NotCaptured
        );
        assert!(first[0].source_stable_id.is_none());
        assert!(!format!("{first:?}").contains(raw_output));

        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn ambiguous_multi_operation_output_stays_not_captured() {
        let path = std::env::temp_dir().join(format!(
            "tokn-result-identity-ambiguous-{}.jsonl",
            std::process::id()
        ));
        let record = serde_json::json!({
            "type": "response_item",
            "payload": {
                "type": "custom_tool_call_output",
                "call_id": "call-many",
                "output": "aggregate output"
            }
        });
        std::fs::write(
            &path,
            format!("{}\n", serde_json::to_string(&record).unwrap()),
        )
        .unwrap();
        let member = AgentEvidence {
            source_path: path.to_string_lossy().to_string(),
            thread_id: "thread-root".into(),
            tools: vec![
                ToolObservation {
                    tool_call_id: "call-many#0".into(),
                    category: "file_read".into(),
                    ..Default::default()
                },
                ToolObservation {
                    tool_call_id: "call-many#1".into(),
                    category: "search".into(),
                    ..Default::default()
                },
            ],
            ..Default::default()
        };

        let activities = build_tool_activities(
            "prj-0123456789abcdef01234567",
            "fixture-project-key",
            None,
            &[member],
        );
        assert_eq!(activities.len(), 2);
        assert!(activities.iter().all(|item| {
            item.content_fingerprint.is_none()
                && item.content_identity_coverage == EvidenceIdentityCoverage::NotCaptured
        }));
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn missing_source_file_keeps_result_identity_unknown() {
        let member = AgentEvidence {
            source_path: std::env::temp_dir()
                .join("tokn-definitely-missing-result-source.jsonl")
                .to_string_lossy()
                .to_string(),
            thread_id: "thread-root".into(),
            tools: vec![ToolObservation {
                tool_call_id: "call-missing#0".into(),
                category: "file_read".into(),
                ..Default::default()
            }],
            ..Default::default()
        };
        let activities = build_tool_activities(
            "prj-0123456789abcdef01234567",
            "fixture-project-key",
            None,
            &[member],
        );
        assert_eq!(
            activities[0].content_identity_coverage,
            EvidenceIdentityCoverage::Unknown
        );
        assert!(activities[0].content_fingerprint.is_none());
    }

    #[test]
    fn source_version_projection_matches_boundaries_without_raw_snapshot_material() {
        let dir = std::env::temp_dir().join(format!(
            "tokn-source-version-projection-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("create temp evidence");
        let private_relative = "private-dir/secret-source.rs";
        let raw_before = "TOKN-SYNTHETIC-BEFORE-SHA-001";
        let raw_after = "TOKN-SYNTHETIC-AFTER-SHA-002";
        for (name, created_at, sha) in [
            (
                "workspace-before-snapshot.json",
                "2026-10-03T10:00:00Z",
                raw_before,
            ),
            (
                "workspace-after-snapshot.json",
                "2026-10-03T10:30:00Z",
                raw_after,
            ),
        ] {
            let snapshot = serde_json::json!({
                "schema_version": PROJECT_SNAPSHOT_SCHEMA_VERSION,
                "created_at": created_at,
                "project_root": "E:\\fixture\\PROJECT",
                "excluded_top_level": [],
                "file_count": 1,
                "total_bytes": 42,
                "files": [{
                    "path": private_relative,
                    "sha256": sha,
                    "bytes": 42,
                    "last_write_utc": created_at
                }]
            });
            std::fs::write(
                dir.join(name),
                serde_json::to_vec_pretty(&snapshot).expect("serialize snapshot"),
            )
            .expect("write snapshot");
        }

        let versions =
            build_source_versions(&dir, "fixture-project-key", Some(r"E:\fixture\PROJECT"))
                .expect("source versions");
        assert_eq!(versions.len(), 2);
        assert_eq!(versions[0].source_stable_id, versions[1].source_stable_id);
        assert_ne!(
            versions[0].version_fingerprint,
            versions[1].version_fingerprint
        );
        assert!(
            versions
                .iter()
                .all(|item| item.source_stable_id.starts_with("src-v1-"))
        );
        assert!(
            versions
                .iter()
                .all(|item| item.version_fingerprint.starts_with("ver-v1-"))
        );
        let debug = format!("{versions:?}");
        assert!(!debug.contains(private_relative));
        assert!(!debug.contains(raw_before));
        assert!(!debug.contains(raw_after));
        let _ = std::fs::remove_dir_all(dir);
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
