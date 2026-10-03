use std::fmt;

use serde::{Deserialize, Serialize};
use tokn_domain::{
    EvidenceIdentityCoverage, HistoricalToolActivityRecord, TOOL_ACTIVITY_HISTORY_SCHEMA_VERSION,
    ToolActivityHistory,
};

pub const SOURCE_MUTATION_WINDOW_HISTORY_SCHEMA_VERSION: u64 = 1;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum MutationWindowObservation {
    ExactContentEqualityObserved,
    ExactContentDifferenceObserved,
    #[default]
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum MutationWindowEvidenceGap {
    MutationNotCompleted,
    MutationSequenceNotObserved,
    MissingReadBefore,
    MissingReadAfter,
    ReadNotCompleted,
    ReadSequenceNotObserved,
    IncompleteReadContentIdentity,
    InterveningSameThreadMutation,
    CrossThreadMutationOrderUnknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum MutationCausalityStatus {
    NotProven,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceMutationWindowObservation {
    pub mutation_activity_id: String,
    pub run_id: String,
    pub thread_id: String,
    pub source_stable_id: String,
    pub mutation_started_seq: Option<u64>,
    pub mutation_ended_seq: Option<u64>,
    pub before_read_activity_id: Option<String>,
    pub after_read_activity_id: Option<String>,
    pub before_content_fingerprint: Option<String>,
    pub after_content_fingerprint: Option<String>,
    pub observation: MutationWindowObservation,
    pub evidence_gaps: Vec<MutationWindowEvidenceGap>,
    pub causality_status: MutationCausalityStatus,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceMutationWindowHistoryReport {
    pub schema_version: u64,
    pub source_activity_history_schema_version: u64,
    pub project_filter: Option<String>,
    pub workspace_filter: Option<String>,
    pub run_limit: u64,
    pub windows: Vec<SourceMutationWindowObservation>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SourceMutationWindowBuildError {
    UnsupportedToolActivityHistorySchema { actual: u64, expected: u64 },
}

impl fmt::Display for SourceMutationWindowBuildError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedToolActivityHistorySchema { actual, expected } => write!(
                formatter,
                "unsupported ToolActivityHistory schema_version {actual}; expected {expected}"
            ),
        }
    }
}

impl std::error::Error for SourceMutationWindowBuildError {}

pub fn build_source_mutation_window_history(
    history: &ToolActivityHistory,
) -> Result<SourceMutationWindowHistoryReport, SourceMutationWindowBuildError> {
    if history.schema_version != TOOL_ACTIVITY_HISTORY_SCHEMA_VERSION {
        return Err(
            SourceMutationWindowBuildError::UnsupportedToolActivityHistorySchema {
                actual: history.schema_version,
                expected: TOOL_ACTIVITY_HISTORY_SCHEMA_VERSION,
            },
        );
    }

    let mut windows = history
        .activities
        .iter()
        .filter(|activity| activity.category == "write_mutation")
        .filter(|activity| activity.source_identity_coverage == EvidenceIdentityCoverage::Observed)
        .filter_map(|mutation| {
            mutation
                .source_stable_id
                .as_ref()
                .map(|source_stable_id| build_window(history, mutation, source_stable_id))
        })
        .collect::<Vec<_>>();

    windows.sort_by(|left, right| {
        let left_activity = history
            .activities
            .iter()
            .find(|activity| activity.activity_id == left.mutation_activity_id);
        let right_activity = history
            .activities
            .iter()
            .find(|activity| activity.activity_id == right.mutation_activity_id);
        let left_key = left_activity
            .map(|activity| (activity.run_created_at_unix, activity.agent_ordinal))
            .unwrap_or_default();
        let right_key = right_activity
            .map(|activity| (activity.run_created_at_unix, activity.agent_ordinal))
            .unwrap_or_default();
        left_key
            .cmp(&right_key)
            .then(left.thread_id.cmp(&right.thread_id))
            .then(left.mutation_activity_id.cmp(&right.mutation_activity_id))
    });

    Ok(SourceMutationWindowHistoryReport {
        schema_version: SOURCE_MUTATION_WINDOW_HISTORY_SCHEMA_VERSION,
        source_activity_history_schema_version: history.schema_version,
        project_filter: history.project_filter.clone(),
        workspace_filter: history.workspace_filter.clone(),
        run_limit: history.run_limit,
        windows,
    })
}

fn build_window(
    history: &ToolActivityHistory,
    mutation: &HistoricalToolActivityRecord,
    source_stable_id: &str,
) -> SourceMutationWindowObservation {
    let mut gaps = Vec::new();

    if !mutation.status.eq_ignore_ascii_case("completed") {
        gaps.push(MutationWindowEvidenceGap::MutationNotCompleted);
    }

    let mutation_interval = sequence_interval(mutation);
    if mutation_interval.is_none() {
        gaps.push(MutationWindowEvidenceGap::MutationSequenceNotObserved);
    }

    let ordered_reads = history
        .activities
        .iter()
        .filter(|activity| activity.run_id == mutation.run_id)
        .filter(|activity| activity.thread_id == mutation.thread_id)
        .filter(|activity| activity.category == "file_read")
        .filter(|activity| {
            activity.source_identity_coverage == EvidenceIdentityCoverage::Observed
                && activity.source_stable_id.as_deref() == Some(source_stable_id)
        })
        .collect::<Vec<_>>();

    let (before_read, after_read) = match mutation_interval {
        Some((mutation_start, mutation_end)) => {
            let before = ordered_reads
                .iter()
                .copied()
                .filter_map(|read| {
                    sequence_interval(read)
                        .filter(|(_, read_end)| *read_end < mutation_start)
                        .map(|(_, read_end)| (read_end, read))
                })
                .max_by_key(|(read_end, _)| *read_end)
                .map(|(_, read)| read);
            let after = ordered_reads
                .iter()
                .copied()
                .filter_map(|read| {
                    sequence_interval(read)
                        .filter(|(read_start, _)| *read_start > mutation_end)
                        .map(|(read_start, _)| (read_start, read))
                })
                .min_by_key(|(read_start, _)| *read_start)
                .map(|(_, read)| read);
            (before, after)
        }
        None => (None, None),
    };

    if before_read.is_none() {
        gaps.push(MutationWindowEvidenceGap::MissingReadBefore);
    }
    if after_read.is_none() {
        gaps.push(MutationWindowEvidenceGap::MissingReadAfter);
    }
    if before_read
        .into_iter()
        .chain(after_read)
        .any(|read| !read.status.eq_ignore_ascii_case("completed"))
    {
        gaps.push(MutationWindowEvidenceGap::ReadNotCompleted);
    }

    if ordered_reads
        .iter()
        .any(|read| sequence_interval(read).is_none())
    {
        gaps.push(MutationWindowEvidenceGap::ReadSequenceNotObserved);
    }

    let before_content = before_read.and_then(exact_content_fingerprint);
    let after_content = after_read.and_then(exact_content_fingerprint);
    if (before_read.is_some() && before_content.is_none())
        || (after_read.is_some() && after_content.is_none())
    {
        gaps.push(MutationWindowEvidenceGap::IncompleteReadContentIdentity);
    }

    if let (Some(before), Some(after), Some((mutation_start, mutation_end))) =
        (before_read, after_read, mutation_interval)
        && has_intervening_same_thread_mutation(
            history,
            mutation,
            source_stable_id,
            before,
            after,
            mutation_start,
            mutation_end,
        )
    {
        gaps.push(MutationWindowEvidenceGap::InterveningSameThreadMutation);
    }

    if history.activities.iter().any(|activity| {
        activity.run_id == mutation.run_id
            && activity.thread_id != mutation.thread_id
            && activity.category == "write_mutation"
            && activity.source_identity_coverage == EvidenceIdentityCoverage::Observed
            && activity.source_stable_id.as_deref() == Some(source_stable_id)
    }) {
        gaps.push(MutationWindowEvidenceGap::CrossThreadMutationOrderUnknown);
    }

    gaps.sort();
    gaps.dedup();

    let observation = if gaps.is_empty() {
        match (before_content, after_content) {
            (Some(before), Some(after)) if before == after => {
                MutationWindowObservation::ExactContentEqualityObserved
            }
            (Some(_), Some(_)) => MutationWindowObservation::ExactContentDifferenceObserved,
            _ => MutationWindowObservation::Unknown,
        }
    } else {
        MutationWindowObservation::Unknown
    };

    SourceMutationWindowObservation {
        mutation_activity_id: mutation.activity_id.clone(),
        run_id: mutation.run_id.clone(),
        thread_id: mutation.thread_id.clone(),
        source_stable_id: source_stable_id.to_string(),
        mutation_started_seq: mutation.started_seq,
        mutation_ended_seq: mutation.ended_seq,
        before_read_activity_id: before_read.map(|read| read.activity_id.clone()),
        after_read_activity_id: after_read.map(|read| read.activity_id.clone()),
        before_content_fingerprint: before_content.map(str::to_string),
        after_content_fingerprint: after_content.map(str::to_string),
        observation,
        evidence_gaps: gaps,
        causality_status: MutationCausalityStatus::NotProven,
    }
}

fn sequence_interval(activity: &HistoricalToolActivityRecord) -> Option<(u64, u64)> {
    let start = activity.started_seq?;
    let end = activity.ended_seq?;
    (start <= end).then_some((start, end))
}

fn exact_content_fingerprint(activity: &HistoricalToolActivityRecord) -> Option<&str> {
    (activity.content_identity_coverage == EvidenceIdentityCoverage::Observed)
        .then_some(activity.content_fingerprint.as_deref())
        .flatten()
}

fn has_intervening_same_thread_mutation(
    history: &ToolActivityHistory,
    target: &HistoricalToolActivityRecord,
    source_stable_id: &str,
    before: &HistoricalToolActivityRecord,
    after: &HistoricalToolActivityRecord,
    mutation_start: u64,
    mutation_end: u64,
) -> bool {
    let Some((_, before_end)) = sequence_interval(before) else {
        return true;
    };
    let Some((after_start, _)) = sequence_interval(after) else {
        return true;
    };

    history.activities.iter().any(|activity| {
        if activity.activity_id == target.activity_id
            || activity.run_id != target.run_id
            || activity.thread_id != target.thread_id
            || activity.category != "write_mutation"
            || activity.source_identity_coverage != EvidenceIdentityCoverage::Observed
            || activity.source_stable_id.as_deref() != Some(source_stable_id)
        {
            return false;
        }
        let Some((start, end)) = sequence_interval(activity) else {
            return true;
        };
        start < after_start && end > before_end && (start < mutation_start || end > mutation_end)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn activity(
        id: &str,
        thread: &str,
        category: &str,
        status: &str,
        start: Option<u64>,
        end: Option<u64>,
        content: Option<&str>,
    ) -> HistoricalToolActivityRecord {
        HistoricalToolActivityRecord {
            activity_id: id.into(),
            run_id: "run-1".into(),
            project_id: "prj-fixture".into(),
            workspace_id: "wsp-fixture".into(),
            thread_id: thread.into(),
            agent_ordinal: start.unwrap_or_default(),
            category: category.into(),
            status: status.into(),
            started_seq: start,
            ended_seq: end,
            source_stable_id: Some("src-v1-fixture".into()),
            source_identity_coverage: EvidenceIdentityCoverage::Observed,
            content_fingerprint: content.map(str::to_string),
            content_identity_coverage: if content.is_some() {
                EvidenceIdentityCoverage::Observed
            } else {
                EvidenceIdentityCoverage::NotCaptured
            },
            run_created_at_unix: 1,
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
    fn reports_exact_difference_across_a_complete_single_thread_mutation_window() {
        let report = build_source_mutation_window_history(&history(vec![
            activity(
                "read-before",
                "thread-a",
                "file_read",
                "completed",
                Some(10),
                Some(11),
                Some("cnt-v1-before"),
            ),
            activity(
                "mutation",
                "thread-a",
                "write_mutation",
                "completed",
                Some(20),
                Some(21),
                None,
            ),
            activity(
                "read-after",
                "thread-a",
                "file_read",
                "completed",
                Some(30),
                Some(31),
                Some("cnt-v1-after"),
            ),
        ]))
        .expect("report");

        assert_eq!(report.windows.len(), 1);
        assert_eq!(
            report.windows[0].observation,
            MutationWindowObservation::ExactContentDifferenceObserved
        );
        assert!(report.windows[0].evidence_gaps.is_empty());
        assert_eq!(
            report.windows[0].causality_status,
            MutationCausalityStatus::NotProven
        );
    }

    #[test]
    fn reports_exact_equality_without_claiming_no_mutation_effect() {
        let report = build_source_mutation_window_history(&history(vec![
            activity(
                "read-before",
                "thread-a",
                "file_read",
                "completed",
                Some(10),
                Some(11),
                Some("cnt-v1-same"),
            ),
            activity(
                "mutation",
                "thread-a",
                "write_mutation",
                "completed",
                Some(20),
                Some(21),
                None,
            ),
            activity(
                "read-after",
                "thread-a",
                "file_read",
                "completed",
                Some(30),
                Some(31),
                Some("cnt-v1-same"),
            ),
        ]))
        .expect("report");

        assert_eq!(
            report.windows[0].observation,
            MutationWindowObservation::ExactContentEqualityObserved
        );
        assert_eq!(
            report.windows[0].causality_status,
            MutationCausalityStatus::NotProven
        );
    }

    #[test]
    fn non_completed_read_stays_unknown_even_with_exact_result_identity() {
        let report = build_source_mutation_window_history(&history(vec![
            activity(
                "read-before",
                "thread-a",
                "file_read",
                "completed",
                Some(10),
                Some(11),
                Some("cnt-v1-before"),
            ),
            activity(
                "mutation",
                "thread-a",
                "write_mutation",
                "completed",
                Some(20),
                Some(21),
                None,
            ),
            activity(
                "read-after",
                "thread-a",
                "file_read",
                "failed",
                Some(30),
                Some(31),
                Some("cnt-v1-error-output"),
            ),
        ]))
        .expect("report");

        assert_eq!(
            report.windows[0].observation,
            MutationWindowObservation::Unknown
        );
        assert!(
            report.windows[0]
                .evidence_gaps
                .contains(&MutationWindowEvidenceGap::ReadNotCompleted)
        );
    }

    #[test]
    fn incomplete_read_identity_stays_unknown() {
        let report = build_source_mutation_window_history(&history(vec![
            activity(
                "read-before",
                "thread-a",
                "file_read",
                "completed",
                Some(10),
                Some(11),
                Some("cnt-v1-before"),
            ),
            activity(
                "mutation",
                "thread-a",
                "write_mutation",
                "completed",
                Some(20),
                Some(21),
                None,
            ),
            activity(
                "read-after",
                "thread-a",
                "file_read",
                "completed",
                Some(30),
                Some(31),
                None,
            ),
        ]))
        .expect("report");

        assert_eq!(
            report.windows[0].observation,
            MutationWindowObservation::Unknown
        );
        assert!(
            report.windows[0]
                .evidence_gaps
                .contains(&MutationWindowEvidenceGap::IncompleteReadContentIdentity)
        );
    }

    #[test]
    fn another_same_source_mutation_in_the_window_blocks_the_observation() {
        let report = build_source_mutation_window_history(&history(vec![
            activity(
                "read-before",
                "thread-a",
                "file_read",
                "completed",
                Some(10),
                Some(11),
                Some("cnt-v1-before"),
            ),
            activity(
                "mutation",
                "thread-a",
                "write_mutation",
                "completed",
                Some(20),
                Some(21),
                None,
            ),
            activity(
                "other-mutation",
                "thread-a",
                "write_mutation",
                "completed",
                Some(25),
                Some(26),
                None,
            ),
            activity(
                "read-after",
                "thread-a",
                "file_read",
                "completed",
                Some(30),
                Some(31),
                Some("cnt-v1-after"),
            ),
        ]))
        .expect("report");

        assert_eq!(
            report.windows[0].observation,
            MutationWindowObservation::Unknown
        );
        assert!(
            report.windows[0]
                .evidence_gaps
                .contains(&MutationWindowEvidenceGap::InterveningSameThreadMutation)
        );
    }

    #[test]
    fn cross_thread_same_source_mutation_keeps_order_unknown() {
        let report = build_source_mutation_window_history(&history(vec![
            activity(
                "read-before",
                "thread-a",
                "file_read",
                "completed",
                Some(10),
                Some(11),
                Some("cnt-v1-before"),
            ),
            activity(
                "mutation",
                "thread-a",
                "write_mutation",
                "completed",
                Some(20),
                Some(21),
                None,
            ),
            activity(
                "read-after",
                "thread-a",
                "file_read",
                "completed",
                Some(30),
                Some(31),
                Some("cnt-v1-after"),
            ),
            activity(
                "other-thread-mutation",
                "thread-b",
                "write_mutation",
                "completed",
                Some(40),
                Some(41),
                None,
            ),
        ]))
        .expect("report");

        assert_eq!(
            report.windows[0].observation,
            MutationWindowObservation::Unknown
        );
        assert!(
            report.windows[0]
                .evidence_gaps
                .contains(&MutationWindowEvidenceGap::CrossThreadMutationOrderUnknown)
        );
    }

    #[test]
    fn rejects_unknown_tool_activity_schema() {
        let mut value = history(Vec::new());
        value.schema_version = 99;
        assert!(matches!(
            build_source_mutation_window_history(&value),
            Err(
                SourceMutationWindowBuildError::UnsupportedToolActivityHistorySchema {
                    actual: 99,
                    ..
                }
            )
        ));
    }
}
