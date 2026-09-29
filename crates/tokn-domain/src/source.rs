use serde::{Deserialize, Serialize};

use crate::SourceId;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SourceKind {
    CodexSession,
    CodexDiagnosticTrace,
    Unknown,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SourceHealthStatus {
    Healthy,
    Partial,
    Empty,
    Malformed,
    Unsupported,
    #[default]
    Unknown,
}

impl SourceHealthStatus {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Healthy => "HEALTHY",
            Self::Partial => "PARTIAL",
            Self::Empty => "EMPTY",
            Self::Malformed => "MALFORMED",
            Self::Unsupported => "UNSUPPORTED",
            Self::Unknown => "UNKNOWN",
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceCoverage {
    pub records_seen: u64,
    pub records_valid: u64,
    pub malformed_records: u64,
    pub inference_events: u64,
    pub usage_records: u64,
    pub tool_events: u64,
    pub terminal_events: u64,
    pub protocol_events: u64,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceHealth {
    pub status: SourceHealthStatus,
    pub coverage: SourceCoverage,
    pub reasons: Vec<String>,
}

impl SourceHealth {
    pub fn has_model_usage(&self) -> bool {
        self.coverage.usage_records > 0
    }

    pub fn is_usable_for_token_accounting(&self) -> bool {
        self.status == SourceHealthStatus::Healthy && self.has_model_usage()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Provenance {
    pub source_id: SourceId,
    pub source_kind: SourceKind,
    pub byte_offset: Option<u64>,
    pub byte_length: Option<u64>,
    pub physical_line: Option<u64>,
    pub source_seq: Option<u64>,
    pub record_hash: String,
    pub adapter_name: String,
    pub adapter_version: String,
}
