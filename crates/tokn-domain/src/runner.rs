use serde::{Deserialize, Serialize};

use crate::{ExperimentIntent, TerminalStatus};

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
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RunnerRequest {
    pub schema_version: u64,
    pub run_id: String,
    pub source: String,
    pub source_root: String,
    pub after_inventory: String,
    #[serde(default)]
    pub before_inventory: Option<String>,
    #[serde(default)]
    pub expected_outputs: Vec<String>,
    pub evidence_dir: String,
    #[serde(default)]
    pub experiment: Option<RunnerExperimentRequest>,
}

impl RunnerRequest {
    pub fn validation_errors(&self) -> Vec<String> {
        let mut errors = Vec::new();

        if self.schema_version != 1 {
            errors.push(format!(
                "unsupported runner request schema_version {}; expected 1",
                self.schema_version
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

        if let Some(experiment) = &self.experiment
            && experiment.experiment_id.trim().is_empty()
        {
            errors.push("experiment.experiment_id cannot be empty".into());
        }

        errors
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct RunnerArtifactPaths {
    pub normalized_request: String,
    pub run_group: String,
    pub workspace_resolution: String,
    pub runner_result: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RunnerResult {
    pub schema_version: u64,
    pub run_id: String,
    pub pipeline_status: RunnerPipelineStatus,
    pub source_requested: String,
    pub root_session: String,
    pub evidence_dir: String,
    pub selected_workspace: Option<String>,
    pub root_terminal: TerminalStatus,
    pub agent_count: u64,
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
            schema_version: 1,
            run_id: "fixture-run".into(),
            source: "fixture.jsonl".into(),
            source_root: "E:/fixture/source".into(),
            after_inventory: "after.json".into(),
            before_inventory: Some("before.json".into()),
            expected_outputs: vec![],
            evidence_dir: "evidence".into(),
            experiment: Some(RunnerExperimentRequest {
                experiment_id: "002-instrumentation".into(),
                intent: ExperimentIntent::Instrumentation,
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
