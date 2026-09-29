use tokn_analysis::{AttributionReport, AttributionWindow, ToolAggregate};

pub fn render_attribution_text(report: &AttributionReport, top: usize) -> String {
    let mut out = String::new();
    out.push_str("TOKN ATTRIBUTION V0\n\n");
    out.push_str("ACTIVITY\n");
    out.push_str(&format!(
        "  inference started       {}\n",
        report.inference_started
    ));
    out.push_str(&format!(
        "  completed               {}\n",
        report.inference_completed
    ));
    out.push_str(&format!(
        "  cancelled               {}\n",
        report.inference_cancelled
    ));
    out.push_str(&format!(
        "  failed                  {}\n",
        report.inference_failed
    ));

    out.push_str(&format!(
        "  usage records           {}\n",
        report.usage_records
    ));
    out.push_str(&format!(
        "  tools                   {}\n",
        report.tools_total
    ));
    out.push_str(&format!(
        "  tool token coverage     {}/{}\n",
        report.tools_with_original_token_count, report.tools_total
    ));
    out.push_str(&format!(
        "  raw tool tokens         {}\n",
        report.total_tool_original_tokens
    ));
    out.push_str(&format!(
        "  output cap coverage     {}/{}\n",
        report.tools_with_output_cap, report.tools_total
    ));
    out.push_str(&format!(
        "  cap-adjusted upper      {}\n",
        report.cap_adjusted_tool_token_upper_bound
    ));
    out.push_str(&format!(
        "  raw exceeds cap         {}\n",
        report.raw_exceeds_cap_count
    ));
    out.push_str(&format!(
        "  unattributed tools      {}\n",
        report.unattributed_tool_calls
    ));
    out.push_str(&format!(
        "  unattributed coverage   {}/{}\n",
        report.unattributed_tools_with_original_token_count, report.unattributed_tool_calls
    ));
    out.push_str(&format!(
        "  unattributed obs toks   {}\n",
        report.unattributed_tool_original_tokens
    ));
    out.push_str(&format!(
        "  code cells              {}\n",
        report.code_cells_started
    ));
    out.push_str(&format!(
        "  compactions             {}\n\n",
        report.compactions
    ));

    render_aggregates(&mut out, "TOOL SURFACES", &report.tool_surfaces);
    render_aggregates(&mut out, "TOOL CATEGORIES", &report.tool_categories);
    render_aggregates(
        &mut out,
        "UNATTRIBUTED TOOL CATEGORIES",
        &report.unattributed_categories,
    );

    let mut by_tool = report.windows.clone();
    by_tool.sort_by_key(|window| std::cmp::Reverse(window.tool_original_tokens));
    render_windows(&mut out, "TOP WINDOWS BY TOOL OUTPUT TOKENS", &by_tool, top);

    let mut by_growth = report.windows.clone();
    by_growth.sort_by_key(|window| std::cmp::Reverse(window.input_growth.unwrap_or(i64::MIN)));
    render_windows(&mut out, "TOP WINDOWS BY INPUT GROWTH", &by_growth, top);

    out.push_str("NOTE\n  Window attribution is temporal correlation, not proof of causality.\n");
    out
}

fn render_aggregates(out: &mut String, title: &str, items: &[ToolAggregate]) {
    out.push_str(title);
    out.push('\n');

    let mut items = items.to_vec();
    items.sort_by(|left, right| {
        right
            .original_tokens
            .cmp(&left.original_tokens)
            .then_with(|| right.count.cmp(&left.count))
            .then_with(|| left.label.cmp(&right.label))
    });

    for item in items {
        out.push_str(&format!(
            "  {:24} count={} raw_coverage={}/{} raw_tokens={} cap_coverage={}/{} cap_upper={} raw_gt_cap={} result_bytes={} output_chars={}\n",
            item.label,
            item.count,
            item.token_count_known,
            item.count,
            item.original_tokens,
            item.cap_count_known,
            item.count,
            item.cap_adjusted_upper_tokens,
            item.raw_exceeds_cap_count,
            item.result_payload_bytes,
            item.result_output_chars,
        ));
    }
    out.push('\n');
}

fn render_windows(out: &mut String, title: &str, windows: &[AttributionWindow], top: usize) {
    out.push_str(title);
    out.push('\n');
    for window in windows.iter().take(top) {
        let growth = window
            .input_growth
            .map(|value| value.to_string())
            .unwrap_or_else(|| "N/A".into());
        let uncached = window
            .uncached_input_tokens
            .map(|value| value.to_string())
            .unwrap_or_else(|| "UNKNOWN".into());
        let kinds = window
            .tool_kinds
            .iter()
            .map(|item| {
                format!(
                    "{}:{} ({} tok)",
                    item.kind, item.count, item.original_tokens
                )
            })
            .collect::<Vec<_>>()
            .join(", ");

        out.push_str(&format!(
            "  #{:02} input={} delta={} uncached={} tools={} tool_raw={} tool_cap_upper={} req_items={} req_bytes={}\n",
            window.index,
            window.input_tokens,
            growth,
            uncached,
            window.tools_since_previous,
            window.tool_original_tokens,
            window.tool_cap_adjusted_upper_tokens,
            option_u64(window.request_input_items),
            option_u64(window.request_payload_bytes),
        ));
        if !kinds.is_empty() {
            out.push_str(&format!("       {}\n", kinds));
        }
    }
    out.push('\n');
}

fn option_u64(value: Option<u64>) -> String {
    value
        .map(|value| value.to_string())
        .unwrap_or_else(|| "UNKNOWN".into())
}
