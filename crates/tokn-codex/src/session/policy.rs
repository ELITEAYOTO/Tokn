use std::collections::BTreeSet;
use std::path::Path;

use anyhow::{Context, Result};
use serde_json::Value;
use tokn_domain::SessionPolicyEvidence;
use tokn_ingest::{JsonlConfig, read_jsonl};

use super::extract_session_tools;

pub fn inspect_session_policy(
    path: &Path,
    marker: &str,
    policy_paths: &[String],
) -> Result<SessionPolicyEvidence> {
    let mut evidence = SessionPolicyEvidence {
        source_path: path.to_string_lossy().to_string(),
        ..Default::default()
    };
    let mut identity_loaded = false;
    let mut matched_paths = BTreeSet::new();
    read_jsonl(path, &JsonlConfig::default(), |record| {
        if !identity_loaded
            && record.value.get("type").and_then(Value::as_str) == Some("session_meta")
            && let Some(payload) = record.value.get("payload")
        {
            evidence.thread_id = payload
                .get("id")
                .and_then(Value::as_str)
                .map(str::to_string);
            evidence.instruction_observed = instruction_fields_contain_marker(payload, marker);
            identity_loaded = true;
        }

        if let Some(batch) = extract_session_tools(&record.value) {
            evidence.parse_failures = evidence.parse_failures.saturating_add(batch.parse_failures);

            for tool in batch.observations {
                if !matches!(tool.category.as_str(), "file_read" | "search") {
                    continue;
                }
                let Some(command) = tool.command.as_deref() else {
                    continue;
                };

                let matches =
                    command_policy_matches(command, tool.workdir.as_deref(), policy_paths);
                if matches.is_empty() {
                    continue;
                }

                evidence.repository_read_observed = true;
                evidence.repository_read_count = evidence.repository_read_count.saturating_add(1);
                matched_paths.extend(matches);
            }
        }
    })
    .with_context(|| format!("inspect session policy {}", path.display()))?;

    evidence.matched_policy_paths = matched_paths.into_iter().collect();
    Ok(evidence)
}
fn instruction_fields_contain_marker(payload: &Value, marker: &str) -> bool {
    if marker.trim().is_empty() {
        return false;
    }

    const KEYS: &[&str] = &[
        "base_instructions",
        "developer_instructions",
        "system_instructions",
        "user_instructions",
        "instructions",
    ];

    KEYS.iter().any(|key| {
        payload
            .get(*key)
            .is_some_and(|value| value_contains_marker(value, marker))
    })
}

fn value_contains_marker(value: &Value, marker: &str) -> bool {
    match value {
        Value::String(text) => text.contains(marker),
        Value::Array(items) => items.iter().any(|item| value_contains_marker(item, marker)),
        Value::Object(map) => map.values().any(|item| value_contains_marker(item, marker)),
        _ => false,
    }
}

fn command_policy_matches(
    command: &str,
    workdir: Option<&str>,
    policy_paths: &[String],
) -> Vec<String> {
    let command = normalize(command);
    let workdir = workdir.map(normalize);
    let mut matches = Vec::new();

    for policy_path in policy_paths {
        let normalized_policy = normalize(policy_path);
        let direct = command.contains(&normalized_policy);
        let relative = workdir.as_deref().is_some_and(|workdir| {
            relative_to(workdir, &normalized_policy)
                .is_some_and(|relative| command.contains(relative))
        });
        if direct || relative {
            matches.push(policy_path.clone());
        }
    }

    matches
}

fn normalize(value: &str) -> String {
    value.replace('\\', "/").to_ascii_lowercase()
}

fn relative_to<'a>(root: &str, path: &'a str) -> Option<&'a str> {
    if path == root {
        return Some("");
    }
    let prefix = format!("{}/", root.trim_end_matches('/'));
    path.strip_prefix(&prefix)
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;
    #[test]
    fn detects_policy_marker_only_in_instruction_fields() {
        let payload = json!({
            "base_instructions": "prefix Tokn Policy Marker suffix",
            "other": "Tokn Policy Marker"
        });

        assert!(instruction_fields_contain_marker(
            &payload,
            "Tokn Policy Marker"
        ));
        assert!(!instruction_fields_contain_marker(&payload, "missing"));
    }

    #[test]
    fn resolves_relative_policy_read_from_tool_workdir() {
        let policy = vec![r"E:\repo\PROJECT\AGENTS.md".to_string()];
        let matches =
            command_policy_matches(r"Get-Content PROJECT\AGENTS.md", Some(r"E:\repo"), &policy);

        assert_eq!(matches, policy);
    }
}
