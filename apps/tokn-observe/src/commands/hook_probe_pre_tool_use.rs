use std::fs::OpenOptions;
use std::io::{self, Read, Write};
use std::path::Path;

use serde_json::{Value, json};
use tokn_codex::tooling::classify_command_text;

pub fn run(audit_jsonl: &Path) -> anyhow::Result<()> {
    let mut raw = String::new();
    io::stdin().read_to_string(&mut raw)?;
    let input: Value = serde_json::from_str(&raw)?;

    if input.get("hook_event_name").and_then(Value::as_str) != Some("PreToolUse") {
        return Ok(());
    }
    if input.get("tool_name").and_then(Value::as_str) != Some("Bash") {
        return Ok(());
    }

    let tool_input = input.get("tool_input").and_then(Value::as_object);
    let keys = tool_input
        .map(|object| object.keys().cloned().collect::<Vec<_>>())
        .unwrap_or_default();
    let command = tool_input
        .and_then(|object| object.get("command"))
        .and_then(Value::as_str);
    let category = command.map(classify_command_text);
    let max_output_tokens = tool_input
        .and_then(|object| object.get("max_output_tokens"))
        .and_then(parse_u64_value);

    let entry = json!({
        "hook_event_name": "PreToolUse",
        "session_id": input.get("session_id").and_then(Value::as_str),
        "turn_id": input.get("turn_id").and_then(Value::as_str),
        "tool_name": input.get("tool_name").and_then(Value::as_str),
        "tool_use_id": input.get("tool_use_id").and_then(Value::as_str),
        "tool_input_keys": keys,
        "command_present": command.is_some(),
        "command_category": category,
        "max_output_tokens_present": tool_input
            .is_some_and(|object| object.contains_key("max_output_tokens")),
        "max_output_tokens": max_output_tokens
    });

    if let Some(parent) = audit_jsonl.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(audit_jsonl)?;
    serde_json::to_writer(&mut file, &entry)?;
    writeln!(file)?;
    file.flush()?;

    // Probe only: stdout intentionally stays empty, so the tool call is not modified.
    Ok(())
}

fn parse_u64_value(value: &Value) -> Option<u64> {
    value
        .as_u64()
        .or_else(|| value.as_str()?.trim().parse::<u64>().ok())
}
