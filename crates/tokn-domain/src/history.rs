use serde::{Deserialize, Serialize};

use crate::{ModelRuntimeProfile, TokenTotals};

pub const HISTORICAL_SNAPSHOT_SCHEMA_VERSION: u64 = 1;
pub const TOOL_ACTIVITY_HISTORY_SCHEMA_VERSION: u64 = 1;

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

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct HistoricalToolActivityRecord {
    pub activity_id: String,
    pub run_id: String,
    pub project_id: String,
    pub workspace_id: String,
    pub thread_id: String,
    pub agent_ordinal: u64,
    pub kind: String,
    pub tool_name: Option<String>,
    pub category: String,
    pub surface: String,
    pub requester_type: Option<String>,
    pub status: String,
    pub started_seq: Option<u64>,
    pub ended_seq: Option<u64>,
    pub invocation_payload_bytes: Option<u64>,
    pub result_payload_bytes: Option<u64>,
    pub result_output_chars: Option<u64>,
    pub max_output_tokens: Option<u64>,
    pub original_token_count: Option<u64>,
    pub operation_fingerprint: Option<String>,
    pub workdir_fingerprint: Option<String>,
    pub parse_error_present: bool,
    pub run_created_at_unix: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ToolActivityHistory {
    pub schema_version: u64,
    pub project_filter: Option<String>,
    pub workspace_filter: Option<String>,
    pub run_limit: u64,
    pub activities: Vec<HistoricalToolActivityRecord>,
}

impl Default for ToolActivityHistory {
    fn default() -> Self {
        Self {
            schema_version: TOOL_ACTIVITY_HISTORY_SCHEMA_VERSION,
            project_filter: None,
            workspace_filter: None,
            run_limit: 0,
            activities: Vec::new(),
        }
    }
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
