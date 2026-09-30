use std::fs::File;
use std::path::{Path, PathBuf};

use tokn_analysis::{
    WorkspaceInventory, WorkspaceResolutionStatus, build_policy_evidence_report, build_run_group,
    check_cap_policy_tools, resolve_workspace,
};
use tokn_codex::diagnostic::{DiagnosticBundle, assess_diagnostic_health};
use tokn_codex::session::{assess_session_health, collect_session_group, inspect_session_policy};
use tokn_domain::{
    PolicyObservationSummary, PolicyPlacement, RunnerArtifactPaths, RunnerPipelineStatus,
    RunnerRequest, RunnerResult, RunnerSourceReport, SourceKind,
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
    let source_health_path = evidence_dir.join("source-health.json");
    let run_group_path = evidence_dir.join("run-group.json");
    let workspace_resolution_path = evidence_dir.join("workspace-resolution.json");
    let policy_evidence_path = request
        .policy
        .as_ref()
        .map(|_| evidence_dir.join("policy-evidence.json"));
    let runner_result_path = evidence_dir.join("runner-result.json");

    let mut owned_paths = vec![
        normalized_request_path.as_path(),
        source_health_path.as_path(),
        run_group_path.as_path(),
        workspace_resolution_path.as_path(),
        runner_result_path.as_path(),
    ];
    if let Some(path) = policy_evidence_path.as_deref() {
        owned_paths.push(path);
    }
    refuse_overwrite(&owned_paths)?;

    write_json(&normalized_request_path, &request)?;

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

    let grouped = collect_session_group(&root_session)?;
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

    let mut warnings = Vec::new();
    if requested_health.status != tokn_domain::SourceHealthStatus::Healthy {
        warnings.push(format!(
            "requested source health is {}",
            requested_health.status.as_str()
        ));
    }
    let pipeline_status = if resolution.status == WorkspaceResolutionStatus::Selected {
        RunnerPipelineStatus::CoreEvidenceReady
    } else {
        warnings.push(format!(
            "workspace resolution is {}; downstream P8 steps are blocked",
            resolution.status.as_str()
        ));
        RunnerPipelineStatus::Blocked
    };

    let mut completed_steps = vec![
        "SOURCE_HEALTH_REPORT".into(),
        "SESSION_ROOT_RESOLUTION".into(),
        "RUN_GROUP".into(),
        "WORKSPACE_RESOLUTION".into(),
    ];
    if policy_report.is_some() {
        completed_steps.push("POLICY_EVIDENCE".into());
    }

    let result = RunnerResult {
        schema_version: 1,
        run_id: request.run_id.clone(),
        pipeline_status,
        source_requested: request.source.clone(),
        source_health_status: requested_health.status,
        fallback_recovered,
        root_session: root_session.to_string_lossy().to_string(),
        evidence_dir: evidence_dir.to_string_lossy().to_string(),
        selected_workspace: resolution.selected_root.clone(),
        root_terminal: group.root_terminal,
        agent_count: group.agents.len() as u64,
        policy_required: request.policy.is_some(),
        policy_observation_status: policy_report.as_ref().map(|report| report.observed.status),
        policy_enforcement_status: policy_report.as_ref().map(|report| report.enforcement),
        completed_steps,
        pending_steps: vec![
            "QUALITY_GATE".into(),
            "EXPERIMENT_VALIDITY".into(),
            "RECOVERY".into(),
        ],
        warnings,
        artifacts: RunnerArtifactPaths {
            normalized_request: normalized_request_path.to_string_lossy().to_string(),
            source_health: source_health_path.to_string_lossy().to_string(),
            run_group: run_group_path.to_string_lossy().to_string(),
            workspace_resolution: workspace_resolution_path.to_string_lossy().to_string(),
            policy_evidence: policy_evidence_path
                .as_ref()
                .map(|path| path.to_string_lossy().to_string()),
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

fn read_request(path: &Path) -> anyhow::Result<RunnerRequest> {
    let bytes = std::fs::read(path)?;
    let bytes = bytes.strip_prefix(&[0xEF, 0xBB, 0xBF]).unwrap_or(&bytes);
    Ok(serde_json::from_slice(bytes)?)
}

fn read_inventory(path: &Path) -> anyhow::Result<WorkspaceInventory> {
    let bytes = std::fs::read(path)?;
    let bytes = bytes.strip_prefix(&[0xEF, 0xBB, 0xBF]).unwrap_or(&bytes);
    Ok(serde_json::from_slice(bytes)?)
}

fn write_json(path: &Path, value: &impl serde::Serialize) -> anyhow::Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    serde_json::to_writer_pretty(File::create(path)?, value)?;
    Ok(())
}

fn refuse_overwrite(paths: &[&Path]) -> anyhow::Result<()> {
    let existing = paths
        .iter()
        .filter(|path| path.exists())
        .map(|path| path.display().to_string())
        .collect::<Vec<_>>();

    if existing.is_empty() {
        Ok(())
    } else {
        anyhow::bail!(
            "runner refuses to overwrite existing owned artifact(s): {}",
            existing.join(", ")
        )
    }
}
