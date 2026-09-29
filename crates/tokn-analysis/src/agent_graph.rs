use std::collections::{HashMap, HashSet};

use tokn_domain::{AgentEvidence, AgentNode, RunGroup, TokenTotals};

use crate::TokenLedger;

pub fn build_run_group(evidence: &[AgentEvidence], root_thread_id: &str) -> Option<RunGroup> {
    let by_thread = evidence
        .iter()
        .map(|item| (item.thread_id.as_str(), item))
        .collect::<HashMap<_, _>>();
    let root = by_thread.get(root_thread_id).copied()?;

    let mut selected = HashSet::from([root_thread_id.to_string()]);
    loop {
        let before = selected.len();
        for item in evidence {
            let linked = item
                .parent_thread_id
                .as_deref()
                .is_some_and(|parent| selected.contains(parent))
                || item
                    .forked_from_id
                    .as_deref()
                    .is_some_and(|parent| selected.contains(parent));
            if linked {
                selected.insert(item.thread_id.clone());
            }
        }
        if selected.len() == before {
            break;
        }
    }

    let mut agents = Vec::new();
    for item in evidence
        .iter()
        .filter(|item| selected.contains(&item.thread_id))
    {
        let depth = depth_for(item, &by_thread, root_thread_id);
        let totals = totals_for(item);
        agents.push(AgentNode {
            source_path: item.source_path.clone(),
            thread_id: item.thread_id.clone(),
            session_id: item.session_id.clone(),
            parent_thread_id: item.parent_thread_id.clone(),
            agent_nickname: item.agent_nickname.clone(),
            agent_path: item.agent_path.clone(),
            cwd: item.cwd.clone(),
            depth,
            totals,
            terminal: item.terminal.clone(),
        });
    }

    agents.sort_by(|left, right| {
        left.depth
            .cmp(&right.depth)
            .then_with(|| left.thread_id.cmp(&right.thread_id))
    });

    let mut totals = TokenTotals::default();
    let mut max_depth = 0_u32;
    for agent in &agents {
        totals.add_totals(&agent.totals);
        max_depth = max_depth.max(agent.depth);
    }

    Some(RunGroup {
        root_thread_id: root.thread_id.clone(),
        session_id: root.session_id.clone(),
        all_threads_completed: agents
            .iter()
            .all(|agent| agent.terminal.status.is_complete()),
        root_terminal: root.terminal.status,
        agents,
        totals,
        max_depth,
    })
}

fn totals_for(evidence: &AgentEvidence) -> TokenTotals {
    let mut ledger = TokenLedger::default();
    for record in &evidence.usages {
        ledger.push_keyed(record.key.clone(), record.usage.clone());
    }

    let mut totals = TokenTotals::default();
    for usage in &ledger.accepted {
        totals.add_usage(usage);
    }
    totals
}

fn depth_for(
    evidence: &AgentEvidence,
    by_thread: &HashMap<&str, &AgentEvidence>,
    root_thread_id: &str,
) -> u32 {
    if evidence.thread_id == root_thread_id {
        return 0;
    }

    let mut current = evidence;
    let mut visited = HashSet::new();
    let mut depth = 0_u32;

    loop {
        if !visited.insert(current.thread_id.as_str()) {
            return depth;
        }
        let parent = current
            .parent_thread_id
            .as_deref()
            .or(current.forked_from_id.as_deref());
        let Some(parent) = parent else {
            return depth;
        };
        depth = depth.saturating_add(1);
        if parent == root_thread_id {
            return depth;
        }
        let Some(next) = by_thread.get(parent).copied() else {
            return depth;
        };
        current = next;
    }
}

#[cfg(test)]
mod tests {
    use tokn_domain::{KeyedTokenUsage, TerminalObservation, TerminalStatus, TokenUsage};

    use super::*;

    fn evidence(
        thread: &str,
        parent: Option<&str>,
        input: u64,
        output: u64,
        terminal: TerminalStatus,
    ) -> AgentEvidence {
        AgentEvidence {
            source_path: format!("{thread}.jsonl"),
            thread_id: thread.into(),
            session_id: Some("session-root".into()),
            parent_thread_id: parent.map(str::to_string),
            forked_from_id: parent.map(str::to_string),
            usages: vec![KeyedTokenUsage {
                key: Some(format!("response:{thread}")),
                usage: TokenUsage {
                    input_tokens: Some(input),
                    cached_input_tokens: Some(input / 2),
                    cache_write_input_tokens: Some(0),
                    output_tokens: Some(output),
                    reasoning_output_tokens: Some(output / 2),
                    ..Default::default()
                },
            }],
            terminal: TerminalObservation {
                status: terminal,
                ..Default::default()
            },
            ..Default::default()
        }
    }

    #[test]
    fn groups_parent_and_three_children_once() {
        let evidence = vec![
            evidence(
                "root",
                None,
                1_000,
                100,
                TerminalStatus::IncompleteUsageLimit,
            ),
            evidence(
                "a",
                Some("root"),
                200,
                20,
                TerminalStatus::IncompleteUsageLimit,
            ),
            evidence(
                "b",
                Some("root"),
                300,
                30,
                TerminalStatus::IncompleteUsageLimit,
            ),
            evidence(
                "c",
                Some("root"),
                400,
                40,
                TerminalStatus::IncompleteUsageLimit,
            ),
            evidence("unrelated", None, 9_999, 999, TerminalStatus::Completed),
        ];

        let group = build_run_group(&evidence, "root").expect("run group");
        assert_eq!(group.agents.len(), 4);
        assert_eq!(group.max_depth, 1);
        assert_eq!(group.totals.usage_records, 4);
        assert_eq!(group.totals.input_tokens, 1_900);
        assert_eq!(group.totals.output_tokens, 190);
        assert_eq!(group.totals.logical_total(), Some(2_090));
        assert_eq!(group.root_terminal, TerminalStatus::IncompleteUsageLimit);
        assert!(!group.all_threads_completed);
    }

    #[test]
    fn computes_nested_depth() {
        let evidence = vec![
            evidence("root", None, 10, 1, TerminalStatus::Completed),
            evidence("child", Some("root"), 10, 1, TerminalStatus::Completed),
            evidence(
                "grandchild",
                Some("child"),
                10,
                1,
                TerminalStatus::Completed,
            ),
        ];
        let group = build_run_group(&evidence, "root").expect("run group");
        assert_eq!(group.max_depth, 2);
    }
}
