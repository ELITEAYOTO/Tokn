use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PolicyEnforcementStatus {
    Unavailable,
    Enforced,
    #[default]
    NotProven,
}

impl PolicyEnforcementStatus {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Unavailable => "UNAVAILABLE",
            Self::Enforced => "ENFORCED",
            Self::NotProven => "NOT_PROVEN",
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PolicyObservationStatus {
    Pass,
    Fail,
    NoEvidence,
    IncompleteEvidence,
    #[default]
    NotEvaluated,
}

impl PolicyObservationStatus {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Pass => "PASS",
            Self::Fail => "FAIL",
            Self::NoEvidence => "NO_EVIDENCE",
            Self::IncompleteEvidence => "INCOMPLETE_EVIDENCE",
            Self::NotEvaluated => "NOT_EVALUATED",
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct PolicyObservationSummary {
    pub status: PolicyObservationStatus,
    pub targeted: u64,
    pub compliant: u64,
    pub violations: u64,
    pub unknown: u64,
    pub parse_failures: u64,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct PolicyPlacement {
    pub path: String,
    pub sha256: Option<String>,
}
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionPolicyEvidence {
    pub source_path: String,
    pub thread_id: Option<String>,
    pub instruction_observed: bool,
    pub repository_read_observed: bool,
    pub repository_read_count: u64,
    pub matched_policy_paths: Vec<String>,
    pub parse_failures: u64,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct PolicyEvidenceReport {
    pub policy_id: String,
    pub marker: String,
    pub placements: Vec<PolicyPlacement>,
    pub sessions: Vec<SessionPolicyEvidence>,
    pub hint_placed: bool,
    pub instruction_observed_threads: u64,
    pub repository_read_threads: u64,
    pub repository_read_count: u64,
    pub parse_failures: u64,
    pub observed: PolicyObservationSummary,
    pub enforcement: PolicyEnforcementStatus,
}
