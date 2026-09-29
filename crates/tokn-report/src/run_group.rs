use tokn_domain::{AgentNode, RunGroup, TokenTotals};

pub fn render_run_group_text(group: &RunGroup) -> String {
    let mut out = String::new();
    out.push_str("TOKN RUN GROUP V0.1\n\n");

    out.push_str("RUN\n");
    out.push_str(&format!(
        "  root thread             {}\n",
        group.root_thread_id
    ));
    out.push_str(&format!(
        "  session id              {}\n",
        group.session_id.as_deref().unwrap_or("UNKNOWN")
    ));
    out.push_str(&format!(
        "  agents                  {}\n",
        group.agents.len()
    ));
    out.push_str(&format!("  max depth               {}\n", group.max_depth));
    out.push_str(&format!(
        "  root terminal           {}\n",
        group.root_terminal.as_str()
    ));
    out.push_str(&format!(
        "  all threads completed   {}\n\n",
        yes_no(group.all_threads_completed)
    ));

    render_totals(&mut out, "TOKENS", &group.totals);

    let root = group
        .agents
        .iter()
        .find(|agent| agent.thread_id == group.root_thread_id);
    if let (Some(total), Some(root_total)) = (
        group.totals.logical_total(),
        root.and_then(|agent| agent.totals.logical_total()),
    ) {
        let descendants = total.saturating_sub(root_total);
        let parent_share = if total == 0 {
            0.0
        } else {
            root_total as f64 * 100.0 / total as f64
        };
        out.push_str("\nCOST SPLIT\n");
        out.push_str(&format!("  parent logical          {}\n", root_total));
        out.push_str(&format!("  descendants logical     {}\n", descendants));
        out.push_str(&format!("  parent share            {:.1}%\n", parent_share));
        out.push_str(&format!(
            "  descendants share       {:.1}%\n",
            100.0 - parent_share
        ));
    }

    out.push_str("\nAGENTS\n");
    for agent in &group.agents {
        render_agent(&mut out, agent);
    }

    out
}

fn render_totals(out: &mut String, title: &str, totals: &TokenTotals) {
    out.push_str(title);
    out.push('\n');
    out.push_str(&format!(
        "  usage records           {}\n",
        totals.usage_records
    ));
    out.push_str(&format!(
        "  input                   {}\n",
        totals.input_tokens
    ));
    out.push_str(&format!(
        "  cached input            {}\n",
        totals.cached_input_tokens
    ));
    out.push_str(&format!(
        "  uncached input          {}\n",
        option_u64(totals.ordinary_uncached_input())
    ));
    out.push_str(&format!(
        "  output                  {}\n",
        totals.output_tokens
    ));
    out.push_str(&format!(
        "  reasoning output        {}\n",
        totals.reasoning_output_tokens
    ));
    out.push_str(&format!(
        "  logical total           {}\n",
        option_u64(totals.logical_total())
    ));
}

fn render_agent(out: &mut String, agent: &AgentNode) {
    let label = agent
        .agent_nickname
        .as_deref()
        .or(agent.agent_path.as_deref())
        .unwrap_or(if agent.depth == 0 {
            "parent"
        } else {
            "subagent"
        });
    out.push_str(&format!(
        "  depth={} {:18} thread={} terminal={} usage={} logical={} uncached={}\n",
        agent.depth,
        label,
        agent.thread_id,
        agent.terminal.status.as_str(),
        agent.totals.usage_records,
        option_u64(agent.totals.logical_total()),
        option_u64(agent.totals.ordinary_uncached_input()),
    ));
}

fn option_u64(value: Option<u64>) -> String {
    value
        .map(|value| value.to_string())
        .unwrap_or_else(|| "UNKNOWN".into())
}

fn yes_no(value: bool) -> &'static str {
    if value { "YES" } else { "NO" }
}
