use std::collections::BTreeMap;
use std::path::Path;

use anyhow::{Context, Result};
use tokn_domain::{AgentEvidence, KeyedTokenUsage, TerminalObservation, TerminalStatus};
use tokn_ingest::{JsonlConfig, read_jsonl};

use super::{
    extract_rate_limit_snapshot, extract_session_tool_result_output, extract_session_tools,
    extract_terminal_observation, extract_usage_record,
};

pub fn read_session_tool_result_outputs(path: &Path) -> Result<BTreeMap<String, Vec<String>>> {
    let mut outputs = BTreeMap::<String, Vec<String>>::new();
    read_jsonl(path, &JsonlConfig::default(), |record| {
        if let Some(result) = extract_session_tool_result_output(&record.value) {
            outputs
                .entry(result.call_id)
                .or_default()
                .push(result.output);
        }
    })
    .with_context(|| format!("read session tool outputs {}", path.display()))?;
    Ok(outputs)
}

pub fn read_session_evidence(path: &Path) -> Result<AgentEvidence> {
    let mut evidence = AgentEvidence {
        source_path: path.to_string_lossy().to_string(),
        terminal: TerminalObservation {
            status: TerminalStatus::UnknownTerminalState,
            ..Default::default()
        },
        ..Default::default()
    };

    let mut identity_loaded = false;
    read_jsonl(path, &JsonlConfig::default(), |record| {
        if !identity_loaded
            && record.value.get("type").and_then(|value| value.as_str()) == Some("session_meta")
            && let Some(payload) = record.value.get("payload")
        {
            evidence.thread_id = string(payload, "id").unwrap_or_default();
            evidence.session_id = string(payload, "session_id");
            evidence.parent_thread_id = string(payload, "parent_thread_id");
            evidence.forked_from_id = string(payload, "forked_from_id");
            evidence.thread_source = string(payload, "thread_source");
            evidence.agent_nickname = string(payload, "agent_nickname");
            evidence.agent_path = string(payload, "agent_path");
            evidence.cwd = string(payload, "cwd");
            evidence.cli_version = string(payload, "cli_version");
            identity_loaded = true;
        }

        if let Some(observation) = extract_usage_record(&record.value) {
            let key = observation
                .response_id
                .map(|response| format!("response:{response}"));
            evidence.usages.push(KeyedTokenUsage {
                key,
                usage: observation.usage,
            });
        }

        if let Some(snapshot) = extract_rate_limit_snapshot(&record.value) {
            evidence.rate_limit_snapshots.push(snapshot);
        }

        if let Some(batch) = extract_session_tools(&record.value) {
            evidence.tool_parse_failures = evidence
                .tool_parse_failures
                .saturating_add(batch.parse_failures);
            evidence.tools.extend(batch.observations);
        }

        if let Some(terminal) = extract_terminal_observation(&record.value) {
            evidence.terminal = terminal;
        }
    })
    .with_context(|| format!("read session evidence {}", path.display()))?;

    if evidence.thread_id.is_empty() {
        anyhow::bail!("session_meta thread id is missing in {}", path.display());
    }

    Ok(evidence)
}

fn string(value: &serde_json::Value, key: &str) -> Option<String> {
    value
        .get(key)
        .and_then(|item| item.as_str())
        .map(str::to_string)
}
