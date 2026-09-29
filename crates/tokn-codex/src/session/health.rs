use std::path::Path;

use anyhow::Result;
use tokn_domain::{SourceCoverage, SourceHealth, SourceHealthStatus};
use tokn_ingest::{JsonlConfig, read_jsonl};

use super::extract_usage_record;

pub fn assess_session_health(path: &Path) -> Result<SourceHealth> {
    let mut coverage = SourceCoverage::default();
    let mut session_meta = 0_u64;

    let stats = read_jsonl(path, &JsonlConfig::default(), |record| {
        let top_type = record.value.get("type").and_then(|value| value.as_str());
        let payload_type = record
            .value
            .get("payload")
            .and_then(|payload| payload.get("type"))
            .and_then(|value| value.as_str());

        if top_type == Some("session_meta") {
            session_meta += 1;
            coverage.protocol_events += 1;
        }
        if top_type == Some("token_usage_record") && extract_usage_record(&record.value).is_some() {
            coverage.usage_records += 1;
        }
        if top_type == Some("response_item")
            && matches!(
                payload_type,
                Some(
                    "custom_tool_call"
                        | "custom_tool_call_output"
                        | "function_call"
                        | "function_call_output"
                )
            )
        {
            coverage.tool_events += 1;
        }
        if top_type == Some("event_msg") {
            match payload_type {
                Some("task_complete") => coverage.terminal_events += 1,
                Some("task_started" | "thread_settings_applied") => coverage.protocol_events += 1,
                _ => {}
            }
        }
    })?;

    coverage.records_seen = stats.records_seen;
    coverage.records_valid = stats.records_valid;
    coverage.malformed_records = stats.malformed_records;

    let mut reasons = Vec::new();
    let status = if coverage.records_seen == 0 {
        reasons.push("session contains no records".into());
        SourceHealthStatus::Empty
    } else if coverage.records_valid == 0 && coverage.malformed_records > 0 {
        reasons.push("session has no valid records".into());
        SourceHealthStatus::Malformed
    } else if coverage.malformed_records > 0 || stats.truncated_tail > 0 {
        reasons.push("session contains malformed or truncated evidence".into());
        SourceHealthStatus::Partial
    } else if session_meta > 0 && coverage.usage_records > 0 {
        SourceHealthStatus::Healthy
    } else {
        if session_meta == 0 {
            reasons.push("session_meta was not observed".into());
        }
        if coverage.usage_records == 0 {
            reasons.push("token_usage_record was not observed".into());
        }
        SourceHealthStatus::Partial
    };

    Ok(SourceHealth {
        status,
        coverage,
        reasons,
    })
}
