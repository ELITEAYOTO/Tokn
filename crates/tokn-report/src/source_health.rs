use tokn_domain::SourceHealth;

pub fn render_source_health_text(label: &str, health: &SourceHealth) -> String {
    let c = &health.coverage;
    let mut out = String::new();

    out.push_str("TOKN SOURCE HEALTH V0.1\n\n");
    out.push_str(&format!("SOURCE\n  {label}\n"));
    out.push_str(&format!("STATUS\n  {}\n\n", health.status.as_str()));

    out.push_str("COVERAGE\n");
    out.push_str(&format!("  records seen            {}\n", c.records_seen));
    out.push_str(&format!("  records valid           {}\n", c.records_valid));
    out.push_str(&format!(
        "  malformed               {}\n",
        c.malformed_records
    ));
    out.push_str(&format!(
        "  inference events        {}\n",
        c.inference_events
    ));
    out.push_str(&format!("  usage records           {}\n", c.usage_records));
    out.push_str(&format!("  tool events             {}\n", c.tool_events));
    out.push_str(&format!(
        "  terminal events         {}\n",
        c.terminal_events
    ));
    out.push_str(&format!(
        "  protocol events         {}\n\n",
        c.protocol_events
    ));

    out.push_str("TOKEN ACCOUNTING\n");
    out.push_str(&format!(
        "  usable                  {}\n",
        if health.is_usable_for_token_accounting() {
            "YES"
        } else {
            "NO"
        }
    ));

    if !health.reasons.is_empty() {
        out.push_str("\nREASONS\n");
        for reason in &health.reasons {
            out.push_str(&format!("  - {reason}\n"));
        }
    }

    out
}
