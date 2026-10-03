use serde::{Deserialize, Serialize};

use crate::{EvidenceIdentityCoverage, ModelRuntimeProfile, TokenTotals};

pub const HISTORICAL_SNAPSHOT_SCHEMA_VERSION: u64 = 1;
pub const TOOL_ACTIVITY_HISTORY_SCHEMA_VERSION: u64 = 3;
pub const RATE_LIMIT_HISTORY_SCHEMA_VERSION: u64 = 1;
pub const SOURCE_VERSION_HISTORY_SCHEMA_VERSION: u64 = 1;

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
pub struct HistoricalRateLimitSnapshotRecord {
    pub snapshot_id: String,
    pub run_id: String,
    pub project_id: String,
    pub workspace_id: String,
    pub observed_at: String,
    pub limit_id: Option<String>,
    pub primary_used_percent: Option<String>,
    pub primary_window_minutes: Option<i64>,
    pub primary_resets_at: Option<String>,
    pub secondary_used_percent: Option<String>,
    pub secondary_window_minutes: Option<i64>,
    pub secondary_resets_at: Option<String>,
    pub rate_limit_reached_type: Option<String>,
    pub created_at_unix: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RateLimitHistory {
    pub schema_version: u64,
    pub project_filter: Option<String>,
    pub workspace_filter: Option<String>,
    pub run_limit: u64,
    pub snapshots: Vec<HistoricalRateLimitSnapshotRecord>,
}

impl Default for RateLimitHistory {
    fn default() -> Self {
        Self {
            schema_version: RATE_LIMIT_HISTORY_SCHEMA_VERSION,
            project_filter: None,
            workspace_filter: None,
            run_limit: 0,
            snapshots: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SourceVersionBoundary {
    Before,
    After,
}

impl SourceVersionBoundary {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Before => "BEFORE",
            Self::After => "AFTER",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HistoricalSourceVersionRecord {
    pub version_id: String,
    pub run_id: String,
    pub project_id: String,
    pub workspace_id: String,
    pub source_stable_id: String,
    pub boundary: SourceVersionBoundary,
    pub version_fingerprint: String,
    pub snapshot_observed_at: Option<String>,
    pub bytes: u64,
    pub run_created_at_unix: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceVersionHistory {
    pub schema_version: u64,
    pub project_filter: Option<String>,
    pub workspace_filter: Option<String>,
    pub run_limit: u64,
    pub versions: Vec<HistoricalSourceVersionRecord>,
}

impl Default for SourceVersionHistory {
    fn default() -> Self {
        Self {
            schema_version: SOURCE_VERSION_HISTORY_SCHEMA_VERSION,
            project_filter: None,
            workspace_filter: None,
            run_limit: 0,
            versions: Vec::new(),
        }
    }
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
