use serde::{Deserialize, Serialize};

use crate::{SourceHealthStatus, TerminalStatus};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ValidityCheckStatus {
    Pass,
    Fail,
    #[default]
    Unknown,
    NotRequired,
}

impl ValidityCheckStatus {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Pass => "PASS",
            Self::Fail => "FAIL",
            Self::Unknown => "UNKNOWN",
            Self::NotRequired => "NOT_REQUIRED",
        }
    }

    pub const fn is_satisfied(self) -> bool {
        matches!(self, Self::Pass | Self::NotRequired)
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ExperimentIntent {
    #[default]
    Instrumentation,
    DescriptiveComparison,
    CausalAb,
}

impl ExperimentIntent {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Instrumentation => "INSTRUMENTATION",
            Self::DescriptiveComparison => "DESCRIPTIVE_COMPARISON",
            Self::CausalAb => "CAUSAL_AB",
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ExperimentValidityVerdict {
    ValidForCausalAb,
    ValidForDescriptiveComparison,
    #[default]
    InstrumentationOnly,
    InvalidCapture,
    IncompleteTask,
}

impl ExperimentValidityVerdict {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ValidForCausalAb => "VALID_FOR_CAUSAL_AB",
            Self::ValidForDescriptiveComparison => "VALID_FOR_DESCRIPTIVE_COMPARISON",
            Self::InstrumentationOnly => "INSTRUMENTATION_ONLY",
            Self::InvalidCapture => "INVALID_CAPTURE",
            Self::IncompleteTask => "INCOMPLETE_TASK",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CaptureValidityInput {
    pub model_usage: ValidityCheckStatus,
    pub tool_evidence: ValidityCheckStatus,
    pub source_health: SourceHealthStatus,
    pub fallback_recovered: bool,
    pub run_group_resolved: ValidityCheckStatus,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TaskValidityInput {
    pub exact_task_captured: ValidityCheckStatus,
    pub terminal_status: TerminalStatus,
    pub completion_required: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WorkspaceValidityInput {
    pub output_workspace_resolved: ValidityCheckStatus,
    pub before_state_captured: ValidityCheckStatus,
    pub after_state_captured: ValidityCheckStatus,
    pub quality_gate_on_output: ValidityCheckStatus,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PolicyValidityInput {
    pub required: bool,
    pub identity_captured: ValidityCheckStatus,
    pub exposure_known: ValidityCheckStatus,
    pub compliance_evidence: ValidityCheckStatus,
    pub enforcement_accurately_labeled: ValidityCheckStatus,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RuntimeValidityInput {
    pub model_recorded: ValidityCheckStatus,
    pub runtime_recorded: ValidityCheckStatus,
    pub configuration_recorded: ValidityCheckStatus,
    pub comparable_to_baseline: ValidityCheckStatus,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct QualityValidityInput {
    pub automated_gates: ValidityCheckStatus,
    pub human_or_host_gates: ValidityCheckStatus,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CausalControlsInput {
    pub baseline_available: ValidityCheckStatus,
    pub same_task: ValidityCheckStatus,
    pub same_starting_workspace: ValidityCheckStatus,
    pub same_runtime_model_config: ValidityCheckStatus,
    pub single_primary_variable: ValidityCheckStatus,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExperimentValidityInput {
    pub experiment_id: String,
    pub intent: ExperimentIntent,
    pub capture: CaptureValidityInput,
    pub task: TaskValidityInput,
    pub workspace: WorkspaceValidityInput,
    pub policy: PolicyValidityInput,
    pub runtime: RuntimeValidityInput,
    pub quality: QualityValidityInput,
    pub causal: CausalControlsInput,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ValidityReason {
    pub code: String,
    pub dimension: String,
    pub status: ValidityCheckStatus,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExperimentValidityReport {
    pub experiment_id: String,
    pub intent: ExperimentIntent,
    pub verdict: ExperimentValidityVerdict,
    pub causal_claims_allowed: bool,
    pub descriptive_metrics_allowed: bool,
    pub reasons: Vec<ValidityReason>,
}
