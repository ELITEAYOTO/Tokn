use std::path::Path;

use tokn_analysis::TokenLedger;
use tokn_codex::diagnostic::{DiagnosticBundle, extract_trace_usage_record_from_bundle};
use tokn_codex::session::{extract_token_usage, extract_usage_record};
use tokn_domain::{RunId, SourceId};
use tokn_ingest::{JsonlConfig, read_jsonl};
use tokn_storage::RunRecord;

use super::common::{open_db, resolve_source};

pub fn run(source: &str) -> anyhow::Result<()> {
    let path = resolve_source(source)?;
    run_path(&path)
}

pub fn run_path(path: &Path) -> anyhow::Result<()> {
    let (data_path, source_kind, bundle_root) = if path.is_dir() {
        let bundle = DiagnosticBundle::detect(path)
            .ok_or_else(|| anyhow::anyhow!("not a valid diagnostic trace bundle"))?;
        (
            bundle.trace.clone(),
            "codex-diagnostic-trace",
            Some(bundle.root),
        )
    } else {
        (path.to_path_buf(), "codex-session", None)
    };

    let mut run_id = RunId::new();
    let mut source_id = SourceId::new();
    let mut structured = TokenLedger::default();
    let mut fallback = TokenLedger::default();
    let mut diagnostic = TokenLedger::default();

    let stats = read_jsonl(&data_path, &JsonlConfig::default(), |record| {
        if source_kind == "codex-session" {
            if let Some(observation) = extract_usage_record(&record.value) {
                let key = observation
                    .response_id
                    .as_ref()
                    .map(|id| format!("response:{id}"));
                structured.push_keyed(key, observation.usage);
            }

            if let Some(usage) = extract_token_usage(&record.value) {
                fallback.push(usage);
            }
        } else if let Some(root) = bundle_root.as_deref()
            && let Some(observation) = extract_trace_usage_record_from_bundle(&record.value, root)
        {
            let key = observation
                .inference_call_id
                .as_ref()
                .map(|id| format!("inference:{id}"))
                .or_else(|| {
                    observation.response_id.as_ref().map(|response| {
                        observation
                            .provider
                            .as_ref()
                            .map(|provider| format!("response:{provider}:{response}"))
                            .unwrap_or_else(|| format!("response:{response}"))
                    })
                });
            diagnostic.push_keyed(key, observation.usage);
        }
    })?;
    if !stats.snapshot_hash.is_empty() {
        let short = &stats.snapshot_hash[..24.min(stats.snapshot_hash.len())];
        run_id = RunId(format!("run-{short}"));
        source_id = SourceId(format!("src-{short}"));
    }

    let (ledger, accounting_mode) = if source_kind == "codex-diagnostic-trace" {
        (&diagnostic, "diagnostic-usage")
    } else if !structured.accepted.is_empty() {
        (&structured, "token_usage_record")
    } else {
        (&fallback, "token_count-fallback")
    };

    let summary = ledger.summary();
    let record = RunRecord {
        run_id: run_id.0.clone(),
        source_path: path.to_string_lossy().to_string(),
        accounting_mode: accounting_mode.to_string(),
        duplicates_suppressed: i64::try_from(ledger.duplicates_suppressed)?,
        usage_conflicts: i64::try_from(ledger.usage_conflicts)?,
        records_seen: i64::try_from(stats.records_seen)?,
        records_valid: i64::try_from(stats.records_valid)?,
        malformed_records: i64::try_from(stats.malformed_records)?,
        oversized_records: i64::try_from(stats.oversized_records)?,
        truncated_tail: i64::try_from(stats.truncated_tail)?,
        usage_records: i64::try_from(summary.records)?,
        input_tokens: i64::try_from(summary.input_tokens)?,
        cached_input_tokens: i64::try_from(summary.cached_input_tokens)?,
        cache_write_input_tokens: i64::try_from(summary.cache_write_input_tokens)?,
        output_tokens: i64::try_from(summary.output_tokens)?,
        reasoning_output_tokens: i64::try_from(summary.reasoning_output_tokens)?,
        logical_tokens: i64::try_from(summary.logical_total())?,
        invariant_conflicts: i64::try_from(ledger.rejected_invariant_count)?,
    };

    let db = open_db()?;
    db.save_run(
        &run_id,
        &source_id,
        path,
        source_kind,
        i64::try_from(stats.snapshot_bytes)?,
        &record,
    )?;

    println!("imported run {}", run_id.0);
    println!("source kind {}", source_kind);
    println!("accounting {}", accounting_mode);
    println!("records {}", stats.records_seen);
    println!("usage records {}", summary.records);
    println!("duplicates suppressed {}", ledger.duplicates_suppressed);
    println!("usage conflicts {}", ledger.usage_conflicts);
    Ok(())
}
