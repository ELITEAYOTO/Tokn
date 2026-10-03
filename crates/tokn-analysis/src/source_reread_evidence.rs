use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use serde::{Deserialize, Serialize};
use tokn_domain::{
    EvidenceIdentityCoverage, HistoricalToolActivityRecord, TOOL_ACTIVITY_HISTORY_SCHEMA_VERSION,
    ToolActivityHistory, parse_rfc3339_unix_ms,
};

pub const SOURCE_REREAD_EVIDENCE_SCHEMA_VERSION: u64 = 1;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SourceReadTimingCoverage {
    CompleteObserved,
    Partial,
    NotCaptured,
    InvalidObservedTimestamp,
    #[default]
    Unknown,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum CrossRunReadChronologyStatus {
    StrictRunOrderObserved,
    OverlapOrEqualObserved,
    #[default]
    Unknown,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum CrossRunReadContentRelation {
    ExactEqualityObserved,
    ExactDifferenceObserved,
    #[default]
    Unknown,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SourceRereadInferenceStatus {
    #[default]
    NotProven,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CrossRunSourceRereadObservation {
    pub earlier_run_id: String,
    pub later_run_id: String,
    pub earlier_workspace_id: String,
    pub later_workspace_id: String,
    pub earlier_last_read_at_unix_ms: u64,
    pub later_first_read_at_unix_ms: u64,
    pub content_relation: CrossRunReadContentRelation,
    pub rediscovery_status: SourceRereadInferenceStatus,
    pub redundancy_status: SourceRereadInferenceStatus,
    pub freshness_status: SourceRereadInferenceStatus,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CrossRunSourceRereadSummary {
    pub project_id: String,
    pub source_stable_id: String,
    pub completed_read_count: u64,
    pub run_count: u64,
    pub timing_coverage: SourceReadTimingCoverage,
    pub chronology_status: CrossRunReadChronologyStatus,
    pub rereads: Vec<CrossRunSourceRereadObservation>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CrossRunSourceRereadReport {
    pub schema_version: u64,
    pub source_activity_history_schema_version: u64,
    pub project_filter: Option<String>,
    pub workspace_filter: Option<String>,
    pub run_limit: u64,
    pub sources: Vec<CrossRunSourceRereadSummary>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SourceRereadEvidenceBuildError {
    UnsupportedToolActivityHistorySchema { actual: u64, expected: u64 },
}

impl fmt::Display for SourceRereadEvidenceBuildError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedToolActivityHistorySchema { actual, expected } => write!(
                formatter,
                "unsupported ToolActivityHistory schema_version {actual}; expected {expected}"
            ),
        }
    }
}

impl std::error::Error for SourceRereadEvidenceBuildError {}

pub fn build_cross_run_source_reread_evidence(
    history: &ToolActivityHistory,
) -> Result<CrossRunSourceRereadReport, SourceRereadEvidenceBuildError> {
    if history.schema_version != TOOL_ACTIVITY_HISTORY_SCHEMA_VERSION {
        return Err(
            SourceRereadEvidenceBuildError::UnsupportedToolActivityHistorySchema {
                actual: history.schema_version,
                expected: TOOL_ACTIVITY_HISTORY_SCHEMA_VERSION,
            },
        );
    }

    let mut grouped = BTreeMap::<(String, String), BTreeMap<String, RunReadGroup>>::new();
    for activity in &history.activities {
        if activity.category != "file_read"
            || activity.status != "completed"
            || activity.source_identity_coverage != EvidenceIdentityCoverage::Observed
        {
            continue;
        }
        let Some(source_stable_id) = activity.source_stable_id.as_ref() else {
            continue;
        };

        let run = grouped
            .entry((activity.project_id.clone(), source_stable_id.clone()))
            .or_default()
            .entry(activity.run_id.clone())
            .or_default();
        run.workspace_ids.insert(activity.workspace_id.clone());
        run.reads.push(ReadPoint::from_activity(activity));
    }

    let mut sources = Vec::new();
    for ((project_id, source_stable_id), runs) in grouped {
        if runs.len() < 2 {
            continue;
        }
        sources.push(build_source_summary(project_id, source_stable_id, runs));
    }

    sources.sort_by(|left, right| {
        right
            .rereads
            .len()
            .cmp(&left.rereads.len())
            .then(right.run_count.cmp(&left.run_count))
            .then(left.project_id.cmp(&right.project_id))
            .then(left.source_stable_id.cmp(&right.source_stable_id))
    });

    Ok(CrossRunSourceRereadReport {
        schema_version: SOURCE_REREAD_EVIDENCE_SCHEMA_VERSION,
        source_activity_history_schema_version: history.schema_version,
        project_filter: history.project_filter.clone(),
        workspace_filter: history.workspace_filter.clone(),
        run_limit: history.run_limit,
        sources,
    })
}

#[derive(Debug, Clone, Default)]
struct RunReadGroup {
    workspace_ids: BTreeSet<String>,
    reads: Vec<ReadPoint>,
}

#[derive(Debug, Clone)]
struct ReadPoint {
    observed_at_unix_ms: Option<u64>,
    timestamp_present: bool,
    timestamp_valid: bool,
    content_identity_coverage: EvidenceIdentityCoverage,
    content_fingerprint: Option<String>,
}

impl ReadPoint {
    fn from_activity(activity: &HistoricalToolActivityRecord) -> Self {
        let raw_timestamp = activity.observed_at.as_deref();
        let parsed_timestamp = raw_timestamp.and_then(parse_rfc3339_unix_ms);
        Self {
            observed_at_unix_ms: parsed_timestamp,
            timestamp_present: raw_timestamp.is_some(),
            timestamp_valid: raw_timestamp.is_none() || parsed_timestamp.is_some(),
            content_identity_coverage: activity.content_identity_coverage,
            content_fingerprint: activity.content_fingerprint.clone(),
        }
    }
}

#[derive(Debug)]
struct SequencedRun {
    run_id: String,
    workspace_id: String,
    first_read_at_unix_ms: u64,
    last_read_at_unix_ms: u64,
    reads: Vec<ReadPoint>,
}

fn build_source_summary(
    project_id: String,
    source_stable_id: String,
    runs: BTreeMap<String, RunReadGroup>,
) -> CrossRunSourceRereadSummary {
    let completed_read_count = runs.values().map(|run| run.reads.len() as u64).sum::<u64>();
    let timing_coverage = timing_coverage(&runs);

    let Some(mut sequenced_runs) = fully_sequenced_runs(&runs, timing_coverage) else {
        return CrossRunSourceRereadSummary {
            project_id,
            source_stable_id,
            completed_read_count,
            run_count: runs.len() as u64,
            timing_coverage,
            chronology_status: CrossRunReadChronologyStatus::Unknown,
            rereads: Vec::new(),
        };
    };

    sequenced_runs.sort_by(|left, right| {
        left.first_read_at_unix_ms
            .cmp(&right.first_read_at_unix_ms)
            .then(left.last_read_at_unix_ms.cmp(&right.last_read_at_unix_ms))
            .then(left.run_id.cmp(&right.run_id))
    });

    if sequenced_runs
        .windows(2)
        .any(|pair| pair[0].last_read_at_unix_ms >= pair[1].first_read_at_unix_ms)
    {
        return CrossRunSourceRereadSummary {
            project_id,
            source_stable_id,
            completed_read_count,
            run_count: runs.len() as u64,
            timing_coverage,
            chronology_status: CrossRunReadChronologyStatus::OverlapOrEqualObserved,
            rereads: Vec::new(),
        };
    }

    let rereads = sequenced_runs
        .windows(2)
        .map(|pair| build_reread(&pair[0], &pair[1]))
        .collect();

    CrossRunSourceRereadSummary {
        project_id,
        source_stable_id,
        completed_read_count,
        run_count: runs.len() as u64,
        timing_coverage,
        chronology_status: CrossRunReadChronologyStatus::StrictRunOrderObserved,
        rereads,
    }
}

fn timing_coverage(runs: &BTreeMap<String, RunReadGroup>) -> SourceReadTimingCoverage {
    let reads = runs
        .values()
        .flat_map(|run| run.reads.iter())
        .collect::<Vec<_>>();
    if reads.is_empty() {
        return SourceReadTimingCoverage::Unknown;
    }
    if reads.iter().any(|read| !read.timestamp_valid) {
        return SourceReadTimingCoverage::InvalidObservedTimestamp;
    }
    let observed = reads
        .iter()
        .filter(|read| read.observed_at_unix_ms.is_some())
        .count();
    if observed == reads.len() {
        SourceReadTimingCoverage::CompleteObserved
    } else if observed == 0 && reads.iter().all(|read| !read.timestamp_present) {
        SourceReadTimingCoverage::NotCaptured
    } else {
        SourceReadTimingCoverage::Partial
    }
}

fn fully_sequenced_runs(
    runs: &BTreeMap<String, RunReadGroup>,
    timing_coverage: SourceReadTimingCoverage,
) -> Option<Vec<SequencedRun>> {
    if timing_coverage != SourceReadTimingCoverage::CompleteObserved {
        return None;
    }

    runs.iter()
        .map(|(run_id, run)| {
            if run.workspace_ids.len() != 1 || run.reads.is_empty() {
                return None;
            }
            let first_read_at_unix_ms = run
                .reads
                .iter()
                .filter_map(|read| read.observed_at_unix_ms)
                .min()?;
            let last_read_at_unix_ms = run
                .reads
                .iter()
                .filter_map(|read| read.observed_at_unix_ms)
                .max()?;
            Some(SequencedRun {
                run_id: run_id.clone(),
                workspace_id: run.workspace_ids.iter().next()?.clone(),
                first_read_at_unix_ms,
                last_read_at_unix_ms,
                reads: run.reads.clone(),
            })
        })
        .collect()
}

fn build_reread(earlier: &SequencedRun, later: &SequencedRun) -> CrossRunSourceRereadObservation {
    let earlier_content =
        boundary_content_fingerprint(&earlier.reads, earlier.last_read_at_unix_ms);
    let later_content = boundary_content_fingerprint(&later.reads, later.first_read_at_unix_ms);
    let content_relation = match (earlier_content, later_content) {
        (Some(left), Some(right)) if left == right => {
            CrossRunReadContentRelation::ExactEqualityObserved
        }
        (Some(_), Some(_)) => CrossRunReadContentRelation::ExactDifferenceObserved,
        _ => CrossRunReadContentRelation::Unknown,
    };

    CrossRunSourceRereadObservation {
        earlier_run_id: earlier.run_id.clone(),
        later_run_id: later.run_id.clone(),
        earlier_workspace_id: earlier.workspace_id.clone(),
        later_workspace_id: later.workspace_id.clone(),
        earlier_last_read_at_unix_ms: earlier.last_read_at_unix_ms,
        later_first_read_at_unix_ms: later.first_read_at_unix_ms,
        content_relation,
        rediscovery_status: SourceRereadInferenceStatus::NotProven,
        redundancy_status: SourceRereadInferenceStatus::NotProven,
        freshness_status: SourceRereadInferenceStatus::NotProven,
    }
}

fn boundary_content_fingerprint(reads: &[ReadPoint], timestamp: u64) -> Option<&str> {
    let matching = reads
        .iter()
        .filter(|read| read.observed_at_unix_ms == Some(timestamp))
        .collect::<Vec<_>>();
    if matching.is_empty()
        || matching.iter().any(|read| {
            read.content_identity_coverage != EvidenceIdentityCoverage::Observed
                || read.content_fingerprint.is_none()
        })
    {
        return None;
    }

    let fingerprints = matching
        .iter()
        .filter_map(|read| read.content_fingerprint.as_deref())
        .collect::<BTreeSet<_>>();
    (fingerprints.len() == 1)
        .then(|| fingerprints.into_iter().next())
        .flatten()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn read(
        project: &str,
        workspace: &str,
        run: &str,
        id: &str,
        observed_at: Option<&str>,
        content: Option<&str>,
    ) -> HistoricalToolActivityRecord {
        HistoricalToolActivityRecord {
            activity_id: id.into(),
            run_id: run.into(),
            project_id: project.into(),
            workspace_id: workspace.into(),
            thread_id: format!("thread-{run}"),
            category: "file_read".into(),
            status: "completed".into(),
            observed_at: observed_at.map(str::to_string),
            source_stable_id: Some("src-v1-shared".into()),
            source_identity_coverage: EvidenceIdentityCoverage::Observed,
            content_fingerprint: content.map(str::to_string),
            content_identity_coverage: if content.is_some() {
                EvidenceIdentityCoverage::Observed
            } else {
                EvidenceIdentityCoverage::NotCaptured
            },
            ..Default::default()
        }
    }

    fn history(activities: Vec<HistoricalToolActivityRecord>) -> ToolActivityHistory {
        ToolActivityHistory {
            schema_version: TOOL_ACTIVITY_HISTORY_SCHEMA_VERSION,
            project_filter: None,
            workspace_filter: None,
            run_limit: 50,
            activities,
        }
    }

    #[test]
    fn reports_strict_cross_run_reread_without_upgrading_interpretation() {
        let report = build_cross_run_source_reread_evidence(&history(vec![
            read(
                "prj-a",
                "wsp-a",
                "run-a",
                "a",
                Some("2026-10-03T12:00:00+02:00"),
                Some("cnt-v1-same"),
            ),
            read(
                "prj-a",
                "wsp-b",
                "run-b",
                "b",
                Some("2026-10-03T10:00:01Z"),
                Some("cnt-v1-same"),
            ),
        ]))
        .expect("report");

        assert_eq!(report.sources.len(), 1);
        let source = &report.sources[0];
        assert_eq!(
            source.timing_coverage,
            SourceReadTimingCoverage::CompleteObserved
        );
        assert_eq!(
            source.chronology_status,
            CrossRunReadChronologyStatus::StrictRunOrderObserved
        );
        assert_eq!(source.rereads.len(), 1);
        assert_eq!(source.rereads[0].earlier_run_id, "run-a");
        assert_eq!(source.rereads[0].later_run_id, "run-b");
        assert_eq!(
            source.rereads[0].content_relation,
            CrossRunReadContentRelation::ExactEqualityObserved
        );
        assert_eq!(
            source.rereads[0].rediscovery_status,
            SourceRereadInferenceStatus::NotProven
        );
        assert_eq!(
            source.rereads[0].redundancy_status,
            SourceRereadInferenceStatus::NotProven
        );
        assert_eq!(
            source.rereads[0].freshness_status,
            SourceRereadInferenceStatus::NotProven
        );
    }

    #[test]
    fn compares_boundary_reads_not_arbitrary_run_content() {
        let report = build_cross_run_source_reread_evidence(&history(vec![
            read(
                "prj-a",
                "wsp-a",
                "run-a",
                "a1",
                Some("2026-10-03T10:00:00Z"),
                Some("cnt-v1-old"),
            ),
            read(
                "prj-a",
                "wsp-a",
                "run-a",
                "a2",
                Some("2026-10-03T10:00:01Z"),
                Some("cnt-v1-boundary"),
            ),
            read(
                "prj-a",
                "wsp-a",
                "run-b",
                "b1",
                Some("2026-10-03T10:00:02Z"),
                Some("cnt-v1-new"),
            ),
        ]))
        .expect("report");

        assert_eq!(
            report.sources[0].rereads[0].content_relation,
            CrossRunReadContentRelation::ExactDifferenceObserved
        );
    }

    #[test]
    fn missing_timestamp_keeps_chronology_unknown() {
        let report = build_cross_run_source_reread_evidence(&history(vec![
            read("prj-a", "wsp-a", "run-a", "a", None, Some("cnt-v1-a")),
            read(
                "prj-a",
                "wsp-a",
                "run-b",
                "b",
                Some("2026-10-03T10:00:02Z"),
                Some("cnt-v1-a"),
            ),
        ]))
        .expect("report");

        assert_eq!(
            report.sources[0].timing_coverage,
            SourceReadTimingCoverage::Partial
        );
        assert_eq!(
            report.sources[0].chronology_status,
            CrossRunReadChronologyStatus::Unknown
        );
        assert!(report.sources[0].rereads.is_empty());
    }

    #[test]
    fn malformed_timestamp_is_explicitly_invalid() {
        let report = build_cross_run_source_reread_evidence(&history(vec![
            read(
                "prj-a",
                "wsp-a",
                "run-a",
                "a",
                Some("not-a-timestamp"),
                Some("cnt-v1-a"),
            ),
            read(
                "prj-a",
                "wsp-a",
                "run-b",
                "b",
                Some("2026-10-03T10:00:02Z"),
                Some("cnt-v1-a"),
            ),
        ]))
        .expect("report");

        assert_eq!(
            report.sources[0].timing_coverage,
            SourceReadTimingCoverage::InvalidObservedTimestamp
        );
        assert_eq!(
            report.sources[0].chronology_status,
            CrossRunReadChronologyStatus::Unknown
        );
    }

    #[test]
    fn overlapping_run_read_windows_do_not_create_reread_order() {
        let report = build_cross_run_source_reread_evidence(&history(vec![
            read(
                "prj-a",
                "wsp-a",
                "run-a",
                "a1",
                Some("2026-10-03T10:00:00Z"),
                Some("cnt-v1-a"),
            ),
            read(
                "prj-a",
                "wsp-a",
                "run-a",
                "a2",
                Some("2026-10-03T10:00:03Z"),
                Some("cnt-v1-a"),
            ),
            read(
                "prj-a",
                "wsp-b",
                "run-b",
                "b",
                Some("2026-10-03T10:00:02Z"),
                Some("cnt-v1-a"),
            ),
        ]))
        .expect("report");

        assert_eq!(
            report.sources[0].chronology_status,
            CrossRunReadChronologyStatus::OverlapOrEqualObserved
        );
        assert!(report.sources[0].rereads.is_empty());
    }

    #[test]
    fn incomplete_boundary_content_identity_stays_unknown() {
        let report = build_cross_run_source_reread_evidence(&history(vec![
            read(
                "prj-a",
                "wsp-a",
                "run-a",
                "a",
                Some("2026-10-03T10:00:00Z"),
                None,
            ),
            read(
                "prj-a",
                "wsp-b",
                "run-b",
                "b",
                Some("2026-10-03T10:00:01Z"),
                Some("cnt-v1-a"),
            ),
        ]))
        .expect("report");

        assert_eq!(
            report.sources[0].rereads[0].content_relation,
            CrossRunReadContentRelation::Unknown
        );
    }

    #[test]
    fn non_completed_reads_and_cross_project_identity_collisions_are_not_joined() {
        let mut failed = read(
            "prj-a",
            "wsp-a",
            "run-a",
            "failed",
            Some("2026-10-03T09:00:00Z"),
            Some("cnt-v1-a"),
        );
        failed.status = "failed".into();
        let report = build_cross_run_source_reread_evidence(&history(vec![
            failed,
            read(
                "prj-a",
                "wsp-a",
                "run-b",
                "a",
                Some("2026-10-03T10:00:00Z"),
                Some("cnt-v1-a"),
            ),
            read(
                "prj-b",
                "wsp-b",
                "run-c",
                "b",
                Some("2026-10-03T10:00:01Z"),
                Some("cnt-v1-a"),
            ),
        ]))
        .expect("report");

        assert!(report.sources.is_empty());
    }

    #[test]
    fn all_missing_timestamps_are_not_captured() {
        let report = build_cross_run_source_reread_evidence(&history(vec![
            read("prj-a", "wsp-a", "run-a", "a", None, Some("cnt-v1-a")),
            read("prj-a", "wsp-b", "run-b", "b", None, Some("cnt-v1-a")),
        ]))
        .expect("report");

        assert_eq!(
            report.sources[0].timing_coverage,
            SourceReadTimingCoverage::NotCaptured
        );
        assert_eq!(
            report.sources[0].chronology_status,
            CrossRunReadChronologyStatus::Unknown
        );
        assert!(report.sources[0].rereads.is_empty());
    }

    #[test]
    fn three_strict_runs_emit_two_consecutive_rereads() {
        let report = build_cross_run_source_reread_evidence(&history(vec![
            read(
                "prj-a",
                "wsp-a",
                "run-a",
                "a",
                Some("2026-10-03T10:00:00Z"),
                Some("cnt-v1-a"),
            ),
            read(
                "prj-a",
                "wsp-b",
                "run-b",
                "b",
                Some("2026-10-03T10:00:01Z"),
                Some("cnt-v1-a"),
            ),
            read(
                "prj-a",
                "wsp-c",
                "run-c",
                "c",
                Some("2026-10-03T10:00:02Z"),
                Some("cnt-v1-b"),
            ),
        ]))
        .expect("report");

        assert_eq!(report.sources[0].rereads.len(), 2);
        assert_eq!(report.sources[0].rereads[0].earlier_run_id, "run-a");
        assert_eq!(report.sources[0].rereads[0].later_run_id, "run-b");
        assert_eq!(report.sources[0].rereads[1].earlier_run_id, "run-b");
        assert_eq!(report.sources[0].rereads[1].later_run_id, "run-c");
    }

    #[test]
    fn ambiguous_same_timestamp_boundary_content_stays_unknown() {
        let report = build_cross_run_source_reread_evidence(&history(vec![
            read(
                "prj-a",
                "wsp-a",
                "run-a",
                "a1",
                Some("2026-10-03T10:00:00Z"),
                Some("cnt-v1-a"),
            ),
            read(
                "prj-a",
                "wsp-a",
                "run-a",
                "a2",
                Some("2026-10-03T10:00:00Z"),
                Some("cnt-v1-b"),
            ),
            read(
                "prj-a",
                "wsp-b",
                "run-b",
                "b",
                Some("2026-10-03T10:00:01Z"),
                Some("cnt-v1-a"),
            ),
        ]))
        .expect("report");

        assert_eq!(
            report.sources[0].rereads[0].content_relation,
            CrossRunReadContentRelation::Unknown
        );
    }

    #[test]
    fn rejects_unknown_tool_activity_schema() {
        let mut value = history(Vec::new());
        value.schema_version = 99;
        assert!(matches!(
            build_cross_run_source_reread_evidence(&value),
            Err(
                SourceRereadEvidenceBuildError::UnsupportedToolActivityHistorySchema {
                    actual: 99,
                    ..
                }
            )
        ));
    }
}
