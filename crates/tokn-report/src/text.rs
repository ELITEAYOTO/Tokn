use tokn_storage::RunRecord;

pub fn render_text(run: &RunRecord) -> String {
    let cache_share = if run.input_tokens == 0 {
        None
    } else {
        Some(run.cached_input_tokens as f64 * 100.0 / run.input_tokens as f64)
    };
    let uncached_input = run
        .input_tokens
        .checked_sub(run.cached_input_tokens)
        .and_then(|value| value.checked_sub(run.cache_write_input_tokens));

    let mut out = String::new();
    out.push_str("TOKN OBSERVER V0.0\n\n");
    out.push_str(&format!("RUN\n  {}\n", run.run_id));
    out.push_str(&format!("SOURCE\n  {}\n", run.source_path));
    out.push_str(&format!("ACCOUNTING\n  {}\n\n", run.accounting_mode));

    out.push_str("INTEGRITY\n");
    out.push_str(&format!("  records seen            {}\n", run.records_seen));
    out.push_str(&format!(
        "  records valid           {}\n",
        run.records_valid
    ));
    out.push_str(&format!(
        "  malformed               {}\n",
        run.malformed_records
    ));
    out.push_str(&format!(
        "  oversized               {}\n",
        run.oversized_records
    ));
    out.push_str(&format!(
        "  truncated tail          {}\n",
        run.truncated_tail
    ));
    out.push_str(&format!(
        "  duplicates suppressed   {}\n",
        run.duplicates_suppressed
    ));
    out.push_str(&format!(
        "  usage conflicts         {}\n",
        run.usage_conflicts
    ));
    out.push_str(&format!(
        "  invariant conflicts     {}\n\n",
        run.invariant_conflicts
    ));

    out.push_str("TOKENS\n");
    out.push_str(&format!(
        "  usage records           {}\n",
        run.usage_records
    ));
    out.push_str(&format!("  input                   {}\n", run.input_tokens));
    out.push_str(&format!(
        "  cached input            {}\n",
        run.cached_input_tokens
    ));
    out.push_str(&format!(
        "  cache write input       {}\n",
        run.cache_write_input_tokens
    ));
    if let Some(uncached) = uncached_input {
        out.push_str(&format!("  uncached input          {}\n", uncached));
    } else {
        out.push_str("  uncached input          UNKNOWN\n");
    }
    out.push_str(&format!(
        "  output                  {}\n",
        run.output_tokens
    ));
    out.push_str(&format!(
        "  reasoning output        {}\n",
        run.reasoning_output_tokens
    ));
    out.push_str(&format!(
        "  logical total           {}\n",
        run.logical_tokens
    ));

    if let Some(share) = cache_share {
        out.push_str(&format!("  cached share            {:.1}%\n", share));
    }

    out
}
