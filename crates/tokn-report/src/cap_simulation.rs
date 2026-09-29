use tokn_analysis::{CapSimulationCategory, CapSimulationReport};

pub fn render_cap_simulation_text(report: &CapSimulationReport) -> String {
    let mut out = String::new();
    out.push_str("TOKN CAP SIMULATION V0\n\n");
    out.push_str("STATUS\n");
    out.push_str("  mode                    WHAT_IF_ONLY\n");
    out.push_str("  quality                 NOT_MEASURED\n");
    out.push_str("  decision                NOT_EVALUATED\n\n");

    out.push_str("COVERAGE\n");
    out.push_str(&format!(
        "  tools                   {}\n",
        report.tools_total
    ));
    out.push_str(&format!(
        "  raw token coverage      {}/{}\n",
        report.raw_coverage, report.tools_total
    ));
    out.push_str(&format!(
        "  baseline cap coverage   {}/{}\n",
        report.baseline_cap_coverage, report.tools_total
    ));
    out.push_str(&format!(
        "  candidate cap coverage  {}/{}\n\n",
        report.candidate_cap_coverage, report.tools_total
    ));

    out.push_str("UPPER BOUNDS\n");
    out.push_str(&format!(
        "  raw observed tokens     {}\n",
        report.raw_tokens
    ));
    out.push_str(&format!(
        "  baseline cap upper      {}\n",
        report.baseline_upper
    ));
    out.push_str(&format!(
        "  candidate cap upper     {}\n",
        report.candidate_upper
    ));
    out.push_str(&format!(
        "  delta upper             {}\n",
        signed_text(report.delta_upper)
    ));
    out.push_str(&format!(
        "  upper-bound change      {}\n\n",
        percent_text(report.baseline_upper, report.candidate_upper)
    ));

    render_categories(&mut out, &report.categories);
    out.push_str("RULES\n");
    out.push_str("  This is an offline upper-bound simulation, not an observed candidate run.\n");
    out.push_str(
        "  Lower output caps can remove useful information; quality must be validated A/B.\n",
    );
    out.push_str("  Unknown/unmeasured tool surfaces are never treated as zero-cost evidence.\n");
    out
}

fn render_categories(out: &mut String, items: &[CapSimulationCategory]) {
    out.push_str("CATEGORIES\n");
    for item in items {
        let override_text = item
            .override_cap
            .map(|cap| cap.to_string())
            .unwrap_or_else(|| "CURRENT".into());
        out.push_str(&format!(
            "  {:20} count={} raw_cov={}/{} raw={} base_cov={}/{} base_upper={} override={} cand_cov={}/{} cand_upper={} delta={} raw_gt_cap {}->{}\n",
            item.label,
            item.count,
            item.raw_coverage,
            item.count,
            item.raw_tokens,
            item.baseline_cap_coverage,
            item.count,
            item.baseline_upper,
            override_text,
            item.candidate_cap_coverage,
            item.count,
            item.candidate_upper,
            signed_text(item.delta_upper),
            item.baseline_raw_exceeds_cap,
            item.candidate_raw_exceeds_cap,
        ));
    }
    out.push('\n');
}

fn percent_text(before: u64, after: u64) -> String {
    if before == 0 {
        return "N/A".into();
    }
    let pct = (after as f64 - before as f64) * 100.0 / before as f64;
    format!("{pct:+.2}%")
}

fn signed_text(value: i64) -> String {
    format!("{value:+}")
}
