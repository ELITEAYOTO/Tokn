use serde::{Deserialize, Serialize};

use crate::TokenUsage;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct InferenceObservation {
    pub inference_call_id: String,
    pub thread_id: Option<String>,
    pub turn_id: Option<String>,
    pub model: Option<String>,
    pub status: String,
    pub started_seq: Option<u64>,
    pub ended_seq: Option<u64>,
    pub started_at_unix_ms: Option<i64>,
    pub ended_at_unix_ms: Option<i64>,
    pub request_payload_bytes: Option<u64>,
    pub request_input_items: Option<u64>,
    pub usage: Option<TokenUsage>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ToolObservation {
    pub tool_call_id: String,
    pub kind: String,
    pub tool_name: Option<String>,
    pub category: String,
    pub surface: String,
    pub requester_type: Option<String>,
    pub status: String,
    pub observed_at: Option<String>,
    pub started_seq: Option<u64>,
    pub ended_seq: Option<u64>,
    pub invocation_payload_bytes: Option<u64>,
    pub result_payload_bytes: Option<u64>,
    pub result_output_chars: Option<u64>,
    pub max_output_tokens: Option<u64>,
    pub original_token_count: Option<u64>,
    pub command: Option<String>,
    pub workdir: Option<String>,
    pub parse_error: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DiagnosticObservations {
    pub inferences: Vec<InferenceObservation>,
    pub tools: Vec<ToolObservation>,
    pub code_cells_started: u64,
    pub compactions: u64,
}
