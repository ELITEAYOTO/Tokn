use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::{MODEL_RUNTIME_PROFILE_SCHEMA_VERSION, SourceKind, ValidityCheckStatus};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum CapabilityEvidenceStatus {
    Observed,
    Unsupported,
    #[default]
    Unknown,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CapabilityObservation {
    pub status: CapabilityEvidenceStatus,
    #[serde(default)]
    pub source_kind: Option<SourceKind>,
    #[serde(default)]
    pub runtime_version: Option<String>,
    #[serde(default)]
    pub first_observed_at: Option<String>,
    #[serde(default)]
    pub last_observed_at: Option<String>,
    #[serde(default)]
    pub note: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RuntimeConfigurationProfile {
    #[serde(default)]
    pub permission_profile: Option<String>,
    #[serde(default)]
    pub approval_policy: Option<String>,
    #[serde(default)]
    pub sandbox_policy: Option<String>,
    #[serde(default)]
    pub collaboration_mode: Option<String>,
    #[serde(default)]
    pub context_management_mode: Option<String>,
    #[serde(default)]
    pub feature_flags: BTreeMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ModelRuntimeProfile {
    pub schema_version: u64,
    pub observed_at: String,
    pub runtime_kind: String,
    #[serde(default)]
    pub runtime_version: Option<String>,
    #[serde(default)]
    pub app_version: Option<String>,
    #[serde(default)]
    pub model: Option<String>,
    #[serde(default)]
    pub model_provider: Option<String>,
    #[serde(default)]
    pub reasoning_effort: Option<String>,
    #[serde(default)]
    pub model_context_window: Option<u64>,
    #[serde(default)]
    pub multi_agent_protocol_version: Option<String>,
    #[serde(default)]
    pub configuration: RuntimeConfigurationProfile,
    pub configuration_complete: ValidityCheckStatus,
    #[serde(default)]
    pub capabilities: BTreeMap<String, CapabilityObservation>,
}

impl ModelRuntimeProfile {
    pub fn validation_errors(&self) -> Vec<String> {
        let mut errors = Vec::new();

        if self.schema_version != MODEL_RUNTIME_PROFILE_SCHEMA_VERSION {
            errors.push(format!(
                "unsupported ModelRuntimeProfile schema_version {}; expected {}",
                self.schema_version, MODEL_RUNTIME_PROFILE_SCHEMA_VERSION
            ));
        }
        if self.observed_at.trim().is_empty() {
            errors.push("observed_at cannot be empty".into());
        }
        if self.runtime_kind.trim().is_empty() {
            errors.push("runtime_kind cannot be empty".into());
        }

        for (name, value) in [
            ("runtime_version", self.runtime_version.as_deref()),
            ("app_version", self.app_version.as_deref()),
            ("model", self.model.as_deref()),
            ("model_provider", self.model_provider.as_deref()),
            ("reasoning_effort", self.reasoning_effort.as_deref()),
            (
                "multi_agent_protocol_version",
                self.multi_agent_protocol_version.as_deref(),
            ),
        ] {
            if value.is_some_and(|item| item.trim().is_empty()) {
                errors.push(format!("{name} cannot be empty when provided"));
            }
        }

        errors
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn profile() -> ModelRuntimeProfile {
        ModelRuntimeProfile {
            schema_version: MODEL_RUNTIME_PROFILE_SCHEMA_VERSION,
            observed_at: "2026-10-01T20:00:00+02:00".into(),
            runtime_kind: "codex".into(),
            runtime_version: Some("codex-cli 0.161.0-alpha.2".into()),
            app_version: Some("26.928.1915.0".into()),
            model: Some("gpt-6.1-sol".into()),
            model_provider: Some("openai".into()),
            reasoning_effort: None,
            model_context_window: None,
            multi_agent_protocol_version: None,
            configuration: RuntimeConfigurationProfile::default(),
            configuration_complete: ValidityCheckStatus::Unknown,
            capabilities: BTreeMap::from([(
                "token_usage_record".into(),
                CapabilityObservation {
                    status: CapabilityEvidenceStatus::Observed,
                    source_kind: Some(SourceKind::CodexSession),
                    runtime_version: Some("codex-cli 0.161.0-alpha.2".into()),
                    ..Default::default()
                },
            )]),
        }
    }

    #[test]
    fn incomplete_configuration_stays_unknown() {
        let value = profile();
        assert_eq!(value.configuration_complete, ValidityCheckStatus::Unknown);
        assert!(value.validation_errors().is_empty());
    }

    #[test]
    fn profile_round_trip_is_stable() {
        let original = profile();
        let json = serde_json::to_string_pretty(&original).expect("serialize profile");
        let parsed: ModelRuntimeProfile = serde_json::from_str(&json).expect("parse profile");
        assert_eq!(parsed, original);
    }

    #[test]
    fn rejects_unknown_schema_and_empty_identity() {
        let mut value = profile();
        value.schema_version = 99;
        value.runtime_kind.clear();

        let errors = value.validation_errors();
        assert!(errors.iter().any(|item| item.contains("schema_version")));
        assert!(errors.iter().any(|item| item.contains("runtime_kind")));
    }
}
