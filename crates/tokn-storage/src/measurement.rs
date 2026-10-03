use std::collections::BTreeSet;
use std::num::TryFromIntError;
use std::time::{SystemTime, UNIX_EPOCH};

use rusqlite::{OptionalExtension, params};
use thiserror::Error;
use tokn_domain::{
    EvidenceIdentityCoverage, HISTORICAL_SNAPSHOT_SCHEMA_VERSION, HistoricalAgentRecord,
    HistoricalProvenanceRecord, HistoricalRateLimitSnapshotRecord, HistoricalRunRecord,
    HistoricalRuntimeProfileRecord, HistoricalSnapshot, HistoricalSourceVersionRecord,
    HistoricalTaskInputRecord, HistoricalToolActivityRecord,
    HistoricalWorkspaceGitProvenanceRecord, HistoricalWorkspaceRecord, MEASUREMENT_CONTRACT_ID,
    MEASUREMENT_CONTRACT_VERSION, MeasurementContractManifest, ModelRuntimeProfile,
    RATE_LIMIT_HISTORY_SCHEMA_VERSION, RateLimitHistory, RunGroup, RunnerQualityStatus,
    RunnerResult, SOURCE_VERSION_HISTORY_SCHEMA_VERSION, SourceVersionBoundary,
    SourceVersionHistory, TASK_INPUT_HISTORY_SCHEMA_VERSION, TOOL_ACTIVITY_HISTORY_SCHEMA_VERSION,
    TaskInputDeliveryStatus, TaskInputHistory, TokenTotals, ToolActivityHistory,
    WORKSPACE_GIT_PROVENANCE_HISTORY_SCHEMA_VERSION, WorkspaceGitProvenanceCoverage,
    WorkspaceGitProvenanceHistory,
};

#[cfg(test)]
use tokn_domain::scoped_source_id_bytes;

use crate::Database;

const PRIVATE_ID_HEX_LEN: usize = 24;
const SCOPED_FINGERPRINT_HEX_LEN: usize = 64;

#[derive(Debug, Error)]
pub enum MeasurementStoreError {
    #[error(transparent)]
    Sql(#[from] rusqlite::Error),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
    #[error(transparent)]
    Integer(#[from] TryFromIntError),
    #[error("invalid measurement store input: {0}")]
    Invalid(String),
}

#[derive(Debug, Clone)]
pub struct MeasurementStoreInput {
    pub project_id: String,
    pub workspace_id: String,
    pub parent_workspace_id: Option<String>,
    pub workspace_snapshot_fingerprint: Option<String>,
    pub source_kind: String,
    pub source_fingerprint: String,
    pub source_snapshot_bytes: Option<u64>,
    pub adapter_name: Option<String>,
    pub adapter_version: Option<String>,
    pub contract: MeasurementContractManifest,
    pub result: RunnerResult,
    pub group: RunGroup,
    pub profile: Option<ModelRuntimeProfile>,
    pub task_input: TaskInputStoreInput,
    pub tool_activities: Vec<ToolActivityStoreInput>,
    pub rate_limit_snapshots: Vec<RateLimitStoreInput>,
    pub source_versions: Vec<SourceVersionStoreInput>,
    pub workspace_git_provenance: Vec<WorkspaceGitProvenanceStoreInput>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToolActivityStoreInput {
    pub source_call_key: String,
    pub thread_id: String,
    pub agent_ordinal: u64,
    pub kind: String,
    pub tool_name: Option<String>,
    pub category: String,
    pub surface: String,
    pub requester_type: Option<String>,
    pub status: String,
    pub observed_at: Option<String>,
    pub started_seq: Option<u64>,
    pub ended_seq: Option<u64>,
    pub invocation_payload_bytes: Option<u64>,
    pub result_payload_bytes: Option<u64>,
    pub result_output_chars: Option<u64>,
    pub max_output_tokens: Option<u64>,
    pub original_token_count: Option<u64>,
    pub operation_fingerprint: Option<String>,
    pub workdir_fingerprint: Option<String>,
    pub source_stable_id: Option<String>,
    pub source_identity_coverage: EvidenceIdentityCoverage,
    pub content_fingerprint: Option<String>,
    pub content_identity_coverage: EvidenceIdentityCoverage,
    pub parse_error_present: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct RateLimitStoreInput {
    pub observed_at: String,
    pub limit_id: Option<String>,
    pub primary_used_percent: Option<String>,
    pub primary_window_minutes: Option<i64>,
    pub primary_resets_at: Option<String>,
    pub secondary_used_percent: Option<String>,
    pub secondary_window_minutes: Option<i64>,
    pub secondary_resets_at: Option<String>,
    pub rate_limit_reached_type: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceVersionStoreInput {
    pub source_stable_id: String,
    pub boundary: SourceVersionBoundary,
    pub version_fingerprint: String,
    pub snapshot_observed_at: Option<String>,
    pub bytes: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TaskInputStoreInput {
    pub coverage: EvidenceIdentityCoverage,
    pub task_fingerprint: Option<String>,
    pub bytes: Option<u64>,
}

impl Default for TaskInputStoreInput {
    fn default() -> Self {
        Self {
            coverage: EvidenceIdentityCoverage::NotCaptured,
            task_fingerprint: None,
            bytes: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkspaceGitProvenanceStoreInput {
    pub boundary: SourceVersionBoundary,
    pub coverage: WorkspaceGitProvenanceCoverage,
    pub head_fingerprint: Option<String>,
    pub dirty: Option<bool>,
    pub snapshot_observed_at: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MeasurementStoreSummary {
    pub run_id: String,
    pub project_id: String,
    pub workspace_id: String,
    pub profile_id: Option<String>,
    pub source_id: String,
    pub agent_count: u64,
    pub tool_activity_count: u64,
    pub rate_limit_snapshot_count: u64,
    pub source_version_count: u64,
    pub task_input_count: u64,
    pub workspace_git_provenance_count: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MeasurementStoreCounts {
    pub schema_version: Option<String>,
    pub projects: i64,
    pub workspaces: i64,
    pub runs: i64,
    pub agents: i64,
    pub runtime_profiles: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MeasurementRunHistoryRecord {
    pub run_id: String,
    pub project_id: String,
    pub workspace_id: String,
    pub profile_id: Option<String>,
    pub measurement_contract_version: i64,
    pub evidence_layout_version: i64,
    pub root_thread_id: String,
    pub root_terminal: String,
    pub source_health: String,
    pub validity_verdict: Option<String>,
    pub quality_status: Option<String>,
    pub agent_count: i64,
    pub usage_records: i64,
    pub input_tokens: i64,
    pub cached_input_tokens: i64,
    pub cache_write_input_tokens: i64,
    pub output_tokens: i64,
    pub reasoning_output_tokens: i64,
    pub logical_tokens: Option<i64>,
    pub created_at_unix: i64,
}

pub fn private_id(prefix: &str, material: &str) -> String {
    let digest = blake3::hash(material.as_bytes()).to_hex().to_string();
    format!("{prefix}-{}", &digest[..PRIVATE_ID_HEX_LEN])
}

pub fn fingerprint_bytes(bytes: &[u8]) -> String {
    blake3::hash(bytes).to_hex().to_string()
}

pub fn scoped_fingerprint_bytes(
    prefix: &str,
    scope_key: &str,
    domain: &str,
    bytes: &[u8],
) -> String {
    let key = blake3::derive_key(
        "tokn.project-scoped-content-fingerprint.v1",
        scope_key.as_bytes(),
    );
    let mut hasher = blake3::Hasher::new_keyed(&key);
    hasher.update(domain.as_bytes());
    hasher.update(&[0]);
    hasher.update(bytes);
    format!("{prefix}-v1-{}", hasher.finalize().to_hex())
}

pub fn scoped_task_input_bytes(scope_key: &str, bytes: &[u8]) -> String {
    let key = blake3::derive_key(
        "tokn.project-scoped-task-input-identity.v1",
        scope_key.as_bytes(),
    );
    let mut hasher = blake3::Hasher::new_keyed(&key);
    hasher.update(b"task-input-artifact-bytes-v1");
    hasher.update(&[0]);
    hasher.update(bytes);
    format!("tsk-v1-{}", hasher.finalize().to_hex())
}

pub fn scoped_source_version_bytes(scope_key: &str, snapshot_sha256: &[u8]) -> String {
    let key = blake3::derive_key(
        "tokn.project-scoped-source-version.v1",
        scope_key.as_bytes(),
    );
    let mut hasher = blake3::Hasher::new_keyed(&key);
    hasher.update(b"workspace-snapshot-sha256-v1");
    hasher.update(&[0]);
    hasher.update(snapshot_sha256);
    format!("ver-v1-{}", hasher.finalize().to_hex())
}

pub fn scoped_git_head_bytes(scope_key: &str, git_head: &[u8]) -> String {
    let key = blake3::derive_key("tokn.project-scoped-git-head.v1", scope_key.as_bytes());
    let mut hasher = blake3::Hasher::new_keyed(&key);
    hasher.update(b"git-head-commit-v1");
    hasher.update(&[0]);
    hasher.update(git_head);
    format!("git-v1-{}", hasher.finalize().to_hex())
}

impl Database {
    pub fn save_measurement(
        &self,
        input: &MeasurementStoreInput,
    ) -> Result<MeasurementStoreSummary, MeasurementStoreError> {
        validate_input(input)?;

        let now = now_unix();
        let profile_json = input
            .profile
            .as_ref()
            .map(serde_json::to_string)
            .transpose()?;
        let profile_id = profile_json.as_deref().map(|json| private_id("rtp", json));
        let source_id = private_id(
            "src",
            &format!("{}:{}", input.source_kind, input.source_fingerprint),
        );

        let conn = self.connection();
        let tx = conn.unchecked_transaction()?;

        tx.execute(
            "INSERT INTO projects_v2(project_id, identity_version, created_at_unix)
             VALUES (?1, 1, ?2)
             ON CONFLICT(project_id) DO NOTHING",
            params![input.project_id, now],
        )?;

        tx.execute(
            "INSERT INTO workspaces_v2(
                workspace_id, project_id, parent_workspace_id, snapshot_fingerprint,
                identity_version, created_at_unix
             ) VALUES (?1, ?2, ?3, ?4, 1, ?5)
             ON CONFLICT(workspace_id) DO UPDATE SET
                project_id=excluded.project_id,
                parent_workspace_id=excluded.parent_workspace_id,
                snapshot_fingerprint=excluded.snapshot_fingerprint",
            params![
                input.workspace_id,
                input.project_id,
                input.parent_workspace_id,
                input.workspace_snapshot_fingerprint,
                now
            ],
        )?;

        if let (Some(profile), Some(profile_json), Some(profile_id)) = (
            input.profile.as_ref(),
            profile_json.as_deref(),
            profile_id.as_deref(),
        ) {
            tx.execute(
                "INSERT INTO runtime_profiles_v2(
                    profile_id, schema_version, observed_at, runtime_kind, runtime_version,
                    app_version, model, model_provider, configuration_complete, profile_json,
                    created_at_unix
                 ) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11)
                 ON CONFLICT(profile_id) DO NOTHING",
                params![
                    profile_id,
                    to_i64(profile.schema_version)?,
                    profile.observed_at,
                    profile.runtime_kind,
                    profile.runtime_version,
                    profile.app_version,
                    profile.model,
                    profile.model_provider,
                    profile.configuration_complete.as_str(),
                    profile_json,
                    now
                ],
            )?;
        }

        let totals = &input.group.totals;
        let logical_tokens = totals.logical_total().map(to_i64).transpose()?;
        let validity_verdict = input
            .result
            .validity_verdict
            .map(|value| value.as_str().to_string());
        let quality_status = input
            .result
            .quality_status
            .map(|value: RunnerQualityStatus| value.as_str().to_string());

        tx.execute(
            "INSERT INTO measurement_runs_v2(
                run_id, project_id, workspace_id, profile_id,
                measurement_contract_version, evidence_layout_version,
                root_thread_id, root_terminal, source_health, validity_verdict, quality_status,
                agent_count, usage_records, input_tokens, cached_input_tokens,
                cache_write_input_tokens, output_tokens, reasoning_output_tokens,
                input_known, cached_input_known, cache_write_input_known, output_known,
                reasoning_output_known, logical_tokens, created_at_unix
             ) VALUES (
                ?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16,?17,
                ?18,?19,?20,?21,?22,?23,?24,?25
             )
             ON CONFLICT(run_id) DO UPDATE SET
                project_id=excluded.project_id,
                workspace_id=excluded.workspace_id,
                profile_id=excluded.profile_id,
                measurement_contract_version=excluded.measurement_contract_version,
                evidence_layout_version=excluded.evidence_layout_version,
                root_thread_id=excluded.root_thread_id,
                root_terminal=excluded.root_terminal,
                source_health=excluded.source_health,
                validity_verdict=excluded.validity_verdict,
                quality_status=excluded.quality_status,
                agent_count=excluded.agent_count,
                usage_records=excluded.usage_records,
                input_tokens=excluded.input_tokens,
                cached_input_tokens=excluded.cached_input_tokens,
                cache_write_input_tokens=excluded.cache_write_input_tokens,
                output_tokens=excluded.output_tokens,
                reasoning_output_tokens=excluded.reasoning_output_tokens,
                input_known=excluded.input_known,
                cached_input_known=excluded.cached_input_known,
                cache_write_input_known=excluded.cache_write_input_known,
                output_known=excluded.output_known,
                reasoning_output_known=excluded.reasoning_output_known,
                logical_tokens=excluded.logical_tokens",
            params![
                input.result.run_id,
                input.project_id,
                input.workspace_id,
                profile_id,
                to_i64(input.contract.measurement_contract_version)?,
                to_i64(input.contract.evidence_layout_version)?,
                input.group.root_thread_id,
                input.group.root_terminal.as_str(),
                input.result.source_health_status.as_str(),
                validity_verdict,
                quality_status,
                to_i64(input.group.agents.len() as u64)?,
                to_i64(totals.usage_records)?,
                to_i64(totals.input_tokens)?,
                to_i64(totals.cached_input_tokens)?,
                to_i64(totals.cache_write_input_tokens)?,
                to_i64(totals.output_tokens)?,
                to_i64(totals.reasoning_output_tokens)?,
                to_i64(totals.input_known)?,
                to_i64(totals.cached_input_known)?,
                to_i64(totals.cache_write_input_known)?,
                to_i64(totals.output_known)?,
                to_i64(totals.reasoning_output_known)?,
                logical_tokens,
                now
            ],
        )?;

        tx.execute(
            "DELETE FROM tool_activity_v1 WHERE run_id = ?1",
            params![input.result.run_id],
        )?;
        tx.execute(
            "DELETE FROM rate_limit_snapshots_v2 WHERE run_id = ?1",
            params![input.result.run_id],
        )?;
        tx.execute(
            "DELETE FROM source_versions_v1 WHERE run_id = ?1",
            params![input.result.run_id],
        )?;
        tx.execute(
            "DELETE FROM workspace_git_provenance_v1 WHERE run_id = ?1",
            params![input.result.run_id],
        )?;
        tx.execute(
            "DELETE FROM task_input_identity_v1 WHERE run_id = ?1",
            params![input.result.run_id],
        )?;
        tx.execute(
            "DELETE FROM agents_v2 WHERE run_id = ?1",
            params![input.result.run_id],
        )?;
        for agent in &input.group.agents {
            let totals = &agent.totals;
            tx.execute(
                "INSERT INTO agents_v2(
                    run_id, thread_id, parent_thread_id, depth, terminal_status, duration_ms,
                    usage_records, input_tokens, cached_input_tokens, cache_write_input_tokens,
                    output_tokens, reasoning_output_tokens, input_known, cached_input_known,
                    cache_write_input_known, output_known, reasoning_output_known, logical_tokens
                 ) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16,?17,?18)",
                params![
                    input.result.run_id,
                    agent.thread_id,
                    agent.parent_thread_id,
                    to_i64(u64::from(agent.depth))?,
                    agent.terminal.status.as_str(),
                    agent.terminal.duration_ms.map(to_i64).transpose()?,
                    to_i64(totals.usage_records)?,
                    to_i64(totals.input_tokens)?,
                    to_i64(totals.cached_input_tokens)?,
                    to_i64(totals.cache_write_input_tokens)?,
                    to_i64(totals.output_tokens)?,
                    to_i64(totals.reasoning_output_tokens)?,
                    to_i64(totals.input_known)?,
                    to_i64(totals.cached_input_known)?,
                    to_i64(totals.cache_write_input_known)?,
                    to_i64(totals.output_known)?,
                    to_i64(totals.reasoning_output_known)?,
                    totals.logical_total().map(to_i64).transpose()?
                ],
            )?;
        }

        for activity in &input.tool_activities {
            let activity_id = private_id(
                "act",
                &format!(
                    "{}:{}:{}:{}",
                    input.result.run_id,
                    activity.thread_id,
                    activity.agent_ordinal,
                    activity.source_call_key
                ),
            );
            tx.execute(
                "INSERT INTO tool_activity_v1(
                    activity_id, run_id, thread_id, agent_ordinal, kind, tool_name,
                    category, surface, requester_type, status, started_seq, ended_seq,
                    invocation_payload_bytes, result_payload_bytes, result_output_chars,
                    max_output_tokens, original_token_count, operation_fingerprint,
                    workdir_fingerprint, source_stable_id, source_identity_coverage,
                    content_fingerprint, content_identity_coverage,
                    parse_error_present, created_at_unix, observed_at
                 ) VALUES (
                    ?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16,
                    ?17,?18,?19,?20,?21,?22,?23,?24,?25,?26
                 )",
                params![
                    activity_id,
                    input.result.run_id,
                    activity.thread_id,
                    to_i64(activity.agent_ordinal)?,
                    activity.kind,
                    activity.tool_name,
                    activity.category,
                    activity.surface,
                    activity.requester_type,
                    activity.status,
                    activity.started_seq.map(to_i64).transpose()?,
                    activity.ended_seq.map(to_i64).transpose()?,
                    activity.invocation_payload_bytes.map(to_i64).transpose()?,
                    activity.result_payload_bytes.map(to_i64).transpose()?,
                    activity.result_output_chars.map(to_i64).transpose()?,
                    activity.max_output_tokens.map(to_i64).transpose()?,
                    activity.original_token_count.map(to_i64).transpose()?,
                    activity.operation_fingerprint,
                    activity.workdir_fingerprint,
                    activity.source_stable_id,
                    activity.source_identity_coverage.as_str(),
                    activity.content_fingerprint,
                    activity.content_identity_coverage.as_str(),
                    if activity.parse_error_present {
                        1_i64
                    } else {
                        0_i64
                    },
                    now,
                    activity.observed_at
                ],
            )?;
        }

        for snapshot in &input.rate_limit_snapshots {
            let snapshot_id = private_id(
                "rls",
                &format!(
                    "{}|{}|{:?}|{:?}|{:?}|{:?}|{:?}|{:?}|{:?}|{:?}",
                    input.result.run_id,
                    snapshot.observed_at,
                    snapshot.limit_id,
                    snapshot.primary_used_percent,
                    snapshot.primary_window_minutes,
                    snapshot.primary_resets_at,
                    snapshot.secondary_used_percent,
                    snapshot.secondary_window_minutes,
                    snapshot.secondary_resets_at,
                    snapshot.rate_limit_reached_type
                ),
            );
            tx.execute(
                "INSERT INTO rate_limit_snapshots_v2(
                    snapshot_id, run_id, observed_at, limit_id,
                    primary_used_percent, primary_window_minutes, primary_resets_at,
                    secondary_used_percent, secondary_window_minutes, secondary_resets_at,
                    rate_limit_reached_type, created_at_unix
                 ) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12)",
                params![
                    snapshot_id,
                    input.result.run_id,
                    snapshot.observed_at,
                    snapshot.limit_id,
                    snapshot.primary_used_percent,
                    snapshot.primary_window_minutes,
                    snapshot.primary_resets_at,
                    snapshot.secondary_used_percent,
                    snapshot.secondary_window_minutes,
                    snapshot.secondary_resets_at,
                    snapshot.rate_limit_reached_type,
                    now
                ],
            )?;
        }

        for version in &input.source_versions {
            let version_id = private_id(
                "svr",
                &format!(
                    "{}|{}|{}|{}",
                    input.result.run_id,
                    version.source_stable_id,
                    version.boundary.as_str(),
                    version.version_fingerprint
                ),
            );
            tx.execute(
                "INSERT INTO source_versions_v1(
                    version_id, run_id, source_stable_id, boundary, version_fingerprint,
                    snapshot_observed_at, bytes, created_at_unix
                 ) VALUES (?1,?2,?3,?4,?5,?6,?7,?8)",
                params![
                    version_id,
                    input.result.run_id,
                    version.source_stable_id,
                    version.boundary.as_str(),
                    version.version_fingerprint,
                    version.snapshot_observed_at,
                    to_i64(version.bytes)?,
                    now
                ],
            )?;
        }

        for provenance in &input.workspace_git_provenance {
            let provenance_id = private_id(
                "wgp",
                &format!(
                    "{}|{}|{}|{:?}|{:?}",
                    input.result.run_id,
                    provenance.boundary.as_str(),
                    provenance.coverage.as_str(),
                    provenance.head_fingerprint,
                    provenance.dirty
                ),
            );
            tx.execute(
                "INSERT INTO workspace_git_provenance_v1(
                    provenance_id, run_id, boundary, coverage, head_fingerprint,
                    dirty, snapshot_observed_at, created_at_unix
                 ) VALUES (?1,?2,?3,?4,?5,?6,?7,?8)",
                params![
                    provenance_id,
                    input.result.run_id,
                    provenance.boundary.as_str(),
                    provenance.coverage.as_str(),
                    provenance.head_fingerprint,
                    provenance.dirty.map(i64::from),
                    provenance.snapshot_observed_at,
                    now
                ],
            )?;
        }

        let task_input_id = private_id(
            "tskobs",
            &format!(
                "{}|{}|{:?}|{:?}",
                input.result.run_id,
                input.task_input.coverage.as_str(),
                input.task_input.task_fingerprint,
                input.task_input.bytes
            ),
        );
        tx.execute(
            "INSERT INTO task_input_identity_v1(
                task_input_id, run_id, coverage, task_fingerprint, bytes, created_at_unix
             ) VALUES (?1,?2,?3,?4,?5,?6)",
            params![
                task_input_id,
                input.result.run_id,
                input.task_input.coverage.as_str(),
                input.task_input.task_fingerprint,
                input.task_input.bytes.map(to_i64).transpose()?,
                now
            ],
        )?;

        tx.execute(
            "INSERT INTO provenance_v2(
                source_id, run_id, source_kind, source_fingerprint, snapshot_bytes,
                adapter_name, adapter_version, created_at_unix
             ) VALUES (?1,?2,?3,?4,?5,?6,?7,?8)
             ON CONFLICT(source_id) DO UPDATE SET
                run_id=excluded.run_id,
                source_kind=excluded.source_kind,
                source_fingerprint=excluded.source_fingerprint,
                snapshot_bytes=excluded.snapshot_bytes,
                adapter_name=excluded.adapter_name,
                adapter_version=excluded.adapter_version",
            params![
                source_id,
                input.result.run_id,
                input.source_kind,
                input.source_fingerprint,
                input.source_snapshot_bytes.map(to_i64).transpose()?,
                input.adapter_name,
                input.adapter_version,
                now
            ],
        )?;

        tx.commit()?;

        Ok(MeasurementStoreSummary {
            run_id: input.result.run_id.clone(),
            project_id: input.project_id.clone(),
            workspace_id: input.workspace_id.clone(),
            profile_id,
            source_id,
            agent_count: input.group.agents.len() as u64,
            tool_activity_count: input.tool_activities.len() as u64,
            rate_limit_snapshot_count: input.rate_limit_snapshots.len() as u64,
            source_version_count: input.source_versions.len() as u64,
            task_input_count: 1,
            workspace_git_provenance_count: input.workspace_git_provenance.len() as u64,
        })
    }

    pub fn measurement_run_count(&self) -> rusqlite::Result<i64> {
        self.connection()
            .query_row("SELECT COUNT(*) FROM measurement_runs_v2", [], |row| {
                row.get(0)
            })
    }

    pub fn measurement_agent_count(&self) -> rusqlite::Result<i64> {
        self.connection()
            .query_row("SELECT COUNT(*) FROM agents_v2", [], |row| row.get(0))
    }

    pub fn runtime_profile_count(&self) -> rusqlite::Result<i64> {
        self.connection()
            .query_row("SELECT COUNT(*) FROM runtime_profiles_v2", [], |row| {
                row.get(0)
            })
    }

    pub fn project_count_v2(&self) -> rusqlite::Result<i64> {
        self.connection()
            .query_row("SELECT COUNT(*) FROM projects_v2", [], |row| row.get(0))
    }

    pub fn workspace_count_v2(&self) -> rusqlite::Result<i64> {
        self.connection()
            .query_row("SELECT COUNT(*) FROM workspaces_v2", [], |row| row.get(0))
    }

    pub fn measurement_store_counts(&self) -> rusqlite::Result<MeasurementStoreCounts> {
        Ok(MeasurementStoreCounts {
            schema_version: self.measurement_store_schema_version()?,
            projects: self.project_count_v2()?,
            workspaces: self.workspace_count_v2()?,
            runs: self.measurement_run_count()?,
            agents: self.measurement_agent_count()?,
            runtime_profiles: self.runtime_profile_count()?,
        })
    }

    pub fn tool_activity_schema_version(&self) -> rusqlite::Result<Option<String>> {
        self.connection()
            .query_row(
                "SELECT value FROM schema_meta WHERE key='tool_activity_schema_version'",
                [],
                |row| row.get(0),
            )
            .optional()
    }

    pub fn tool_activity_count(&self) -> rusqlite::Result<i64> {
        self.connection()
            .query_row("SELECT COUNT(*) FROM tool_activity_v1", [], |row| {
                row.get(0)
            })
    }

    pub fn rate_limit_snapshot_count(&self) -> rusqlite::Result<i64> {
        self.connection()
            .query_row("SELECT COUNT(*) FROM rate_limit_snapshots_v2", [], |row| {
                row.get(0)
            })
    }

    pub fn source_version_schema_version(&self) -> rusqlite::Result<Option<String>> {
        self.connection()
            .query_row(
                "SELECT value FROM schema_meta WHERE key='source_version_schema_version'",
                [],
                |row| row.get(0),
            )
            .optional()
    }

    pub fn source_version_count(&self) -> rusqlite::Result<i64> {
        self.connection()
            .query_row("SELECT COUNT(*) FROM source_versions_v1", [], |row| {
                row.get(0)
            })
    }

    pub fn source_version_history(
        &self,
        project_filter: Option<&str>,
        workspace_filter: Option<&str>,
        run_limit: usize,
    ) -> Result<SourceVersionHistory, MeasurementStoreError> {
        let mut history = SourceVersionHistory {
            schema_version: SOURCE_VERSION_HISTORY_SCHEMA_VERSION,
            project_filter: project_filter.map(str::to_string),
            workspace_filter: workspace_filter.map(str::to_string),
            run_limit: u64::try_from(run_limit).unwrap_or(u64::MAX),
            ..Default::default()
        };
        if run_limit == 0 {
            return Ok(history);
        }

        let mut stmt = self.connection().prepare(
            "WITH selected_runs AS (
                SELECT run_id, project_id, workspace_id, created_at_unix
                FROM measurement_runs_v2
                WHERE (?1 IS NULL OR project_id = ?1)
                  AND (?2 IS NULL OR workspace_id = ?2)
                ORDER BY created_at_unix DESC, run_id ASC
                LIMIT ?3
             )
             SELECT version.version_id, version.run_id, selected.project_id,
                selected.workspace_id, version.source_stable_id, version.boundary,
                version.version_fingerprint, version.snapshot_observed_at, version.bytes,
                selected.created_at_unix
             FROM source_versions_v1 AS version
             INNER JOIN selected_runs AS selected ON selected.run_id = version.run_id
             ORDER BY selected.created_at_unix DESC, version.run_id ASC,
                version.source_stable_id ASC, version.boundary ASC",
        )?;

        history.versions = stmt
            .query_map(
                params![
                    project_filter,
                    workspace_filter,
                    i64::try_from(run_limit).unwrap_or(i64::MAX)
                ],
                |row| {
                    Ok(HistoricalSourceVersionRecord {
                        version_id: row.get(0)?,
                        run_id: row.get(1)?,
                        project_id: row.get(2)?,
                        workspace_id: row.get(3)?,
                        source_stable_id: row.get(4)?,
                        boundary: parse_source_version_boundary(row.get::<_, String>(5)?)?,
                        version_fingerprint: row.get(6)?,
                        snapshot_observed_at: row.get(7)?,
                        bytes: row_u64(row, 8)?,
                        run_created_at_unix: row.get(9)?,
                    })
                },
            )?
            .collect::<rusqlite::Result<Vec<_>>>()?;

        Ok(history)
    }

    pub fn task_input_identity_schema_version(&self) -> rusqlite::Result<Option<String>> {
        self.connection()
            .query_row(
                "SELECT value FROM schema_meta WHERE key='task_input_identity_schema_version'",
                [],
                |row| row.get(0),
            )
            .optional()
    }

    pub fn task_input_count(&self) -> rusqlite::Result<i64> {
        self.connection()
            .query_row("SELECT COUNT(*) FROM task_input_identity_v1", [], |row| {
                row.get(0)
            })
    }

    pub fn task_input_history(
        &self,
        project_filter: Option<&str>,
        workspace_filter: Option<&str>,
        run_limit: usize,
    ) -> Result<TaskInputHistory, MeasurementStoreError> {
        let mut history = TaskInputHistory {
            schema_version: TASK_INPUT_HISTORY_SCHEMA_VERSION,
            project_filter: project_filter.map(str::to_string),
            workspace_filter: workspace_filter.map(str::to_string),
            run_limit: u64::try_from(run_limit).unwrap_or(u64::MAX),
            ..Default::default()
        };
        if run_limit == 0 {
            return Ok(history);
        }

        let mut stmt = self.connection().prepare(
            "WITH selected_runs AS (
                SELECT run_id, project_id, workspace_id, created_at_unix
                FROM measurement_runs_v2
                WHERE (?1 IS NULL OR project_id = ?1)
                  AND (?2 IS NULL OR workspace_id = ?2)
                ORDER BY created_at_unix DESC, run_id ASC
                LIMIT ?3
             )
             SELECT task.task_input_id, task.run_id, selected.project_id,
                selected.workspace_id, task.coverage, task.task_fingerprint, task.bytes,
                selected.created_at_unix
             FROM task_input_identity_v1 AS task
             INNER JOIN selected_runs AS selected ON selected.run_id = task.run_id
             ORDER BY selected.created_at_unix DESC, task.run_id ASC",
        )?;

        history.observations = stmt
            .query_map(
                params![
                    project_filter,
                    workspace_filter,
                    i64::try_from(run_limit).unwrap_or(i64::MAX)
                ],
                |row| {
                    Ok(HistoricalTaskInputRecord {
                        task_input_id: row.get(0)?,
                        run_id: row.get(1)?,
                        project_id: row.get(2)?,
                        workspace_id: row.get(3)?,
                        coverage: parse_identity_coverage(row.get::<_, String>(4)?)?,
                        task_fingerprint: row.get(5)?,
                        bytes: row_optional_u64(row, 6)?,
                        delivery_status: TaskInputDeliveryStatus::NotProven,
                        run_created_at_unix: row.get(7)?,
                    })
                },
            )?
            .collect::<rusqlite::Result<Vec<_>>>()?;

        Ok(history)
    }

    pub fn workspace_git_provenance_schema_version(&self) -> rusqlite::Result<Option<String>> {
        self.connection()
            .query_row(
                "SELECT value FROM schema_meta WHERE key='workspace_git_provenance_schema_version'",
                [],
                |row| row.get(0),
            )
            .optional()
    }

    pub fn workspace_git_provenance_count(&self) -> rusqlite::Result<i64> {
        self.connection().query_row(
            "SELECT COUNT(*) FROM workspace_git_provenance_v1",
            [],
            |row| row.get(0),
        )
    }

    pub fn workspace_git_provenance_history(
        &self,
        project_filter: Option<&str>,
        workspace_filter: Option<&str>,
        run_limit: usize,
    ) -> Result<WorkspaceGitProvenanceHistory, MeasurementStoreError> {
        let mut history = WorkspaceGitProvenanceHistory {
            schema_version: WORKSPACE_GIT_PROVENANCE_HISTORY_SCHEMA_VERSION,
            project_filter: project_filter.map(str::to_string),
            workspace_filter: workspace_filter.map(str::to_string),
            run_limit: u64::try_from(run_limit).unwrap_or(u64::MAX),
            ..Default::default()
        };
        if run_limit == 0 {
            return Ok(history);
        }

        let mut stmt = self.connection().prepare(
            "WITH selected_runs AS (
                SELECT run_id, project_id, workspace_id, created_at_unix
                FROM measurement_runs_v2
                WHERE (?1 IS NULL OR project_id = ?1)
                  AND (?2 IS NULL OR workspace_id = ?2)
                ORDER BY created_at_unix DESC, run_id ASC
                LIMIT ?3
             )
             SELECT provenance.provenance_id, provenance.run_id, selected.project_id,
                selected.workspace_id, provenance.boundary, provenance.coverage,
                provenance.head_fingerprint, provenance.dirty,
                provenance.snapshot_observed_at, selected.created_at_unix
             FROM workspace_git_provenance_v1 AS provenance
             INNER JOIN selected_runs AS selected ON selected.run_id = provenance.run_id
             ORDER BY selected.created_at_unix DESC, provenance.run_id ASC,
                provenance.boundary ASC",
        )?;

        history.observations = stmt
            .query_map(
                params![
                    project_filter,
                    workspace_filter,
                    i64::try_from(run_limit).unwrap_or(i64::MAX)
                ],
                |row| {
                    Ok(HistoricalWorkspaceGitProvenanceRecord {
                        provenance_id: row.get(0)?,
                        run_id: row.get(1)?,
                        project_id: row.get(2)?,
                        workspace_id: row.get(3)?,
                        boundary: parse_source_version_boundary(row.get::<_, String>(4)?)?,
                        coverage: parse_workspace_git_provenance_coverage(
                            row.get::<_, String>(5)?,
                        )?,
                        head_fingerprint: row.get(6)?,
                        dirty: row.get::<_, Option<i64>>(7)?.map(|value| value != 0),
                        snapshot_observed_at: row.get(8)?,
                        run_created_at_unix: row.get(9)?,
                    })
                },
            )?
            .collect::<rusqlite::Result<Vec<_>>>()?;

        Ok(history)
    }

    pub fn rate_limit_history(
        &self,
        project_filter: Option<&str>,
        workspace_filter: Option<&str>,
        run_limit: usize,
    ) -> Result<RateLimitHistory, MeasurementStoreError> {
        let mut history = RateLimitHistory {
            schema_version: RATE_LIMIT_HISTORY_SCHEMA_VERSION,
            project_filter: project_filter.map(str::to_string),
            workspace_filter: workspace_filter.map(str::to_string),
            run_limit: u64::try_from(run_limit).unwrap_or(u64::MAX),
            ..Default::default()
        };
        if run_limit == 0 {
            return Ok(history);
        }

        let mut stmt = self.connection().prepare(
            "WITH selected_runs AS (
                SELECT run_id, project_id, workspace_id, created_at_unix
                FROM measurement_runs_v2
                WHERE (?1 IS NULL OR project_id = ?1)
                  AND (?2 IS NULL OR workspace_id = ?2)
                ORDER BY created_at_unix DESC, run_id ASC
                LIMIT ?3
             )
             SELECT
                snapshot.snapshot_id, snapshot.run_id, selected.project_id,
                selected.workspace_id, snapshot.observed_at, snapshot.limit_id,
                snapshot.primary_used_percent, snapshot.primary_window_minutes,
                snapshot.primary_resets_at, snapshot.secondary_used_percent,
                snapshot.secondary_window_minutes, snapshot.secondary_resets_at,
                snapshot.rate_limit_reached_type, snapshot.created_at_unix
             FROM rate_limit_snapshots_v2 AS snapshot
             INNER JOIN selected_runs AS selected ON selected.run_id = snapshot.run_id
             ORDER BY selected.created_at_unix DESC, snapshot.observed_at ASC,
                snapshot.limit_id ASC, snapshot.snapshot_id ASC",
        )?;

        history.snapshots = stmt
            .query_map(
                params![
                    project_filter,
                    workspace_filter,
                    i64::try_from(run_limit).unwrap_or(i64::MAX)
                ],
                |row| {
                    Ok(HistoricalRateLimitSnapshotRecord {
                        snapshot_id: row.get(0)?,
                        run_id: row.get(1)?,
                        project_id: row.get(2)?,
                        workspace_id: row.get(3)?,
                        observed_at: row.get(4)?,
                        limit_id: row.get(5)?,
                        primary_used_percent: row.get(6)?,
                        primary_window_minutes: row.get(7)?,
                        primary_resets_at: row.get(8)?,
                        secondary_used_percent: row.get(9)?,
                        secondary_window_minutes: row.get(10)?,
                        secondary_resets_at: row.get(11)?,
                        rate_limit_reached_type: row.get(12)?,
                        created_at_unix: row.get(13)?,
                    })
                },
            )?
            .collect::<rusqlite::Result<Vec<_>>>()?;

        Ok(history)
    }

    pub fn recent_measurement_runs(
        &self,
        limit: usize,
    ) -> rusqlite::Result<Vec<MeasurementRunHistoryRecord>> {
        if limit == 0 {
            return Ok(Vec::new());
        }

        let mut stmt = self.connection().prepare(
            "SELECT
                run_id, project_id, workspace_id, profile_id,
                measurement_contract_version, evidence_layout_version,
                root_thread_id, root_terminal, source_health,
                validity_verdict, quality_status,
                agent_count, usage_records, input_tokens, cached_input_tokens,
                cache_write_input_tokens, output_tokens, reasoning_output_tokens,
                logical_tokens, created_at_unix
             FROM measurement_runs_v2
             ORDER BY created_at_unix DESC, run_id ASC
             LIMIT ?1",
        )?;

        stmt.query_map(params![i64::try_from(limit).unwrap_or(i64::MAX)], |row| {
            Ok(MeasurementRunHistoryRecord {
                run_id: row.get(0)?,
                project_id: row.get(1)?,
                workspace_id: row.get(2)?,
                profile_id: row.get(3)?,
                measurement_contract_version: row.get(4)?,
                evidence_layout_version: row.get(5)?,
                root_thread_id: row.get(6)?,
                root_terminal: row.get(7)?,
                source_health: row.get(8)?,
                validity_verdict: row.get(9)?,
                quality_status: row.get(10)?,
                agent_count: row.get(11)?,
                usage_records: row.get(12)?,
                input_tokens: row.get(13)?,
                cached_input_tokens: row.get(14)?,
                cache_write_input_tokens: row.get(15)?,
                output_tokens: row.get(16)?,
                reasoning_output_tokens: row.get(17)?,
                logical_tokens: row.get(18)?,
                created_at_unix: row.get(19)?,
            })
        })?
        .collect()
    }

    pub fn tool_activity_history(
        &self,
        project_filter: Option<&str>,
        workspace_filter: Option<&str>,
        run_limit: usize,
    ) -> Result<ToolActivityHistory, MeasurementStoreError> {
        let mut history = ToolActivityHistory {
            schema_version: TOOL_ACTIVITY_HISTORY_SCHEMA_VERSION,
            project_filter: project_filter.map(str::to_string),
            workspace_filter: workspace_filter.map(str::to_string),
            run_limit: u64::try_from(run_limit).unwrap_or(u64::MAX),
            ..Default::default()
        };
        if run_limit == 0 {
            return Ok(history);
        }

        let mut stmt = self.connection().prepare(
            "WITH selected_runs AS (
                SELECT run_id, project_id, workspace_id, created_at_unix
                FROM measurement_runs_v2
                WHERE (?1 IS NULL OR project_id = ?1)
                  AND (?2 IS NULL OR workspace_id = ?2)
                ORDER BY created_at_unix DESC, run_id ASC
                LIMIT ?3
             )
             SELECT
                activity.activity_id, activity.run_id,
                selected.project_id, selected.workspace_id,
                activity.thread_id, activity.agent_ordinal,
                activity.kind, activity.tool_name, activity.category, activity.surface,
                activity.requester_type, activity.status,
                activity.started_seq, activity.ended_seq,
                activity.invocation_payload_bytes, activity.result_payload_bytes,
                activity.result_output_chars, activity.max_output_tokens,
                activity.original_token_count, activity.operation_fingerprint,
                activity.workdir_fingerprint, activity.source_stable_id,
                activity.source_identity_coverage, activity.content_fingerprint,
                activity.content_identity_coverage, activity.parse_error_present,
                selected.created_at_unix, activity.observed_at
             FROM tool_activity_v1 AS activity
             INNER JOIN selected_runs AS selected ON selected.run_id = activity.run_id
             ORDER BY
                selected.created_at_unix DESC,
                activity.run_id ASC,
                activity.thread_id ASC,
                activity.agent_ordinal ASC",
        )?;

        history.activities = stmt
            .query_map(
                params![
                    project_filter,
                    workspace_filter,
                    i64::try_from(run_limit).unwrap_or(i64::MAX)
                ],
                |row| {
                    Ok(HistoricalToolActivityRecord {
                        activity_id: row.get(0)?,
                        run_id: row.get(1)?,
                        project_id: row.get(2)?,
                        workspace_id: row.get(3)?,
                        thread_id: row.get(4)?,
                        agent_ordinal: row_u64(row, 5)?,
                        kind: row.get(6)?,
                        tool_name: row.get(7)?,
                        category: row.get(8)?,
                        surface: row.get(9)?,
                        requester_type: row.get(10)?,
                        status: row.get(11)?,
                        started_seq: row_optional_u64(row, 12)?,
                        ended_seq: row_optional_u64(row, 13)?,
                        invocation_payload_bytes: row_optional_u64(row, 14)?,
                        result_payload_bytes: row_optional_u64(row, 15)?,
                        result_output_chars: row_optional_u64(row, 16)?,
                        max_output_tokens: row_optional_u64(row, 17)?,
                        original_token_count: row_optional_u64(row, 18)?,
                        operation_fingerprint: row.get(19)?,
                        workdir_fingerprint: row.get(20)?,
                        source_stable_id: row.get(21)?,
                        source_identity_coverage: parse_identity_coverage(
                            row.get::<_, String>(22)?,
                        )?,
                        content_fingerprint: row.get(23)?,
                        content_identity_coverage: parse_identity_coverage(
                            row.get::<_, String>(24)?,
                        )?,
                        parse_error_present: row.get::<_, i64>(25)? != 0,
                        run_created_at_unix: row.get(26)?,
                        observed_at: row.get(27)?,
                    })
                },
            )?
            .collect::<rusqlite::Result<Vec<_>>>()?;

        Ok(history)
    }

    pub fn historical_snapshot(
        &self,
        project_filter: Option<&str>,
        workspace_filter: Option<&str>,
        limit: usize,
    ) -> Result<HistoricalSnapshot, MeasurementStoreError> {
        let mut snapshot = HistoricalSnapshot {
            schema_version: HISTORICAL_SNAPSHOT_SCHEMA_VERSION,
            project_filter: project_filter.map(str::to_string),
            workspace_filter: workspace_filter.map(str::to_string),
            ..Default::default()
        };
        if limit == 0 {
            return Ok(snapshot);
        }

        let limit = i64::try_from(limit).unwrap_or(i64::MAX);
        let mut stmt = self.connection().prepare(
            "SELECT
                run_id, project_id, workspace_id, profile_id,
                measurement_contract_version, evidence_layout_version,
                root_thread_id, root_terminal, source_health,
                validity_verdict, quality_status, agent_count,
                usage_records, input_tokens, cached_input_tokens,
                cache_write_input_tokens, output_tokens, reasoning_output_tokens,
                input_known, cached_input_known, cache_write_input_known,
                output_known, reasoning_output_known, logical_tokens, created_at_unix
             FROM measurement_runs_v2
             WHERE (?1 IS NULL OR project_id = ?1)
               AND (?2 IS NULL OR workspace_id = ?2)
             ORDER BY created_at_unix DESC, run_id ASC
             LIMIT ?3",
        )?;

        snapshot.runs = stmt
            .query_map(params![project_filter, workspace_filter, limit], |row| {
                Ok(HistoricalRunRecord {
                    run_id: row.get(0)?,
                    project_id: row.get(1)?,
                    workspace_id: row.get(2)?,
                    profile_id: row.get(3)?,
                    measurement_contract_version: row_u64(row, 4)?,
                    evidence_layout_version: row_u64(row, 5)?,
                    root_thread_id: row.get(6)?,
                    root_terminal: row.get(7)?,
                    source_health: row.get(8)?,
                    validity_verdict: row.get(9)?,
                    quality_status: row.get(10)?,
                    agent_count: row_u64(row, 11)?,
                    totals: TokenTotals {
                        usage_records: row_u64(row, 12)?,
                        input_tokens: row_u64(row, 13)?,
                        cached_input_tokens: row_u64(row, 14)?,
                        cache_write_input_tokens: row_u64(row, 15)?,
                        output_tokens: row_u64(row, 16)?,
                        reasoning_output_tokens: row_u64(row, 17)?,
                        input_known: row_u64(row, 18)?,
                        cached_input_known: row_u64(row, 19)?,
                        cache_write_input_known: row_u64(row, 20)?,
                        output_known: row_u64(row, 21)?,
                        reasoning_output_known: row_u64(row, 22)?,
                    },
                    logical_tokens: row_optional_u64(row, 23)?,
                    created_at_unix: row.get(24)?,
                })
            })?
            .collect::<rusqlite::Result<Vec<_>>>()?;

        let mut workspace_ids = BTreeSet::new();
        let mut profile_ids = BTreeSet::new();

        for run in &snapshot.runs {
            workspace_ids.insert(run.workspace_id.clone());
            if let Some(profile_id) = run.profile_id.as_ref() {
                profile_ids.insert(profile_id.clone());
            }

            snapshot
                .agents
                .extend(self.historical_agents_for_run(&run.run_id)?);
            snapshot
                .provenance
                .extend(self.historical_provenance_for_run(&run.run_id)?);
        }

        let mut pending = workspace_ids.into_iter().collect::<Vec<_>>();
        let mut seen = BTreeSet::new();
        while let Some(workspace_id) = pending.pop() {
            if !seen.insert(workspace_id.clone()) {
                continue;
            }
            if let Some(workspace) = self.historical_workspace(&workspace_id)? {
                if let Some(parent) = workspace.parent_workspace_id.as_ref() {
                    pending.push(parent.clone());
                }
                snapshot.workspaces.push(workspace);
            }
        }
        snapshot
            .workspaces
            .sort_by(|left, right| left.workspace_id.cmp(&right.workspace_id));

        for profile_id in profile_ids {
            if let Some(profile) = self.load_runtime_profile(&profile_id)? {
                snapshot
                    .runtime_profiles
                    .push(HistoricalRuntimeProfileRecord {
                        profile_id,
                        profile,
                    });
            }
        }
        snapshot
            .runtime_profiles
            .sort_by(|left, right| left.profile_id.cmp(&right.profile_id));
        snapshot.agents.sort_by(|left, right| {
            left.run_id
                .cmp(&right.run_id)
                .then(left.depth.cmp(&right.depth))
                .then(left.thread_id.cmp(&right.thread_id))
        });
        snapshot.provenance.sort_by(|left, right| {
            left.run_id
                .cmp(&right.run_id)
                .then(left.source_id.cmp(&right.source_id))
        });

        Ok(snapshot)
    }

    fn historical_agents_for_run(
        &self,
        run_id: &str,
    ) -> Result<Vec<HistoricalAgentRecord>, MeasurementStoreError> {
        let mut stmt = self.connection().prepare(
            "SELECT
                thread_id, parent_thread_id, depth, terminal_status, duration_ms,
                usage_records, input_tokens, cached_input_tokens, cache_write_input_tokens,
                output_tokens, reasoning_output_tokens, input_known, cached_input_known,
                cache_write_input_known, output_known, reasoning_output_known, logical_tokens
             FROM agents_v2
             WHERE run_id = ?1
             ORDER BY depth ASC, thread_id ASC",
        )?;

        Ok(stmt
            .query_map(params![run_id], |row| {
                Ok(HistoricalAgentRecord {
                    run_id: run_id.to_string(),
                    thread_id: row.get(0)?,
                    parent_thread_id: row.get(1)?,
                    depth: row.get(2)?,
                    terminal_status: row.get(3)?,
                    duration_ms: row_optional_u64(row, 4)?,
                    totals: TokenTotals {
                        usage_records: row_u64(row, 5)?,
                        input_tokens: row_u64(row, 6)?,
                        cached_input_tokens: row_u64(row, 7)?,
                        cache_write_input_tokens: row_u64(row, 8)?,
                        output_tokens: row_u64(row, 9)?,
                        reasoning_output_tokens: row_u64(row, 10)?,
                        input_known: row_u64(row, 11)?,
                        cached_input_known: row_u64(row, 12)?,
                        cache_write_input_known: row_u64(row, 13)?,
                        output_known: row_u64(row, 14)?,
                        reasoning_output_known: row_u64(row, 15)?,
                    },
                    logical_tokens: row_optional_u64(row, 16)?,
                })
            })?
            .collect::<rusqlite::Result<Vec<_>>>()?)
    }

    fn historical_provenance_for_run(
        &self,
        run_id: &str,
    ) -> Result<Vec<HistoricalProvenanceRecord>, MeasurementStoreError> {
        let mut stmt = self.connection().prepare(
            "SELECT
                source_id, source_kind, source_fingerprint, snapshot_bytes,
                adapter_name, adapter_version, created_at_unix
             FROM provenance_v2
             WHERE run_id = ?1
             ORDER BY source_id ASC",
        )?;

        Ok(stmt
            .query_map(params![run_id], |row| {
                Ok(HistoricalProvenanceRecord {
                    source_id: row.get(0)?,
                    run_id: run_id.to_string(),
                    source_kind: row.get(1)?,
                    source_fingerprint: row.get(2)?,
                    snapshot_bytes: row_optional_u64(row, 3)?,
                    adapter_name: row.get(4)?,
                    adapter_version: row.get(5)?,
                    created_at_unix: row.get(6)?,
                })
            })?
            .collect::<rusqlite::Result<Vec<_>>>()?)
    }

    fn historical_workspace(
        &self,
        workspace_id: &str,
    ) -> Result<Option<HistoricalWorkspaceRecord>, MeasurementStoreError> {
        Ok(self
            .connection()
            .query_row(
                "SELECT
                    workspace_id, project_id, parent_workspace_id, snapshot_fingerprint,
                    identity_version, created_at_unix
                 FROM workspaces_v2
                 WHERE workspace_id = ?1",
                params![workspace_id],
                |row| {
                    Ok(HistoricalWorkspaceRecord {
                        workspace_id: row.get(0)?,
                        project_id: row.get(1)?,
                        parent_workspace_id: row.get(2)?,
                        snapshot_fingerprint: row.get(3)?,
                        identity_version: row_u64(row, 4)?,
                        created_at_unix: row.get(5)?,
                    })
                },
            )
            .optional()?)
    }

    pub fn load_runtime_profile(
        &self,
        profile_id: &str,
    ) -> Result<Option<ModelRuntimeProfile>, MeasurementStoreError> {
        let mut stmt = self
            .connection()
            .prepare("SELECT profile_json FROM runtime_profiles_v2 WHERE profile_id = ?1")?;
        let mut rows = stmt.query(params![profile_id])?;
        let Some(row) = rows.next()? else {
            return Ok(None);
        };
        let json: String = row.get(0)?;
        Ok(Some(serde_json::from_str(&json)?))
    }

    pub fn measurement_store_schema_version(&self) -> rusqlite::Result<Option<String>> {
        let mut stmt = self.connection().prepare(
            "SELECT value FROM schema_meta WHERE key='measurement_store_schema_version'",
        )?;
        let mut rows = stmt.query([])?;
        rows.next()?.map(|row| row.get(0)).transpose()
    }
}

fn validate_input(input: &MeasurementStoreInput) -> Result<(), MeasurementStoreError> {
    validate_private_id("project_id", &input.project_id, "prj")?;
    validate_private_id("workspace_id", &input.workspace_id, "wsp")?;
    if let Some(parent) = input.parent_workspace_id.as_deref() {
        validate_private_id("parent_workspace_id", parent, "wsp")?;
    }
    if input.source_kind.trim().is_empty() {
        return Err(MeasurementStoreError::Invalid(
            "source_kind cannot be empty".into(),
        ));
    }
    if input.source_fingerprint.trim().is_empty() {
        return Err(MeasurementStoreError::Invalid(
            "source_fingerprint cannot be empty".into(),
        ));
    }
    if input.contract.contract_id != MEASUREMENT_CONTRACT_ID
        || input.contract.measurement_contract_version != MEASUREMENT_CONTRACT_VERSION
    {
        return Err(MeasurementStoreError::Invalid(format!(
            "unsupported measurement contract {} version {}",
            input.contract.contract_id, input.contract.measurement_contract_version
        )));
    }
    if input.result.run_id.trim().is_empty() {
        return Err(MeasurementStoreError::Invalid(
            "run_id cannot be empty".into(),
        ));
    }
    if input.group.root_thread_id.trim().is_empty() {
        return Err(MeasurementStoreError::Invalid(
            "root_thread_id cannot be empty".into(),
        ));
    }
    if input.result.agent_count != input.group.agents.len() as u64 {
        return Err(MeasurementStoreError::Invalid(format!(
            "agent_count mismatch: result={} group={}",
            input.result.agent_count,
            input.group.agents.len()
        )));
    }
    if input.result.root_terminal != input.group.root_terminal {
        return Err(MeasurementStoreError::Invalid(
            "root terminal mismatch between RunnerResult and RunGroup".into(),
        ));
    }
    let agent_ids = input
        .group
        .agents
        .iter()
        .map(|agent| agent.thread_id.as_str())
        .collect::<BTreeSet<_>>();
    let mut activity_slots = BTreeSet::new();
    for activity in &input.tool_activities {
        if activity.source_call_key.trim().is_empty() {
            return Err(MeasurementStoreError::Invalid(
                "tool activity source_call_key cannot be empty".into(),
            ));
        }
        if !agent_ids.contains(activity.thread_id.as_str()) {
            return Err(MeasurementStoreError::Invalid(format!(
                "tool activity thread {} is not present in RunGroup",
                activity.thread_id
            )));
        }
        if !activity_slots.insert((activity.thread_id.as_str(), activity.agent_ordinal)) {
            return Err(MeasurementStoreError::Invalid(format!(
                "duplicate tool activity ordinal {} for thread {}",
                activity.agent_ordinal, activity.thread_id
            )));
        }
        for (name, value) in [
            ("kind", activity.kind.as_str()),
            ("category", activity.category.as_str()),
            ("surface", activity.surface.as_str()),
            ("status", activity.status.as_str()),
        ] {
            if value.trim().is_empty() {
                return Err(MeasurementStoreError::Invalid(format!(
                    "tool activity {name} cannot be empty"
                )));
            }
        }
        if activity
            .observed_at
            .as_deref()
            .is_some_and(|value| value.trim().is_empty())
        {
            return Err(MeasurementStoreError::Invalid(
                "tool activity observed_at cannot be empty".into(),
            ));
        }
        if let Some(value) = activity.operation_fingerprint.as_deref() {
            validate_private_id("operation_fingerprint", value, "op")?;
        }
        if let Some(value) = activity.workdir_fingerprint.as_deref() {
            validate_private_id("workdir_fingerprint", value, "cwd")?;
        }
        match activity.source_stable_id.as_deref() {
            Some(value) => {
                validate_scoped_fingerprint("source_stable_id", value, "src")?;
                if activity.source_identity_coverage != EvidenceIdentityCoverage::Observed {
                    return Err(MeasurementStoreError::Invalid(
                        "source_stable_id requires OBSERVED source identity coverage".into(),
                    ));
                }
            }
            None if activity.source_identity_coverage == EvidenceIdentityCoverage::Observed => {
                return Err(MeasurementStoreError::Invalid(
                    "OBSERVED source identity coverage requires source_stable_id".into(),
                ));
            }
            None => {}
        }
        match activity.content_fingerprint.as_deref() {
            Some(value) => {
                validate_scoped_fingerprint("content_fingerprint", value, "cnt")?;
                if activity.content_identity_coverage != EvidenceIdentityCoverage::Observed {
                    return Err(MeasurementStoreError::Invalid(
                        "content_fingerprint requires OBSERVED content identity coverage".into(),
                    ));
                }
            }
            None if activity.content_identity_coverage == EvidenceIdentityCoverage::Observed => {
                return Err(MeasurementStoreError::Invalid(
                    "OBSERVED content identity coverage requires content_fingerprint".into(),
                ));
            }
            None => {}
        }
    }

    let mut source_version_slots = BTreeSet::new();
    for version in &input.source_versions {
        validate_scoped_fingerprint(
            "source_version source_stable_id",
            &version.source_stable_id,
            "src",
        )?;
        validate_scoped_fingerprint(
            "source_version version_fingerprint",
            &version.version_fingerprint,
            "ver",
        )?;
        if version
            .snapshot_observed_at
            .as_deref()
            .is_some_and(|value| value.trim().is_empty())
        {
            return Err(MeasurementStoreError::Invalid(
                "source version snapshot_observed_at cannot be empty".into(),
            ));
        }
        if !source_version_slots
            .insert((version.source_stable_id.as_str(), version.boundary.as_str()))
        {
            return Err(MeasurementStoreError::Invalid(format!(
                "duplicate source version boundary {} for {}",
                version.boundary.as_str(),
                version.source_stable_id
            )));
        }
    }

    match input.task_input.coverage {
        EvidenceIdentityCoverage::Observed => {
            let fingerprint = input
                .task_input
                .task_fingerprint
                .as_deref()
                .ok_or_else(|| {
                    MeasurementStoreError::Invalid(
                        "OBSERVED task input identity requires task_fingerprint".into(),
                    )
                })?;
            validate_scoped_fingerprint("task input task_fingerprint", fingerprint, "tsk")?;
            if input.task_input.bytes.is_none() {
                return Err(MeasurementStoreError::Invalid(
                    "OBSERVED task input identity requires byte length".into(),
                ));
            }
        }
        EvidenceIdentityCoverage::Partial
        | EvidenceIdentityCoverage::NotCaptured
        | EvidenceIdentityCoverage::Unknown => {
            if input.task_input.task_fingerprint.is_some() || input.task_input.bytes.is_some() {
                return Err(MeasurementStoreError::Invalid(
                    "non-observed task input identity cannot carry fingerprint or byte length"
                        .into(),
                ));
            }
        }
    }

    let mut git_provenance_boundaries = BTreeSet::new();
    for provenance in &input.workspace_git_provenance {
        if !git_provenance_boundaries.insert(provenance.boundary.as_str()) {
            return Err(MeasurementStoreError::Invalid(format!(
                "duplicate workspace Git provenance boundary {}",
                provenance.boundary.as_str()
            )));
        }
        if provenance
            .snapshot_observed_at
            .as_deref()
            .is_some_and(|value| value.trim().is_empty())
        {
            return Err(MeasurementStoreError::Invalid(
                "workspace Git provenance snapshot_observed_at cannot be empty".into(),
            ));
        }
        match provenance.coverage {
            WorkspaceGitProvenanceCoverage::Observed => {
                let head = provenance.head_fingerprint.as_deref().ok_or_else(|| {
                    MeasurementStoreError::Invalid(
                        "OBSERVED workspace Git provenance requires head_fingerprint".into(),
                    )
                })?;
                validate_scoped_fingerprint("workspace Git head_fingerprint", head, "git")?;
                if provenance.dirty.is_none() {
                    return Err(MeasurementStoreError::Invalid(
                        "OBSERVED workspace Git provenance requires dirty state".into(),
                    ));
                }
            }
            WorkspaceGitProvenanceCoverage::NotCaptured
            | WorkspaceGitProvenanceCoverage::Unknown => {
                if provenance.head_fingerprint.is_some() || provenance.dirty.is_some() {
                    return Err(MeasurementStoreError::Invalid(
                        "non-observed workspace Git provenance cannot carry head or dirty state"
                            .into(),
                    ));
                }
            }
        }
    }

    for snapshot in &input.rate_limit_snapshots {
        if snapshot.observed_at.trim().is_empty() {
            return Err(MeasurementStoreError::Invalid(
                "rate-limit observed_at cannot be empty".into(),
            ));
        }
        if snapshot
            .limit_id
            .as_deref()
            .is_some_and(|value| value.trim().is_empty())
        {
            return Err(MeasurementStoreError::Invalid(
                "rate-limit limit_id cannot be empty when provided".into(),
            ));
        }
        if snapshot.primary_used_percent.is_none()
            && snapshot.secondary_used_percent.is_none()
            && snapshot.rate_limit_reached_type.is_none()
        {
            return Err(MeasurementStoreError::Invalid(
                "rate-limit snapshot has no diagnostic window or reached type".into(),
            ));
        }
        if snapshot.primary_used_percent.is_none()
            && (snapshot.primary_window_minutes.is_some() || snapshot.primary_resets_at.is_some())
        {
            return Err(MeasurementStoreError::Invalid(
                "primary rate-limit metadata requires primary_used_percent".into(),
            ));
        }
        if snapshot.secondary_used_percent.is_none()
            && (snapshot.secondary_window_minutes.is_some()
                || snapshot.secondary_resets_at.is_some())
        {
            return Err(MeasurementStoreError::Invalid(
                "secondary rate-limit metadata requires secondary_used_percent".into(),
            ));
        }
        for (name, raw) in [
            (
                "primary_used_percent",
                snapshot.primary_used_percent.as_deref(),
            ),
            (
                "secondary_used_percent",
                snapshot.secondary_used_percent.as_deref(),
            ),
        ] {
            if let Some(raw) = raw {
                let parsed = raw.parse::<f64>().map_err(|_| {
                    MeasurementStoreError::Invalid(format!("{name} is not numeric"))
                })?;
                if !parsed.is_finite() {
                    return Err(MeasurementStoreError::Invalid(format!(
                        "{name} must be finite"
                    )));
                }
            }
        }
    }

    if let Some(profile) = input.profile.as_ref() {
        let errors = profile.validation_errors();
        if !errors.is_empty() {
            return Err(MeasurementStoreError::Invalid(format!(
                "invalid ModelRuntimeProfile: {}",
                errors.join("; ")
            )));
        }
    }
    Ok(())
}

fn validate_private_id(name: &str, value: &str, prefix: &str) -> Result<(), MeasurementStoreError> {
    let expected_prefix = format!("{prefix}-");
    let Some(hex) = value.strip_prefix(&expected_prefix) else {
        return Err(MeasurementStoreError::Invalid(format!(
            "{name} must be a privacy-preserving {prefix}-<hex> identifier"
        )));
    };
    if hex.len() != PRIVATE_ID_HEX_LEN || !hex.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err(MeasurementStoreError::Invalid(format!(
            "{name} must contain exactly {PRIVATE_ID_HEX_LEN} hexadecimal characters"
        )));
    }
    Ok(())
}

fn validate_scoped_fingerprint(
    name: &str,
    value: &str,
    prefix: &str,
) -> Result<(), MeasurementStoreError> {
    let expected_prefix = format!("{prefix}-v1-");
    let Some(hex) = value.strip_prefix(&expected_prefix) else {
        return Err(MeasurementStoreError::Invalid(format!(
            "{name} must be a privacy-preserving {prefix}-v1-<hex> fingerprint"
        )));
    };
    if hex.len() != SCOPED_FINGERPRINT_HEX_LEN || !hex.bytes().all(|byte| byte.is_ascii_hexdigit())
    {
        return Err(MeasurementStoreError::Invalid(format!(
            "{name} must contain exactly {SCOPED_FINGERPRINT_HEX_LEN} hexadecimal characters"
        )));
    }
    Ok(())
}

fn parse_source_version_boundary(value: String) -> rusqlite::Result<SourceVersionBoundary> {
    match value.as_str() {
        "BEFORE" => Ok(SourceVersionBoundary::Before),
        "AFTER" => Ok(SourceVersionBoundary::After),
        _ => Err(rusqlite::Error::FromSqlConversionFailure(
            0,
            rusqlite::types::Type::Text,
            Box::new(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("unknown source version boundary: {value}"),
            )),
        )),
    }
}

fn parse_workspace_git_provenance_coverage(
    value: String,
) -> rusqlite::Result<WorkspaceGitProvenanceCoverage> {
    match value.as_str() {
        "OBSERVED" => Ok(WorkspaceGitProvenanceCoverage::Observed),
        "NOT_CAPTURED" => Ok(WorkspaceGitProvenanceCoverage::NotCaptured),
        "UNKNOWN" => Ok(WorkspaceGitProvenanceCoverage::Unknown),
        _ => Err(rusqlite::Error::FromSqlConversionFailure(
            0,
            rusqlite::types::Type::Text,
            Box::new(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("unknown workspace Git provenance coverage: {value}"),
            )),
        )),
    }
}

fn parse_identity_coverage(value: String) -> rusqlite::Result<EvidenceIdentityCoverage> {
    EvidenceIdentityCoverage::parse(&value).ok_or_else(|| {
        rusqlite::Error::FromSqlConversionFailure(
            0,
            rusqlite::types::Type::Text,
            std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("unknown evidence identity coverage: {value}"),
            )
            .into(),
        )
    })
}

fn row_u64(row: &rusqlite::Row<'_>, index: usize) -> rusqlite::Result<u64> {
    let value: i64 = row.get(index)?;
    u64::try_from(value).map_err(|_| rusqlite::Error::IntegralValueOutOfRange(index, value))
}

fn row_optional_u64(row: &rusqlite::Row<'_>, index: usize) -> rusqlite::Result<Option<u64>> {
    let value: Option<i64> = row.get(index)?;
    value
        .map(|value| {
            u64::try_from(value).map_err(|_| rusqlite::Error::IntegralValueOutOfRange(index, value))
        })
        .transpose()
}

fn now_unix() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}

fn to_i64(value: u64) -> Result<i64, TryFromIntError> {
    i64::try_from(value)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;
    use tokn_domain::{
        AgentNode, ExperimentValidityVerdict, MODEL_RUNTIME_PROFILE_SCHEMA_VERSION,
        RUNNER_RESULT_SCHEMA_VERSION, RunnerArtifactPaths, RunnerPipelineStatus,
        RunnerRecoveryStatus, SourceHealthStatus, TerminalObservation, TerminalStatus, TokenTotals,
        ValidityCheckStatus,
    };

    fn temp_db(name: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "tokn-measurement-{name}-{}.sqlite3",
            std::process::id()
        ))
    }

    #[test]
    fn scoped_identity_validator_accepts_only_versioned_project_scoped_fingerprints() {
        let source = scoped_source_id_bytes("project-a", b"file:src/lib.rs");
        let content =
            scoped_fingerprint_bytes("cnt", "project-a", "tool-result-output-v1", b"fixture");
        assert!(validate_scoped_fingerprint("source", &source, "src").is_ok());
        assert!(validate_scoped_fingerprint("content", &content, "cnt").is_ok());
        assert!(
            validate_scoped_fingerprint("source", &private_id("src", "raw-path"), "src").is_err()
        );
        assert!(validate_scoped_fingerprint("content", "cnt-v1-deadbeef", "cnt").is_err());
    }

    #[test]
    fn scoped_content_fingerprint_is_stable_within_project_and_separated_across_projects() {
        let raw = b"privacy-sensitive fixture output";
        let first = scoped_fingerprint_bytes("cnt", "project-a", "tool-result-output-v1", raw);
        let repeat = scoped_fingerprint_bytes("cnt", "project-a", "tool-result-output-v1", raw);
        let other_project =
            scoped_fingerprint_bytes("cnt", "project-b", "tool-result-output-v1", raw);
        let other_domain = scoped_fingerprint_bytes("cnt", "project-a", "other-domain", raw);

        assert_eq!(first, repeat);
        assert_ne!(first, other_project);
        assert_ne!(first, other_domain);
        assert!(first.starts_with("cnt-v1-"));
        assert!(!first.contains("privacy-sensitive"));
    }

    fn totals() -> TokenTotals {
        TokenTotals {
            usage_records: 2,
            input_tokens: 100,
            cached_input_tokens: 70,
            cache_write_input_tokens: 10,
            output_tokens: 20,
            reasoning_output_tokens: 5,
            input_known: 2,
            cached_input_known: 2,
            cache_write_input_known: 1,
            output_known: 2,
            reasoning_output_known: 2,
        }
    }

    fn result(private_path: &str) -> RunnerResult {
        RunnerResult {
            schema_version: RUNNER_RESULT_SCHEMA_VERSION,
            run_id: "run-store-v2".into(),
            pipeline_status: RunnerPipelineStatus::Complete,
            source_requested: private_path.into(),
            source_health_status: SourceHealthStatus::Healthy,
            fallback_recovered: false,
            root_session: private_path.into(),
            evidence_dir: private_path.into(),
            selected_workspace: Some(private_path.into()),
            workspace_diff_added_count: Some(1),
            workspace_diff_modified_count: Some(0),
            workspace_diff_removed_count: Some(0),
            workspace_diff_targets_selected_workspace: Some(true),
            root_terminal: TerminalStatus::Completed,
            agent_count: 1,
            policy_required: false,
            policy_observation_status: None,
            policy_enforcement_status: None,
            quality_required: Some(true),
            quality_status: Some(RunnerQualityStatus::Pass),
            validity_verdict: Some(ExperimentValidityVerdict::InstrumentationOnly),
            causal_claims_allowed: Some(false),
            descriptive_metrics_allowed: Some(true),
            recovery_status: RunnerRecoveryStatus::NotRequired,
            completed_steps: vec!["RUN_GROUP".into()],
            pending_steps: vec![],
            warnings: vec![],
            artifacts: RunnerArtifactPaths::default(),
        }
    }

    fn group(private_path: &str) -> RunGroup {
        RunGroup {
            root_thread_id: "thread-root".into(),
            session_id: Some("session-root".into()),
            agents: vec![AgentNode {
                source_path: private_path.into(),
                thread_id: "thread-root".into(),
                session_id: Some("session-root".into()),
                parent_thread_id: None,
                agent_nickname: None,
                agent_path: None,
                cwd: Some(private_path.into()),
                depth: 0,
                totals: totals(),
                terminal: TerminalObservation {
                    status: TerminalStatus::Completed,
                    error_code: None,
                    duration_ms: Some(1234),
                    turn_id: Some("turn-root".into()),
                },
            }],
            totals: totals(),
            max_depth: 0,
            root_terminal: TerminalStatus::Completed,
            all_threads_completed: true,
        }
    }

    fn profile() -> ModelRuntimeProfile {
        ModelRuntimeProfile {
            schema_version: MODEL_RUNTIME_PROFILE_SCHEMA_VERSION,
            observed_at: "2026-10-01T20:00:00+02:00".into(),
            runtime_kind: "codex".into(),
            runtime_version: Some("codex-cli fixture".into()),
            app_version: None,
            model: Some("fixture-model".into()),
            model_provider: Some("openai".into()),
            reasoning_effort: None,
            model_context_window: None,
            multi_agent_protocol_version: None,
            configuration: Default::default(),
            configuration_complete: ValidityCheckStatus::Unknown,
            capabilities: Default::default(),
        }
    }

    fn input(private_path: &str) -> MeasurementStoreInput {
        let project_id = private_id("prj", private_path);
        let workspace_id = private_id("wsp", &format!("{private_path}:workspace"));
        MeasurementStoreInput {
            project_id,
            workspace_id,
            parent_workspace_id: None,
            workspace_snapshot_fingerprint: Some(fingerprint_bytes(b"snapshot")),
            source_kind: "codex-session".into(),
            source_fingerprint: fingerprint_bytes(private_path.as_bytes()),
            source_snapshot_bytes: Some(2048),
            adapter_name: Some("tokn-codex-session".into()),
            adapter_version: Some("1".into()),
            contract: MeasurementContractManifest::default(),
            result: result(private_path),
            group: group(private_path),
            profile: Some(profile()),
            task_input: TaskInputStoreInput::default(),
            tool_activities: Vec::new(),
            rate_limit_snapshots: Vec::new(),
            source_versions: Vec::new(),
            workspace_git_provenance: Vec::new(),
        }
    }

    #[test]
    fn task_input_identity_is_private_queryable_and_idempotent() {
        let path = temp_db("task-input-identity");
        let _ = fs::remove_file(&path);
        let raw_task = b"privacy-sensitive exact task fixture";
        let mut input = input(r"C:\fixture\session.jsonl");
        input.task_input = TaskInputStoreInput {
            coverage: EvidenceIdentityCoverage::Observed,
            task_fingerprint: Some(scoped_task_input_bytes("fixture-scope", raw_task)),
            bytes: Some(raw_task.len() as u64),
        };

        let db = Database::open(&path).expect("open db");
        let first = db.save_measurement(&input).expect("first save");
        let second = db.save_measurement(&input).expect("second save");
        assert_eq!(first, second);
        assert_eq!(first.task_input_count, 1);
        assert_eq!(db.task_input_count().unwrap(), 1);
        assert_eq!(
            db.task_input_identity_schema_version().unwrap().as_deref(),
            Some("1")
        );

        let history = db.task_input_history(None, None, 50).unwrap();
        assert_eq!(history.observations.len(), 1);
        let observed = &history.observations[0];
        assert_eq!(observed.coverage, EvidenceIdentityCoverage::Observed);
        assert_eq!(observed.delivery_status, TaskInputDeliveryStatus::NotProven);
        assert_eq!(observed.bytes, Some(raw_task.len() as u64));
        assert!(
            observed
                .task_fingerprint
                .as_deref()
                .is_some_and(|value| value.starts_with("tsk-v1-"))
        );
        assert_ne!(
            scoped_task_input_bytes("fixture-scope", raw_task),
            scoped_task_input_bytes("other-scope", raw_task)
        );
        drop(db);
        let persisted = fs::read(&path).expect("read sqlite file");
        assert!(
            !persisted
                .windows(raw_task.len())
                .any(|window| window == raw_task)
        );
        let _ = fs::remove_file(path);
    }

    #[test]
    fn workspace_git_provenance_is_private_queryable_and_idempotent() {
        let path = temp_db("git-provenance");
        let _ = fs::remove_file(&path);
        let raw_head = "0123456789abcdef0123456789abcdef01234567";
        let mut input = input(r"C:\fixture\session.jsonl");
        input.workspace_git_provenance = vec![
            WorkspaceGitProvenanceStoreInput {
                boundary: SourceVersionBoundary::Before,
                coverage: WorkspaceGitProvenanceCoverage::Observed,
                head_fingerprint: Some(scoped_git_head_bytes("fixture-scope", raw_head.as_bytes())),
                dirty: Some(false),
                snapshot_observed_at: Some("2026-10-03T15:00:00Z".into()),
            },
            WorkspaceGitProvenanceStoreInput {
                boundary: SourceVersionBoundary::After,
                coverage: WorkspaceGitProvenanceCoverage::Unknown,
                head_fingerprint: None,
                dirty: None,
                snapshot_observed_at: Some("2026-10-03T15:05:00Z".into()),
            },
        ];

        let db = Database::open(&path).expect("open db");
        let first = db.save_measurement(&input).expect("first save");
        let second = db.save_measurement(&input).expect("second save");
        assert_eq!(first, second);
        assert_eq!(first.workspace_git_provenance_count, 2);
        assert_eq!(db.workspace_git_provenance_count().unwrap(), 2);
        assert_eq!(
            db.workspace_git_provenance_schema_version()
                .unwrap()
                .as_deref(),
            Some("1")
        );
        let history = db.workspace_git_provenance_history(None, None, 50).unwrap();
        assert_eq!(history.observations.len(), 2);
        assert_eq!(
            history.observations[0].boundary,
            SourceVersionBoundary::After
        );
        assert_eq!(
            history.observations[0].coverage,
            WorkspaceGitProvenanceCoverage::Unknown
        );
        assert_eq!(
            history.observations[1].boundary,
            SourceVersionBoundary::Before
        );
        assert_eq!(
            history.observations[1].coverage,
            WorkspaceGitProvenanceCoverage::Observed
        );
        assert_eq!(history.observations[1].dirty, Some(false));
        assert!(
            history.observations[1]
                .head_fingerprint
                .as_deref()
                .is_some_and(|value| value.starts_with("git-v1-"))
        );

        drop(db);
        let bytes = fs::read(&path).expect("read sqlite");
        let haystack = String::from_utf8_lossy(&bytes);
        assert!(!haystack.contains(raw_head));
        let _ = fs::remove_file(path);
    }

    #[test]
    fn duplicate_workspace_git_provenance_boundary_fails_closed() {
        let path = temp_db("git-provenance-duplicate");
        let _ = fs::remove_file(&path);
        let mut input = input(r"C:\fixture\session.jsonl");
        let fingerprint =
            scoped_git_head_bytes("fixture-scope", b"0123456789abcdef0123456789abcdef01234567");
        input.workspace_git_provenance = vec![
            WorkspaceGitProvenanceStoreInput {
                boundary: SourceVersionBoundary::Before,
                coverage: WorkspaceGitProvenanceCoverage::Observed,
                head_fingerprint: Some(fingerprint.clone()),
                dirty: Some(false),
                snapshot_observed_at: None,
            },
            WorkspaceGitProvenanceStoreInput {
                boundary: SourceVersionBoundary::Before,
                coverage: WorkspaceGitProvenanceCoverage::Observed,
                head_fingerprint: Some(fingerprint),
                dirty: Some(true),
                snapshot_observed_at: None,
            },
        ];

        let db = Database::open(&path).expect("open db");
        let error = db
            .save_measurement(&input)
            .expect_err("duplicate boundary must fail");
        assert!(
            error
                .to_string()
                .contains("duplicate workspace Git provenance boundary")
        );
        let _ = fs::remove_file(path);
    }

    #[test]
    fn measurement_store_is_idempotent_and_does_not_persist_raw_paths() {
        let path = temp_db("privacy");
        let _ = fs::remove_file(&path);
        let private_path = format!(
            "C:\\{}\\private-user\\Secret Project\\session.jsonl",
            "Users"
        );
        let input = input(&private_path);

        let db = Database::open(&path).expect("open db");
        let first = db.save_measurement(&input).expect("first save");
        let second = db.save_measurement(&input).expect("second save");

        assert_eq!(first, second);
        assert_eq!(
            db.measurement_store_schema_version().unwrap().as_deref(),
            Some("2")
        );
        assert_eq!(db.measurement_run_count().unwrap(), 1);
        assert_eq!(db.measurement_agent_count().unwrap(), 1);
        assert_eq!(db.runtime_profile_count().unwrap(), 1);
        assert_eq!(db.project_count_v2().unwrap(), 1);
        assert_eq!(db.workspace_count_v2().unwrap(), 1);

        let counts = db.measurement_store_counts().unwrap();
        assert_eq!(counts.schema_version.as_deref(), Some("2"));
        assert_eq!(counts.projects, 1);
        assert_eq!(counts.workspaces, 1);
        assert_eq!(counts.runs, 1);
        assert_eq!(counts.agents, 1);
        assert_eq!(counts.runtime_profiles, 1);

        let recent = db.recent_measurement_runs(10).unwrap();
        assert_eq!(recent.len(), 1);
        assert_eq!(recent[0].run_id, "run-store-v2");
        assert_eq!(recent[0].agent_count, 1);
        assert_eq!(recent[0].usage_records, 2);
        assert_eq!(recent[0].logical_tokens, Some(120));

        let profile_id = first.profile_id.as_deref().expect("profile id");
        assert_eq!(db.load_runtime_profile(profile_id).unwrap(), input.profile);

        drop(db);
        let bytes = fs::read(&path).expect("read sqlite");
        let haystack = String::from_utf8_lossy(&bytes);
        assert!(!haystack.contains(&private_path));
        assert!(!haystack.contains("private-user"));
        assert!(!haystack.contains("Secret Project"));

        let _ = fs::remove_file(path);
    }

    #[test]
    fn rate_limit_history_is_idempotent_and_filterable() {
        let path = temp_db("rate-limit-history");
        let _ = fs::remove_file(&path);
        let private_path = format!(r"C:\{}\rate-user\Secret Project\session.jsonl", "Users");
        let mut input = input(&private_path);
        input.rate_limit_snapshots.push(RateLimitStoreInput {
            observed_at: "2026-10-03T07:15:00Z".into(),
            limit_id: Some("codex".into()),
            primary_used_percent: Some("12.5".into()),
            primary_window_minutes: Some(300),
            primary_resets_at: Some("1791018000".into()),
            secondary_used_percent: Some("42".into()),
            secondary_window_minutes: Some(10_080),
            secondary_resets_at: Some("1791622800".into()),
            rate_limit_reached_type: Some("primary".into()),
        });

        let db = Database::open(&path).expect("open db");
        let first = db.save_measurement(&input).expect("first save");
        let second = db.save_measurement(&input).expect("second save");
        assert_eq!(first, second);
        assert_eq!(first.rate_limit_snapshot_count, 1);
        assert_eq!(db.rate_limit_snapshot_count().unwrap(), 1);

        let history = db
            .rate_limit_history(Some(&input.project_id), Some(&input.workspace_id), 10)
            .expect("rate-limit history");
        assert_eq!(history.schema_version, RATE_LIMIT_HISTORY_SCHEMA_VERSION);
        assert_eq!(history.snapshots.len(), 1);
        let snapshot = &history.snapshots[0];
        assert_eq!(snapshot.limit_id.as_deref(), Some("codex"));
        assert_eq!(snapshot.primary_used_percent.as_deref(), Some("12.5"));
        assert_eq!(snapshot.secondary_window_minutes, Some(10_080));
        assert_eq!(snapshot.run_id, "run-store-v2");

        drop(db);
        let bytes = fs::read(&path).expect("read sqlite");
        let haystack = String::from_utf8_lossy(&bytes);
        assert!(!haystack.contains("rate-user"));
        assert!(!haystack.contains("Secret Project"));
        let _ = fs::remove_file(path);
    }

    #[test]
    fn source_versions_are_idempotent_filterable_and_privacy_scoped() {
        let path = temp_db("source-version-history");
        let _ = fs::remove_file(&path);
        let private_path = format!(r"C:\{}\version-user\Secret Project\session.jsonl", "Users");
        let mut input = input(&private_path);
        let source_id = scoped_source_id_bytes("fixture-project-key", b"file:src/example.rs");
        let raw_snapshot_hash = "TOKN-SYNTHETIC-SHA256-SECRET-abcdef";
        input.source_versions.push(SourceVersionStoreInput {
            source_stable_id: source_id.clone(),
            boundary: SourceVersionBoundary::Before,
            version_fingerprint: scoped_source_version_bytes(
                "fixture-project-key",
                raw_snapshot_hash.as_bytes(),
            ),
            snapshot_observed_at: Some("2026-10-03T10:00:00Z".into()),
            bytes: 42,
        });
        input.source_versions.push(SourceVersionStoreInput {
            source_stable_id: source_id.clone(),
            boundary: SourceVersionBoundary::After,
            version_fingerprint: scoped_source_version_bytes(
                "fixture-project-key",
                b"TOKN-SYNTHETIC-SHA256-CHANGED-fedcba",
            ),
            snapshot_observed_at: Some("2026-10-03T10:30:00Z".into()),
            bytes: 43,
        });

        let db = Database::open(&path).expect("open db");
        let first = db.save_measurement(&input).expect("first save");
        let second = db.save_measurement(&input).expect("second save");
        assert_eq!(first, second);
        assert_eq!(first.source_version_count, 2);
        assert_eq!(db.source_version_count().unwrap(), 2);
        assert_eq!(
            db.source_version_schema_version().unwrap().as_deref(),
            Some("1")
        );

        let history = db
            .source_version_history(Some(&input.project_id), Some(&input.workspace_id), 10)
            .expect("source version history");
        assert_eq!(
            history.schema_version,
            SOURCE_VERSION_HISTORY_SCHEMA_VERSION
        );
        assert_eq!(history.versions.len(), 2);
        assert!(
            history
                .versions
                .iter()
                .all(|item| item.source_stable_id == source_id)
        );
        assert!(
            history
                .versions
                .iter()
                .any(|item| item.boundary == SourceVersionBoundary::Before)
        );
        assert!(
            history
                .versions
                .iter()
                .any(|item| item.boundary == SourceVersionBoundary::After)
        );

        drop(db);
        let bytes = fs::read(&path).expect("read sqlite");
        let haystack = String::from_utf8_lossy(&bytes);
        assert!(!haystack.contains(raw_snapshot_hash));
        assert!(!haystack.contains("version-user"));
        assert!(!haystack.contains("Secret Project"));
        assert!(!haystack.contains("src/example.rs"));
        let _ = fs::remove_file(path);
    }

    #[test]
    fn tool_activity_is_idempotent_and_does_not_persist_raw_command_or_workdir() {
        let path = temp_db("tool-activity-privacy");
        let _ = fs::remove_file(&path);
        let synthetic_secret = ["sk", "-", "test-TOKN-SYNTHETIC-ONLY-0123456789"].concat();
        let private_path = format!(r"C:\{}\activity-user\Secret Project\session.jsonl", "Users");
        let private_command = format!(
            r"Get-Content C:\{}\activity-user\Secret Project\secret-notes.txt; Write-Output {}",
            "Users", synthetic_secret
        );
        let private_workdir = format!(r"C:\{}\activity-user\Secret Project", "Users");
        let raw_result = format!("private-result-{synthetic_secret}");

        let mut input = input(&private_path);
        let operation_fingerprint = private_id(
            "op",
            &format!("{}:file_read:{private_command}", input.project_id),
        );
        let workdir_fingerprint =
            private_id("cwd", &format!("{}:{private_workdir}", input.project_id));
        let content_fingerprint = scoped_fingerprint_bytes(
            "cnt",
            "fixture-project-key",
            "tool-result-output-v1",
            raw_result.as_bytes(),
        );
        input.tool_activities.push(ToolActivityStoreInput {
            source_call_key: "call-private#0".into(),
            thread_id: "thread-root".into(),
            agent_ordinal: 0,
            kind: "exec_command".into(),
            tool_name: Some("exec_command".into()),
            category: "file_read".into(),
            surface: "session_rollout".into(),
            requester_type: None,
            status: "completed".into(),
            observed_at: Some("2026-10-03T12:00:00Z".into()),
            started_seq: None,
            ended_seq: None,
            invocation_payload_bytes: None,
            result_payload_bytes: None,
            result_output_chars: None,
            max_output_tokens: Some(4_000),
            original_token_count: None,
            operation_fingerprint: Some(operation_fingerprint),
            workdir_fingerprint: Some(workdir_fingerprint),
            source_stable_id: None,
            source_identity_coverage: EvidenceIdentityCoverage::NotCaptured,
            content_fingerprint: Some(content_fingerprint.clone()),
            content_identity_coverage: EvidenceIdentityCoverage::Observed,
            parse_error_present: false,
        });

        let db = Database::open(&path).expect("open db");
        let first = db.save_measurement(&input).expect("first save");
        let second = db.save_measurement(&input).expect("second save");

        assert_eq!(first, second);
        assert_eq!(first.tool_activity_count, 1);
        assert_eq!(db.tool_activity_count().unwrap(), 1);
        assert_eq!(
            db.tool_activity_schema_version().unwrap().as_deref(),
            Some("3")
        );

        let history = db
            .tool_activity_history(Some(&input.project_id), Some(&input.workspace_id), 10)
            .expect("tool activity history");
        assert_eq!(history.schema_version, TOOL_ACTIVITY_HISTORY_SCHEMA_VERSION);
        assert_eq!(history.activities.len(), 1);
        assert_eq!(history.activities[0].category, "file_read");
        assert_eq!(history.activities[0].thread_id, "thread-root");
        assert_eq!(history.activities[0].agent_ordinal, 0);
        assert!(
            history.activities[0]
                .operation_fingerprint
                .as_deref()
                .is_some_and(|value| value.starts_with("op-"))
        );
        assert!(
            history.activities[0]
                .workdir_fingerprint
                .as_deref()
                .is_some_and(|value| value.starts_with("cwd-"))
        );
        assert_eq!(
            history.activities[0].content_fingerprint.as_deref(),
            Some(content_fingerprint.as_str())
        );
        assert_eq!(
            history.activities[0].content_identity_coverage,
            EvidenceIdentityCoverage::Observed
        );
        assert_eq!(
            history.activities[0].source_identity_coverage,
            EvidenceIdentityCoverage::NotCaptured
        );

        drop(db);
        let bytes = fs::read(&path).expect("read sqlite");
        let haystack = String::from_utf8_lossy(&bytes);
        assert!(!haystack.contains(&private_command));
        assert!(!haystack.contains(&private_workdir));
        assert!(!haystack.contains("secret-notes.txt"));
        assert!(!haystack.contains("activity-user"));
        assert!(!haystack.contains(&synthetic_secret));
        assert!(!haystack.contains(&raw_result));

        let _ = fs::remove_file(path);
    }

    #[test]
    fn historical_snapshot_filters_runs_and_keeps_workspace_lineage() {
        let path = temp_db("history-snapshot");
        let _ = fs::remove_file(&path);
        let db = Database::open(&path).expect("open db");

        let private_path = format!("C:\\{}\\history-user\\Project\\session.jsonl", "Users");
        let parent = input(&private_path);
        let project_id = parent.project_id.clone();
        let parent_workspace_id = parent.workspace_id.clone();
        db.save_measurement(&parent).expect("save parent");

        let mut child = parent.clone();
        child.result.run_id = "run-store-v2-child".into();
        child.workspace_id = private_id("wsp", &format!("{private_path}:workspace:child"));
        child.parent_workspace_id = Some(parent_workspace_id.clone());
        child.workspace_snapshot_fingerprint = Some(fingerprint_bytes(b"child-snapshot"));
        child.source_fingerprint = fingerprint_bytes(b"child-source");
        let child_workspace_id = child.workspace_id.clone();
        db.save_measurement(&child).expect("save child");

        let snapshot = db
            .historical_snapshot(Some(&project_id), Some(&child_workspace_id), 10)
            .expect("historical snapshot");

        assert_eq!(snapshot.schema_version, HISTORICAL_SNAPSHOT_SCHEMA_VERSION);
        assert_eq!(
            snapshot.project_filter.as_deref(),
            Some(project_id.as_str())
        );
        assert_eq!(
            snapshot.workspace_filter.as_deref(),
            Some(child_workspace_id.as_str())
        );
        assert_eq!(snapshot.runs.len(), 1);
        assert_eq!(snapshot.runs[0].run_id, "run-store-v2-child");
        assert_eq!(snapshot.runs[0].totals.usage_records, 2);
        assert_eq!(snapshot.runs[0].totals.input_known, 2);
        assert_eq!(snapshot.agents.len(), 1);
        assert_eq!(snapshot.provenance.len(), 1);
        assert_eq!(snapshot.runtime_profiles.len(), 1);
        assert_eq!(snapshot.workspaces.len(), 2);

        let child_workspace = snapshot
            .workspaces
            .iter()
            .find(|item| item.workspace_id == child_workspace_id)
            .expect("child workspace");
        assert_eq!(
            child_workspace.parent_workspace_id.as_deref(),
            Some(parent_workspace_id.as_str())
        );
        assert!(
            snapshot
                .workspaces
                .iter()
                .any(|item| item.workspace_id == parent_workspace_id)
        );

        drop(db);
        let _ = fs::remove_file(path);
    }

    #[test]
    fn rejects_raw_or_unversioned_project_identity() {
        let path = temp_db("reject-raw-id");
        let _ = fs::remove_file(&path);
        let db = Database::open(&path).expect("open db");
        let raw_id = format!("C:\\{}\\private-user\\project", "Users");
        let mut value = input(&raw_id);
        value.project_id = raw_id;

        let error = db.save_measurement(&value).expect_err("raw id must fail");
        assert!(error.to_string().contains("privacy-preserving"));

        drop(db);
        let _ = fs::remove_file(path);
    }
}
