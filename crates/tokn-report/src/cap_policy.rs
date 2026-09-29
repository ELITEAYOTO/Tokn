use tokn_analysis::CapPolicyReport;

pub fn render_cap_policy_text(report: &CapPolicyReport) -> String {
    let mut out = String::new();
    out.push_str("TOKN CAP POLICY CHECK V0.1\n\n");

    out.push_str("SUMMARY\n");
    out.push_str(&format!(
        "  status                  {}\n",
        report.status.as_str()
    ));
    out.push_str(&format!(
        "  targeted tools          {}\n",
        report.targeted_tools
    ));
    out.push_str(&format!("  cap known               {}\n", report.cap_known));
    out.push_str(&format!("  compliant               {}\n", report.compliant));
    out.push_str(&format!(
        "  violations              {}\n",
        report.violations
    ));
    out.push_str(&format!("  unknown                 {}\n", report.unknown));
    out.push_str(&format!(
        "  parse failures          {}\n",
        report.parse_failures
    ));
    out.push_str(&format!(
        "  known caps compliant    {}\n",
        yes_no(report.all_known_caps_compliant)
    ));
    out.push_str(&format!(
        "  evidence complete       {}\n\n",
        yes_no(report.complete_evidence)
    ));

    out.push_str("CATEGORIES\n");
    for item in &report.categories {
        out.push_str(&format!(
            "  {:20} required_max={} tools={} known={} compliant={} violations={} unknown={} min_cap={} max_cap={}\n",
            item.label,
            item.required_max,
            item.tool_count,
            item.cap_known,
            item.compliant,
            item.violations,
            item.unknown,
            option_u64(item.min_observed_cap),
            option_u64(item.max_observed_cap),
        ));
    }

    out.push_str("\nRULE\n");
    out.push_str(
        "  PASS requires at least one applicable target, no violations, no unknown caps, and no parse failures.\n",
    );
    out.push_str(
        "  NO_EVIDENCE is never treated as PASS. This check does not prove quality or token savings.\n",
    );
    out
}

fn option_u64(value: Option<u64>) -> String {
    value
        .map(|value| value.to_string())
        .unwrap_or_else(|| "UNKNOWN".into())
}

fn yes_no(value: bool) -> &'static str {
    if value { "YES" } else { "NO" }
}
