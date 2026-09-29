use tokn_domain::{
    EventId, EventScope, PbEvent, Provenance, RunId, SourceId, SourceKind, TruthLevel,
};
use tokn_ingest::ParsedRecord;

use super::extract_token_usage;

pub fn adapt_record(run_id: &RunId, source_id: &SourceId, record: &ParsedRecord) -> PbEvent {
    let kind = record
        .value
        .get("payload")
        .and_then(|v| v.get("type"))
        .and_then(|v| v.as_str())
        .or_else(|| record.value.get("type").and_then(|v| v.as_str()))
        .unwrap_or("unknown")
        .to_string();

    let source_timestamp = record
        .value
        .get("timestamp")
        .and_then(|v| v.as_str())
        .map(str::to_string);

    PbEvent {
        schema: "toknevent/0.1".into(),
        event_id: EventId::new(),
        run_id: run_id.clone(),
        kind,
        source_timestamp,
        scope: EventScope::default(),
        truth: TruthLevel::Observed,
        source: Provenance {
            source_id: source_id.clone(),
            source_kind: SourceKind::CodexSession,
            byte_offset: Some(record.offset),
            byte_length: Some(record.byte_length),
            physical_line: Some(record.line),
            source_seq: record.value.get("ordinal").and_then(|v| v.as_u64()),
            record_hash: record.hash.clone(),
            adapter_name: "codex-session".into(),
            adapter_version: "0.1".into(),
        },
        token_usage: extract_token_usage(&record.value),
        metadata: serde_json::json!({
            "top_level_type": record.value.get("type").and_then(|v| v.as_str()),
            "payload_type": record.value.get("payload")
                .and_then(|v| v.get("type"))
                .and_then(|v| v.as_str())
        }),
    }
}
