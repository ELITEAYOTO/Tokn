use std::io::{self, BufRead, BufWriter, Write};

use anyhow::Context;
use serde_json::{Value, json};
use tokn_domain::{MEASUREMENT_CONTRACT_ID, MEASUREMENT_CONTRACT_VERSION};
use tokn_storage::{Database, MeasurementRunHistoryRecord};

const SERVER_NAME: &str = "tokn-mcp";
const LATEST_PROTOCOL_VERSION: &str = "2025-11-25";
const SUPPORTED_PROTOCOL_VERSIONS: &[&str] =
    &["2025-11-25", "2025-06-18", "2025-03-26", "2024-11-05"];

pub fn run_stdio(db: Database) -> anyhow::Result<()> {
    let stdin = io::stdin();
    let stdout = io::stdout();
    let mut input = stdin.lock();
    let mut output = BufWriter::new(stdout.lock());
    let mut server = McpServer::new(db);
    let mut line = String::new();

    loop {
        line.clear();
        let read = input.read_line(&mut line).context("read MCP stdin")?;
        if read == 0 {
            break;
        }
        if line.trim().is_empty() {
            continue;
        }

        let response = match serde_json::from_str::<Value>(line.trim_end()) {
            Ok(message) => server.handle_message(message),
            Err(_) => Some(rpc_error(Value::Null, -32700, "parse error")),
        };

        if let Some(response) = response {
            serde_json::to_writer(&mut output, &response).context("write MCP response")?;
            output.write_all(b"\n").context("terminate MCP response")?;
            output.flush().context("flush MCP response")?;
        }
    }

    Ok(())
}

struct McpServer {
    db: Database,
    initialized: bool,
    protocol_version: Option<String>,
}

impl McpServer {
    fn new(db: Database) -> Self {
        Self {
            db,
            initialized: false,
            protocol_version: None,
        }
    }

    fn handle_message(&mut self, message: Value) -> Option<Value> {
        let Some(object) = message.as_object() else {
            return Some(rpc_error(Value::Null, -32600, "invalid request"));
        };

        let id = object.get("id").cloned();
        let is_notification = id.is_none();

        if object.get("jsonrpc").and_then(Value::as_str) != Some("2.0") {
            return if is_notification {
                None
            } else {
                Some(rpc_error(
                    id.unwrap_or(Value::Null),
                    -32600,
                    "invalid request",
                ))
            };
        }

        let Some(method) = object.get("method").and_then(Value::as_str) else {
            return if is_notification {
                None
            } else {
                Some(rpc_error(
                    id.unwrap_or(Value::Null),
                    -32600,
                    "invalid request",
                ))
            };
        };

        if is_notification {
            self.handle_notification(method);
            return None;
        }

        let id = id.unwrap_or(Value::Null);
        let params = object.get("params").cloned().unwrap_or_else(|| json!({}));

        match method {
            "initialize" => Some(match self.initialize(&params) {
                Ok(result) => rpc_result(id, result),
                Err(error) => rpc_error(id, error.code, &error.message),
            }),
            "ping" => Some(rpc_result(id, json!({}))),
            "tools/list" => Some(if self.initialized {
                rpc_result(id, self.tools_list())
            } else {
                rpc_error(id, -32002, "server not initialized")
            }),
            "tools/call" => Some(if self.initialized {
                match self.tools_call(&params) {
                    Ok(result) => rpc_result(id, result),
                    Err(error) => rpc_error(id, error.code, &error.message),
                }
            } else {
                rpc_error(id, -32002, "server not initialized")
            }),
            _ => Some(rpc_error(id, -32601, "method not found")),
        }
    }

    fn handle_notification(&mut self, method: &str) {
        if method == "notifications/initialized" {
            self.initialized = true;
        }
    }

    fn initialize(&mut self, params: &Value) -> Result<Value, RpcFailure> {
        let requested = params
            .get("protocolVersion")
            .and_then(Value::as_str)
            .unwrap_or(LATEST_PROTOCOL_VERSION);
        let negotiated = if SUPPORTED_PROTOCOL_VERSIONS.contains(&requested) {
            requested
        } else {
            LATEST_PROTOCOL_VERSION
        };

        self.protocol_version = Some(negotiated.to_string());
        // Compatibility: allow tools immediately after initialize response while
        // still accepting the standard notifications/initialized lifecycle message.
        self.initialized = true;

        Ok(json!({
            "protocolVersion": negotiated,
            "capabilities": {
                "tools": {
                    "listChanged": false
                }
            },
            "serverInfo": {
                "name": SERVER_NAME,
                "version": env!("CARGO_PKG_VERSION")
            },
            "instructions": "Tokn local read-only prototype. It reports measurement/store status and recent persisted runs. It performs no active optimization."
        }))
    }

    fn tools_list(&self) -> Value {
        json!({
            "tools": [
                {
                    "name": "tokn_status",
                    "description": "Return Tokn MCP, measurement-contract and local Store status. Read-only.",
                    "inputSchema": {
                        "type": "object",
                        "properties": {},
                        "additionalProperties": false
                    }
                },
                {
                    "name": "tokn_recent_runs",
                    "description": "Return recent persisted Tokn measurement runs from the shared local Store. Read-only.",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "limit": {
                                "type": "integer",
                                "minimum": 1,
                                "maximum": 100,
                                "default": 10
                            }
                        },
                        "additionalProperties": false
                    }
                }
            ]
        })
    }

    fn tools_call(&self, params: &Value) -> Result<Value, RpcFailure> {
        let name = params
            .get("name")
            .and_then(Value::as_str)
            .ok_or_else(|| RpcFailure::invalid_params("missing tool name"))?;
        let arguments = params
            .get("arguments")
            .cloned()
            .unwrap_or_else(|| json!({}));

        match name {
            "tokn_status" => {
                ensure_object(&arguments)?;
                let counts = self
                    .db
                    .measurement_store_counts()
                    .map_err(|_| RpcFailure::internal("Store status query failed"))?;
                Ok(tool_text(json!({
                    "server": {
                        "name": SERVER_NAME,
                        "version": env!("CARGO_PKG_VERSION"),
                        "transport": "stdio",
                        "read_only": true,
                        "protocol_version": self.protocol_version
                    },
                    "measurement_contract": {
                        "id": MEASUREMENT_CONTRACT_ID,
                        "version": MEASUREMENT_CONTRACT_VERSION
                    },
                    "store": {
                        "schema_version": counts.schema_version,
                        "projects": counts.projects,
                        "workspaces": counts.workspaces,
                        "runs": counts.runs,
                        "agents": counts.agents,
                        "runtime_profiles": counts.runtime_profiles
                    },
                    "tools": [
                        "tokn_status",
                        "tokn_recent_runs"
                    ]
                })))
            }
            "tokn_recent_runs" => {
                ensure_object(&arguments)?;
                let limit = arguments
                    .get("limit")
                    .map(|value| {
                        value
                            .as_u64()
                            .ok_or_else(|| RpcFailure::invalid_params("limit must be an integer"))
                    })
                    .transpose()?
                    .unwrap_or(10);
                if !(1..=100).contains(&limit) {
                    return Err(RpcFailure::invalid_params(
                        "limit must be between 1 and 100",
                    ));
                }

                let runs = self
                    .db
                    .recent_measurement_runs(limit as usize)
                    .map_err(|_| RpcFailure::internal("Store history query failed"))?;
                let records = runs.iter().map(run_json).collect::<Vec<_>>();
                Ok(tool_text(json!({
                    "count": records.len(),
                    "runs": records
                })))
            }
            _ => Err(RpcFailure::invalid_params("unknown tool")),
        }
    }
}

fn ensure_object(value: &Value) -> Result<(), RpcFailure> {
    if value.is_object() {
        Ok(())
    } else {
        Err(RpcFailure::invalid_params(
            "tool arguments must be an object",
        ))
    }
}

fn run_json(run: &MeasurementRunHistoryRecord) -> Value {
    json!({
        "run_id": run.run_id,
        "project_id": run.project_id,
        "workspace_id": run.workspace_id,
        "profile_id": run.profile_id,
        "measurement_contract_version": run.measurement_contract_version,
        "evidence_layout_version": run.evidence_layout_version,
        "root_thread_id": run.root_thread_id,
        "root_terminal": run.root_terminal,
        "source_health": run.source_health,
        "validity_verdict": run.validity_verdict,
        "quality_status": run.quality_status,
        "agent_count": run.agent_count,
        "usage_records": run.usage_records,
        "input_tokens": run.input_tokens,
        "cached_input_tokens": run.cached_input_tokens,
        "cache_write_input_tokens": run.cache_write_input_tokens,
        "output_tokens": run.output_tokens,
        "reasoning_output_tokens": run.reasoning_output_tokens,
        "logical_tokens": run.logical_tokens,
        "created_at_unix": run.created_at_unix
    })
}

fn tool_text(value: Value) -> Value {
    let text = serde_json::to_string_pretty(&value).unwrap_or_else(|_| "{}".into());
    json!({
        "content": [
            {
                "type": "text",
                "text": text
            }
        ]
    })
}

fn rpc_result(id: Value, result: Value) -> Value {
    json!({
        "jsonrpc": "2.0",
        "id": id,
        "result": result
    })
}

fn rpc_error(id: Value, code: i64, message: &str) -> Value {
    json!({
        "jsonrpc": "2.0",
        "id": id,
        "error": {
            "code": code,
            "message": message
        }
    })
}

struct RpcFailure {
    code: i64,
    message: String,
}

impl RpcFailure {
    fn invalid_params(message: &str) -> Self {
        Self {
            code: -32602,
            message: message.into(),
        }
    }

    fn internal(message: &str) -> Self {
        Self {
            code: -32603,
            message: message.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::PathBuf;

    fn temp_db(name: &str) -> PathBuf {
        std::env::temp_dir().join(format!("tokn-mcp-{name}-{}.sqlite3", std::process::id()))
    }

    fn server(name: &str) -> (McpServer, PathBuf) {
        let path = temp_db(name);
        let _ = fs::remove_file(&path);
        let db = Database::open(&path).expect("open test store");
        (McpServer::new(db), path)
    }

    fn initialize(server: &mut McpServer) -> Value {
        server
            .handle_message(json!({
                "jsonrpc": "2.0",
                "id": 1,
                "method": "initialize",
                "params": {
                    "protocolVersion": "2025-06-18",
                    "clientInfo": {
                        "name": "fixture-client",
                        "version": "1"
                    },
                    "capabilities": {}
                }
            }))
            .expect("initialize response")
    }

    #[test]
    fn initialize_and_list_tools() {
        let (mut server, path) = server("initialize");
        let response = initialize(&mut server);
        assert_eq!(
            response["result"]["protocolVersion"].as_str(),
            Some("2025-06-18")
        );
        assert_eq!(
            response["result"]["serverInfo"]["name"].as_str(),
            Some(SERVER_NAME)
        );

        let tools = server
            .handle_message(json!({
                "jsonrpc": "2.0",
                "id": 2,
                "method": "tools/list",
                "params": {}
            }))
            .expect("tools response");
        assert_eq!(tools["result"]["tools"].as_array().map(Vec::len), Some(2));

        drop(server);
        let _ = fs::remove_file(path);
    }

    #[test]
    fn status_tool_reports_empty_store_without_database_path() {
        let (mut server, path) = server("status");
        initialize(&mut server);

        let response = server
            .handle_message(json!({
                "jsonrpc": "2.0",
                "id": "status",
                "method": "tools/call",
                "params": {
                    "name": "tokn_status",
                    "arguments": {}
                }
            }))
            .expect("status response");
        let text = response["result"]["content"][0]["text"]
            .as_str()
            .expect("tool text");
        let payload: Value = serde_json::from_str(text).expect("status json");
        assert_eq!(payload["store"]["schema_version"].as_str(), Some("2"));
        assert_eq!(payload["store"]["runs"].as_i64(), Some(0));
        assert!(payload.get("db").is_none());
        assert!(!text.contains(&path.to_string_lossy().to_string()));

        drop(server);
        let _ = fs::remove_file(path);
    }

    #[test]
    fn recent_runs_is_read_only_and_empty_on_fresh_store() {
        let (mut server, path) = server("history");
        initialize(&mut server);

        let response = server
            .handle_message(json!({
                "jsonrpc": "2.0",
                "id": 3,
                "method": "tools/call",
                "params": {
                    "name": "tokn_recent_runs",
                    "arguments": {
                        "limit": 5
                    }
                }
            }))
            .expect("history response");
        let text = response["result"]["content"][0]["text"]
            .as_str()
            .expect("tool text");
        let payload: Value = serde_json::from_str(text).expect("history json");
        assert_eq!(payload["count"].as_u64(), Some(0));
        assert_eq!(payload["runs"].as_array().map(Vec::len), Some(0));

        drop(server);
        let _ = fs::remove_file(path);
    }

    #[test]
    fn notifications_and_errors_are_structured() {
        let (mut server, path) = server("errors");
        assert!(
            server
                .handle_message(json!({
                    "jsonrpc": "2.0",
                    "method": "notifications/initialized"
                }))
                .is_none()
        );

        let missing = server
            .handle_message(json!({
                "jsonrpc": "2.0",
                "id": 4,
                "method": "does/not/exist"
            }))
            .expect("error response");
        assert_eq!(missing["error"]["code"].as_i64(), Some(-32601));

        let invalid = server
            .handle_message(json!({
                "jsonrpc": "2.0",
                "id": 5,
                "method": "tools/call",
                "params": {
                    "name": "tokn_recent_runs",
                    "arguments": {
                        "limit": 0
                    }
                }
            }))
            .expect("invalid params response");
        assert_eq!(invalid["error"]["code"].as_i64(), Some(-32602));

        drop(server);
        let _ = fs::remove_file(path);
    }
}
