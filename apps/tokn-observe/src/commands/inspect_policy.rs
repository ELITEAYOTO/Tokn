use std::fs::File;
use std::path::{Path, PathBuf};

use tokn_analysis::{CapPolicyStatus, build_policy_evidence_report, check_cap_policy_tools};
use tokn_codex::session::{collect_session_group, inspect_session_policy};
use tokn_domain::{
    PolicyEnforcementStatus, PolicyObservationStatus, PolicyObservationSummary, PolicyPlacement,
};
use tokn_report::render_policy_evidence_text;

use super::common::{parse_cap_overrides, resolve_session_root_from_source, resolve_source};

pub fn run(
    source: &str,
    policy_id: &str,
    marker: &str,
    policy_paths: &[PathBuf],
    caps: &[String],
    enforcement: &str,
    output_json: Option<&Path>,
) -> anyhow::Result<()> {
    let source_path = resolve_source(source)?;
    let root_session = resolve_session_root_from_source(&source_path)?;
    let grouped = collect_session_group(&root_session)?;

    let policy_path_strings = policy_paths
        .iter()
        .map(|path| path.to_string_lossy().to_string())
        .collect::<Vec<_>>();
    let placements = policy_path_strings
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
            marker,
            &policy_path_strings,
        )?);
    }

    let policy = parse_cap_overrides(caps)?;
    let observed = if policy.is_empty() {
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
        let check = check_cap_policy_tools(&tools, parse_failures, &policy);

        PolicyObservationSummary {
            status: map_observation_status(check.status),
            targeted: check.targeted_tools,
            compliant: check.compliant,
            violations: check.violations,
            unknown: check.unknown,
            parse_failures: check.parse_failures,
        }
    };

    let report = build_policy_evidence_report(
        policy_id,
        marker,
        placements,
        session_reports,
        observed,
        parse_enforcement(enforcement)?,
    );

    print!("{}", render_policy_evidence_text(&report));

    if let Some(path) = output_json {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        serde_json::to_writer_pretty(File::create(path)?, &report)?;
        println!("\nJSON\n  {}", path.display());
    }

    Ok(())
}

fn map_observation_status(status: CapPolicyStatus) -> PolicyObservationStatus {
    match status {
        CapPolicyStatus::Pass => PolicyObservationStatus::Pass,
        CapPolicyStatus::Fail => PolicyObservationStatus::Fail,
        CapPolicyStatus::NoEvidence => PolicyObservationStatus::NoEvidence,
        CapPolicyStatus::IncompleteEvidence => PolicyObservationStatus::IncompleteEvidence,
    }
}

fn parse_enforcement(value: &str) -> anyhow::Result<PolicyEnforcementStatus> {
    match value {
        "unavailable" => Ok(PolicyEnforcementStatus::Unavailable),
        "enforced" => Ok(PolicyEnforcementStatus::Enforced),
        "not-proven" => Ok(PolicyEnforcementStatus::NotProven),
        _ => anyhow::bail!("unsupported enforcement status: {value}"),
    }
}
