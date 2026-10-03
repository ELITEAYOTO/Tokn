use std::collections::BTreeMap;

use serde_json::Value;
use tokn_domain::ToolObservation;

use crate::tooling::classify_command_text;

#[derive(Debug, Clone, Default)]
pub struct SessionToolBatch {
    pub observations: Vec<ToolObservation>,
    pub parse_failures: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionToolResultOutput {
    pub call_id: String,
    pub output: String,
}

pub fn extract_session_tool_result_output(value: &Value) -> Option<SessionToolResultOutput> {
    if value.get("type").and_then(Value::as_str) != Some("response_item") {
        return None;
    }
    let payload = value.get("payload")?;
    if payload.get("type").and_then(Value::as_str) != Some("custom_tool_call_output") {
        return None;
    }
    Some(SessionToolResultOutput {
        call_id: payload.get("call_id")?.as_str()?.to_string(),
        output: payload.get("output")?.as_str()?.to_string(),
    })
}

pub fn extract_session_tools(value: &Value) -> Option<SessionToolBatch> {
    if value.get("type").and_then(Value::as_str) != Some("response_item") {
        return None;
    }
    let payload = value.get("payload")?;
    if payload.get("type").and_then(Value::as_str) != Some("custom_tool_call")
        || payload.get("name").and_then(Value::as_str) != Some("exec")
    {
        return None;
    }

    let base_call_id = payload
        .get("call_id")
        .or_else(|| payload.get("id"))
        .and_then(Value::as_str)
        .unwrap_or("unknown-session-tool")
        .to_string();
    let status = payload
        .get("status")
        .and_then(Value::as_str)
        .unwrap_or("unknown")
        .to_string();
    let input = payload
        .get("input")
        .and_then(Value::as_str)
        .unwrap_or_default();

    let offsets = exec_command_offsets(input);
    if offsets.is_empty() {
        return None;
    }

    let mut batch = SessionToolBatch::default();
    for (index, marker_start) in offsets.into_iter().enumerate() {
        let tool_call_id = format!("{base_call_id}#{index}");
        match parse_exec_command_at(input, marker_start) {
            Ok(args) => {
                let command = args
                    .get("cmd")
                    .and_then(|raw| parse_js_string(raw))
                    .or_else(|| args.get("command").and_then(|raw| parse_js_string(raw)));
                let workdir = args.get("workdir").and_then(|raw| parse_js_string(raw));
                let max_output_tokens = args
                    .get("max_output_tokens")
                    .and_then(|raw| raw.trim().trim_matches('"').parse::<u64>().ok());
                let category = command
                    .as_deref()
                    .map(classify_command_text)
                    .unwrap_or_else(|| "command_unknown".into());

                batch.observations.push(ToolObservation {
                    tool_call_id,
                    kind: "exec_command".into(),
                    tool_name: Some("exec_command".into()),
                    category,
                    surface: "session_rollout".into(),
                    status: status.clone(),
                    max_output_tokens,
                    command,
                    workdir,
                    ..Default::default()
                });
            }
            Err(error) => {
                batch.parse_failures = batch.parse_failures.saturating_add(1);
                batch.observations.push(ToolObservation {
                    tool_call_id,
                    kind: "exec_command".into(),
                    tool_name: Some("exec_command".into()),
                    category: "command_unknown".into(),
                    surface: "session_rollout".into(),
                    status: status.clone(),
                    parse_error: Some(error),
                    ..Default::default()
                });
            }
        }
    }

    Some(batch)
}

fn exec_command_offsets(input: &str) -> Vec<usize> {
    const MARKER: &[u8] = b"tools.exec_command";
    let bytes = input.as_bytes();
    let mut out = Vec::new();
    let mut quote = None;
    let mut escaped = false;
    let mut index = 0_usize;

    while index < bytes.len() {
        let byte = bytes[index];

        if let Some(active) = quote {
            if escaped {
                escaped = false;
            } else if byte == b'\\' {
                escaped = true;
            } else if byte == active {
                quote = None;
            }
            index += 1;
            continue;
        }

        if matches!(byte, b'"' | b'\'' | b'`') {
            quote = Some(byte);
            index += 1;
            continue;
        }

        if bytes[index..].starts_with(MARKER) {
            out.push(index);
            index += MARKER.len();
            continue;
        }

        index += 1;
    }

    out
}

fn parse_exec_command_at(
    input: &str,
    marker_start: usize,
) -> Result<BTreeMap<String, String>, String> {
    let marker = "tools.exec_command";
    let after_marker_start = marker_start.saturating_add(marker.len());
    let after_marker = input
        .get(after_marker_start..)
        .ok_or_else(|| "invalid exec_command marker offset".to_string())?;

    let paren = after_marker
        .find('(')
        .ok_or_else(|| "exec_command opening parenthesis not found".to_string())?;
    let call_start = after_marker_start + paren + 1;
    let call = input
        .get(call_start..)
        .ok_or_else(|| "invalid exec_command call offset".to_string())?;
    let brace = call
        .find('{')
        .ok_or_else(|| "exec_command argument object not found".to_string())?;
    let absolute_brace = call_start + brace;

    let object = balanced_slice(input, absolute_brace, '{', '}')
        .ok_or_else(|| "unterminated exec_command argument object".to_string())?;
    parse_object_members(object)
}

fn balanced_slice(text: &str, start: usize, open: char, close: char) -> Option<&str> {
    let mut depth = 0_u32;
    let mut quote = None;
    let mut escaped = false;

    for (offset, ch) in text.get(start..)?.char_indices() {
        if let Some(active) = quote {
            if escaped {
                escaped = false;
                continue;
            }
            if ch == '\\' {
                escaped = true;
                continue;
            }
            if ch == active {
                quote = None;
            }
            continue;
        }

        if matches!(ch, '"' | '\'' | '`') {
            quote = Some(ch);
            continue;
        }
        if ch == open {
            depth += 1;
        } else if ch == close {
            depth = depth.checked_sub(1)?;
            if depth == 0 {
                return text.get(start..start + offset + ch.len_utf8());
            }
        }
    }
    None
}

fn parse_object_members(object: &str) -> Result<BTreeMap<String, String>, String> {
    let inner = object
        .strip_prefix('{')
        .and_then(|value| value.strip_suffix('}'))
        .ok_or_else(|| "argument is not an object".to_string())?;
    let mut out = BTreeMap::new();

    for (index, part) in split_top_level(inner, ',').into_iter().enumerate() {
        let part = part.trim();
        if part.is_empty() {
            continue;
        }
        let Some(colon) = find_top_level(part, ':') else {
            return Err(format!("object member {index} has no colon"));
        };
        let key = part[..colon].trim().trim_matches(['"', '\'', '`']);
        if key.is_empty() {
            return Err(format!("object member {index} has an empty key"));
        }
        out.insert(key.to_string(), part[colon + 1..].trim().to_string());
    }

    Ok(out)
}

fn split_top_level(text: &str, delimiter: char) -> Vec<&str> {
    let mut parts = Vec::new();
    let mut quote = None;
    let mut escaped = false;
    let mut depth = 0_i32;
    let mut start = 0_usize;

    for (index, ch) in text.char_indices() {
        if let Some(active) = quote {
            if escaped {
                escaped = false;
                continue;
            }
            if ch == '\\' {
                escaped = true;
                continue;
            }
            if ch == active {
                quote = None;
            }
            continue;
        }

        if matches!(ch, '"' | '\'' | '`') {
            quote = Some(ch);
            continue;
        }

        match ch {
            '{' | '[' | '(' => depth += 1,
            '}' | ']' | ')' => depth -= 1,
            _ => {}
        }

        if ch == delimiter && depth == 0 {
            parts.push(&text[start..index]);
            start = index + ch.len_utf8();
        }
    }

    parts.push(&text[start..]);
    parts
}

fn find_top_level(text: &str, needle: char) -> Option<usize> {
    let mut quote = None;
    let mut escaped = false;
    let mut depth = 0_i32;

    for (index, ch) in text.char_indices() {
        if let Some(active) = quote {
            if escaped {
                escaped = false;
                continue;
            }
            if ch == '\\' {
                escaped = true;
                continue;
            }
            if ch == active {
                quote = None;
            }
            continue;
        }

        if matches!(ch, '"' | '\'' | '`') {
            quote = Some(ch);
            continue;
        }

        match ch {
            '{' | '[' | '(' => depth += 1,
            '}' | ']' | ')' => depth -= 1,
            _ => {}
        }

        if ch == needle && depth == 0 {
            return Some(index);
        }
    }

    None
}

fn parse_js_string(raw: &str) -> Option<String> {
    let raw = raw.trim();
    if raw.starts_with('"') {
        return serde_json::from_str::<String>(raw).ok();
    }

    if raw.len() >= 2
        && ((raw.starts_with('\'') && raw.ends_with('\''))
            || (raw.starts_with('`') && raw.ends_with('`')))
    {
        return Some(
            raw[1..raw.len() - 1]
                .replace("\\'", "'")
                .replace("\\\\", "\\"),
        );
    }

    None
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn parses_capped_file_read() {
        let value = json!({
            "type": "response_item",
            "payload": {
                "type": "custom_tool_call",
                "call_id": "call-1",
                "name": "exec",
                "status": "completed",
                "input": r#"text(await tools.exec_command({cmd:"Get-Content README.md",workdir:"C:\\fixture",max_output_tokens:5000}))"#
            }
        });

        let batch = extract_session_tools(&value).expect("tools");
        assert_eq!(batch.parse_failures, 0);
        assert_eq!(batch.observations.len(), 1);
        assert_eq!(batch.observations[0].category, "file_read");
        assert_eq!(batch.observations[0].max_output_tokens, Some(5_000));
    }

    #[test]
    fn missing_cap_is_known_absence_not_parse_failure() {
        let value = json!({
            "type": "response_item",
            "payload": {
                "type": "custom_tool_call",
                "call_id": "call-2",
                "name": "exec",
                "status": "completed",
                "input": r#"text(await tools.exec_command({cmd:"rg TODO src",workdir:"C:\\fixture"}))"#
            }
        });

        let batch = extract_session_tools(&value).expect("tools");
        assert_eq!(batch.parse_failures, 0);
        assert_eq!(batch.observations[0].category, "search");
        assert_eq!(batch.observations[0].max_output_tokens, None);
    }

    #[test]
    fn non_exec_code_mode_call_is_not_a_parse_failure() {
        let value = json!({
            "type": "response_item",
            "payload": {
                "type": "custom_tool_call",
                "call_id": "call-web",
                "name": "exec",
                "status": "completed",
                "input": r#"text(await tools.web__run("fast|fixture"))"#
            }
        });

        assert!(extract_session_tools(&value).is_none());
    }

    #[test]
    fn parses_multiple_exec_commands_in_one_code_mode_call() {
        let value = json!({
            "type": "response_item",
            "payload": {
                "type": "custom_tool_call",
                "call_id": "call-many",
                "name": "exec",
                "status": "completed",
                "input": r#"const a=await tools.exec_command({cmd:"Get-Content a.txt",max_output_tokens:5000}); const b=await tools.exec_command({cmd:"rg TODO src",max_output_tokens:3000}); text(a); text(b);"#
            }
        });

        let batch = extract_session_tools(&value).expect("tools");
        assert_eq!(batch.parse_failures, 0);
        assert_eq!(batch.observations.len(), 2);
        assert_eq!(batch.observations[0].category, "file_read");
        assert_eq!(batch.observations[1].category, "search");
    }

    #[test]
    fn malformed_exec_is_explicit_parse_failure() {
        let value = json!({
            "type": "response_item",
            "payload": {
                "type": "custom_tool_call",
                "call_id": "call-3",
                "name": "exec",
                "status": "completed",
                "input": "text(await tools.exec_command("
            }
        });

        let batch = extract_session_tools(&value).expect("tools");
        assert_eq!(batch.parse_failures, 1);
        assert_eq!(batch.observations.len(), 1);
        assert!(batch.observations[0].parse_error.is_some());
    }

    #[test]
    fn extracts_custom_tool_result_output_without_normalizing_raw_content() {
        let value = json!({
            "type": "response_item",
            "payload": {
                "type": "custom_tool_call_output",
                "call_id": "call-1",
                "output": "fixture result"
            }
        });

        let result = extract_session_tool_result_output(&value).expect("tool output");
        assert_eq!(result.call_id, "call-1");
        assert_eq!(result.output, "fixture result");
    }

    #[test]
    fn ignores_non_tool_output_records() {
        let value = json!({
            "type": "response_item",
            "payload": {"type": "message", "output": "fixture"}
        });
        assert!(extract_session_tool_result_output(&value).is_none());
    }
}
