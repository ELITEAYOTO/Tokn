use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::{
    ExperimentIntent, ExperimentValidityVerdict, PolicyEnforcementStatus, PolicyObservationStatus,
    RUNNER_REQUEST_SCHEMA_VERSION, SourceHealth, SourceHealthStatus, SourceKind, TerminalStatus,
    ValidityCheckStatus,
};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum RunnerPipelineStatus {
    #[default]
    CoreEvidenceReady,
    Blocked,
    Complete,
    Failed,
}

impl RunnerPipelineStatus {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::CoreEvidenceReady => "CORE_EVIDENCE_READY",
            Self::Blocked => "BLOCKED",
            Self::Complete => "COMPLETE",
            Self::Failed => "FAILED",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RunnerExperimentRequest {
    pub experiment_id: String,
    pub intent: ExperimentIntent,
    #[serde(default)]
    pub validity: RunnerValidityHints,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RunnerValidityHints {
    pub exact_task_captured: ValidityCheckStatus,
    pub completion_required: bool,
    pub tool_evidence_required: bool,
    pub policy_enforcement_accurately_labeled: ValidityCheckStatus,
    pub model_recorded: ValidityCheckStatus,
    pub configuration_recorded: ValidityCheckStatus,
    pub comparable_to_baseline: ValidityCheckStatus,
    pub human_or_host_gates: ValidityCheckStatus,
    pub baseline_available: ValidityCheckStatus,
    pub same_task: ValidityCheckStatus,
    pub same_starting_workspace: ValidityCheckStatus,
    pub same_runtime_model_config: ValidityCheckStatus,
    pub single_primary_variable: ValidityCheckStatus,
}

impl Default for RunnerValidityHints {
    fn default() -> Self {
        Self {
            exact_task_captured: ValidityCheckStatus::Unknown,
            completion_required: true,
            tool_evidence_required: true,
            policy_enforcement_accurately_labeled: ValidityCheckStatus::Unknown,
            model_recorded: ValidityCheckStatus::Unknown,
            configuration_recorded: ValidityCheckStatus::Unknown,
            comparable_to_baseline: ValidityCheckStatus::Unknown,
            human_or_host_gates: ValidityCheckStatus::Unknown,
            baseline_available: ValidityCheckStatus::Unknown,
            same_task: ValidityCheckStatus::Unknown,
            same_starting_workspace: ValidityCheckStatus::Unknown,
            same_runtime_model_config: ValidityCheckStatus::Unknown,
            single_primary_variable: ValidityCheckStatus::Unknown,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RunnerPolicyRequest {
    pub policy_id: String,
    pub marker: String,
    #[serde(default)]
    pub policy_paths: Vec<String>,
    #[serde(default)]
    pub caps: BTreeMap<String, u64>,
    #[serde(default)]
    pub enforcement: PolicyEnforcementStatus,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum RunnerQualityStatus {
    Pass,
    Fail,
    Unavailable,
    NotRequired,
    #[default]
    Unknown,
}

impl RunnerQualityStatus {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Pass => "PASS",
            Self::Fail => "FAIL",
            Self::Unavailable => "UNAVAILABLE",
            Self::NotRequired => "NOT_REQUIRED",
            Self::Unknown => "UNKNOWN",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RunnerQualityRequest {
    pub required: bool,
    #[serde(default)]
    pub program: Option<String>,
    #[serde(default)]
    pub args: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RunnerQualityReport {
    pub required: bool,
    pub status: RunnerQualityStatus,
    pub workspace: Option<String>,
    pub program: Option<String>,
    pub args: Vec<String>,
    pub exit_code: Option<i32>,
    pub stdout: String,
    pub stderr: String,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum RunnerRecoveryStatus {
    #[default]
    NotRequired,
    RecoveredPartial,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RunnerRecoveryReport {
    pub status: RunnerRecoveryStatus,
    pub removed_artifacts: Vec<String>,
    pub policy_placements_mutated: bool,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RunnerRequest {
    pub schema_version: u64,
    pub run_id: String,
    pub source: String,
    #[serde(default)]
    pub session_candidates: Vec<String>,
    pub source_root: String,
    pub after_inventory: String,
    #[serde(default)]
    pub before_inventory: Option<String>,
    #[serde(default)]
    pub before_snapshot: Option<String>,
    #[serde(default)]
    pub after_snapshot: Option<String>,
    #[serde(default)]
    pub expected_outputs: Vec<String>,
    pub evidence_dir: String,
    #[serde(default)]
    pub experiment: Option<RunnerExperimentRequest>,
    #[serde(default)]
    pub policy: Option<RunnerPolicyRequest>,
    #[serde(default)]
    pub quality: Option<RunnerQualityRequest>,
}

impl RunnerRequest {
    pub fn validation_errors(&self) -> Vec<String> {
        let mut errors = Vec::new();

        if self.schema_version != RUNNER_REQUEST_SCHEMA_VERSION {
            errors.push(format!(
                "unsupported runner request schema_version {}; expected {}",
                self.schema_version, RUNNER_REQUEST_SCHEMA_VERSION
            ));
        }
        for (name, value) in [
            ("run_id", self.run_id.as_str()),
            ("source", self.source.as_str()),
            ("source_root", self.source_root.as_str()),
            ("after_inventory", self.after_inventory.as_str()),
            ("evidence_dir", self.evidence_dir.as_str()),
        ] {
            if value.trim().is_empty() {
                errors.push(format!("{name} cannot be empty"));
            }
        }

        if self
            .session_candidates
            .iter()
            .any(|path| path.trim().is_empty())
        {
            errors.push("session_candidates cannot contain empty paths".into());
        }

        if self.before_snapshot.is_some() != self.after_snapshot.is_some() {
            errors.push(
                "before_snapshot and after_snapshot must either both be provided or both omitted"
                    .into(),
            );
        }

        if let Some(experiment) = &self.experiment
            && experiment.experiment_id.trim().is_empty()
        {
            errors.push("experiment.experiment_id cannot be empty".into());
        }

        if let Some(policy) = &self.policy {
            if policy.policy_id.trim().is_empty() {
                errors.push("policy.policy_id cannot be empty".into());
            }
            if policy.marker.trim().is_empty() {
                errors.push("policy.marker cannot be empty".into());
            }
            if policy.caps.values().any(|tokens| *tokens == 0) {
                errors.push("policy caps must be greater than zero".into());
            }
        }

        if let Some(quality) = &self.quality {
            if quality
                .program
                .as_deref()
                .is_some_and(|program| program.trim().is_empty())
            {
                errors.push("quality.program cannot be empty when provided".into());
            }
            if quality.required && quality.program.is_none() {
                errors.push("quality.program is required when quality.required=true".into());
            }
        }

        errors
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RunnerSourceReport {
    pub requested_source: String,
    pub requested_kind: SourceKind,
    pub requested_health: SourceHealth,
    pub fallback_recovered: bool,
    pub root_session: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct RunnerArtifactPaths {
    pub normalized_request: String,
    #[serde(default)]
    pub measurement_contract: Option<String>,
    pub source_health: String,
    pub session_evidence: String,
    pub run_group: String,
    pub workspace_resolution: String,
    pub workspace_before_snapshot: Option<String>,
    pub workspace_after_snapshot: Option<String>,
    pub workspace_diff: Option<String>,
    pub policy_evidence: Option<String>,
    pub quality_gate: Option<String>,
    pub validity_input: Option<String>,
    pub validity_report: Option<String>,
    pub recovery_report: String,
    pub runner_result: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RunnerResult {
    pub schema_version: u64,
    pub run_id: String,
    pub pipeline_status: RunnerPipelineStatus,
    pub source_requested: String,
    pub source_health_status: SourceHealthStatus,
    pub fallback_recovered: bool,
    pub root_session: String,
    pub evidence_dir: String,
    pub selected_workspace: Option<String>,
    pub workspace_diff_added_count: Option<u64>,
    pub workspace_diff_modified_count: Option<u64>,
    pub workspace_diff_removed_count: Option<u64>,
    pub workspace_diff_targets_selected_workspace: Option<bool>,
    pub root_terminal: TerminalStatus,
    pub agent_count: u64,
    pub policy_required: bool,
    pub policy_observation_status: Option<PolicyObservationStatus>,
    pub policy_enforcement_status: Option<PolicyEnforcementStatus>,
    pub quality_required: Option<bool>,
    pub quality_status: Option<RunnerQualityStatus>,
    pub validity_verdict: Option<ExperimentValidityVerdict>,
    pub causal_claims_allowed: Option<bool>,
    pub descriptive_metrics_allowed: Option<bool>,
    pub recovery_status: RunnerRecoveryStatus,
    pub completed_steps: Vec<String>,
    pub pending_steps: Vec<String>,
    pub warnings: Vec<String>,
    pub artifacts: RunnerArtifactPaths,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn request() -> RunnerRequest {
        RunnerRequest {
            schema_version: RUNNER_REQUEST_SCHEMA_VERSION,
            run_id: "fixture-run".into(),
            source: "fixture.jsonl".into(),
            session_candidates: vec!["child-a.jsonl".into(), "child-b.jsonl".into()],
            source_root: "E:/fixture/source".into(),
            after_inventory: "after.json".into(),
            before_inventory: Some("before.json".into()),
            before_snapshot: Some("before-snapshot.json".into()),
            after_snapshot: Some("after-snapshot.json".into()),
            expected_outputs: vec![],
            evidence_dir: "evidence".into(),
            experiment: Some(RunnerExperimentRequest {
                experiment_id: "002-instrumentation".into(),
                intent: ExperimentIntent::Instrumentation,
                validity: RunnerValidityHints::default(),
            }),
            policy: Some(RunnerPolicyRequest {
                policy_id: "fixture-policy".into(),
                marker: "fixture marker".into(),
                policy_paths: vec!["E:/fixture/source/AGENTS.md".into()],
                caps: BTreeMap::from([("file_read".into(), 5000)]),
                enforcement: PolicyEnforcementStatus::NotProven,
            }),
            quality: Some(RunnerQualityRequest {
                required: true,
                program: Some("fixture-quality".into()),
                args: vec!["--check".into()],
            }),
        }
    }

    #[test]
    fn valid_request_has_no_validation_errors() {
        assert!(request().validation_errors().is_empty());
    }

    #[test]
    fn rejects_unknown_schema_and_empty_required_fields() {
        let mut value = request();
        value.schema_version = 99;
        value.source.clear();

        let errors = value.validation_errors();
        assert!(errors.iter().any(|item| item.contains("schema_version")));
        assert!(
            errors
                .iter()
                .any(|item| item.contains("source cannot be empty"))
        );
    }

    #[test]
    fn request_round_trip_is_stable() {
        let original = request();
        let json = serde_json::to_string_pretty(&original).expect("serialize request");
        let parsed: RunnerRequest = serde_json::from_str(&json).expect("parse request");
        assert_eq!(parsed, original);
    }
}
