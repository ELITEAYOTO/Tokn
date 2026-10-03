use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use serde::{Deserialize, Serialize};
use tokn_domain::{
    EvidenceIdentityCoverage, HISTORICAL_SNAPSHOT_SCHEMA_VERSION, HistoricalAgentRecord,
    HistoricalSnapshot, TOOL_ACTIVITY_HISTORY_SCHEMA_VERSION, ToolActivityHistory,
};

pub const CROSS_AGENT_EVIDENCE_SCHEMA_VERSION: u64 = 2;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ResultIdentityMatch {
    Same,
    Different,
    #[default]
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum CrossAgentRelationKind {
    DirectParentChild,
    AncestorDescendant,
    Sibling,
    OtherKnownLineage,
    UnknownLineage,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CrossAgentParticipant {
    pub thread_id: String,
    pub parent_thread_id: Option<String>,
    pub depth: Option<u32>,
    pub occurrences: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CrossAgentPairRelation {
    pub first_thread_id: String,
    pub second_thread_id: String,
    pub relation: CrossAgentRelationKind,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CrossAgentOperationOverlap {
    pub run_id: String,
    pub project_id: String,
    pub workspace_id: String,
    pub run_created_at_unix: i64,
    pub operation_fingerprint: String,
    pub category: String,
    pub occurrences: u64,
    pub thread_count: u64,
    pub participants: Vec<CrossAgentParticipant>,
    pub relations: Vec<CrossAgentPairRelation>,
    pub result_identity_coverage: EvidenceIdentityCoverage,
    pub result_identity_match: ResultIdentityMatch,
    pub distinct_result_fingerprint_count: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CrossAgentEvidenceReport {
    pub schema_version: u64,
    pub source_snapshot_schema_version: u64,
    pub source_activity_history_schema_version: u64,
    pub project_filter: Option<String>,
    pub workspace_filter: Option<String>,
    pub run_limit: u64,
    pub overlaps: Vec<CrossAgentOperationOverlap>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CrossAgentEvidenceBuildError {
    UnsupportedHistoricalSnapshotSchema { actual: u64, expected: u64 },
    UnsupportedToolActivityHistorySchema { actual: u64, expected: u64 },
    ProjectFilterMismatch,
    WorkspaceFilterMismatch,
}

impl fmt::Display for CrossAgentEvidenceBuildError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedHistoricalSnapshotSchema { actual, expected } => write!(
                formatter,
                "unsupported HistoricalSnapshot schema_version {actual}; expected {expected}"
            ),
            Self::UnsupportedToolActivityHistorySchema { actual, expected } => write!(
                formatter,
                "unsupported ToolActivityHistory schema_version {actual}; expected {expected}"
            ),
            Self::ProjectFilterMismatch => {
                write!(
                    formatter,
                    "HistoricalSnapshot and ToolActivityHistory project filters differ"
                )
            }
            Self::WorkspaceFilterMismatch => write!(
                formatter,
                "HistoricalSnapshot and ToolActivityHistory workspace filters differ"
            ),
        }
    }
}

impl std::error::Error for CrossAgentEvidenceBuildError {}

pub fn build_cross_agent_evidence(
    snapshot: &HistoricalSnapshot,
    activity_history: &ToolActivityHistory,
) -> Result<CrossAgentEvidenceReport, CrossAgentEvidenceBuildError> {
    if snapshot.schema_version != HISTORICAL_SNAPSHOT_SCHEMA_VERSION {
        return Err(
            CrossAgentEvidenceBuildError::UnsupportedHistoricalSnapshotSchema {
                actual: snapshot.schema_version,
                expected: HISTORICAL_SNAPSHOT_SCHEMA_VERSION,
            },
        );
    }
    if activity_history.schema_version != TOOL_ACTIVITY_HISTORY_SCHEMA_VERSION {
        return Err(
            CrossAgentEvidenceBuildError::UnsupportedToolActivityHistorySchema {
                actual: activity_history.schema_version,
                expected: TOOL_ACTIVITY_HISTORY_SCHEMA_VERSION,
            },
        );
    }
    if snapshot.project_filter != activity_history.project_filter {
        return Err(CrossAgentEvidenceBuildError::ProjectFilterMismatch);
    }
    if snapshot.workspace_filter != activity_history.workspace_filter {
        return Err(CrossAgentEvidenceBuildError::WorkspaceFilterMismatch);
    }

    let agents = snapshot
        .agents
        .iter()
        .map(|agent| ((agent.run_id.clone(), agent.thread_id.clone()), agent))
        .collect::<BTreeMap<_, _>>();

    let mut groups = BTreeMap::<(String, String, String), OperationGroup>::new();
    for activity in &activity_history.activities {
        let Some(fingerprint) = activity.operation_fingerprint.as_ref() else {
            continue;
        };
        let key = (
            activity.run_id.clone(),
            activity.category.clone(),
            fingerprint.clone(),
        );
        let entry = groups.entry(key).or_insert_with(|| OperationGroup {
            project_id: activity.project_id.clone(),
            workspace_id: activity.workspace_id.clone(),
            run_created_at_unix: activity.run_created_at_unix,
            occurrences: 0,
            per_thread: BTreeMap::new(),
            result_fingerprints: BTreeSet::new(),
            result_fingerprint_occurrences: 0,
            result_identity_coverages: Vec::new(),
        });
        entry.occurrences += 1;
        if let Some(result_fingerprint) = activity.content_fingerprint.as_ref() {
            entry.result_fingerprints.insert(result_fingerprint.clone());
            entry.result_fingerprint_occurrences += 1;
        }
        entry
            .result_identity_coverages
            .push(activity.content_identity_coverage);
        *entry
            .per_thread
            .entry(activity.thread_id.clone())
            .or_default() += 1;
    }

    let mut overlaps = Vec::new();
    for ((run_id, category, operation_fingerprint), group) in groups {
        if group.per_thread.len() < 2 {
            continue;
        }
        let participants = group
            .per_thread
            .iter()
            .map(|(thread_id, occurrences)| {
                let agent = agents.get(&(run_id.clone(), thread_id.clone())).copied();
                CrossAgentParticipant {
                    thread_id: thread_id.clone(),
                    parent_thread_id: agent.and_then(|item| item.parent_thread_id.clone()),
                    depth: agent.map(|item| item.depth),
                    occurrences: *occurrences,
                }
            })
            .collect::<Vec<_>>();

        let mut relations = Vec::new();
        for first_index in 0..participants.len() {
            for second_index in first_index + 1..participants.len() {
                let first = &participants[first_index];
                let second = &participants[second_index];
                relations.push(CrossAgentPairRelation {
                    first_thread_id: first.thread_id.clone(),
                    second_thread_id: second.thread_id.clone(),
                    relation: relation_kind(&run_id, &first.thread_id, &second.thread_id, &agents),
                });
            }
        }

        let (result_identity_coverage, result_identity_match) = summarize_result_identity(&group);
        overlaps.push(CrossAgentOperationOverlap {
            run_id,
            project_id: group.project_id,
            workspace_id: group.workspace_id,
            run_created_at_unix: group.run_created_at_unix,
            operation_fingerprint,
            category,
            occurrences: group.occurrences,
            thread_count: participants.len() as u64,
            participants,
            relations,
            result_identity_coverage,
            result_identity_match,
            distinct_result_fingerprint_count: group.result_fingerprints.len() as u64,
        });
    }

    overlaps.sort_by(|left, right| {
        right
            .run_created_at_unix
            .cmp(&left.run_created_at_unix)
            .then(right.occurrences.cmp(&left.occurrences))
            .then(left.category.cmp(&right.category))
            .then(left.operation_fingerprint.cmp(&right.operation_fingerprint))
    });

    Ok(CrossAgentEvidenceReport {
        schema_version: CROSS_AGENT_EVIDENCE_SCHEMA_VERSION,
        source_snapshot_schema_version: snapshot.schema_version,
        source_activity_history_schema_version: activity_history.schema_version,
        project_filter: activity_history.project_filter.clone(),
        workspace_filter: activity_history.workspace_filter.clone(),
        run_limit: activity_history.run_limit,
        overlaps,
    })
}

struct OperationGroup {
    project_id: String,
    workspace_id: String,
    run_created_at_unix: i64,
    occurrences: u64,
    per_thread: BTreeMap<String, u64>,
    result_fingerprints: BTreeSet<String>,
    result_fingerprint_occurrences: u64,
    result_identity_coverages: Vec<EvidenceIdentityCoverage>,
}

fn summarize_result_identity(
    group: &OperationGroup,
) -> (EvidenceIdentityCoverage, ResultIdentityMatch) {
    let observed = group
        .result_identity_coverages
        .iter()
        .filter(|coverage| **coverage == EvidenceIdentityCoverage::Observed)
        .count();
    let has_partial = group
        .result_identity_coverages
        .contains(&EvidenceIdentityCoverage::Partial);
    let has_unknown = group
        .result_identity_coverages
        .contains(&EvidenceIdentityCoverage::Unknown);
    let complete = observed == group.occurrences as usize
        && group.result_fingerprint_occurrences == group.occurrences
        && observed > 0;

    if complete {
        let result_match = if group.result_fingerprints.len() == 1 {
            ResultIdentityMatch::Same
        } else {
            ResultIdentityMatch::Different
        };
        return (EvidenceIdentityCoverage::Observed, result_match);
    }
    if observed > 0 || has_partial {
        return (EvidenceIdentityCoverage::Partial, ResultIdentityMatch::Unknown);
    }
    if has_unknown {
        return (EvidenceIdentityCoverage::Unknown, ResultIdentityMatch::Unknown);
    }
    (
        EvidenceIdentityCoverage::NotCaptured,
        ResultIdentityMatch::Unknown,
    )
}

fn relation_kind(
    run_id: &str,
    first_thread_id: &str,
    second_thread_id: &str,
    agents: &BTreeMap<(String, String), &HistoricalAgentRecord>,
) -> CrossAgentRelationKind {
    let first = agents
        .get(&(run_id.to_string(), first_thread_id.to_string()))
        .copied();
    let second = agents
        .get(&(run_id.to_string(), second_thread_id.to_string()))
        .copied();
    let (Some(first), Some(second)) = (first, second) else {
        return CrossAgentRelationKind::UnknownLineage;
    };

    if first.parent_thread_id.as_deref() == Some(second_thread_id)
        || second.parent_thread_id.as_deref() == Some(first_thread_id)
    {
        return CrossAgentRelationKind::DirectParentChild;
    }

    if first.parent_thread_id.is_some() && first.parent_thread_id == second.parent_thread_id {
        return CrossAgentRelationKind::Sibling;
    }

    let first_to_second = ancestor_status(run_id, first_thread_id, second_thread_id, agents);
    let second_to_first = ancestor_status(run_id, second_thread_id, first_thread_id, agents);
    if first_to_second == Some(true) || second_to_first == Some(true) {
        return CrossAgentRelationKind::AncestorDescendant;
    }
    if first_to_second.is_none() || second_to_first.is_none() {
        return CrossAgentRelationKind::UnknownLineage;
    }

    CrossAgentRelationKind::OtherKnownLineage
}

fn ancestor_status(
    run_id: &str,
    ancestor_thread_id: &str,
    descendant_thread_id: &str,
    agents: &BTreeMap<(String, String), &HistoricalAgentRecord>,
) -> Option<bool> {
    let mut current = descendant_thread_id.to_string();
    let mut visited = BTreeSet::new();

    loop {
        if !visited.insert(current.clone()) {
            return None;
        }
        let record = agents.get(&(run_id.to_string(), current)).copied()?;
        let Some(parent) = record.parent_thread_id.as_deref() else {
            return Some(false);
        };
        if parent == ancestor_thread_id {
            return Some(true);
        }
        current = parent.to_string();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokn_domain::HistoricalToolActivityRecord;

    const PROJECT: &str = "prj-0123456789abcdef01234567";
    const WORKSPACE: &str = "wsp-0123456789abcdef01234567";

    fn agent(thread_id: &str, parent_thread_id: Option<&str>, depth: u32) -> HistoricalAgentRecord {
        HistoricalAgentRecord {
            run_id: "run-1".into(),
            thread_id: thread_id.into(),
            parent_thread_id: parent_thread_id.map(str::to_string),
            depth,
            terminal_status: "completed".into(),
            ..Default::default()
        }
    }

    fn activity(
        run_id: &str,
        thread_id: &str,
        ordinal: u64,
        fingerprint: Option<&str>,
    ) -> HistoricalToolActivityRecord {
        HistoricalToolActivityRecord {
            activity_id: format!("act-{run_id}-{thread_id}-{ordinal}"),
            run_id: run_id.into(),
            project_id: PROJECT.into(),
            workspace_id: WORKSPACE.into(),
            thread_id: thread_id.into(),
            agent_ordinal: ordinal,
            kind: "exec_command".into(),
            category: "file_read".into(),
            surface: "session_rollout".into(),
            status: "completed".into(),
            operation_fingerprint: fingerprint.map(str::to_string),
            run_created_at_unix: if run_id == "run-2" { 2 } else { 1 },
            ..Default::default()
        }
    }

    fn activity_with_result(
        run_id: &str,
        thread_id: &str,
        ordinal: u64,
        operation_fingerprint: &str,
        content_fingerprint: Option<&str>,
        coverage: EvidenceIdentityCoverage,
    ) -> HistoricalToolActivityRecord {
        let mut item = activity(
            run_id,
            thread_id,
            ordinal,
            Some(operation_fingerprint),
        );
        item.content_fingerprint = content_fingerprint.map(str::to_string);
        item.content_identity_coverage = coverage;
        item
    }

    fn snapshot(agents: Vec<HistoricalAgentRecord>) -> HistoricalSnapshot {
        HistoricalSnapshot {
            schema_version: HISTORICAL_SNAPSHOT_SCHEMA_VERSION,
            project_filter: Some(PROJECT.into()),
            workspace_filter: None,
            agents,
            ..Default::default()
        }
    }

    fn history(activities: Vec<HistoricalToolActivityRecord>) -> ToolActivityHistory {
        ToolActivityHistory {
            schema_version: TOOL_ACTIVITY_HISTORY_SCHEMA_VERSION,
            project_filter: Some(PROJECT.into()),
            workspace_filter: None,
            run_limit: 10,
            activities,
        }
    }

    #[test]
    fn reports_parent_child_operation_overlap_without_claiming_result_identity() {
        let report = build_cross_agent_evidence(
            &snapshot(vec![
                agent("root", None, 0),
                agent("child", Some("root"), 1),
            ]),
            &history(vec![
                activity("run-1", "root", 0, Some("op-same")),
                activity("run-1", "child", 0, Some("op-same")),
            ]),
        )
        .unwrap();
        assert_eq!(report.overlaps.len(), 1);
        let overlap = &report.overlaps[0];
        assert_eq!(overlap.occurrences, 2);
        assert_eq!(overlap.thread_count, 2);
        assert_eq!(
            overlap.result_identity_coverage,
            EvidenceIdentityCoverage::NotCaptured
        );
        assert_eq!(overlap.result_identity_match, ResultIdentityMatch::Unknown);
        assert_eq!(overlap.distinct_result_fingerprint_count, 0);
        assert_eq!(overlap.relations.len(), 1);
        assert_eq!(
            overlap.relations[0].relation,
            CrossAgentRelationKind::DirectParentChild
        );
    }

    #[test]
    fn reports_same_result_only_when_all_occurrences_have_same_observed_identity() {
        let report = build_cross_agent_evidence(
            &snapshot(vec![agent("root", None, 0), agent("child", Some("root"), 1)]),
            &history(vec![
                activity_with_result(
                    "run-1",
                    "root",
                    0,
                    "op-same",
                    Some("cnt-v1-a"),
                    EvidenceIdentityCoverage::Observed,
                ),
                activity_with_result(
                    "run-1",
                    "child",
                    0,
                    "op-same",
                    Some("cnt-v1-a"),
                    EvidenceIdentityCoverage::Observed,
                ),
            ]),
        )
        .unwrap();

        let overlap = &report.overlaps[0];
        assert_eq!(overlap.result_identity_coverage, EvidenceIdentityCoverage::Observed);
        assert_eq!(overlap.result_identity_match, ResultIdentityMatch::Same);
        assert_eq!(overlap.distinct_result_fingerprint_count, 1);
    }

    #[test]
    fn reports_different_result_when_complete_identity_disagrees() {
        let report = build_cross_agent_evidence(
            &snapshot(vec![agent("root", None, 0), agent("child", Some("root"), 1)]),
            &history(vec![
                activity_with_result(
                    "run-1", "root", 0, "op-same", Some("cnt-v1-a"),
                    EvidenceIdentityCoverage::Observed,
                ),
                activity_with_result(
                    "run-1", "child", 0, "op-same", Some("cnt-v1-b"),
                    EvidenceIdentityCoverage::Observed,
                ),
            ]),
        )
        .unwrap();

        let overlap = &report.overlaps[0];
        assert_eq!(overlap.result_identity_coverage, EvidenceIdentityCoverage::Observed);
        assert_eq!(overlap.result_identity_match, ResultIdentityMatch::Different);
        assert_eq!(overlap.distinct_result_fingerprint_count, 2);
    }

    #[test]
    fn incomplete_result_identity_is_partial_and_never_claims_match() {
        let report = build_cross_agent_evidence(
            &snapshot(vec![agent("root", None, 0), agent("child", Some("root"), 1)]),
            &history(vec![
                activity_with_result(
                    "run-1", "root", 0, "op-same", Some("cnt-v1-a"),
                    EvidenceIdentityCoverage::Observed,
                ),
                activity_with_result(
                    "run-1", "child", 0, "op-same", None,
                    EvidenceIdentityCoverage::NotCaptured,
                ),
            ]),
        )
        .unwrap();

        let overlap = &report.overlaps[0];
        assert_eq!(overlap.result_identity_coverage, EvidenceIdentityCoverage::Partial);
        assert_eq!(overlap.result_identity_match, ResultIdentityMatch::Unknown);
        assert_eq!(overlap.distinct_result_fingerprint_count, 1);
    }

    #[test]
    fn unavailable_result_identity_is_unknown() {
        let report = build_cross_agent_evidence(
            &snapshot(vec![agent("root", None, 0), agent("child", Some("root"), 1)]),
            &history(vec![
                activity_with_result(
                    "run-1", "root", 0, "op-same", None,
                    EvidenceIdentityCoverage::Unknown,
                ),
                activity_with_result(
                    "run-1", "child", 0, "op-same", None,
                    EvidenceIdentityCoverage::NotCaptured,
                ),
            ]),
        )
        .unwrap();

        let overlap = &report.overlaps[0];
        assert_eq!(overlap.result_identity_coverage, EvidenceIdentityCoverage::Unknown);
        assert_eq!(overlap.result_identity_match, ResultIdentityMatch::Unknown);
    }

    #[test]
    fn same_thread_repeat_is_not_cross_agent_overlap() {
        let report = build_cross_agent_evidence(
            &snapshot(vec![agent("root", None, 0)]),
            &history(vec![
                activity("run-1", "root", 0, Some("op-same")),
                activity("run-1", "root", 1, Some("op-same")),
            ]),
        )
        .unwrap();

        assert!(report.overlaps.is_empty());
    }

    #[test]
    fn distinguishes_ancestor_and_sibling_relations() {
        let report = build_cross_agent_evidence(
            &snapshot(vec![
                agent("root", None, 0),
                agent("child-a", Some("root"), 1),
                agent("child-b", Some("root"), 1),
                agent("grandchild", Some("child-a"), 2),
            ]),
            &history(vec![
                activity("run-1", "root", 0, Some("op-tree")),
                activity("run-1", "child-a", 0, Some("op-tree")),
                activity("run-1", "child-b", 0, Some("op-tree")),
                activity("run-1", "grandchild", 0, Some("op-tree")),
            ]),
        )
        .unwrap();

        let relations = report.overlaps[0]
            .relations
            .iter()
            .map(|relation| relation.relation)
            .collect::<Vec<_>>();
        assert!(relations.contains(&CrossAgentRelationKind::DirectParentChild));
        assert!(relations.contains(&CrossAgentRelationKind::AncestorDescendant));
        assert!(relations.contains(&CrossAgentRelationKind::Sibling));
    }

    #[test]
    fn missing_agent_lineage_is_explicitly_unknown() {
        let report = build_cross_agent_evidence(
            &snapshot(vec![agent("root", None, 0)]),
            &history(vec![
                activity("run-1", "root", 0, Some("op-same")),
                activity("run-1", "missing-child", 0, Some("op-same")),
            ]),
        )
        .unwrap();

        assert_eq!(
            report.overlaps[0].relations[0].relation,
            CrossAgentRelationKind::UnknownLineage
        );
        let missing = report.overlaps[0]
            .participants
            .iter()
            .find(|participant| participant.thread_id == "missing-child")
            .expect("missing participant");
        assert_eq!(missing.depth, None);
    }

    #[test]
    fn same_operation_across_runs_alone_is_not_cross_agent_overlap() {
        let report = build_cross_agent_evidence(
            &snapshot(vec![agent("root", None, 0)]),
            &history(vec![
                activity("run-1", "root", 0, Some("op-same")),
                activity("run-2", "root", 0, Some("op-same")),
            ]),
        )
        .unwrap();
        assert!(report.overlaps.is_empty());
    }

    #[test]
    fn filter_mismatch_fails_closed() {
        let mut activity_history = history(Vec::new());
        activity_history.project_filter = Some("prj-other".into());
        assert!(matches!(
            build_cross_agent_evidence(&snapshot(Vec::new()), &activity_history),
            Err(CrossAgentEvidenceBuildError::ProjectFilterMismatch)
        ));
    }

    #[test]
    fn unknown_source_schema_fails_closed() {
        let mut historical_snapshot = snapshot(Vec::new());
        historical_snapshot.schema_version = 99;
        assert!(matches!(
            build_cross_agent_evidence(&historical_snapshot, &history(Vec::new())),
            Err(
                CrossAgentEvidenceBuildError::UnsupportedHistoricalSnapshotSchema {
                    actual: 99,
                    ..
                }
            )
        ));
    }
}
