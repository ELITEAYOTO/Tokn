use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use tokn_domain::{
    MODEL_RUNTIME_PROFILE_SCHEMA_VERSION, ModelRuntimeProfile, RuntimeConfigurationProfile,
    ValidityCheckStatus,
};

pub const RUNTIME_PROFILE_COMPATIBILITY_SCHEMA_VERSION: u64 = 1;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RuntimeCompatibilityCheck {
    pub code: String,
    pub status: ValidityCheckStatus,
    pub required_for_causal: bool,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RuntimeProfileCompatibilityReport {
    pub schema_version: u64,
    pub status: ValidityCheckStatus,
    pub causal_compatible: bool,
    pub checks: Vec<RuntimeCompatibilityCheck>,
}

pub fn reduce_runtime_profile_compatibility(
    baseline: Option<&ModelRuntimeProfile>,
    candidate: Option<&ModelRuntimeProfile>,
) -> RuntimeProfileCompatibilityReport {
    let (Some(baseline), Some(candidate)) = (baseline, candidate) else {
        return report(vec![check(
            "PROFILE_PRESENT",
            ValidityCheckStatus::Unknown,
            true,
            "baseline and candidate runtime profiles are both required",
        )]);
    };

    let schema_status = if baseline.schema_version == MODEL_RUNTIME_PROFILE_SCHEMA_VERSION
        && candidate.schema_version == MODEL_RUNTIME_PROFILE_SCHEMA_VERSION
    {
        ValidityCheckStatus::Pass
    } else {
        ValidityCheckStatus::Unknown
    };

    let mut checks = vec![check(
        "PROFILE_SCHEMA",
        schema_status,
        true,
        "both runtime profiles must use the supported schema",
    )];

    if schema_status != ValidityCheckStatus::Pass
        || !baseline.validation_errors().is_empty()
        || !candidate.validation_errors().is_empty()
    {
        checks.push(check(
            "PROFILE_VALID",
            ValidityCheckStatus::Unknown,
            true,
            "invalid or unsupported profile evidence cannot prove compatibility",
        ));
        return report(checks);
    }
    checks.push(required_str_check(
        "RUNTIME_KIND",
        Some(&baseline.runtime_kind),
        Some(&candidate.runtime_kind),
    ));
    checks.push(required_str_check(
        "RUNTIME_VERSION",
        baseline.runtime_version.as_deref(),
        candidate.runtime_version.as_deref(),
    ));
    checks.push(optional_str_check(
        "APP_VERSION",
        baseline.app_version.as_deref(),
        candidate.app_version.as_deref(),
    ));
    checks.push(required_str_check(
        "MODEL",
        baseline.model.as_deref(),
        candidate.model.as_deref(),
    ));
    checks.push(required_str_check(
        "MODEL_PROVIDER",
        baseline.model_provider.as_deref(),
        candidate.model_provider.as_deref(),
    ));
    checks.push(required_str_check(
        "REASONING_EFFORT",
        baseline.reasoning_effort.as_deref(),
        candidate.reasoning_effort.as_deref(),
    ));
    checks.push(optional_u64_check(
        "MODEL_CONTEXT_WINDOW",
        baseline.model_context_window,
        candidate.model_context_window,
    ));
    checks.push(optional_str_check(
        "MULTI_AGENT_PROTOCOL_VERSION",
        baseline.multi_agent_protocol_version.as_deref(),
        candidate.multi_agent_protocol_version.as_deref(),
    ));

    let configuration_complete = baseline.configuration_complete == ValidityCheckStatus::Pass
        && candidate.configuration_complete == ValidityCheckStatus::Pass;
    checks.push(check(
        "CONFIGURATION_COMPLETE",
        if configuration_complete {
            ValidityCheckStatus::Pass
        } else {
            ValidityCheckStatus::Unknown
        },
        true,
        "both profiles must explicitly record a complete runtime configuration",
    ));
    checks.extend(configuration_checks(
        &baseline.configuration,
        &candidate.configuration,
        configuration_complete,
    ));

    report(checks)
}

fn configuration_checks(
    baseline: &RuntimeConfigurationProfile,
    candidate: &RuntimeConfigurationProfile,
    complete: bool,
) -> Vec<RuntimeCompatibilityCheck> {
    vec![
        configuration_option_check(
            "CONFIG_PERMISSION_PROFILE",
            baseline.permission_profile.as_deref(),
            candidate.permission_profile.as_deref(),
            complete,
        ),
        configuration_option_check(
            "CONFIG_APPROVAL_POLICY",
            baseline.approval_policy.as_deref(),
            candidate.approval_policy.as_deref(),
            complete,
        ),
        configuration_option_check(
            "CONFIG_SANDBOX_POLICY",
            baseline.sandbox_policy.as_deref(),
            candidate.sandbox_policy.as_deref(),
            complete,
        ),
        configuration_option_check(
            "CONFIG_COLLABORATION_MODE",
            baseline.collaboration_mode.as_deref(),
            candidate.collaboration_mode.as_deref(),
            complete,
        ),
        configuration_option_check(
            "CONFIG_CONTEXT_MANAGEMENT_MODE",
            baseline.context_management_mode.as_deref(),
            candidate.context_management_mode.as_deref(),
            complete,
        ),
        configuration_map_check(
            "CONFIG_FEATURE_FLAGS",
            &baseline.feature_flags,
            &candidate.feature_flags,
            complete,
        ),
    ]
}

fn required_str_check(
    code: &str,
    baseline: Option<&str>,
    candidate: Option<&str>,
) -> RuntimeCompatibilityCheck {
    let status = match (baseline.map(str::trim), candidate.map(str::trim)) {
        (Some(left), Some(right)) if left == right => ValidityCheckStatus::Pass,
        (Some(_), Some(_)) => ValidityCheckStatus::Fail,
        _ => ValidityCheckStatus::Unknown,
    };
    check(code, status, true, "required runtime evidence comparison")
}

fn optional_str_check(
    code: &str,
    baseline: Option<&str>,
    candidate: Option<&str>,
) -> RuntimeCompatibilityCheck {
    match (baseline.map(str::trim), candidate.map(str::trim)) {
        (None, None) => check(
            code,
            ValidityCheckStatus::NotRequired,
            false,
            "optional evidence absent from both profiles",
        ),
        (Some(left), Some(right)) if left == right => check(
            code,
            ValidityCheckStatus::Pass,
            true,
            "optional evidence is present and equal",
        ),
        (Some(_), Some(_)) => check(
            code,
            ValidityCheckStatus::Fail,
            true,
            "optional evidence is present and differs",
        ),
        _ => check(
            code,
            ValidityCheckStatus::Unknown,
            true,
            "optional evidence is present on only one side",
        ),
    }
}

fn optional_u64_check(
    code: &str,
    baseline: Option<u64>,
    candidate: Option<u64>,
) -> RuntimeCompatibilityCheck {
    match (baseline, candidate) {
        (None, None) => check(
            code,
            ValidityCheckStatus::NotRequired,
            false,
            "optional evidence absent from both profiles",
        ),
        (Some(left), Some(right)) if left == right => check(
            code,
            ValidityCheckStatus::Pass,
            true,
            "optional evidence is present and equal",
        ),
        (Some(_), Some(_)) => check(
            code,
            ValidityCheckStatus::Fail,
            true,
            "optional evidence is present and differs",
        ),
        _ => check(
            code,
            ValidityCheckStatus::Unknown,
            true,
            "optional evidence is present on only one side",
        ),
    }
}

fn configuration_option_check(
    code: &str,
    baseline: Option<&str>,
    candidate: Option<&str>,
    complete: bool,
) -> RuntimeCompatibilityCheck {
    let status = if complete {
        if baseline == candidate {
            ValidityCheckStatus::Pass
        } else {
            ValidityCheckStatus::Fail
        }
    } else {
        match (baseline, candidate) {
            (Some(left), Some(right)) if left != right => ValidityCheckStatus::Fail,
            _ => ValidityCheckStatus::Unknown,
        }
    };
    check(
        code,
        status,
        true,
        "runtime configuration values must be equal with complete evidence",
    )
}

fn configuration_map_check(
    code: &str,
    baseline: &BTreeMap<String, String>,
    candidate: &BTreeMap<String, String>,
    complete: bool,
) -> RuntimeCompatibilityCheck {
    let status = if complete {
        if baseline == candidate {
            ValidityCheckStatus::Pass
        } else {
            ValidityCheckStatus::Fail
        }
    } else if !baseline.is_empty() && !candidate.is_empty() && baseline != candidate {
        ValidityCheckStatus::Fail
    } else {
        ValidityCheckStatus::Unknown
    };
    check(
        code,
        status,
        true,
        "runtime feature flags must be equal with complete evidence",
    )
}

fn check(
    code: &str,
    status: ValidityCheckStatus,
    required_for_causal: bool,
    message: &str,
) -> RuntimeCompatibilityCheck {
    RuntimeCompatibilityCheck {
        code: code.into(),
        status,
        required_for_causal,
        message: message.into(),
    }
}

fn report(checks: Vec<RuntimeCompatibilityCheck>) -> RuntimeProfileCompatibilityReport {
    let status = if checks
        .iter()
        .any(|item| item.status == ValidityCheckStatus::Fail)
    {
        ValidityCheckStatus::Fail
    } else if checks
        .iter()
        .any(|item| item.required_for_causal && item.status == ValidityCheckStatus::Unknown)
    {
        ValidityCheckStatus::Unknown
    } else {
        ValidityCheckStatus::Pass
    };

    RuntimeProfileCompatibilityReport {
        schema_version: RUNTIME_PROFILE_COMPATIBILITY_SCHEMA_VERSION,
        status,
        causal_compatible: status == ValidityCheckStatus::Pass,
        checks,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn profile() -> ModelRuntimeProfile {
        ModelRuntimeProfile {
            schema_version: MODEL_RUNTIME_PROFILE_SCHEMA_VERSION,
            observed_at: "2026-10-02T20:00:00+02:00".into(),
            runtime_kind: "codex".into(),
            runtime_version: Some("codex-cli 0.161.0-alpha.2".into()),
            app_version: None,
            model: Some("gpt-6.1-sol".into()),
            model_provider: Some("openai".into()),
            reasoning_effort: Some("high".into()),
            model_context_window: None,
            multi_agent_protocol_version: None,
            configuration: RuntimeConfigurationProfile {
                permission_profile: Some("workspace-write".into()),
                approval_policy: Some("on-request".into()),
                sandbox_policy: Some("workspace-write".into()),
                collaboration_mode: Some("multi-agent".into()),
                context_management_mode: Some("default".into()),
                feature_flags: BTreeMap::from([("feature-a".into(), "enabled".into())]),
            },
            configuration_complete: ValidityCheckStatus::Pass,
            capabilities: BTreeMap::new(),
        }
    }

    #[test]
    fn identical_complete_profiles_pass() {
        let baseline = profile();
        let candidate = baseline.clone();
        let result = reduce_runtime_profile_compatibility(Some(&baseline), Some(&candidate));
        assert_eq!(result.status, ValidityCheckStatus::Pass);
        assert!(result.causal_compatible);
    }

    #[test]
    fn model_mismatch_fails_even_if_another_required_field_is_unknown() {
        let baseline = profile();
        let mut candidate = profile();
        candidate.model = Some("different-model".into());
        candidate.reasoning_effort = None;

        let result = reduce_runtime_profile_compatibility(Some(&baseline), Some(&candidate));
        assert_eq!(result.status, ValidityCheckStatus::Fail);
        assert!(!result.causal_compatible);
    }

    #[test]
    fn missing_required_evidence_is_unknown() {
        let baseline = profile();
        let mut candidate = profile();
        candidate.reasoning_effort = None;

        let result = reduce_runtime_profile_compatibility(Some(&baseline), Some(&candidate));
        assert_eq!(result.status, ValidityCheckStatus::Unknown);
    }

    #[test]
    fn incomplete_configuration_is_unknown() {
        let baseline = profile();
        let mut candidate = profile();
        candidate.configuration_complete = ValidityCheckStatus::Unknown;

        let result = reduce_runtime_profile_compatibility(Some(&baseline), Some(&candidate));
        assert_eq!(result.status, ValidityCheckStatus::Unknown);
    }

    #[test]
    fn observed_configuration_difference_fails() {
        let baseline = profile();
        let mut candidate = profile();
        candidate.configuration.context_management_mode = Some("compact".into());

        let result = reduce_runtime_profile_compatibility(Some(&baseline), Some(&candidate));
        assert_eq!(result.status, ValidityCheckStatus::Fail);
    }

    #[test]
    fn optional_evidence_absent_from_both_sides_does_not_block() {
        let baseline = profile();
        let candidate = profile();

        let result = reduce_runtime_profile_compatibility(Some(&baseline), Some(&candidate));
        let app = result
            .checks
            .iter()
            .find(|item| item.code == "APP_VERSION")
            .expect("app version check");
        assert_eq!(app.status, ValidityCheckStatus::NotRequired);
        assert_eq!(result.status, ValidityCheckStatus::Pass);
    }

    #[test]
    fn asymmetric_optional_evidence_is_unknown() {
        let baseline = profile();
        let mut candidate = profile();
        candidate.app_version = Some("26.928.1915.0".into());

        let result = reduce_runtime_profile_compatibility(Some(&baseline), Some(&candidate));
        assert_eq!(result.status, ValidityCheckStatus::Unknown);
    }

    #[test]
    fn missing_profile_is_unknown() {
        let candidate = profile();
        let result = reduce_runtime_profile_compatibility(None, Some(&candidate));
        assert_eq!(result.status, ValidityCheckStatus::Unknown);
        assert!(!result.causal_compatible);
    }

    #[test]
    fn unsupported_profile_schema_is_unknown() {
        let baseline = profile();
        let mut candidate = profile();
        candidate.schema_version = 99;

        let result = reduce_runtime_profile_compatibility(Some(&baseline), Some(&candidate));
        assert_eq!(result.status, ValidityCheckStatus::Unknown);
    }

    #[test]
    fn feature_flag_difference_fails() {
        let baseline = profile();
        let mut candidate = profile();
        candidate
            .configuration
            .feature_flags
            .insert("feature-a".into(), "disabled".into());

        let result = reduce_runtime_profile_compatibility(Some(&baseline), Some(&candidate));
        assert_eq!(result.status, ValidityCheckStatus::Fail);
    }
}
