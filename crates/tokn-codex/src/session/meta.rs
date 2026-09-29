use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionMetadata {
    pub thread_id: Option<String>,
    pub session_id: Option<String>,
    pub parent_thread_id: Option<String>,
    pub forked_from_id: Option<String>,
    pub thread_source: Option<String>,
    pub agent_nickname: Option<String>,
    pub agent_path: Option<String>,
    pub cwd: Option<String>,
    pub originator: Option<String>,
    pub cli_version: Option<String>,
    pub timestamp: Option<String>,
}

impl SessionMetadata {
    pub fn is_subagent(&self) -> bool {
        self.parent_thread_id.is_some() || self.thread_source.as_deref() == Some("subagent")
    }

    pub fn is_root_candidate(&self) -> bool {
        !self.is_subagent()
    }
}

pub fn read_session_metadata(path: &Path) -> Result<Option<SessionMetadata>> {
    let file = File::open(path).with_context(|| format!("open {}", path.display()))?;

    for line in BufReader::new(file).lines().take(64) {
        let line = line?;
        let Ok(record) = serde_json::from_str::<Value>(&line) else {
            continue;
        };
        if record.get("type").and_then(Value::as_str) != Some("session_meta") {
            continue;
        }
        let Some(payload) = record.get("payload") else {
            continue;
        };

        return Ok(Some(SessionMetadata {
            thread_id: text(payload, "id"),
            session_id: text(payload, "session_id"),
            parent_thread_id: text(payload, "parent_thread_id"),
            forked_from_id: text(payload, "forked_from_id"),
            thread_source: text(payload, "thread_source"),
            agent_nickname: text(payload, "agent_nickname"),
            agent_path: text(payload, "agent_path"),
            cwd: text(payload, "cwd"),
            originator: text(payload, "originator"),
            cli_version: text(payload, "cli_version"),
            timestamp: text(payload, "timestamp"),
        }));
    }

    Ok(None)
}

fn text(value: &Value, key: &str) -> Option<String> {
    value.get(key).and_then(Value::as_str).map(str::to_string)
}
