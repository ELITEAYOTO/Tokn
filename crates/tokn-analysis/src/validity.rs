use tokn_domain::{
    ExperimentIntent, ExperimentValidityInput, ExperimentValidityReport, ExperimentValidityVerdict,
    SourceHealthStatus, TerminalStatus, ValidityCheckStatus, ValidityReason,
};

pub fn reduce_experiment_validity(input: &ExperimentValidityInput) -> ExperimentValidityReport {
    let mut reasons = Vec::new();

    let source_usable = match input.capture.source_health {
        SourceHealthStatus::Healthy => true,
        SourceHealthStatus::Partial if input.capture.fallback_recovered => true,
        _ => false,
    };

    if !source_usable {
        reasons.push(reason(
            "CAPTURE_SOURCE_UNUSABLE",
            "capture",
            ValidityCheckStatus::Fail,
            format!(
                "source health {} is not sufficient and no usable fallback was recovered",
                input.capture.source_health.as_str()
            ),
        ));
    } else if input.capture.source_health == SourceHealthStatus::Partial {
        reasons.push(reason(
            "CAPTURE_FALLBACK_RECOVERED",
            "capture",
            ValidityCheckStatus::Pass,
            "partial primary source was recovered through fallback evidence",
        ));
    }

    record_check(
        &mut reasons,
        "CAPTURE_MODEL_USAGE",
        "capture",
        input.capture.model_usage,
        "model usage evidence",
    );
    record_check(
        &mut reasons,
        "CAPTURE_TOOL_EVIDENCE",
        "capture",
        input.capture.tool_evidence,
        "tool evidence",
    );
    record_check(
        &mut reasons,
        "CAPTURE_RUN_GROUP",
        "capture",
        input.capture.run_group_resolved,
        "parent/subagent RunGroup resolution",
    );
    record_check(
        &mut reasons,
        "TASK_IDENTITY",
        "task",
        input.task.exact_task_captured,
        "exact task identity",
    );
    record_check(
        &mut reasons,
        "WORKSPACE_OUTPUT",
        "workspace",
        input.workspace.output_workspace_resolved,
        "actual output workspace resolution",
    );
    record_check(
        &mut reasons,
        "WORKSPACE_BEFORE",
        "workspace",
        input.workspace.before_state_captured,
        "before-workspace state",
    );
    record_check(
        &mut reasons,
        "WORKSPACE_AFTER",
        "workspace",
        input.workspace.after_state_captured,
        "after-workspace state",
    );
    record_check(
        &mut reasons,
        "WORKSPACE_QUALITY_TARGET",
        "workspace",
        input.workspace.quality_gate_on_output,
        "quality gate executed on the resolved output workspace",
    );

    if input.policy.required {
        record_check(
            &mut reasons,
            "POLICY_IDENTITY",
            "policy",
            input.policy.identity_captured,
            "policy identity",
        );
        record_check(
            &mut reasons,
            "POLICY_EXPOSURE",
            "policy",
            input.policy.exposure_known,
            "policy exposure",
        );
        record_check(
            &mut reasons,
            "POLICY_COMPLIANCE_EVIDENCE",
            "policy",
            input.policy.compliance_evidence,
            "policy compliance evidence",
        );
        record_check(
            &mut reasons,
            "POLICY_ENFORCEMENT_LABEL",
            "policy",
            input.policy.enforcement_accurately_labeled,
            "policy enforcement classification",
        );
    }

    record_check(
        &mut reasons,
        "RUNTIME_MODEL_RECORDED",
        "runtime",
        input.runtime.model_recorded,
        "model identity",
    );
    record_check(
        &mut reasons,
        "RUNTIME_VERSION_RECORDED",
        "runtime",
        input.runtime.runtime_recorded,
        "runtime version",
    );
    record_check(
        &mut reasons,
        "RUNTIME_CONFIG_RECORDED",
        "runtime",
        input.runtime.configuration_recorded,
        "important runtime configuration",
    );
    record_check(
        &mut reasons,
        "QUALITY_AUTOMATED_GATES",
        "quality",
        input.quality.automated_gates,
        "automated quality gates",
    );
    record_check(
        &mut reasons,
        "QUALITY_HOST_GATES",
        "quality",
        input.quality.human_or_host_gates,
        "required human or host quality gates",
    );

    if input.task.completion_required && input.task.terminal_status != TerminalStatus::Completed {
        reasons.push(reason(
            "TASK_NOT_COMPLETED",
            "task",
            ValidityCheckStatus::Fail,
            format!(
                "required task completion was not achieved: {}",
                input.task.terminal_status.as_str()
            ),
        ));
    }

    let capture_usable = source_usable
        && input.capture.model_usage == ValidityCheckStatus::Pass
        && input.capture.run_group_resolved == ValidityCheckStatus::Pass;

    let verdict = if !capture_usable {
        ExperimentValidityVerdict::InvalidCapture
    } else if input.intent == ExperimentIntent::Instrumentation {
        ExperimentValidityVerdict::InstrumentationOnly
    } else if input.task.completion_required
        && input.task.terminal_status != TerminalStatus::Completed
    {
        ExperimentValidityVerdict::IncompleteTask
    } else if input.intent == ExperimentIntent::DescriptiveComparison {
        ExperimentValidityVerdict::ValidForDescriptiveComparison
    } else if causal_requirements_satisfied(input, source_usable) {
        ExperimentValidityVerdict::ValidForCausalAb
    } else {
        record_causal_reasons(input, &mut reasons);
        ExperimentValidityVerdict::ValidForDescriptiveComparison
    };

    ExperimentValidityReport {
        experiment_id: input.experiment_id.clone(),
        intent: input.intent,
        causal_claims_allowed: verdict == ExperimentValidityVerdict::ValidForCausalAb,
        descriptive_metrics_allowed: verdict != ExperimentValidityVerdict::InvalidCapture,
        verdict,
        reasons,
    }
}

fn causal_requirements_satisfied(input: &ExperimentValidityInput, source_usable: bool) -> bool {
    source_usable
        && input.capture.model_usage == ValidityCheckStatus::Pass
        && input.capture.tool_evidence.is_satisfied()
        && input.capture.run_group_resolved == ValidityCheckStatus::Pass
        && input.task.exact_task_captured == ValidityCheckStatus::Pass
        && (!input.task.completion_required
            || input.task.terminal_status == TerminalStatus::Completed)
        && all_satisfied([
            input.workspace.output_workspace_resolved,
            input.workspace.before_state_captured,
            input.workspace.after_state_captured,
            input.workspace.quality_gate_on_output,
            input.runtime.model_recorded,
            input.runtime.runtime_recorded,
            input.runtime.configuration_recorded,
            input.runtime.comparable_to_baseline,
            input.quality.automated_gates,
            input.quality.human_or_host_gates,
            input.causal.baseline_available,
            input.causal.same_task,
            input.causal.same_starting_workspace,
            input.causal.same_runtime_model_config,
            input.causal.single_primary_variable,
        ])
        && (!input.policy.required
            || all_satisfied([
                input.policy.identity_captured,
                input.policy.exposure_known,
                input.policy.compliance_evidence,
                input.policy.enforcement_accurately_labeled,
            ]))
}

fn record_causal_reasons(input: &ExperimentValidityInput, reasons: &mut Vec<ValidityReason>) {
    let checks = [
        (
            "CAUSAL_BASELINE",
            input.causal.baseline_available,
            "baseline run availability",
        ),
        (
            "CAUSAL_SAME_TASK",
            input.causal.same_task,
            "same frozen task",
        ),
        (
            "CAUSAL_SAME_START_WORKSPACE",
            input.causal.same_starting_workspace,
            "same starting workspace",
        ),
        (
            "CAUSAL_RUNTIME_COMPARABLE",
            input.causal.same_runtime_model_config,
            "same runtime/model configuration",
        ),
        (
            "CAUSAL_SINGLE_VARIABLE",
            input.causal.single_primary_variable,
            "single primary intervention variable",
        ),
        (
            "RUNTIME_BASELINE_COMPARABLE",
            input.runtime.comparable_to_baseline,
            "runtime comparability to baseline",
        ),
    ];
    for (code, status, label) in checks {
        record_check(reasons, code, "causal", status, label);
    }
}

fn all_satisfied<const N: usize>(checks: [ValidityCheckStatus; N]) -> bool {
    checks.into_iter().all(ValidityCheckStatus::is_satisfied)
}

fn record_check(
    reasons: &mut Vec<ValidityReason>,
    code: &str,
    dimension: &str,
    status: ValidityCheckStatus,
    label: &str,
) {
    if status.is_satisfied() {
        return;
    }
    reasons.push(reason(
        code,
        dimension,
        status,
        format!("{label} is {}", status.as_str()),
    ));
}

fn reason(
    code: impl Into<String>,
    dimension: impl Into<String>,
    status: ValidityCheckStatus,
    message: impl Into<String>,
) -> ValidityReason {
    ValidityReason {
        code: code.into(),
        dimension: dimension.into(),
        status,
        message: message.into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokn_domain::{
        CaptureValidityInput, CausalControlsInput, PolicyValidityInput, QualityValidityInput,
        RuntimeValidityInput, TaskValidityInput, WorkspaceValidityInput,
    };

    fn pass() -> ValidityCheckStatus {
        ValidityCheckStatus::Pass
    }

    fn complete_input(intent: ExperimentIntent) -> ExperimentValidityInput {
        ExperimentValidityInput {
            experiment_id: "test".into(),
            intent,
            capture: CaptureValidityInput {
                model_usage: pass(),
                tool_evidence: pass(),
                source_health: SourceHealthStatus::Healthy,
                fallback_recovered: false,
                run_group_resolved: pass(),
            },
            task: TaskValidityInput {
                exact_task_captured: pass(),
                terminal_status: TerminalStatus::Completed,
                completion_required: true,
            },
            workspace: WorkspaceValidityInput {
                output_workspace_resolved: pass(),
                before_state_captured: pass(),
                after_state_captured: pass(),
                quality_gate_on_output: pass(),
            },
            policy: PolicyValidityInput {
                required: true,
                identity_captured: pass(),
                exposure_known: pass(),
                compliance_evidence: pass(),
                enforcement_accurately_labeled: pass(),
            },
            runtime: RuntimeValidityInput {
                model_recorded: pass(),
                runtime_recorded: pass(),
                configuration_recorded: pass(),
                comparable_to_baseline: pass(),
            },
            quality: QualityValidityInput {
                automated_gates: pass(),
                human_or_host_gates: ValidityCheckStatus::NotRequired,
            },
            causal: CausalControlsInput {
                baseline_available: pass(),
                same_task: pass(),
                same_starting_workspace: pass(),
                same_runtime_model_config: pass(),
                single_primary_variable: pass(),
            },
        }
    }

    #[test]
    fn complete_causal_ab_allows_causal_claims() {
        let report = reduce_experiment_validity(&complete_input(ExperimentIntent::CausalAb));
        assert_eq!(report.verdict, ExperimentValidityVerdict::ValidForCausalAb);
        assert!(report.causal_claims_allowed);
        assert!(report.descriptive_metrics_allowed);
    }

    #[test]
    fn causal_mismatch_degrades_to_descriptive() {
        let mut input = complete_input(ExperimentIntent::CausalAb);
        input.causal.same_runtime_model_config = ValidityCheckStatus::Fail;
        let report = reduce_experiment_validity(&input);
        assert_eq!(
            report.verdict,
            ExperimentValidityVerdict::ValidForDescriptiveComparison
        );
        assert!(!report.causal_claims_allowed);
        assert!(
            report
                .reasons
                .iter()
                .any(|item| item.code == "CAUSAL_RUNTIME_COMPARABLE")
        );
    }

    #[test]
    fn unusable_capture_blocks_even_instrumentation_metrics() {
        let mut input = complete_input(ExperimentIntent::Instrumentation);
        input.capture.model_usage = ValidityCheckStatus::Unknown;
        let report = reduce_experiment_validity(&input);
        assert_eq!(report.verdict, ExperimentValidityVerdict::InvalidCapture);
        assert!(!report.descriptive_metrics_allowed);
    }

    #[test]
    fn incomplete_required_task_is_explicit() {
        let mut input = complete_input(ExperimentIntent::CausalAb);
        input.task.terminal_status = TerminalStatus::IncompleteUsageLimit;
        let report = reduce_experiment_validity(&input);
        assert_eq!(report.verdict, ExperimentValidityVerdict::IncompleteTask);
        assert!(!report.causal_claims_allowed);
        assert!(report.descriptive_metrics_allowed);
    }

    #[test]
    fn serialized_validity_fixtures_have_stable_verdicts() {
        let cases = [
            (
                include_str!("../../../fixtures/experiments/validity/causal-valid.json"),
                ExperimentValidityVerdict::ValidForCausalAb,
            ),
            (
                include_str!("../../../fixtures/experiments/validity/descriptive-only.json"),
                ExperimentValidityVerdict::ValidForDescriptiveComparison,
            ),
            (
                include_str!("../../../fixtures/experiments/validity/incomplete-task.json"),
                ExperimentValidityVerdict::IncompleteTask,
            ),
            (
                include_str!("../../../fixtures/experiments/validity/invalid-capture.json"),
                ExperimentValidityVerdict::InvalidCapture,
            ),
        ];

        for (json, expected) in cases {
            let input: ExperimentValidityInput =
                serde_json::from_str(json).expect("valid validity fixture");
            assert_eq!(reduce_experiment_validity(&input).verdict, expected);
        }
    }

    #[test]
    fn experiment_001_golden_is_instrumentation_only() {
        let input: ExperimentValidityInput = serde_json::from_str(include_str!(
            "../../../fixtures/experiments/001-validity.json"
        ))
        .expect("valid Experiment 001 validity fixture");

        let report = reduce_experiment_validity(&input);
        assert_eq!(
            report.verdict,
            ExperimentValidityVerdict::InstrumentationOnly
        );
        assert!(!report.causal_claims_allowed);
        assert!(report.descriptive_metrics_allowed);
        assert!(
            report
                .reasons
                .iter()
                .any(|item| item.code == "CAPTURE_FALLBACK_RECOVERED")
        );
    }
}
