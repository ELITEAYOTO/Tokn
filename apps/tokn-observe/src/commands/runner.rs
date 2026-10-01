use std::fs::File;
use std::path::{Path, PathBuf};
use std::process::Command as ProcessCommand;

use tokn_analysis::{
    ProjectSnapshot, WorkspaceInventory, WorkspaceResolutionStatus, build_policy_evidence_report,
    build_run_group, check_cap_policy_tools, diff_project_snapshots, reduce_experiment_validity,
    resolve_workspace,
};
use tokn_codex::diagnostic::{DiagnosticBundle, assess_diagnostic_health};
use tokn_codex::session::{
    assess_session_health, collect_session_group, collect_session_group_from_files,
    inspect_session_policy,
};
use tokn_domain::{
    CaptureValidityInput, CausalControlsInput, ExperimentValidityInput,
    MeasurementContractManifest, PROJECT_SNAPSHOT_SCHEMA_VERSION, PolicyEvidenceReport,
    PolicyObservationStatus, PolicyObservationSummary, PolicyPlacement, PolicyValidityInput,
    QualityValidityInput, RUNNER_RESULT_SCHEMA_VERSION, RunnerArtifactPaths, RunnerPipelineStatus,
    RunnerQualityReport, RunnerQualityRequest, RunnerQualityStatus, RunnerRecoveryReport,
    RunnerRecoveryStatus, RunnerRequest, RunnerResult, RunnerSourceReport, RuntimeValidityInput,
    SourceKind, TaskValidityInput, ValidityCheckStatus, WORKSPACE_INVENTORY_SCHEMA_VERSION,
    WorkspaceValidityInput,
};

use super::common::{
    map_policy_observation_status, resolve_session_root_from_source, resolve_source,
};

pub fn run(request_path: &Path) -> anyhow::Result<()> {
    let request = read_request(request_path)?;
    let validation_errors = request.validation_errors();
    if !validation_errors.is_empty() {
        anyhow::bail!(
            "invalid runner request:\n- {}",
            validation_errors.join("\n- ")
        );
    }

    let evidence_dir = PathBuf::from(&request.evidence_dir);
    std::fs::create_dir_all(&evidence_dir)?;

    let normalized_request_path = evidence_dir.join("runner-request.json");
    let measurement_contract_path = evidence_dir.join("measurement-contract.json");
    let source_health_path = evidence_dir.join("source-health.json");
    let session_evidence_path = evidence_dir.join("session-evidence.json");
    let run_group_path = evidence_dir.join("run-group.json");
    let workspace_resolution_path = evidence_dir.join("workspace-resolution.json");
    let workspace_before_snapshot_path = request
        .before_snapshot
        .as_ref()
        .map(|_| evidence_dir.join("workspace-before-snapshot.json"));
    let workspace_after_snapshot_path = request
        .after_snapshot
        .as_ref()
        .map(|_| evidence_dir.join("workspace-after-snapshot.json"));
    let workspace_diff_path = request
        .before_snapshot
        .as_ref()
        .map(|_| evidence_dir.join("workspace-diff.json"));
    let policy_evidence_path = request
        .policy
        .as_ref()
        .map(|_| evidence_dir.join("policy-evidence.json"));
    let quality_gate_path = request
        .quality
        .as_ref()
        .map(|_| evidence_dir.join("quality-gate.json"));
    let validity_input_path = request
        .experiment
        .as_ref()
        .map(|_| evidence_dir.join("validity-input.json"));
    let validity_report_path = request
        .experiment
        .as_ref()
        .map(|_| evidence_dir.join("validity-report.json"));
    let recovery_report_path = evidence_dir.join("recovery-report.json");
    let runner_result_path = evidence_dir.join("runner-result.json");

    let mut owned_paths = vec![
        normalized_request_path.as_path(),
        measurement_contract_path.as_path(),
        source_health_path.as_path(),
        session_evidence_path.as_path(),
        run_group_path.as_path(),
        workspace_resolution_path.as_path(),
        recovery_report_path.as_path(),
        runner_result_path.as_path(),
    ];
    if let Some(path) = workspace_before_snapshot_path.as_deref() {
        owned_paths.push(path);
    }
    if let Some(path) = workspace_after_snapshot_path.as_deref() {
        owned_paths.push(path);
    }
    if let Some(path) = workspace_diff_path.as_deref() {
        owned_paths.push(path);
    }
    if let Some(path) = policy_evidence_path.as_deref() {
        owned_paths.push(path);
    }
    if let Some(path) = quality_gate_path.as_deref() {
        owned_paths.push(path);
    }
    if let Some(path) = validity_input_path.as_deref() {
        owned_paths.push(path);
    }
    if let Some(path) = validity_report_path.as_deref() {
        owned_paths.push(path);
    }
    let recovery_report = prepare_owned_artifacts(&owned_paths, &runner_result_path)?;

    write_json(&normalized_request_path, &request)?;
    write_json(
        &measurement_contract_path,
        &MeasurementContractManifest::default(),
    )?;
    write_json(&recovery_report_path, &recovery_report)?;

    let source_path = resolve_source(&request.source)?;
    let (requested_kind, requested_health) = if source_path.is_dir() {
        let bundle = DiagnosticBundle::detect(&source_path)
            .ok_or_else(|| anyhow::anyhow!("not a valid diagnostic trace bundle"))?;
        (
            SourceKind::CodexDiagnosticTrace,
            assess_diagnostic_health(&bundle)?,
        )
    } else {
        (
            SourceKind::CodexSession,
            assess_session_health(&source_path)?,
        )
    };

    let root_session = resolve_session_root_from_source(&source_path)?;
    let fallback_recovered = source_path.is_dir();
    let source_report = RunnerSourceReport {
        requested_source: source_path.to_string_lossy().to_string(),
        requested_kind,
        requested_health: requested_health.clone(),
        fallback_recovered,
        root_session: root_session.to_string_lossy().to_string(),
    };
    write_json(&source_health_path, &source_report)?;

    let grouped = if request.session_candidates.is_empty() {
        collect_session_group(&root_session)?
    } else {
        let candidates = request
            .session_candidates
            .iter()
            .map(PathBuf::from)
            .collect::<Vec<_>>();
        collect_session_group_from_files(&root_session, &candidates)?
    };
    write_json(&session_evidence_path, &grouped.members)?;
    let group = build_run_group(&grouped.members, &grouped.root_thread_id)
        .ok_or_else(|| anyhow::anyhow!("failed to build run group"))?;
    write_json(&run_group_path, &group)?;

    let policy_report = if let Some(policy) = &request.policy {
        let placements = policy
            .policy_paths
            .iter()
            .map(|path| PolicyPlacement {
                path: path.clone(),
                sha256: None,
            })
            .collect::<Vec<_>>();

        let mut session_reports = Vec::new();
        for member in &grouped.members {
            session_reports.push(inspect_session_policy(
                Path::new(&member.source_path),
                &policy.marker,
                &policy.policy_paths,
            )?);
        }

        let observed = if policy.caps.is_empty() {
            PolicyObservationSummary::default()
        } else {
            let tools = grouped
                .members
                .iter()
                .flat_map(|member| member.tools.iter().cloned())
                .collect::<Vec<_>>();
            let parse_failures = grouped
                .members
                .iter()
                .map(|member| member.tool_parse_failures)
                .sum();
            let check = check_cap_policy_tools(&tools, parse_failures, &policy.caps);
            PolicyObservationSummary {
                status: map_policy_observation_status(check.status),
                targeted: check.targeted_tools,
                compliant: check.compliant,
                violations: check.violations,
                unknown: check.unknown,
                parse_failures: check.parse_failures,
            }
        };

        let report = build_policy_evidence_report(
            &policy.policy_id,
            &policy.marker,
            placements,
            session_reports,
            observed,
            policy.enforcement,
        );
        write_json(
            policy_evidence_path
                .as_deref()
                .expect("policy evidence path exists when policy is configured"),
            &report,
        )?;
        Some(report)
    } else {
        None
    };

    let after = read_inventory(Path::new(&request.after_inventory))?;
    let before = request
        .before_inventory
        .as_deref()
        .map(Path::new)
        .map(read_inventory)
        .transpose()?;

    let resolution = resolve_workspace(
        before.as_ref(),
        &after,
        &request.source_root,
        &request.expected_outputs,
        &grouped.members,
    );
    write_json(&workspace_resolution_path, &resolution)?;

    let workspace_diff = if let (Some(before_source), Some(after_source)) =
        (&request.before_snapshot, &request.after_snapshot)
    {
        let before_snapshot = read_project_snapshot(Path::new(before_source))?;
        let after_snapshot = read_project_snapshot(Path::new(after_source))?;
        let before_copy = workspace_before_snapshot_path
            .as_deref()
            .expect("before snapshot artifact path exists");
        let after_copy = workspace_after_snapshot_path
            .as_deref()
            .expect("after snapshot artifact path exists");
        write_json(before_copy, &before_snapshot)?;
        write_json(after_copy, &after_snapshot)?;

        let report = diff_project_snapshots(
            &before_snapshot,
            &after_snapshot,
            before_copy.to_string_lossy().to_string(),
            after_copy.to_string_lossy().to_string(),
        );
        write_json(
            workspace_diff_path
                .as_deref()
                .expect("workspace diff path exists when snapshots are configured"),
            &report,
        )?;
        let targets_selected_workspace = resolution
            .selected_root
            .as_deref()
            .is_some_and(|selected| same_path(selected, &after_snapshot.project_root));
        Some((report, targets_selected_workspace))
    } else {
        None
    };

    let quality_report = if let Some(quality) = &request.quality {
        let report = run_quality_gate(quality, resolution.selected_root.as_deref());
        write_json(
            quality_gate_path
                .as_deref()
                .expect("quality gate path exists when quality is configured"),
            &report,
        )?;
        Some(report)
    } else {
        None
    };

    let validity_report = if let Some(experiment) = &request.experiment {
        let hints = &experiment.validity;
        let input = ExperimentValidityInput {
            experiment_id: experiment.experiment_id.clone(),
            intent: experiment.intent,
            capture: CaptureValidityInput {
                model_usage: if group.totals.usage_records > 0 {
                    ValidityCheckStatus::Pass
                } else {
                    ValidityCheckStatus::Unknown
                },
                tool_evidence: if grouped
                    .members
                    .iter()
                    .any(|member| !member.tools.is_empty())
                {
                    ValidityCheckStatus::Pass
                } else if hints.tool_evidence_required {
                    ValidityCheckStatus::Unknown
                } else {
                    ValidityCheckStatus::NotRequired
                },
                source_health: requested_health.status,
                fallback_recovered,
                run_group_resolved: ValidityCheckStatus::Pass,
            },
            task: TaskValidityInput {
                exact_task_captured: hints.exact_task_captured,
                terminal_status: group.root_terminal,
                completion_required: hints.completion_required,
            },
            workspace: WorkspaceValidityInput {
                output_workspace_resolved: if resolution.status
                    == WorkspaceResolutionStatus::Selected
                {
                    ValidityCheckStatus::Pass
                } else {
                    ValidityCheckStatus::Fail
                },
                before_state_captured: if workspace_diff.is_some() {
                    ValidityCheckStatus::Pass
                } else {
                    ValidityCheckStatus::Unknown
                },
                after_state_captured: if workspace_diff.is_some() {
                    ValidityCheckStatus::Pass
                } else {
                    ValidityCheckStatus::Unknown
                },
                quality_gate_on_output: quality_execution_validity(quality_report.as_ref()),
            },
            policy: PolicyValidityInput {
                required: request.policy.is_some(),
                identity_captured: if policy_report.is_some() {
                    ValidityCheckStatus::Pass
                } else {
                    ValidityCheckStatus::NotRequired
                },
                exposure_known: policy_exposure_validity(policy_report.as_ref()),
                compliance_evidence: policy_compliance_validity(policy_report.as_ref()),
                enforcement_accurately_labeled: if policy_report.is_some() {
                    hints.policy_enforcement_accurately_labeled
                } else {
                    ValidityCheckStatus::NotRequired
                },
            },
            runtime: RuntimeValidityInput {
                model_recorded: hints.model_recorded,
                runtime_recorded: if grouped.members.iter().any(|member| {
                    member
                        .cli_version
                        .as_deref()
                        .is_some_and(|value| !value.trim().is_empty())
                }) {
                    ValidityCheckStatus::Pass
                } else {
                    ValidityCheckStatus::Unknown
                },
                configuration_recorded: hints.configuration_recorded,
                comparable_to_baseline: hints.comparable_to_baseline,
            },
            quality: QualityValidityInput {
                automated_gates: quality_acceptance_validity(quality_report.as_ref()),
                human_or_host_gates: hints.human_or_host_gates,
            },
            causal: CausalControlsInput {
                baseline_available: hints.baseline_available,
                same_task: hints.same_task,
                same_starting_workspace: hints.same_starting_workspace,
                same_runtime_model_config: hints.same_runtime_model_config,
                single_primary_variable: hints.single_primary_variable,
            },
        };
        write_json(
            validity_input_path
                .as_deref()
                .expect("validity input path exists when experiment is configured"),
            &input,
        )?;
        let report = reduce_experiment_validity(&input);
        write_json(
            validity_report_path
                .as_deref()
                .expect("validity report path exists when experiment is configured"),
            &report,
        )?;
        Some(report)
    } else {
        None
    };

    let mut warnings = Vec::new();
    if requested_health.status != tokn_domain::SourceHealthStatus::Healthy {
        warnings.push(format!(
            "requested source health is {}",
            requested_health.status.as_str()
        ));
    }
    let pipeline_status = if resolution.status != WorkspaceResolutionStatus::Selected {
        warnings.push(format!(
            "workspace resolution is {}; downstream P8 steps are blocked",
            resolution.status.as_str()
        ));
        RunnerPipelineStatus::Blocked
    } else if workspace_diff
        .as_ref()
        .is_some_and(|(_, targets_selected)| !targets_selected)
    {
        warnings.push("after snapshot does not target the selected workspace".into());
        RunnerPipelineStatus::Blocked
    } else if quality_report
        .as_ref()
        .is_some_and(|report| report.required && report.status == RunnerQualityStatus::Unavailable)
    {
        warnings.push("required quality gate is unavailable".into());
        RunnerPipelineStatus::Blocked
    } else {
        RunnerPipelineStatus::CoreEvidenceReady
    };

    let mut completed_steps = vec![
        "RECOVERY".into(),
        "MEASUREMENT_CONTRACT".into(),
        "SOURCE_HEALTH_REPORT".into(),
        "SESSION_ROOT_RESOLUTION".into(),
        "SESSION_EVIDENCE".into(),
        "RUN_GROUP".into(),
        "WORKSPACE_RESOLUTION".into(),
    ];
    if workspace_diff.is_some() {
        completed_steps.push("WORKSPACE_DIFF".into());
    }
    if policy_report.is_some() {
        completed_steps.push("POLICY_EVIDENCE".into());
    }
    if quality_report.is_some() {
        completed_steps.push("QUALITY_GATE".into());
    }
    if validity_report.is_some() {
        completed_steps.push("EXPERIMENT_VALIDITY".into());
    }

    let mut pending_steps = Vec::new();
    if workspace_diff.is_none() {
        pending_steps.push("WORKSPACE_DIFF".into());
    }
    if quality_report.is_none() {
        pending_steps.push("QUALITY_GATE".into());
    }

    let pipeline_status =
        if pipeline_status == RunnerPipelineStatus::CoreEvidenceReady && pending_steps.is_empty() {
            RunnerPipelineStatus::Complete
        } else {
            pipeline_status
        };

    let result = RunnerResult {
        schema_version: RUNNER_RESULT_SCHEMA_VERSION,
        run_id: request.run_id.clone(),
        pipeline_status,
        source_requested: request.source.clone(),
        source_health_status: requested_health.status,
        fallback_recovered,
        root_session: root_session.to_string_lossy().to_string(),
        evidence_dir: evidence_dir.to_string_lossy().to_string(),
        selected_workspace: resolution.selected_root.clone(),
        workspace_diff_added_count: workspace_diff
            .as_ref()
            .map(|(report, _)| report.added_count),
        workspace_diff_modified_count: workspace_diff
            .as_ref()
            .map(|(report, _)| report.modified_count),
        workspace_diff_removed_count: workspace_diff
            .as_ref()
            .map(|(report, _)| report.removed_count),
        workspace_diff_targets_selected_workspace: workspace_diff
            .as_ref()
            .map(|(_, targets_selected)| *targets_selected),
        root_terminal: group.root_terminal,
        agent_count: group.agents.len() as u64,
        policy_required: request.policy.is_some(),
        policy_observation_status: policy_report.as_ref().map(|report| report.observed.status),
        policy_enforcement_status: policy_report.as_ref().map(|report| report.enforcement),
        quality_required: request.quality.as_ref().map(|quality| quality.required),
        quality_status: quality_report.as_ref().map(|report| report.status),
        validity_verdict: validity_report.as_ref().map(|report| report.verdict),
        causal_claims_allowed: validity_report
            .as_ref()
            .map(|report| report.causal_claims_allowed),
        descriptive_metrics_allowed: validity_report
            .as_ref()
            .map(|report| report.descriptive_metrics_allowed),
        recovery_status: recovery_report.status,
        completed_steps,
        pending_steps,
        warnings,
        artifacts: RunnerArtifactPaths {
            normalized_request: normalized_request_path.to_string_lossy().to_string(),
            measurement_contract: Some(measurement_contract_path.to_string_lossy().to_string()),
            source_health: source_health_path.to_string_lossy().to_string(),
            session_evidence: session_evidence_path.to_string_lossy().to_string(),
            run_group: run_group_path.to_string_lossy().to_string(),
            workspace_resolution: workspace_resolution_path.to_string_lossy().to_string(),
            workspace_before_snapshot: workspace_before_snapshot_path
                .as_ref()
                .map(|path| path.to_string_lossy().to_string()),
            workspace_after_snapshot: workspace_after_snapshot_path
                .as_ref()
                .map(|path| path.to_string_lossy().to_string()),
            workspace_diff: workspace_diff_path
                .as_ref()
                .map(|path| path.to_string_lossy().to_string()),
            policy_evidence: policy_evidence_path
                .as_ref()
                .map(|path| path.to_string_lossy().to_string()),
            quality_gate: quality_gate_path
                .as_ref()
                .map(|path| path.to_string_lossy().to_string()),
            validity_input: validity_input_path
                .as_ref()
                .map(|path| path.to_string_lossy().to_string()),
            validity_report: validity_report_path
                .as_ref()
                .map(|path| path.to_string_lossy().to_string()),
            recovery_report: recovery_report_path.to_string_lossy().to_string(),
            runner_result: runner_result_path.to_string_lossy().to_string(),
        },
    };
    write_json(&runner_result_path, &result)?;

    println!("TOKN RUNNER");
    println!("  run_id: {}", result.run_id);
    println!("  status: {}", result.pipeline_status.as_str());
    println!("  root_session: {}", result.root_session);
    println!("  agents: {}", result.agent_count);
    println!(
        "  selected_workspace: {}",
        result.selected_workspace.as_deref().unwrap_or("<none>")
    );
    println!("  evidence_dir: {}", result.evidence_dir);

    Ok(())
}

fn policy_exposure_validity(report: Option<&PolicyEvidenceReport>) -> ValidityCheckStatus {
    match report {
        Some(report) if report.parse_failures == 0 => ValidityCheckStatus::Pass,
        Some(_) => ValidityCheckStatus::Unknown,
        None => ValidityCheckStatus::NotRequired,
    }
}

fn policy_compliance_validity(report: Option<&PolicyEvidenceReport>) -> ValidityCheckStatus {
    match report.map(|report| report.observed.status) {
        Some(PolicyObservationStatus::Pass | PolicyObservationStatus::Fail) => {
            ValidityCheckStatus::Pass
        }
        Some(
            PolicyObservationStatus::NoEvidence
            | PolicyObservationStatus::IncompleteEvidence
            | PolicyObservationStatus::NotEvaluated,
        ) => ValidityCheckStatus::Unknown,
        None => ValidityCheckStatus::NotRequired,
    }
}

fn quality_execution_validity(report: Option<&RunnerQualityReport>) -> ValidityCheckStatus {
    match report.map(|report| report.status) {
        Some(RunnerQualityStatus::Pass | RunnerQualityStatus::Fail) => ValidityCheckStatus::Pass,
        Some(RunnerQualityStatus::NotRequired) => ValidityCheckStatus::NotRequired,
        Some(RunnerQualityStatus::Unavailable | RunnerQualityStatus::Unknown) | None => {
            ValidityCheckStatus::Unknown
        }
    }
}

fn quality_acceptance_validity(report: Option<&RunnerQualityReport>) -> ValidityCheckStatus {
    match report.map(|report| report.status) {
        Some(RunnerQualityStatus::Pass) => ValidityCheckStatus::Pass,
        Some(RunnerQualityStatus::Fail) => ValidityCheckStatus::Fail,
        Some(RunnerQualityStatus::NotRequired) => ValidityCheckStatus::NotRequired,
        Some(RunnerQualityStatus::Unavailable | RunnerQualityStatus::Unknown) | None => {
            ValidityCheckStatus::Unknown
        }
    }
}

fn run_quality_gate(
    request: &RunnerQualityRequest,
    workspace: Option<&str>,
) -> RunnerQualityReport {
    if !request.required && request.program.is_none() {
        return RunnerQualityReport {
            required: false,
            status: RunnerQualityStatus::NotRequired,
            workspace: workspace.map(str::to_string),
            program: None,
            args: request.args.clone(),
            exit_code: None,
            stdout: String::new(),
            stderr: String::new(),
            error: None,
        };
    }

    let Some(workspace) = workspace else {
        return RunnerQualityReport {
            required: request.required,
            status: RunnerQualityStatus::Unavailable,
            workspace: None,
            program: request.program.clone(),
            args: request.args.clone(),
            exit_code: None,
            stdout: String::new(),
            stderr: String::new(),
            error: Some("quality gate cannot run without a selected workspace".into()),
        };
    };

    let Some(program) = request.program.as_deref() else {
        return RunnerQualityReport {
            required: request.required,
            status: RunnerQualityStatus::Unavailable,
            workspace: Some(workspace.to_string()),
            program: None,
            args: request.args.clone(),
            exit_code: None,
            stdout: String::new(),
            stderr: String::new(),
            error: Some("quality gate program is unavailable".into()),
        };
    };

    match ProcessCommand::new(program)
        .args(&request.args)
        .current_dir(workspace)
        .output()
    {
        Ok(output) => RunnerQualityReport {
            required: request.required,
            status: if output.status.success() {
                RunnerQualityStatus::Pass
            } else {
                RunnerQualityStatus::Fail
            },
            workspace: Some(workspace.to_string()),
            program: Some(program.to_string()),
            args: request.args.clone(),
            exit_code: output.status.code(),
            stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
            stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
            error: None,
        },
        Err(error) => RunnerQualityReport {
            required: request.required,
            status: RunnerQualityStatus::Unavailable,
            workspace: Some(workspace.to_string()),
            program: Some(program.to_string()),
            args: request.args.clone(),
            exit_code: None,
            stdout: String::new(),
            stderr: String::new(),
            error: Some(error.to_string()),
        },
    }
}

fn read_request(path: &Path) -> anyhow::Result<RunnerRequest> {
    let bytes = std::fs::read(path)?;
    let bytes = bytes.strip_prefix(&[0xEF, 0xBB, 0xBF]).unwrap_or(&bytes);
    Ok(serde_json::from_slice(bytes)?)
}

fn read_inventory(path: &Path) -> anyhow::Result<WorkspaceInventory> {
    let bytes = std::fs::read(path)?;
    let bytes = bytes.strip_prefix(&[0xEF, 0xBB, 0xBF]).unwrap_or(&bytes);
    let inventory: WorkspaceInventory = serde_json::from_slice(bytes)?;
    if inventory.schema_version != WORKSPACE_INVENTORY_SCHEMA_VERSION {
        anyhow::bail!(
            "unsupported workspace inventory schema_version {}; expected {}",
            inventory.schema_version,
            WORKSPACE_INVENTORY_SCHEMA_VERSION
        );
    }
    Ok(inventory)
}

fn read_project_snapshot(path: &Path) -> anyhow::Result<ProjectSnapshot> {
    let bytes = std::fs::read(path)?;
    let bytes = bytes.strip_prefix(&[0xEF, 0xBB, 0xBF]).unwrap_or(&bytes);
    let snapshot: ProjectSnapshot = serde_json::from_slice(bytes)?;
    if snapshot.schema_version != PROJECT_SNAPSHOT_SCHEMA_VERSION {
        anyhow::bail!(
            "unsupported project snapshot schema_version {}; expected {}",
            snapshot.schema_version,
            PROJECT_SNAPSHOT_SCHEMA_VERSION
        );
    }
    Ok(snapshot)
}

fn same_path(left: &str, right: &str) -> bool {
    normalize_path(left) == normalize_path(right)
}

fn normalize_path(path: &str) -> String {
    path.replace('/', "\\")
        .trim_end_matches('\\')
        .to_ascii_lowercase()
}

fn write_json(path: &Path, value: &impl serde::Serialize) -> anyhow::Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    serde_json::to_writer_pretty(File::create(path)?, value)?;
    Ok(())
}

fn prepare_owned_artifacts(
    paths: &[&Path],
    runner_result_path: &Path,
) -> anyhow::Result<RunnerRecoveryReport> {
    if runner_result_path.exists() {
        anyhow::bail!(
            "runner refuses to overwrite completed evidence: {}",
            runner_result_path.display()
        );
    }

    let existing = paths
        .iter()
        .filter(|path| path.exists())
        .filter(|path| **path != runner_result_path)
        .map(|path| (*path).to_path_buf())
        .collect::<Vec<_>>();

    let mut removed_artifacts = Vec::new();
    for path in existing {
        if !path.is_file() {
            anyhow::bail!(
                "runner owned artifact path is not a file: {}",
                path.display()
            );
        }
        std::fs::remove_file(&path)?;
        removed_artifacts.push(path.to_string_lossy().to_string());
    }

    let status = if removed_artifacts.is_empty() {
        RunnerRecoveryStatus::NotRequired
    } else {
        RunnerRecoveryStatus::RecoveredPartial
    };
    let message = if removed_artifacts.is_empty() {
        "no partial Runner artifacts required recovery; Runner V0.1 does not install temporary policy files"
            .into()
    } else {
        format!(
            "recovered {} partial Runner artifact(s); Runner V0.1 does not install temporary policy files",
            removed_artifacts.len()
        )
    };

    Ok(RunnerRecoveryReport {
        status,
        removed_artifacts,
        policy_placements_mutated: false,
        message,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_json(name: &str, content: &str) -> PathBuf {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock")
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "tokn-runner-{name}-{}-{stamp}.json",
            std::process::id()
        ));
        std::fs::write(&path, content).expect("write fixture");
        path
    }

    #[test]
    fn rejects_unknown_workspace_inventory_schema() {
        let path = temp_json(
            "inventory-schema",
            r#"{"schema_version":99,"watch_root":"E:/fixture","candidates":[]}"#,
        );
        let error = read_inventory(&path).expect_err("unknown inventory schema must fail");
        std::fs::remove_file(path).ok();
        assert!(
            error
                .to_string()
                .contains("workspace inventory schema_version 99")
        );
    }

    #[test]
    fn rejects_unknown_project_snapshot_schema() {
        let path = temp_json(
            "snapshot-schema",
            r#"{"schema_version":99,"project_root":"E:/fixture","file_count":0,"total_bytes":0,"files":[]}"#,
        );
        let error = read_project_snapshot(&path).expect_err("unknown snapshot schema must fail");
        std::fs::remove_file(path).ok();
        assert!(
            error
                .to_string()
                .contains("project snapshot schema_version 99")
        );
    }
}
