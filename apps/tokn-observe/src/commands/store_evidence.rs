use std::path::{Path, PathBuf};

use anyhow::Context;
use tokn_domain::{
    MeasurementContractManifest, ModelRuntimeProfile, RunGroup, RunnerResult, RunnerSourceReport,
    SourceKind,
};
use tokn_storage::{Database, MeasurementStoreInput, fingerprint_bytes, private_id};

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
}
