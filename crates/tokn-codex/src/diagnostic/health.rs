use anyhow::Result;
use tokn_domain::{SourceCoverage, SourceHealth, SourceHealthStatus};
use tokn_ingest::{JsonlConfig, read_jsonl};

use super::{DiagnosticBundle, extract_trace_usage_record_from_bundle};

pub fn assess_diagnostic_health(bundle: &DiagnosticBundle) -> Result<SourceHealth> {
    let mut coverage = SourceCoverage::default();

    let stats = read_jsonl(&bundle.trace, &JsonlConfig::default(), |record| {
        let payload_type = record
            .value
            .get("payload")
            .and_then(|payload| payload.get("type"))
            .and_then(|value| value.as_str());

        match payload_type {
            Some(
                "inference_started"
                | "inference_completed"
                | "inference_cancelled"
                | "inference_failed",
            ) => coverage.inference_events += 1,
            Some("tool_call_started" | "tool_call_ended") => coverage.tool_events += 1,
            Some("task_complete") => coverage.terminal_events += 1,
            Some(
                "rollout_started"
                | "thread_started"
                | "protocol_event_observed"
                | "thread_settings_applied",
            ) => coverage.protocol_events += 1,
            _ => {}
        }

        if extract_trace_usage_record_from_bundle(&record.value, &bundle.root).is_some() {
            coverage.usage_records += 1;
        }
    })?;

    coverage.records_seen = stats.records_seen;
    coverage.records_valid = stats.records_valid;
    coverage.malformed_records = stats.malformed_records;

    let mut reasons = Vec::new();
    let status = if coverage.records_seen == 0 {
        reasons.push("trace contains no records".into());
        SourceHealthStatus::Empty
    } else if coverage.records_valid == 0 && coverage.malformed_records > 0 {
        reasons.push("trace has no valid records".into());
        SourceHealthStatus::Malformed
    } else if coverage.malformed_records > 0 || stats.truncated_tail > 0 {
        reasons.push("trace contains malformed or truncated evidence".into());
        SourceHealthStatus::Partial
    } else if coverage.usage_records > 0 && coverage.inference_events > 0 {
        SourceHealthStatus::Healthy
    } else {
        if coverage.inference_events == 0 {
            reasons.push("no inference events observed".into());
        }
        if coverage.usage_records == 0 {
            reasons.push("no model usage observed".into());
        }
        if coverage.tool_events == 0 {
            reasons.push("no diagnostic tool events observed".into());
        }
        SourceHealthStatus::Partial
    };

    Ok(SourceHealth {
        status,
        coverage,
        reasons,
    })
}
