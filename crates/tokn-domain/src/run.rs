use serde::{Deserialize, Serialize};

use crate::{TokenTotals, TokenUsage, ToolObservation};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TerminalStatus {
    Completed,
    IncompleteUsageLimit,
    IncompleteCancelled,
    IncompleteError,
    IncompleteUserAbort,
    #[default]
    UnknownTerminalState,
}

impl TerminalStatus {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Completed => "COMPLETED",
            Self::IncompleteUsageLimit => "INCOMPLETE_USAGE_LIMIT",
            Self::IncompleteCancelled => "INCOMPLETE_CANCELLED",
            Self::IncompleteError => "INCOMPLETE_ERROR",
            Self::IncompleteUserAbort => "INCOMPLETE_USER_ABORT",
            Self::UnknownTerminalState => "UNKNOWN_TERMINAL_STATE",
        }
    }

    pub fn is_complete(self) -> bool {
        self == Self::Completed
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct TerminalObservation {
    pub status: TerminalStatus,
    pub error_code: Option<String>,
    pub duration_ms: Option<u64>,
    pub turn_id: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct KeyedTokenUsage {
    pub key: Option<String>,
    pub usage: TokenUsage,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct RateLimitWindowObservation {
    pub used_percent: String,
    pub window_minutes: Option<i64>,
    pub resets_at: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct RateLimitSnapshotObservation {
    pub observed_at: String,
    pub limit_id: Option<String>,
    pub primary: Option<RateLimitWindowObservation>,
    pub secondary: Option<RateLimitWindowObservation>,
    pub rate_limit_reached_type: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentEvidence {
    pub source_path: String,
    pub thread_id: String,
    pub session_id: Option<String>,
    pub parent_thread_id: Option<String>,
    pub forked_from_id: Option<String>,
    pub thread_source: Option<String>,
    pub agent_nickname: Option<String>,
    pub agent_path: Option<String>,
    pub cwd: Option<String>,
    pub cli_version: Option<String>,
    pub usages: Vec<KeyedTokenUsage>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub rate_limit_snapshots: Vec<RateLimitSnapshotObservation>,
    pub tools: Vec<ToolObservation>,
    pub tool_parse_failures: u64,
    pub terminal: TerminalObservation,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentNode {
    pub source_path: String,
    pub thread_id: String,
    pub session_id: Option<String>,
    pub parent_thread_id: Option<String>,
    pub agent_nickname: Option<String>,
    pub agent_path: Option<String>,
    pub cwd: Option<String>,
    pub depth: u32,
    pub totals: TokenTotals,
    pub terminal: TerminalObservation,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct RunGroup {
    pub root_thread_id: String,
    pub session_id: Option<String>,
    pub agents: Vec<AgentNode>,
    pub totals: TokenTotals,
    pub max_depth: u32,
    pub root_terminal: TerminalStatus,
    pub all_threads_completed: bool,
}
