use std::collections::BTreeSet;
use std::num::TryFromIntError;
use std::time::{SystemTime, UNIX_EPOCH};

use rusqlite::{OptionalExtension, params};
use thiserror::Error;
use tokn_domain::{
    HISTORICAL_SNAPSHOT_SCHEMA_VERSION, HistoricalAgentRecord, HistoricalProvenanceRecord,
    HistoricalRunRecord, HistoricalRuntimeProfileRecord, HistoricalSnapshot,
    HistoricalWorkspaceRecord, MEASUREMENT_CONTRACT_ID, MEASUREMENT_CONTRACT_VERSION,
    MeasurementContractManifest, ModelRuntimeProfile, RunGroup, RunnerQualityStatus, RunnerResult,
    TokenTotals,
};

use crate::Database;

const PRIVATE_ID_HEX_LEN: usize = 24;

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
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MeasurementStoreSummary {
    pub run_id: String,
    pub project_id: String,
    pub workspace_id: String,
    pub profile_id: Option<String>,
    pub source_id: String,
    pub agent_count: u64,
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
        }
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
