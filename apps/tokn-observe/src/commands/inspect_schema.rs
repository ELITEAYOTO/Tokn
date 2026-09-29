use tokn_ingest::{JsonlConfig, SchemaInspector, read_jsonl};

use super::common::resolve_source;

pub fn run(source: &str) -> anyhow::Result<()> {
    let path = resolve_source(source)?;
    let mut inspector = SchemaInspector::default();
    let stats = read_jsonl(&path, &JsonlConfig::default(), |record| {
        inspector.observe(&record.value);
    })?;

    println!("TOKN SCHEMA INSPECTOR");
    println!("source: {}", path.display());
    println!("records: {}", stats.records_seen);
    println!("valid: {}", stats.records_valid);
    println!("malformed: {}", stats.malformed_records);
    println!("truncated tail: {}", stats.truncated_tail);
    println!();
    println!("TOP LEVEL TYPES");
    for (kind, count) in inspector.top_types {
        println!("  {:>8}  {}", count, kind);
    }
    println!();
    println!("PAYLOAD TYPES");
    for (kind, count) in inspector.payload_types {
        println!("  {:>8}  {}", count, kind);
    }
    Ok(())
}
