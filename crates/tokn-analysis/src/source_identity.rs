use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use serde::{Deserialize, Serialize};
use tokn_domain::{
    EvidenceIdentityCoverage, TOOL_ACTIVITY_HISTORY_SCHEMA_VERSION, ToolActivityHistory,
};

pub const SOURCE_IDENTITY_HISTORY_SCHEMA_VERSION: u64 = 1;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ObservedContentEvolution {
    UnchangedObserved,
    ChangedObserved,
    #[default]
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceIdentitySummary {
    pub source_stable_id: String,
    pub occurrence_count: u64,
    pub run_count: u64,
    pub thread_count: u64,
    pub content_identity_coverage: EvidenceIdentityCoverage,
    pub observed_content_occurrence_count: u64,
    pub distinct_content_fingerprint_count: u64,
    pub content_evolution: ObservedContentEvolution,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceIdentityHistoryReport {
    pub schema_version: u64,
    pub source_activity_history_schema_version: u64,
    pub project_filter: Option<String>,
    pub workspace_filter: Option<String>,
    pub run_limit: u64,
    pub sources: Vec<SourceIdentitySummary>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SourceIdentityHistoryBuildError {
    UnsupportedToolActivityHistorySchema { actual: u64, expected: u64 },
}

impl fmt::Display for SourceIdentityHistoryBuildError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedToolActivityHistorySchema { actual, expected } => write!(
                formatter,
                "unsupported ToolActivityHistory schema_version {actual}; expected {expected}"
            ),
        }
    }
}

impl std::error::Error for SourceIdentityHistoryBuildError {}

pub fn build_source_identity_history(
    history: &ToolActivityHistory,
) -> Result<SourceIdentityHistoryReport, SourceIdentityHistoryBuildError> {
    if history.schema_version != TOOL_ACTIVITY_HISTORY_SCHEMA_VERSION {
        return Err(
            SourceIdentityHistoryBuildError::UnsupportedToolActivityHistorySchema {
                actual: history.schema_version,
                expected: TOOL_ACTIVITY_HISTORY_SCHEMA_VERSION,
            },
        );
    }

    let mut groups = BTreeMap::<String, SourceGroup>::new();
    for activity in &history.activities {
        if activity.source_identity_coverage != EvidenceIdentityCoverage::Observed {
            continue;
        }
        let Some(source_stable_id) = activity.source_stable_id.as_ref() else {
            continue;
        };

        let entry = groups.entry(source_stable_id.clone()).or_default();
        entry.occurrences = entry.occurrences.saturating_add(1);
        entry.runs.insert(activity.run_id.clone());
        entry.threads.insert(activity.thread_id.clone());
        entry
            .content_coverages
            .push(activity.content_identity_coverage);
        if activity.content_identity_coverage == EvidenceIdentityCoverage::Observed
            && let Some(fingerprint) = activity.content_fingerprint.as_ref()
        {
            entry.observed_content_occurrences =
                entry.observed_content_occurrences.saturating_add(1);
            entry.content_fingerprints.insert(fingerprint.clone());
        }
    }

    let mut sources = groups
        .into_iter()
        .map(|(source_stable_id, group)| {
            let coverage = aggregate_content_coverage(
                group.occurrences,
                group.observed_content_occurrences,
                &group.content_coverages,
            );
            let content_evolution = if coverage == EvidenceIdentityCoverage::Observed {
                match group.content_fingerprints.len() {
                    1 => ObservedContentEvolution::UnchangedObserved,
                    count if count > 1 => ObservedContentEvolution::ChangedObserved,
                    _ => ObservedContentEvolution::Unknown,
                }
            } else {
                ObservedContentEvolution::Unknown
            };

            SourceIdentitySummary {
                source_stable_id,
                occurrence_count: group.occurrences,
                run_count: group.runs.len() as u64,
                thread_count: group.threads.len() as u64,
                content_identity_coverage: coverage,
                observed_content_occurrence_count: group.observed_content_occurrences,
                distinct_content_fingerprint_count: group.content_fingerprints.len() as u64,
                content_evolution,
            }
        })
        .collect::<Vec<_>>();

    sources.sort_by(|left, right| {
        right
            .occurrence_count
            .cmp(&left.occurrence_count)
            .then(left.source_stable_id.cmp(&right.source_stable_id))
    });

    Ok(SourceIdentityHistoryReport {
        schema_version: SOURCE_IDENTITY_HISTORY_SCHEMA_VERSION,
        source_activity_history_schema_version: history.schema_version,
        project_filter: history.project_filter.clone(),
        workspace_filter: history.workspace_filter.clone(),
        run_limit: history.run_limit,
        sources,
    })
}

#[derive(Default)]
struct SourceGroup {
    occurrences: u64,
    observed_content_occurrences: u64,
    runs: BTreeSet<String>,
    threads: BTreeSet<String>,
    content_fingerprints: BTreeSet<String>,
    content_coverages: Vec<EvidenceIdentityCoverage>,
}

fn aggregate_content_coverage(
    occurrences: u64,
    observed: u64,
    coverages: &[EvidenceIdentityCoverage],
) -> EvidenceIdentityCoverage {
    if occurrences > 0 && observed == occurrences {
        EvidenceIdentityCoverage::Observed
    } else if observed > 0 || coverages.contains(&EvidenceIdentityCoverage::Partial) {`r`n        EvidenceIdentityCoverage::Partial
    } else if coverages.contains(&EvidenceIdentityCoverage::Unknown) {
        EvidenceIdentityCoverage::Unknown
    } else {
        EvidenceIdentityCoverage::NotCaptured
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokn_domain::HistoricalToolActivityRecord;

    fn activity(
        run_id: &str,
        thread_id: &str,
        source_id: Option<&str>,
        source_coverage: EvidenceIdentityCoverage,
        content: Option<&str>,
        content_coverage: EvidenceIdentityCoverage,
    ) -> HistoricalToolActivityRecord {
        HistoricalToolActivityRecord {
            activity_id: format!("act-{run_id}-{thread_id}-{}", content.unwrap_or("none")),
            run_id: run_id.into(),
            project_id: "prj-fixture".into(),
            workspace_id: "wsp-fixture".into(),
            thread_id: thread_id.into(),
            source_stable_id: source_id.map(str::to_string),
            source_identity_coverage: source_coverage,
            content_fingerprint: content.map(str::to_string),
            content_identity_coverage: content_coverage,
            ..Default::default()
        }
    }

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
    fn reports_unchanged_content_only_with_complete_observed_identity() {
        let report = build_source_identity_history(&history(vec![
            activity(
                "run-1",
                "thread-a",
                Some("src-v1-same"),
                EvidenceIdentityCoverage::Observed,
                Some("cnt-v1-a"),
                EvidenceIdentityCoverage::Observed,
            ),
            activity(
                "run-2",
                "thread-b",
                Some("src-v1-same"),
                EvidenceIdentityCoverage::Observed,
                Some("cnt-v1-a"),
                EvidenceIdentityCoverage::Observed,
            ),
        ]))
        .expect("report");

        assert_eq!(report.sources.len(), 1);
        assert_eq!(
            report.sources[0].content_evolution,
            ObservedContentEvolution::UnchangedObserved
        );
        assert_eq!(report.sources[0].run_count, 2);
    }

    #[test]
    fn reports_changed_content_when_complete_identity_differs() {
        let report = build_source_identity_history(&history(vec![
            activity(
                "run-1",
                "thread-a",
                Some("src-v1-same"),
                EvidenceIdentityCoverage::Observed,
                Some("cnt-v1-a"),
                EvidenceIdentityCoverage::Observed,
            ),
            activity(
                "run-2",
                "thread-a",
                Some("src-v1-same"),
                EvidenceIdentityCoverage::Observed,
                Some("cnt-v1-b"),
                EvidenceIdentityCoverage::Observed,
            ),
        ]))
        .expect("report");

        assert_eq!(
            report.sources[0].content_evolution,
            ObservedContentEvolution::ChangedObserved
        );
        assert_eq!(report.sources[0].distinct_content_fingerprint_count, 2);
    }

    #[test]
    fn incomplete_content_identity_stays_unknown() {
        let report = build_source_identity_history(&history(vec![
            activity(
                "run-1",
                "thread-a",
                Some("src-v1-same"),
                EvidenceIdentityCoverage::Observed,
                Some("cnt-v1-a"),
                EvidenceIdentityCoverage::Observed,
            ),
            activity(
                "run-2",
                "thread-a",
                Some("src-v1-same"),
                EvidenceIdentityCoverage::Observed,
                None,
                EvidenceIdentityCoverage::NotCaptured,
            ),
        ]))
        .expect("report");

        assert_eq!(
            report.sources[0].content_identity_coverage,
            EvidenceIdentityCoverage::Partial
        );
        assert_eq!(
            report.sources[0].content_evolution,
            ObservedContentEvolution::Unknown
        );
    }

    #[test]
    fn ignores_records_without_observed_stable_source_identity() {
        let report = build_source_identity_history(&history(vec![activity(
            "run-1",
            "thread-a",
            None,
            EvidenceIdentityCoverage::NotCaptured,
            Some("cnt-v1-a"),
            EvidenceIdentityCoverage::Observed,
        )]))
        .expect("report");
        assert!(report.sources.is_empty());
    }

    #[test]
    fn rejects_unknown_tool_activity_schema() {
        let mut value = history(Vec::new());
        value.schema_version = 99;
        assert!(matches!(
            build_source_identity_history(&value),
            Err(
                SourceIdentityHistoryBuildError::UnsupportedToolActivityHistorySchema {
                    actual: 99,
                    ..
                }
            )
        ));
    }
}
