use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use serde::{Deserialize, Serialize};
use tokn_domain::{
    HistoricalToolActivityRecord, TOOL_ACTIVITY_HISTORY_SCHEMA_VERSION, ToolActivityHistory,
};

pub const ACTIVITY_TIMELINE_SCHEMA_VERSION: u64 = 1;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ActivityOrderStatus {
    SourceSequenceComplete,
    PartialSourceSequence,
    AgentOrdinalOnly,
    #[default]
    CrossAgentUnknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentActivityTimeline {
    pub thread_id: String,
    pub ordering_status: ActivityOrderStatus,
    pub activities: Vec<HistoricalToolActivityRecord>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RunActivityTimeline {
    pub run_id: String,
    pub project_id: String,
    pub workspace_id: String,
    pub run_created_at_unix: i64,
    pub cross_agent_order_status: ActivityOrderStatus,
    pub agents: Vec<AgentActivityTimeline>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExactRepeatedOperation {
    pub operation_fingerprint: String,
    pub category: String,
    pub occurrences: u64,
    pub run_count: u64,
    pub thread_count: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ActivityTimeline {
    pub schema_version: u64,
    pub source_history_schema_version: u64,
    pub project_filter: Option<String>,
    pub workspace_filter: Option<String>,
    pub run_limit: u64,
    pub runs: Vec<RunActivityTimeline>,
    pub exact_repetitions: Vec<ExactRepeatedOperation>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ActivityTimelineBuildError {
    UnsupportedToolActivityHistorySchema { actual: u64, expected: u64 },
}

impl fmt::Display for ActivityTimelineBuildError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedToolActivityHistorySchema { actual, expected } => write!(
                formatter,
                "unsupported ToolActivityHistory schema_version {actual}; expected {expected}"
            ),
        }
    }
}

impl std::error::Error for ActivityTimelineBuildError {}

pub fn build_activity_timeline(
    history: &ToolActivityHistory,
) -> Result<ActivityTimeline, ActivityTimelineBuildError> {
    if history.schema_version != TOOL_ACTIVITY_HISTORY_SCHEMA_VERSION {
        return Err(
            ActivityTimelineBuildError::UnsupportedToolActivityHistorySchema {
                actual: history.schema_version,
                expected: TOOL_ACTIVITY_HISTORY_SCHEMA_VERSION,
            },
        );
    }

    let mut run_groups = BTreeMap::<
        String,
        (
            String,
            String,
            i64,
            BTreeMap<String, Vec<HistoricalToolActivityRecord>>,
        ),
    >::new();

    for activity in &history.activities {
        let entry = run_groups.entry(activity.run_id.clone()).or_insert_with(|| {
            (
                activity.project_id.clone(),
                activity.workspace_id.clone(),
                activity.run_created_at_unix,
                BTreeMap::new(),
            )
        });
        entry
            .3
            .entry(activity.thread_id.clone())
            .or_default()
            .push(activity.clone());
    }

    let mut runs = Vec::with_capacity(run_groups.len());
    for (run_id, (project_id, workspace_id, created_at, agents)) in run_groups {
        let mut agent_timelines = Vec::with_capacity(agents.len());
        for (thread_id, mut activities) in agents {
            activities.sort_by_key(|activity| activity.agent_ordinal);
            let ordering_status = activity_order_status(&activities);
            agent_timelines.push(AgentActivityTimeline {
                thread_id,
                ordering_status,
                activities,
            });
        }

        runs.push(RunActivityTimeline {
            run_id,
            project_id,
            workspace_id,
            run_created_at_unix: created_at,
            cross_agent_order_status: ActivityOrderStatus::CrossAgentUnknown,
            agents: agent_timelines,
        });
    }
    runs.sort_by(|left, right| {
        right
            .run_created_at_unix
            .cmp(&left.run_created_at_unix)
            .then(left.run_id.cmp(&right.run_id))
    });

    let exact_repetitions = exact_repetitions(&history.activities);

    Ok(ActivityTimeline {
        schema_version: ACTIVITY_TIMELINE_SCHEMA_VERSION,
        source_history_schema_version: history.schema_version,
        project_filter: history.project_filter.clone(),
        workspace_filter: history.workspace_filter.clone(),
        run_limit: history.run_limit,
        runs,
        exact_repetitions,
    })
}

fn activity_order_status(activities: &[HistoricalToolActivityRecord]) -> ActivityOrderStatus {
    if activities.is_empty() {
        return ActivityOrderStatus::AgentOrdinalOnly;
    }

    let sequenced = activities
        .iter()
        .filter(|activity| activity.started_seq.is_some())
        .count();

    if sequenced == activities.len() {
        ActivityOrderStatus::SourceSequenceComplete
    } else if sequenced > 0 {
        ActivityOrderStatus::PartialSourceSequence
    } else {
        ActivityOrderStatus::AgentOrdinalOnly
    }
}

fn exact_repetitions(
    activities: &[HistoricalToolActivityRecord],
) -> Vec<ExactRepeatedOperation> {
    let mut groups = BTreeMap::<
        (String, String),
        (u64, BTreeSet<String>, BTreeSet<(String, String)>),
    >::new();

    for activity in activities {
        let Some(fingerprint) = activity.operation_fingerprint.as_ref() else {
            continue;
        };
        let entry = groups
            .entry((activity.category.clone(), fingerprint.clone()))
            .or_insert_with(|| (0, BTreeSet::new(), BTreeSet::new()));
        entry.0 += 1;
        entry.1.insert(activity.run_id.clone());
        entry
            .2
            .insert((activity.run_id.clone(), activity.thread_id.clone()));
    }

    let mut repetitions = groups
        .into_iter()
        .filter_map(
            |((category, operation_fingerprint), (occurrences, runs, threads))| {
                (occurrences > 1).then(|| ExactRepeatedOperation {
                    operation_fingerprint,
                    category,
                    occurrences,
                    run_count: runs.len() as u64,
                    thread_count: threads.len() as u64,
                })
            },
        )
        .collect::<Vec<_>>();

    repetitions.sort_by(|left, right| {
        right
            .occurrences
            .cmp(&left.occurrences)
            .then(left.category.cmp(&right.category))
            .then(
                left.operation_fingerprint
                    .cmp(&right.operation_fingerprint),
            )
    });
    repetitions
}

#[cfg(test)]
mod tests {
    use super::*;

    fn activity(
        run_id: &str,
        thread_id: &str,
        ordinal: u64,
        started_seq: Option<u64>,
        fingerprint: Option<&str>,
    ) -> HistoricalToolActivityRecord {
        HistoricalToolActivityRecord {
            activity_id: format!("act-{run_id}-{thread_id}-{ordinal}"),
            run_id: run_id.into(),
            project_id: "prj-0123456789abcdef01234567".into(),
            workspace_id: "wsp-0123456789abcdef01234567".into(),
            thread_id: thread_id.into(),
            agent_ordinal: ordinal,
            kind: "exec_command".into(),
            tool_name: Some("exec_command".into()),
            category: "file_read".into(),
            surface: "session_rollout".into(),
            requester_type: None,
            status: "completed".into(),
            started_seq,
            ended_seq: started_seq.map(|value| value + 1),
            invocation_payload_bytes: None,
            result_payload_bytes: None,
            result_output_chars: None,
            max_output_tokens: None,
            original_token_count: None,
            operation_fingerprint: fingerprint.map(str::to_string),
            workdir_fingerprint: Some("cwd-0123456789abcdef01234567".into()),
            parse_error_present: false,
            run_created_at_unix: if run_id == "run-2" { 2 } else { 1 },
        }
    }

    fn history(activities: Vec<HistoricalToolActivityRecord>) -> ToolActivityHistory {
        ToolActivityHistory {
            schema_version: TOOL_ACTIVITY_HISTORY_SCHEMA_VERSION,
            project_filter: Some("prj-0123456789abcdef01234567".into()),
            workspace_filter: None,
            run_limit: 10,
            activities,
        }
    }

    #[test]
    fn builds_per_agent_timelines_without_inventing_cross_agent_order() {
        let report = build_activity_timeline(&history(vec![
            activity("run-1", "thread-a", 1, Some(20), None),
            activity("run-1", "thread-a", 0, Some(10), None),
            activity("run-1", "thread-b", 0, None, None),
        ]))
        .unwrap();

        assert_eq!(report.runs.len(), 1);
        assert_eq!(
            report.runs[0].cross_agent_order_status,
            ActivityOrderStatus::CrossAgentUnknown
        );
        assert_eq!(report.runs[0].agents.len(), 2);
        assert_eq!(
            report.runs[0].agents[0].ordering_status,
            ActivityOrderStatus::SourceSequenceComplete
        );
        assert_eq!(report.runs[0].agents[0].activities[0].agent_ordinal, 0);
        assert_eq!(
            report.runs[0].agents[1].ordering_status,
            ActivityOrderStatus::AgentOrdinalOnly
        );
    }

    #[test]
    fn reports_partial_source_sequence_coverage() {
        let report = build_activity_timeline(&history(vec![
            activity("run-1", "thread-a", 0, Some(10), None),
            activity("run-1", "thread-a", 1, None, None),
        ]))
        .unwrap();

        assert_eq!(
            report.runs[0].agents[0].ordering_status,
            ActivityOrderStatus::PartialSourceSequence
        );
    }

    #[test]
    fn groups_only_exact_repeated_operation_fingerprints() {
        let report = build_activity_timeline(&history(vec![
            activity("run-1", "thread-a", 0, None, Some("op-same")),
            activity("run-1", "thread-b", 0, None, Some("op-same")),
            activity("run-2", "thread-a", 0, None, Some("op-same")),
            activity("run-2", "thread-a", 1, None, Some("op-other")),
        ]))
        .unwrap();

        assert_eq!(report.exact_repetitions.len(), 1);
        let repeated = &report.exact_repetitions[0];
        assert_eq!(repeated.operation_fingerprint, "op-same");
        assert_eq!(repeated.occurrences, 3);
        assert_eq!(repeated.run_count, 2);
        assert_eq!(repeated.thread_count, 3);
    }

    #[test]
    fn unknown_history_schema_fails_closed() {
        let mut history = history(Vec::new());
        history.schema_version = 99;

        assert!(matches!(
            build_activity_timeline(&history),
            Err(
                ActivityTimelineBuildError::UnsupportedToolActivityHistorySchema {
                    actual: 99,
                    ..
                }
            )
        ));
    }
}
