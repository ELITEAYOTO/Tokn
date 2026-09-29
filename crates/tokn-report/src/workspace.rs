use tokn_analysis::WorkspaceResolution;

pub fn render_workspace_resolution_text(result: &WorkspaceResolution) -> String {
    let mut out = String::new();

    out.push_str("TOKN WORKSPACE RESOLUTION V0.1\n\n");
    out.push_str(&format!("STATUS\n  {}\n\n", result.status.as_str()));
    out.push_str(&format!("WATCH ROOT\n  {}\n", result.watch_root));
    out.push_str(&format!("SOURCE ROOT\n  {}\n", result.source_root));
    out.push_str(&format!(
        "SELECTED ROOT\n  {}\n\n",
        result.selected_root.as_deref().unwrap_or("NONE")
    ));

    out.push_str("CANDIDATES\n");
    for candidate in &result.candidates {
        out.push_str(&format!(
            "  score={:<5} new={:<7} expected={:<3} source={:<3} direct_hits={:<3} parent_hits={:<3} cwd_hits={:<3} {}\n",
            candidate.score,
            option_bool(candidate.existed_before.map(|value| !value)),
            yes_no(candidate.declared_expected),
            yes_no(candidate.is_source_root),
            candidate.direct_tool_workdir_hits,
            candidate.parent_tool_workdir_hits,
            candidate.agent_cwd_hits,
            candidate.root
        ));

        for reason in &candidate.reasons {
            out.push_str(&format!("    - {reason}\n"));
        }
    }

    out
}

fn yes_no(value: bool) -> &'static str {
    if value { "YES" } else { "NO" }
}

fn option_bool(value: Option<bool>) -> &'static str {
    match value {
        Some(true) => "YES",
        Some(false) => "NO",
        None => "UNKNOWN",
    }
}
