use serde_json::Value;

#[derive(Debug, Clone)]
pub struct ParsedRecord {
    pub offset: u64,
    pub line: u64,
    pub byte_length: u64,
    pub hash: String,
    pub value: Value,
}

#[derive(Debug, Clone, Default)]
pub struct IngestStats {
    pub records_seen: u64,
    pub records_valid: u64,
    pub malformed_records: u64,
    pub oversized_records: u64,
    pub truncated_tail: u64,
    pub snapshot_bytes: u64,
    pub snapshot_hash: String,
}
