use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::{Component, Path, PathBuf};

use anyhow::{Context, Result};
use serde_json::Value;
use tokn_domain::{DiagnosticObservations, InferenceObservation, TokenUsage, ToolObservation};

use super::DiagnosticBundle;

pub fn read_diagnostic_observations(bundle: &DiagnosticBundle) -> Result<DiagnosticObservations> {
    let file =
        File::open(&bundle.trace).with_context(|| format!("open {}", bundle.trace.display()))?;
    let mut inferences = HashMap::<String, InferenceObservation>::new();
    let mut tools = HashMap::<String, ToolObservation>::new();
    let mut code_cells_started = 0_u64;
    let mut compactions = 0_u64;

    for line in BufReader::new(file).lines() {
        let line = line?;
        let event: Value = serde_json::from_str(&line)?;
        let Some(envelope) = event.as_object() else {
            continue;
        };
        let seq = envelope.get("seq").and_then(Value::as_u64);
        let wall = envelope.get("wall_time_unix_ms").and_then(Value::as_i64);
        let thread_id = string(envelope, "thread_id");
        let turn_id = string(envelope, "codex_turn_id");
        let Some(payload) = envelope.get("payload").and_then(Value::as_object) else {
            continue;
        };
        let Some(kind) = payload.get("type").and_then(Value::as_str) else {
            continue;
        };

        if kind.contains("compaction") {
            compactions += 1;
        }

        match kind {
            "code_cell_started" => code_cells_started += 1,
            "inference_started" => {
                let Some(id) = string(payload, "inference_call_id") else {
                    continue;
                };
                let entry = inferences.entry(id.clone()).or_default();

                entry.inference_call_id = id;
                entry.thread_id = string(payload, "thread_id").or(thread_id);
                entry.turn_id = string(payload, "codex_turn_id").or(turn_id);
                entry.model = string(payload, "model");
                entry.status = "started".into();
                entry.started_seq = seq;
                entry.started_at_unix_ms = wall;

                if let Some(reference) = payload.get("request_payload")
                    && let Some((request, bytes)) = load_payload(&bundle.root, reference)
                {
                    entry.request_payload_bytes = Some(bytes);
                    entry.request_input_items = request
                        .get("input")
                        .and_then(Value::as_array)
                        .map(|items| items.len() as u64);
                }
            }
            "inference_completed" | "inference_cancelled" | "inference_failed" => {
                let Some(id) = string(payload, "inference_call_id") else {
                    continue;
                };
                let entry = inferences.entry(id.clone()).or_default();
                entry.inference_call_id = id;
                entry.ended_seq = seq;
                entry.ended_at_unix_ms = wall;
                entry.status = kind.trim_start_matches("inference_").to_string();

                if kind == "inference_completed"
                    && let Some(reference) = payload.get("response_payload")
                    && let Some((response, _)) = load_payload(&bundle.root, reference)
                {
                    entry.usage = response.get("token_usage").and_then(parse_usage);
                }
            }
            "tool_call_started" => {
                let Some(id) = string(payload, "tool_call_id") else {
                    continue;
                };
                let entry = tools.entry(id.clone()).or_default();
                entry.tool_call_id = id;
                entry.kind = tool_kind(payload);
                entry.surface = tool_surface(payload);
                entry.requester_type = payload
                    .get("requester")
                    .and_then(Value::as_object)
                    .and_then(|requester| string(requester, "type"));
                entry.status = "started".into();
                entry.started_seq = seq;

                if let Some(reference) = payload.get("invocation_payload")
                    && let Some((invocation, bytes)) = load_payload(&bundle.root, reference)
                {
                    entry.invocation_payload_bytes = Some(bytes);
                    entry.tool_name = invocation
                        .get("tool_name")
                        .and_then(Value::as_str)
                        .map(str::to_string);
                    entry.max_output_tokens = invocation_arguments(&invocation)
                        .and_then(|args| args.get("max_output_tokens").and_then(Value::as_u64));
                    entry.category =
                        classify_tool(&entry.kind, entry.tool_name.as_deref(), &invocation);
                }
            }
            "tool_call_ended" => {
                let Some(id) = string(payload, "tool_call_id") else {
                    continue;
                };
                let entry = tools.entry(id.clone()).or_default();
                entry.tool_call_id = id;
                entry.status = string(payload, "status").unwrap_or_else(|| "ended".into());
                entry.ended_seq = seq;

                if let Some(reference) = payload.get("result_payload")
                    && let Some((result, bytes)) = load_payload(&bundle.root, reference)
                {
                    entry.result_payload_bytes = Some(bytes);
                    entry.result_output_chars = result_output_chars(&result);
                    entry.original_token_count = original_token_count(&result);
                }
            }
            _ => {}
        }
    }

    let mut inferences = inferences.into_values().collect::<Vec<_>>();
    inferences.sort_by_key(|item| item.started_seq.unwrap_or(u64::MAX));
    let mut tools = tools.into_values().collect::<Vec<_>>();
    tools.sort_by_key(|item| item.started_seq.unwrap_or(u64::MAX));

    Ok(DiagnosticObservations {
        inferences,
        tools,
        code_cells_started,
        compactions,
    })
}

fn load_payload(root: &Path, reference: &Value) -> Option<(Value, u64)> {
    let relative = reference.get("path")?.as_str()?;
    let path = safe_path(root, relative)?;
    let bytes = std::fs::metadata(&path).ok()?.len();
    let value = serde_json::from_reader(File::open(path).ok()?).ok()?;
    Some((value, bytes))
}

fn safe_path(root: &Path, relative: &str) -> Option<PathBuf> {
    let relative = Path::new(relative);
    if relative
        .components()
        .any(|part| !matches!(part, Component::Normal(_) | Component::CurDir))
    {
        return None;
    }
    Some(root.join(relative))
}

fn parse_usage(value: &Value) -> Option<TokenUsage> {
    let input = value.get("input_tokens").and_then(Value::as_u64);
    let output = value.get("output_tokens").and_then(Value::as_u64);
    if input.is_none() && output.is_none() {
        return None;
    }

    Some(TokenUsage {
        input_tokens: input,
        cached_input_tokens: value.get("cached_input_tokens").and_then(Value::as_u64),
        cache_write_input_tokens: value
            .get("cache_write_input_tokens")
            .and_then(Value::as_u64),
        output_tokens: output,
        reasoning_output_tokens: value.get("reasoning_output_tokens").and_then(Value::as_u64),
        reported_total_tokens: value.get("total_tokens").and_then(Value::as_u64),
    })
}

fn original_token_count(value: &Value) -> Option<u64> {
    value
        .get("value")
        .and_then(|inner| inner.get("original_token_count"))
        .and_then(Value::as_u64)
}

fn result_output_chars(value: &Value) -> Option<u64> {
    value
        .get("value")
        .and_then(|inner| inner.get("output"))
        .and_then(Value::as_str)
        .map(|output| output.chars().count() as u64)
}

fn tool_kind(payload: &serde_json::Map<String, Value>) -> String {
    payload
        .get("kind")
        .and_then(Value::as_object)
        .and_then(|kind| kind.get("type").or_else(|| kind.get("name")))
        .and_then(Value::as_str)
        .unwrap_or("unknown")
        .to_string()
}

fn tool_surface(payload: &serde_json::Map<String, Value>) -> String {
    if payload
        .get("model_visible_call_id")
        .is_some_and(|value| !value.is_null())
    {
        "direct_model_visible".into()
    } else if payload
        .get("code_mode_runtime_tool_id")
        .is_some_and(|value| !value.is_null())
    {
        "runtime_mediated".into()
    } else {
        "unknown".into()
    }
}

fn classify_tool(kind: &str, tool_name: Option<&str>, invocation: &Value) -> String {
    match kind {
        "assign_agent_task" => "agent_task".into(),
        "send_message" => "agent_message".into(),
        "write_stdin" => "process_interaction".into(),
        "other" if tool_name == Some("list_agents") => "agent_control".into(),
        "exec_command" => classify_command(invocation),
        _ => kind.to_string(),
    }
}

fn classify_command(invocation: &Value) -> String {
    let Some(command) = invocation_arguments(invocation)
        .and_then(|args| args.get("cmd").and_then(Value::as_str).map(str::to_string))
    else {
        return "command_unknown".into();
    };
    let command = command.to_ascii_lowercase();

    if contains_any(
        &command,
        &[
            "cargo test",
            "cargo clippy",
            "cargo check",
            "cargo build",
            "npm test",
            "npm run test",
            "npm run build",
            "npm run lint",
            "pnpm test",
            "pnpm build",
            "pnpm lint",
            "pytest",
            "dotnet test",
        ],
    ) {
        "test_build".into()
    } else if contains_any(
        &command,
        &[
            "set-content",
            "add-content",
            "out-file",
            "copy-item",
            "move-item",
            "remove-item",
            "new-item",
            "mkdir",
            "write_text(",
            "write_bytes(",
        ],
    ) {
        "write_mutation".into()
    } else if contains_any(
        &command,
        &[" rg ", "rg ", "grep ", "findstr", "select-string"],
    ) {
        "search".into()
    } else if contains_any(
        &command,
        &[
            "get-content",
            " cat ",
            "cat ",
            " head ",
            "head ",
            " tail ",
            "tail ",
            "type ",
        ],
    ) {
        "file_read".into()
    } else if contains_any(&command, &["get-childitem", " dir ", "dir ", " ls ", "ls "]) {
        "directory_list".into()
    } else if contains_any(&command, &["git ", " git "]) {
        "git".into()
    } else if contains_any(
        &command,
        &["start-process", "stop-process", "taskkill", "kill-process"],
    ) {
        "process_control".into()
    } else {
        "command_other".into()
    }
}

fn invocation_arguments(invocation: &Value) -> Option<Value> {
    let raw = invocation.get("payload")?.get("arguments")?;
    match raw {
        Value::String(text) => serde_json::from_str(text).ok(),
        Value::Object(_) => Some(raw.clone()),
        _ => None,
    }
}

fn contains_any(text: &str, needles: &[&str]) -> bool {
    needles.iter().any(|needle| text.contains(needle))
}

fn string(map: &serde_json::Map<String, Value>, key: &str) -> Option<String> {
    map.get(key).and_then(Value::as_str).map(str::to_string)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn invocation(command: &str) -> Value {
        json!({
            "payload": {
                "arguments": serde_json::to_string(&json!({
                    "cmd": command,
                    "workdir": "C:\\fixture"
                }))
                .unwrap()
            }
        })
    }

    #[test]
    fn classifies_common_command_families() {
        assert_eq!(
            classify_command(&invocation("Get-Content README.md")),
            "file_read"
        );
        assert_eq!(classify_command(&invocation("rg TODO src")), "search");

        assert_eq!(
            classify_command(&invocation("cargo test --workspace")),
            "test_build"
        );
        assert_eq!(
            classify_command(&invocation("$x=Get-Content a.txt; Set-Content b.txt $x")),
            "write_mutation"
        );
    }
}
