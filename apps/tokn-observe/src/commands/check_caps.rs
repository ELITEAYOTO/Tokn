use std::path::Path;

use tokn_analysis::{CapPolicyStatus, check_cap_policy_tools};
use tokn_codex::diagnostic::{
    DiagnosticBundle, assess_diagnostic_health, read_diagnostic_observations,
};
use tokn_codex::session::collect_session_group;
use tokn_domain::ToolObservation;
use tokn_report::render_cap_policy_text;

use super::common::{parse_cap_overrides, resolve_session_root_from_source, resolve_source};

pub fn run(source: &str, caps: &[String]) -> anyhow::Result<()> {
    let policy = parse_cap_overrides(caps)?;
    if policy.is_empty() {
        anyhow::bail!("at least one --cap CATEGORY=TOKENS policy is required");
    }

    let path = resolve_source(source)?;
    let (tools, parse_failures) = collect_tools(&path)?;
    let report = check_cap_policy_tools(&tools, parse_failures, &policy);
    print!("{}", render_cap_policy_text(&report));

    match report.status {
        CapPolicyStatus::Pass => Ok(()),
        CapPolicyStatus::Fail => std::process::exit(3),
        CapPolicyStatus::IncompleteEvidence => std::process::exit(4),
        CapPolicyStatus::NoEvidence => std::process::exit(5),
    }
}

fn collect_tools(path: &Path) -> anyhow::Result<(Vec<ToolObservation>, u64)> {
    if !path.is_dir() {
        let root = resolve_session_root_from_source(path)?;
        return collect_session_tools(&root);
    }

    let bundle = DiagnosticBundle::detect(path).ok_or_else(|| {
        anyhow::anyhow!("not a valid diagnostic trace bundle: {}", path.display())
    })?;
    let health = assess_diagnostic_health(&bundle)?;

    if health.is_usable_for_token_accounting() {
        let observations = read_diagnostic_observations(&bundle)?;
        return Ok((observations.tools, 0));
    }

    let root = resolve_session_root_from_source(path)?;
    collect_session_tools(&root)
}

fn collect_session_tools(root: &Path) -> anyhow::Result<(Vec<ToolObservation>, u64)> {
    let group = collect_session_group(root)?;
    let parse_failures = group
        .members
        .iter()
        .map(|member| member.tool_parse_failures)
        .sum();

    let tools = group
        .members
        .into_iter()
        .flat_map(|member| member.tools)
        .collect();

    Ok((tools, parse_failures))
}
