use serde::{Deserialize, Serialize};

use crate::{ModelRuntimeProfile, TokenTotals};

pub const HISTORICAL_SNAPSHOT_SCHEMA_VERSION: u64 = 1;

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct HistoricalWorkspaceRecord {
    pub workspace_id: String,
    pub project_id: String,
    pub parent_workspace_id: Option<String>,
    pub snapshot_fingerprint: Option<String>,
    pub identity_version: u64,
    pub created_at_unix: i64,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct HistoricalRunRecord {
    pub run_id: String,
    pub project_id: String,
    pub workspace_id: String,
    pub profile_id: Option<String>,
    pub measurement_contract_version: u64,
    pub evidence_layout_version: u64,
    pub root_thread_id: String,
    pub root_terminal: String,
    pub source_health: String,
    pub validity_verdict: Option<String>,
    pub quality_status: Option<String>,
    pub agent_count: u64,
    pub totals: TokenTotals,
    pub logical_tokens: Option<u64>,
    pub created_at_unix: i64,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct HistoricalAgentRecord {
    pub run_id: String,
    pub thread_id: String,
    pub parent_thread_id: Option<String>,
    pub depth: u32,
    pub terminal_status: String,
    pub duration_ms: Option<u64>,
    pub totals: TokenTotals,
    pub logical_tokens: Option<u64>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct HistoricalProvenanceRecord {
    pub source_id: String,
    pub run_id: String,
    pub source_kind: String,
    pub source_fingerprint: String,
    pub snapshot_bytes: Option<u64>,
    pub adapter_name: Option<String>,
    pub adapter_version: Option<String>,
    pub created_at_unix: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HistoricalRuntimeProfileRecord {
    pub profile_id: String,
    pub profile: ModelRuntimeProfile,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HistoricalSnapshot {
    pub schema_version: u64,
    pub project_filter: Option<String>,
    pub workspace_filter: Option<String>,
    pub workspaces: Vec<HistoricalWorkspaceRecord>,
    pub runs: Vec<HistoricalRunRecord>,
    pub agents: Vec<HistoricalAgentRecord>,
    pub provenance: Vec<HistoricalProvenanceRecord>,
    pub runtime_profiles: Vec<HistoricalRuntimeProfileRecord>,
}

impl Default for HistoricalSnapshot {
    fn default() -> Self {
        Self {
            schema_version: HISTORICAL_SNAPSHOT_SCHEMA_VERSION,
            project_filter: None,
            workspace_filter: None,
            workspaces: Vec::new(),
            runs: Vec::new(),
            agents: Vec::new(),
            provenance: Vec::new(),
            runtime_profiles: Vec::new(),
        }
    }
}
