use std::fmt;

use serde::{Deserialize, Serialize};
use tokn_domain::{EvidenceIdentityCoverage, TOOL_ACTIVITY_HISTORY_SCHEMA_VERSION, ToolActivityHistory};

pub const SOURCE_MUTATION_HISTORY_SCHEMA_VERSION: u64 = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum MutationTimingCoverage {
    Observed,
    NotCaptured,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum MutationEffectStatus {
    NotVerified,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceMutationEvent {
    pub activity_id: String,
    pub run_id: String,
    pub thread_id: String,
    pub source_stable_id: String,
    pub observed_at: Option<String>,
    pub timing_coverage: MutationTimingCoverage,
    pub tool_status: String,
    pub effect_status: MutationEffectStatus,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceMutationHistoryReport {
    pub schema_version: u64,
    pub source_activity_history_schema_version: u64,
    pub project_filter: Option<String>,
    pub workspace_filter: Option<String>,
    pub run_limit: u64,
    pub events: Vec<SourceMutationEvent>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SourceMutationHistoryBuildError {
    UnsupportedToolActivityHistorySchema { actual: u64, expected: u64 },
}

impl fmt::Display for SourceMutationHistoryBuildError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedToolActivityHistorySchema { actual, expected } => write!(
                formatter,
                "unsupported ToolActivityHistory schema_version {actual}; expected {expected}"
            ),
        }
    }
}

impl std::error::Error for SourceMutationHistoryBuildError {}

pub fn build_source_mutation_history(
    history: &ToolActivityHistory,
) -> Result<SourceMutationHistoryReport, SourceMutationHistoryBuildError> {
    if history.schema_version != TOOL_ACTIVITY_HISTORY_SCHEMA_VERSION {
        return Err(SourceMutationHistoryBuildError::UnsupportedToolActivityHistorySchema {
            actual: history.schema_version,
            expected: TOOL_ACTIVITY_HISTORY_SCHEMA_VERSION,
        });
    }

    let mut events = history
        .activities
        .iter()
        .filter(|activity| activity.category == "write_mutation")
        .filter(|activity| activity.source_identity_coverage == EvidenceIdentityCoverage::Observed)
        .filter_map(|activity| {
            let source_stable_id = activity.source_stable_id.clone()?;
            let observed_at = activity
                .observed_at
                .as_deref()
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(str::to_string);
            let timing_coverage = if observed_at.is_some() {
                MutationTimingCoverage::Observed
            } else {
                MutationTimingCoverage::NotCaptured
            };
            Some((
                activity.run_created_at_unix,
                SourceMutationEvent {
                    activity_id: activity.activity_id.clone(),
                    run_id: activity.run_id.clone(),
                    thread_id: activity.thread_id.clone(),
                    source_stable_id,
                    observed_at,
                    timing_coverage,
                    tool_status: activity.status.clone(),
                    effect_status: MutationEffectStatus::NotVerified,
                },
            ))
        })
        .collect::<Vec<_>>();

    events.sort_by(|(left_run, left), (right_run, right)| {
        left_run
            .cmp(right_run)
            .then(left.observed_at.cmp(&right.observed_at))
            .then(left.activity_id.cmp(&right.activity_id))
    });
    let events = events.into_iter().map(|(_, event)| event).collect();

    Ok(SourceMutationHistoryReport {
        schema_version: SOURCE_MUTATION_HISTORY_SCHEMA_VERSION,
        source_activity_history_schema_version: history.schema_version,
        project_filter: history.project_filter.clone(),
        workspace_filter: history.workspace_filter.clone(),
        run_limit: history.run_limit,
        events,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokn_domain::HistoricalToolActivityRecord;
    fn history(activities: Vec<HistoricalToolActivityRecord>) -> ToolActivityHistory {
        ToolActivityHistory {
            schema_version: TOOL_ACTIVITY_HISTORY_SCHEMA_VERSION,
            project_filter: Some("prj-fixture".into()),
            workspace_filter: None,
            run_limit: 50,
            activities,
        }
    }

    #[test]
    fn reports_observed_mutation_timing_without_claiming_effect() {
        let activity = HistoricalToolActivityRecord {
            activity_id: "act-1".into(),
            run_id: "run-1".into(),
            thread_id: "thread-1".into(),
            category: "write_mutation".into(),
            status: "completed".into(),
            observed_at: Some("2026-10-03T12:00:00Z".into()),
            source_stable_id: Some("src-v1-fixture".into()),
            source_identity_coverage: EvidenceIdentityCoverage::Observed,
            run_created_at_unix: 1,
            ..Default::default()
        };
        let report = build_source_mutation_history(&history(vec![activity])).expect("report");
        assert_eq!(report.events.len(), 1);
        assert_eq!(report.events[0].timing_coverage, MutationTimingCoverage::Observed);
        assert_eq!(report.events[0].effect_status, MutationEffectStatus::NotVerified);
    }

    #[test]
    fn missing_timestamp_is_explicitly_not_captured() {
        let activity = HistoricalToolActivityRecord {
            activity_id: "act-2".into(),
            run_id: "run-1".into(),
            thread_id: "thread-1".into(),
            category: "write_mutation".into(),
            source_stable_id: Some("src-v1-fixture".into()),
            source_identity_coverage: EvidenceIdentityCoverage::Observed,
            ..Default::default()
        };
        let report = build_source_mutation_history(&history(vec![activity])).expect("report");
        assert_eq!(report.events[0].timing_coverage, MutationTimingCoverage::NotCaptured);
        assert!(report.events[0].observed_at.is_none());
    }

    #[test]
    fn ignores_unproven_source_identity() {
        let activity = HistoricalToolActivityRecord {
            category: "write_mutation".into(),
            source_identity_coverage: EvidenceIdentityCoverage::NotCaptured,
            ..Default::default()
        };
        assert!(build_source_mutation_history(&history(vec![activity]))
            .expect("report")
            .events
            .is_empty());
    }
}
