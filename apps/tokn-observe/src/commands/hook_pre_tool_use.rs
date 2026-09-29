use std::collections::BTreeMap;
use std::fs::OpenOptions;
use std::io::{self, Read, Write};
use std::path::Path;

use serde_json::{Value, json};
use tokn_codex::tooling::classify_command_text;

use super::common::parse_cap_overrides;

#[derive(Debug)]
struct HookEvaluation {
    category: String,
    required_max: u64,
    observed_cap: Option<u64>,
    observable: bool,
    denied: bool,
    output: Option<Value>,
}

pub fn run(caps: &[String], audit_jsonl: Option<&Path>) -> anyhow::Result<()> {
    let policy = parse_cap_overrides(caps)?;
    let mut raw = String::new();
    io::stdin().read_to_string(&mut raw)?;
    let input: Value = serde_json::from_str(&raw)?;

    if let Some(evaluation) = evaluate_pre_tool_use(&input, &policy) {
        if let Some(path) = audit_jsonl {
            append_audit(path, &input, &evaluation)?;
        }
        if let Some(output) = evaluation.output {
            println!("{}", serde_json::to_string(&output)?);
        }
    }

    Ok(())
}

fn evaluate_pre_tool_use(input: &Value, policy: &BTreeMap<String, u64>) -> Option<HookEvaluation> {
    if input.get("hook_event_name").and_then(Value::as_str) != Some("PreToolUse") {
        return None;
    }

    let tool_name = input.get("tool_name").and_then(Value::as_str)?;
    if !matches!(tool_name, "Bash" | "exec_command") {
        return None;
    }

    let tool_input = input.get("tool_input")?;
    let command = tool_input.get("command").and_then(Value::as_str)?;
    let category = classify_command_text(command);
    let required_max = *policy.get(&category)?;

    let observed_cap = tool_input
        .get("max_output_tokens")
        .and_then(parse_u64_value);

    let Some(observed_cap) = observed_cap else {
        return Some(HookEvaluation {
            category,
            required_max,
            observed_cap: None,
            observable: false,
            denied: false,
            output: None,
        });
    };

    if observed_cap <= required_max {
        return Some(HookEvaluation {
            category,
            required_max,
            observed_cap: Some(observed_cap),
            observable: true,
            denied: false,
            output: None,
        });
    }

    let reason = format!(
        "Tokn policy: {category} max_output_tokens={observed_cap} exceeds required maximum {required_max}."
    );

    let output = json!({
        "hookSpecificOutput": {
            "hookEventName": "PreToolUse",
            "permissionDecision": "deny",
            "permissionDecisionReason": reason
        }
    });

    Some(HookEvaluation {
        category,
        required_max,
        observed_cap: Some(observed_cap),
        observable: true,
        denied: true,
        output: Some(output),
    })
}

fn append_audit(path: &Path, input: &Value, evaluation: &HookEvaluation) -> anyhow::Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let entry = json!({
        "session_id": input.get("session_id").and_then(Value::as_str),
        "turn_id": input.get("turn_id").and_then(Value::as_str),
        "tool_use_id": input.get("tool_use_id").and_then(Value::as_str),
        "category": evaluation.category,
        "required_max_output_tokens": evaluation.required_max,
        "observed_max_output_tokens": evaluation.observed_cap,
        "decision": if !evaluation.observable {
            "UNOBSERVABLE"
        } else if evaluation.denied {
            "DENY"
        } else {
            "ALLOW"
        }
    });

    let mut file = OpenOptions::new().create(true).append(true).open(path)?;
    serde_json::to_writer(&mut file, &entry)?;
    writeln!(file)?;
    file.flush()?;
    Ok(())
}

fn parse_u64_value(value: &Value) -> Option<u64> {
    value
        .as_u64()
        .or_else(|| value.as_str()?.trim().parse::<u64>().ok())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn policy() -> BTreeMap<String, u64> {
        BTreeMap::from([
            ("file_read".to_string(), 5_000),
            ("search".to_string(), 3_000),
        ])
    }

    #[test]
    fn allows_compliant_targeted_command() {
        let input = json!({
            "hook_event_name": "PreToolUse",
            "tool_name": "Bash",
            "tool_input": {
                "command": "Get-Content README.md",
                "max_output_tokens": 5000
            }
        });

        let evaluation = evaluate_pre_tool_use(&input, &policy()).unwrap();
        assert!(!evaluation.denied);
        assert!(evaluation.output.is_none());
    }

    #[test]
    fn leaves_unobservable_missing_cap_unblocked() {
        let input = json!({
            "hook_event_name": "PreToolUse",
            "tool_name": "Bash",
            "tool_input": {"command": "rg TODO src"}
        });

        let evaluation = evaluate_pre_tool_use(&input, &policy()).unwrap();
        assert!(!evaluation.observable);
        assert!(!evaluation.denied);
        assert_eq!(evaluation.observed_cap, None);
        assert!(evaluation.output.is_none());
    }

    #[test]
    fn ignores_non_targeted_command() {
        let input = json!({
            "hook_event_name": "PreToolUse",
            "tool_name": "Bash",
            "tool_input": {"command": "cargo test --workspace"}
        });

        assert!(evaluate_pre_tool_use(&input, &policy()).is_none());
    }

    #[test]
    fn denies_cap_above_policy() {
        let input = json!({
            "hook_event_name": "PreToolUse",
            "tool_name": "exec_command",
            "tool_input": {
                "command": "Get-Content README.md",
                "max_output_tokens": "8000"
            }
        });

        let evaluation = evaluate_pre_tool_use(&input, &policy()).unwrap();
        assert!(evaluation.denied);
        assert_eq!(evaluation.observed_cap, Some(8_000));
    }
}
