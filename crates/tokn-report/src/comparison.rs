use tokn_analysis::{CategoryComparison, RunComparison, RunMetrics};

pub fn render_comparison_text(report: &RunComparison) -> String {
    let mut out = String::new();
    out.push_str("TOKN COMPARE V0\n\n");
    out.push_str("QUALITY\n");
    out.push_str("  status                  NOT_MEASURED\n");
    out.push_str("  decision                NOT_EVALUATED\n\n");

    out.push_str("TOKENS\n");
    metric(
        &mut out,
        "input",
        report.baseline.input_tokens,
        report.candidate.input_tokens,
        report.delta_input_tokens,
    );
    metric(
        &mut out,
        "cached input",
        report.baseline.cached_input_tokens,
        report.candidate.cached_input_tokens,
        report.delta_cached_input_tokens,
    );

    optional_metric(
        &mut out,
        "uncached input",
        report.baseline.uncached_input_tokens,
        report.candidate.uncached_input_tokens,
        report.delta_uncached_input_tokens,
    );
    metric(
        &mut out,
        "output",
        report.baseline.output_tokens,
        report.candidate.output_tokens,
        report.delta_output_tokens,
    );
    metric(
        &mut out,
        "reasoning output",
        report.baseline.reasoning_output_tokens,
        report.candidate.reasoning_output_tokens,
        report.delta_reasoning_output_tokens,
    );
    metric(
        &mut out,
        "logical total",
        report.baseline.logical_total_tokens,
        report.candidate.logical_total_tokens,
        report.delta_logical_total_tokens,
    );

    out.push('\n');
    out.push_str("ACTIVITY\n");
    count_metric(
        &mut out,
        "completed inferences",
        report.baseline.inference_completed,
        report.candidate.inference_completed,
        report.delta_completed_inferences,
    );
    count_metric(
        &mut out,
        "tools",
        report.baseline.tools_total,
        report.candidate.tools_total,
        signed(report.candidate.tools_total, report.baseline.tools_total),
    );
    metric(
        &mut out,
        "raw tool tokens",
        report.baseline.observed_tool_tokens,
        report.candidate.observed_tool_tokens,
        report.delta_observed_tool_tokens,
    );
    metric(
        &mut out,
        "cap-adjusted upper",
        report.baseline.cap_adjusted_tool_token_upper_bound,
        report.candidate.cap_adjusted_tool_token_upper_bound,
        report.delta_cap_adjusted_tool_token_upper_bound,
    );
    count_metric(
        &mut out,
        "raw exceeds cap",
        report.baseline.raw_exceeds_cap_count,
        report.candidate.raw_exceeds_cap_count,
        report.delta_raw_exceeds_cap_count,
    );
    span_metric(&mut out, &report.baseline, &report.candidate);

    out.push('\n');
    out.push_str("TOOL TOKEN COVERAGE\n");
    out.push_str(&format!(
        "  baseline                {}/{}\n",
        report.baseline.tools_with_token_count, report.baseline.tools_total
    ));
    out.push_str(&format!(
        "  candidate               {}/{}\n",
        report.candidate.tools_with_token_count, report.candidate.tools_total
    ));
    out.push_str("OUTPUT CAP COVERAGE\n");
    out.push_str(&format!(
        "  baseline                {}/{}\n",
        report.baseline.tools_with_output_cap, report.baseline.tools_total
    ));
    out.push_str(&format!(
        "  candidate               {}/{}\n\n",
        report.candidate.tools_with_output_cap, report.candidate.tools_total
    ));

    render_categories(&mut out, &report.category_deltas);

    out.push_str("RULE\n");
    out.push_str("  Fewer tokens alone is not a win. Quality/work must be validated separately.\n");
    out.push_str(
        "  Observed duration is informational; Tokn does not optimize for speed by default.\n",
    );
    out
}

fn metric(out: &mut String, label: &str, before: u64, after: u64, delta: i64) {
    out.push_str(&format!(
        "  {:22} baseline={} candidate={} delta={} change={}\n",
        label,
        before,
        after,
        signed_text(delta),
        percent_text(before, after),
    ));
}

fn optional_metric(
    out: &mut String,
    label: &str,
    before: Option<u64>,
    after: Option<u64>,
    delta: Option<i64>,
) {
    match (before, after, delta) {
        (Some(before), Some(after), Some(delta)) => metric(out, label, before, after, delta),
        _ => out.push_str(&format!(
            "  {:22} baseline=UNKNOWN candidate=UNKNOWN delta=UNKNOWN change=UNKNOWN\n",
            label
        )),
    }
}

fn count_metric(out: &mut String, label: &str, before: u64, after: u64, delta: i64) {
    out.push_str(&format!(
        "  {:22} baseline={} candidate={} delta={}\n",
        label,
        before,
        after,
        signed_text(delta),
    ));
}

fn span_metric(out: &mut String, before: &RunMetrics, after: &RunMetrics) {
    let before = before.observed_span_ms.map(|value| value as f64 / 1000.0);
    let after = after.observed_span_ms.map(|value| value as f64 / 1000.0);
    match (before, after) {
        (Some(before), Some(after)) => out.push_str(&format!(
            "  observed span (sec)    baseline={before:.2} candidate={after:.2}\n"
        )),
        _ => out.push_str("  observed span (sec)    baseline=UNKNOWN candidate=UNKNOWN\n"),
    }
}

fn render_categories(out: &mut String, items: &[CategoryComparison]) {
    out.push_str("TOOL CATEGORIES\n");
    for item in items {
        out.push_str(&format!(
            "  {:20} count {}->{} raw_cov {}/{}->{}/{} raw {}->{} d={} cap_cov {}/{}->{}/{} cap_upper {}->{} d={} raw_gt_cap {}->{}\n",
            item.label,
            item.baseline_count,
            item.candidate_count,
            item.baseline_token_coverage,
            item.baseline_count,
            item.candidate_token_coverage,
            item.candidate_count,
            item.baseline_observed_tokens,
            item.candidate_observed_tokens,
            signed_text(item.delta_observed_tokens),
            item.baseline_cap_coverage,
            item.baseline_count,
            item.candidate_cap_coverage,
            item.candidate_count,
            item.baseline_cap_upper,
            item.candidate_cap_upper,
            signed_text(item.delta_cap_upper),
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

fn signed(current: u64, baseline: u64) -> i64 {
    let delta = i128::from(current) - i128::from(baseline);
    delta.clamp(i128::from(i64::MIN), i128::from(i64::MAX)) as i64
}
