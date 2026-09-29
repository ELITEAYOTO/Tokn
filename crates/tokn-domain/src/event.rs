use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{EventId, Provenance, RunId, TokenUsage, TruthLevel};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EventScope {
    pub thread_id: Option<String>,
    pub root_turn_id: Option<String>,
    pub turn_id: Option<String>,
    pub inference_call_id: Option<String>,
    pub response_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PbEvent {
    pub schema: String,
    pub event_id: EventId,
    pub run_id: RunId,
    pub kind: String,
    pub source_timestamp: Option<String>,
    pub scope: EventScope,
    pub truth: TruthLevel,
    pub source: Provenance,
    pub token_usage: Option<TokenUsage>,
    pub metadata: Value,
}
