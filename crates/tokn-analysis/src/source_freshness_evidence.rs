use std::collections::BTreeMap;
use std::fmt;

use serde::{Deserialize, Serialize};
use tokn_domain::{
    SourceVersionBoundary, SourceVersionHistory, ToolActivityHistory,
    WORKSPACE_GIT_PROVENANCE_HISTORY_SCHEMA_VERSION, WorkspaceGitProvenanceCoverage,
    WorkspaceGitProvenanceHistory,
};

use crate::{
    MutationWindowEvidenceGap, MutationWindowObservation, SourceBoundaryComparison,
    SourceBoundaryVersionBuildError, SourceMutationWindowBuildError,
    build_source_boundary_version_report, build_source_mutation_window_history,
};

pub const SOURCE_FRESHNESS_EVIDENCE_REPORT_SCHEMA_VERSION: u64 = 1;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum FreshnessEvidenceObservation {
    ChangeAndRereadWithBoundaryChangeObserved,
    ChangeAndRereadObserved,
    SameContentRereadObserved,
    #[default]
    Unknown,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum WorkspaceGitBoundaryComparison {
    HeadChangedObserved,
    HeadUnchangedObserved,
    #[default]
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum FreshnessInferenceStatus {
    NotProven,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceFreshnessEvidenceObservation {
    pub run_id: String,
    pub project_id: String,
    pub workspace_id: String,
    pub thread_id: String,
    pub source_stable_id: String,
    pub mutation_activity_id: String,
    pub prior_read_activity_id: Option<String>,
    pub reread_activity_id: Option<String>,
    pub prior_content_fingerprint: Option<String>,
    pub reread_content_fingerprint: Option<String>,
    pub mutation_window_observation: MutationWindowObservation,
    pub mutation_window_evidence_gaps: Vec<MutationWindowEvidenceGap>,
    pub source_boundary_comparison: SourceBoundaryComparison,
    pub workspace_git_boundary_comparison: WorkspaceGitBoundaryComparison,
    pub git_before_dirty: Option<bool>,
    pub git_after_dirty: Option<bool>,
    pub evidence_observation: FreshnessEvidenceObservation,
    pub freshness_status: FreshnessInferenceStatus,
    pub invalidation_status: FreshnessInferenceStatus,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceFreshnessEvidenceReport {
    pub schema_version: u64,
    pub project_filter: Option<String>,
    pub workspace_filter: Option<String>,
    pub run_limit: u64,
    pub observations: Vec<SourceFreshnessEvidenceObservation>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SourceFreshnessEvidenceBuildError {
    MutationWindow(SourceMutationWindowBuildError),
    SourceBoundary(SourceBoundaryVersionBuildError),
    UnsupportedWorkspaceGitProvenanceHistorySchema {
        actual: u64,
        expected: u64,
    },
    InputScopeMismatch,
    DuplicateWorkspaceGitBoundary {
        run_id: String,
        project_id: String,
        workspace_id: String,
        boundary: SourceVersionBoundary,
    },
}

impl fmt::Display for SourceFreshnessEvidenceBuildError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MutationWindow(error) => write!(formatter, "mutation-window evidence: {error}"),
            Self::SourceBoundary(error) => write!(formatter, "source-boundary evidence: {error}"),
            Self::UnsupportedWorkspaceGitProvenanceHistorySchema { actual, expected } => write!(
                formatter,
                "unsupported WorkspaceGitProvenanceHistory schema_version {actual}; expected {expected}"
            ),
            Self::InputScopeMismatch => write!(
                formatter,
                "freshness evidence inputs must use identical project/workspace filters and run limits"
            ),
            Self::DuplicateWorkspaceGitBoundary {
                run_id,
                project_id,
                workspace_id,
                boundary,
            } => write!(
                formatter,
                "duplicate {} workspace Git provenance boundary for run {run_id} project {project_id} workspace {workspace_id}",
                boundary.as_str()
            ),
        }
    }
}

impl std::error::Error for SourceFreshnessEvidenceBuildError {}

impl From<SourceMutationWindowBuildError> for SourceFreshnessEvidenceBuildError {
    fn from(value: SourceMutationWindowBuildError) -> Self {
        Self::MutationWindow(value)
    }
}

impl From<SourceBoundaryVersionBuildError> for SourceFreshnessEvidenceBuildError {
    fn from(value: SourceBoundaryVersionBuildError) -> Self {
        Self::SourceBoundary(value)
    }
}

#[derive(Debug, Clone)]
struct GitBoundaryPair {
    before: Option<GitBoundaryValue>,
    after: Option<GitBoundaryValue>,
}

#[derive(Debug, Clone)]
struct GitBoundaryValue {
    coverage: WorkspaceGitProvenanceCoverage,
    head_fingerprint: Option<String>,
    dirty: Option<bool>,
}

pub fn build_source_freshness_evidence_report(
    tool_history: &ToolActivityHistory,
    source_versions: &SourceVersionHistory,
    workspace_git: &WorkspaceGitProvenanceHistory,
) -> Result<SourceFreshnessEvidenceReport, SourceFreshnessEvidenceBuildError> {
    ensure_matching_scope(tool_history, source_versions, workspace_git)?;
    if workspace_git.schema_version != WORKSPACE_GIT_PROVENANCE_HISTORY_SCHEMA_VERSION {
        return Err(
            SourceFreshnessEvidenceBuildError::UnsupportedWorkspaceGitProvenanceHistorySchema {
                actual: workspace_git.schema_version,
                expected: WORKSPACE_GIT_PROVENANCE_HISTORY_SCHEMA_VERSION,
            },
        );
    }

    let mutation_windows = build_source_mutation_window_history(tool_history)?;
    let source_boundaries = build_source_boundary_version_report(source_versions)?;
    let git_boundaries = build_git_boundary_map(workspace_git)?;

    let mut observations = Vec::with_capacity(mutation_windows.windows.len());
    for window in mutation_windows.windows {
        let Some(mutation) = tool_history
            .activities
            .iter()
            .find(|activity| activity.activity_id == window.mutation_activity_id)
        else {
            continue;
        };

        let source_boundary_comparison = source_boundaries
            .sources
            .iter()
            .find(|source| {
                source.run_id == window.run_id
                    && source.project_id == mutation.project_id
                    && source.workspace_id == mutation.workspace_id
                    && source.source_stable_id == window.source_stable_id
            })
            .map(|source| source.comparison)
            .unwrap_or(SourceBoundaryComparison::Unknown);

        let git_pair = git_boundaries.get(&(
            window.run_id.clone(),
            mutation.project_id.clone(),
            mutation.workspace_id.clone(),
        ));
        let (workspace_git_boundary_comparison, git_before_dirty, git_after_dirty) =
            compare_git_pair(git_pair);

        let evidence_observation = match (window.observation, source_boundary_comparison) {
            (
                MutationWindowObservation::ExactContentDifferenceObserved,
                SourceBoundaryComparison::ChangedObserved,
            ) => FreshnessEvidenceObservation::ChangeAndRereadWithBoundaryChangeObserved,
            (MutationWindowObservation::ExactContentDifferenceObserved, _) => {
                FreshnessEvidenceObservation::ChangeAndRereadObserved
            }
            (MutationWindowObservation::ExactContentEqualityObserved, _) => {
                FreshnessEvidenceObservation::SameContentRereadObserved
            }
            _ => FreshnessEvidenceObservation::Unknown,
        };

        observations.push(SourceFreshnessEvidenceObservation {
            run_id: window.run_id,
            project_id: mutation.project_id.clone(),
            workspace_id: mutation.workspace_id.clone(),
            thread_id: window.thread_id,
            source_stable_id: window.source_stable_id,
            mutation_activity_id: window.mutation_activity_id,
            prior_read_activity_id: window.before_read_activity_id,
            reread_activity_id: window.after_read_activity_id,
            prior_content_fingerprint: window.before_content_fingerprint,
            reread_content_fingerprint: window.after_content_fingerprint,
            mutation_window_observation: window.observation,
            mutation_window_evidence_gaps: window.evidence_gaps,
            source_boundary_comparison,
            workspace_git_boundary_comparison,
            git_before_dirty,
            git_after_dirty,
            evidence_observation,
            freshness_status: FreshnessInferenceStatus::NotProven,
            invalidation_status: FreshnessInferenceStatus::NotProven,
        });
    }

    Ok(SourceFreshnessEvidenceReport {
        schema_version: SOURCE_FRESHNESS_EVIDENCE_REPORT_SCHEMA_VERSION,
        project_filter: tool_history.project_filter.clone(),
        workspace_filter: tool_history.workspace_filter.clone(),
        run_limit: tool_history.run_limit,
        observations,
    })
}

fn ensure_matching_scope(
    tool_history: &ToolActivityHistory,
    source_versions: &SourceVersionHistory,
    workspace_git: &WorkspaceGitProvenanceHistory,
) -> Result<(), SourceFreshnessEvidenceBuildError> {
    let filters_match = tool_history.project_filter == source_versions.project_filter
        && tool_history.project_filter == workspace_git.project_filter
        && tool_history.workspace_filter == source_versions.workspace_filter
        && tool_history.workspace_filter == workspace_git.workspace_filter;
    let limits_match = tool_history.run_limit == source_versions.run_limit
        && tool_history.run_limit == workspace_git.run_limit;
    if !filters_match || !limits_match {
        return Err(SourceFreshnessEvidenceBuildError::InputScopeMismatch);
    }
    Ok(())
}

fn build_git_boundary_map(
    history: &WorkspaceGitProvenanceHistory,
) -> Result<BTreeMap<(String, String, String), GitBoundaryPair>, SourceFreshnessEvidenceBuildError>
{
    let mut grouped = BTreeMap::<(String, String, String), GitBoundaryPair>::new();
    for observation in &history.observations {
        let key = (
            observation.run_id.clone(),
            observation.project_id.clone(),
            observation.workspace_id.clone(),
        );
        let pair = grouped.entry(key).or_insert_with(|| GitBoundaryPair {
            before: None,
            after: None,
        });
        let slot = match observation.boundary {
            SourceVersionBoundary::Before => &mut pair.before,
            SourceVersionBoundary::After => &mut pair.after,
        };
        if slot.is_some() {
            return Err(
                SourceFreshnessEvidenceBuildError::DuplicateWorkspaceGitBoundary {
                    run_id: observation.run_id.clone(),
                    project_id: observation.project_id.clone(),
                    workspace_id: observation.workspace_id.clone(),
                    boundary: observation.boundary,
                },
            );
        }
        *slot = Some(GitBoundaryValue {
            coverage: observation.coverage,
            head_fingerprint: observation.head_fingerprint.clone(),
            dirty: observation.dirty,
        });
    }
    Ok(grouped)
}

fn compare_git_pair(
    pair: Option<&GitBoundaryPair>,
) -> (WorkspaceGitBoundaryComparison, Option<bool>, Option<bool>) {
    let Some(pair) = pair else {
        return (WorkspaceGitBoundaryComparison::Unknown, None, None);
    };
    let before_dirty = pair
        .before
        .as_ref()
        .filter(|value| value.coverage == WorkspaceGitProvenanceCoverage::Observed)
        .and_then(|value| value.dirty);
    let after_dirty = pair
        .after
        .as_ref()
        .filter(|value| value.coverage == WorkspaceGitProvenanceCoverage::Observed)
        .and_then(|value| value.dirty);

    let comparison = match (&pair.before, &pair.after) {
        (Some(before), Some(after))
            if before.coverage == WorkspaceGitProvenanceCoverage::Observed
                && after.coverage == WorkspaceGitProvenanceCoverage::Observed =>
        {
            match (&before.head_fingerprint, &after.head_fingerprint) {
                (Some(before_head), Some(after_head)) if before_head == after_head => {
                    WorkspaceGitBoundaryComparison::HeadUnchangedObserved
                }
                (Some(_), Some(_)) => WorkspaceGitBoundaryComparison::HeadChangedObserved,
                _ => WorkspaceGitBoundaryComparison::Unknown,
            }
        }
        _ => WorkspaceGitBoundaryComparison::Unknown,
    };

    (comparison, before_dirty, after_dirty)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokn_domain::{
        EvidenceIdentityCoverage, HistoricalSourceVersionRecord, HistoricalToolActivityRecord,
        HistoricalWorkspaceGitProvenanceRecord, SOURCE_VERSION_HISTORY_SCHEMA_VERSION,
        TOOL_ACTIVITY_HISTORY_SCHEMA_VERSION,
    };

    fn activity(
        id: &str,
        category: &str,
        status: &str,
        start: u64,
        end: u64,
        content: Option<&str>,
    ) -> HistoricalToolActivityRecord {
        HistoricalToolActivityRecord {
            activity_id: id.into(),
            run_id: "run-1".into(),
            project_id: "prj-fixture".into(),
            workspace_id: "wsp-fixture".into(),
            thread_id: "thread-a".into(),
            agent_ordinal: start,
            category: category.into(),
            status: status.into(),
            started_seq: Some(start),
            ended_seq: Some(end),
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

    fn tool_history(after_content: &str) -> ToolActivityHistory {
        ToolActivityHistory {
            schema_version: TOOL_ACTIVITY_HISTORY_SCHEMA_VERSION,
            project_filter: Some("prj-fixture".into()),
            workspace_filter: Some("wsp-fixture".into()),
            run_limit: 50,
            activities: vec![
                activity(
                    "read-before",
                    "file_read",
                    "completed",
                    10,
                    11,
                    Some("cnt-v1-before"),
                ),
                activity("mutation", "write_mutation", "completed", 20, 21, None),
                activity(
                    "read-after",
                    "file_read",
                    "completed",
                    30,
                    31,
                    Some(after_content),
                ),
            ],
        }
    }

    fn source_versions(before: &str, after: &str) -> SourceVersionHistory {
        SourceVersionHistory {
            schema_version: SOURCE_VERSION_HISTORY_SCHEMA_VERSION,
            project_filter: Some("prj-fixture".into()),
            workspace_filter: Some("wsp-fixture".into()),
            run_limit: 50,
            versions: vec![
                HistoricalSourceVersionRecord {
                    version_id: "version-before".into(),
                    run_id: "run-1".into(),
                    project_id: "prj-fixture".into(),
                    workspace_id: "wsp-fixture".into(),
                    source_stable_id: "src-v1-fixture".into(),
                    boundary: SourceVersionBoundary::Before,
                    version_fingerprint: before.into(),
                    snapshot_observed_at: None,
                    bytes: 10,
                    run_created_at_unix: 1,
                },
                HistoricalSourceVersionRecord {
                    version_id: "version-after".into(),
                    run_id: "run-1".into(),
                    project_id: "prj-fixture".into(),
                    workspace_id: "wsp-fixture".into(),
                    source_stable_id: "src-v1-fixture".into(),
                    boundary: SourceVersionBoundary::After,
                    version_fingerprint: after.into(),
                    snapshot_observed_at: None,
                    bytes: 11,
                    run_created_at_unix: 1,
                },
            ],
        }
    }

    fn git_history(
        before_head: Option<&str>,
        after_head: Option<&str>,
        coverage: WorkspaceGitProvenanceCoverage,
    ) -> WorkspaceGitProvenanceHistory {
        let make = |boundary, id: &str, head: Option<&str>, dirty| {
            HistoricalWorkspaceGitProvenanceRecord {
                provenance_id: id.into(),
                run_id: "run-1".into(),
                project_id: "prj-fixture".into(),
                workspace_id: "wsp-fixture".into(),
                boundary,
                coverage,
                head_fingerprint: head.map(str::to_string),
                dirty,
                snapshot_observed_at: None,
                run_created_at_unix: 1,
            }
        };
        WorkspaceGitProvenanceHistory {
            schema_version: WORKSPACE_GIT_PROVENANCE_HISTORY_SCHEMA_VERSION,
            project_filter: Some("prj-fixture".into()),
            workspace_filter: Some("wsp-fixture".into()),
            run_limit: 50,
            observations: vec![
                make(
                    SourceVersionBoundary::Before,
                    "git-before",
                    before_head,
                    Some(false),
                ),
                make(
                    SourceVersionBoundary::After,
                    "git-after",
                    after_head,
                    Some(true),
                ),
            ],
        }
    }

    #[test]
    fn change_reread_and_boundary_change_are_reported_without_freshness_claim() {
        let report = build_source_freshness_evidence_report(
            &tool_history("cnt-v1-after"),
            &source_versions("ver-v1-before", "ver-v1-after"),
            &git_history(
                Some("git-v1-head"),
                Some("git-v1-head"),
                WorkspaceGitProvenanceCoverage::Observed,
            ),
        )
        .expect("report");

        let observation = &report.observations[0];
        assert_eq!(
            observation.evidence_observation,
            FreshnessEvidenceObservation::ChangeAndRereadWithBoundaryChangeObserved
        );
        assert_eq!(
            observation.workspace_git_boundary_comparison,
            WorkspaceGitBoundaryComparison::HeadUnchangedObserved
        );
        assert_eq!(
            observation.freshness_status,
            FreshnessInferenceStatus::NotProven
        );
        assert_eq!(
            observation.invalidation_status,
            FreshnessInferenceStatus::NotProven
        );
    }

    #[test]
    fn in_run_change_remains_observed_when_run_boundaries_match() {
        let report = build_source_freshness_evidence_report(
            &tool_history("cnt-v1-after"),
            &source_versions("ver-v1-same", "ver-v1-same"),
            &git_history(
                Some("git-v1-before"),
                Some("git-v1-after"),
                WorkspaceGitProvenanceCoverage::Observed,
            ),
        )
        .expect("report");

        let observation = &report.observations[0];
        assert_eq!(
            observation.evidence_observation,
            FreshnessEvidenceObservation::ChangeAndRereadObserved
        );
        assert_eq!(
            observation.source_boundary_comparison,
            SourceBoundaryComparison::UnchangedObserved
        );
        assert_eq!(
            observation.workspace_git_boundary_comparison,
            WorkspaceGitBoundaryComparison::HeadChangedObserved
        );
    }

    #[test]
    fn same_content_reread_is_not_promoted_to_fresh() {
        let report = build_source_freshness_evidence_report(
            &tool_history("cnt-v1-before"),
            &source_versions("ver-v1-before", "ver-v1-after"),
            &git_history(
                Some("git-v1-head"),
                Some("git-v1-head"),
                WorkspaceGitProvenanceCoverage::Observed,
            ),
        )
        .expect("report");

        assert_eq!(
            report.observations[0].evidence_observation,
            FreshnessEvidenceObservation::SameContentRereadObserved
        );
        assert_eq!(
            report.observations[0].freshness_status,
            FreshnessInferenceStatus::NotProven
        );
    }

    #[test]
    fn unknown_git_coverage_stays_unknown_without_erasing_read_evidence() {
        let report = build_source_freshness_evidence_report(
            &tool_history("cnt-v1-after"),
            &source_versions("ver-v1-before", "ver-v1-after"),
            &git_history(None, None, WorkspaceGitProvenanceCoverage::Unknown),
        )
        .expect("report");

        assert_eq!(
            report.observations[0].evidence_observation,
            FreshnessEvidenceObservation::ChangeAndRereadWithBoundaryChangeObserved
        );
        assert_eq!(
            report.observations[0].workspace_git_boundary_comparison,
            WorkspaceGitBoundaryComparison::Unknown
        );
        assert_eq!(report.observations[0].git_before_dirty, None);
        assert_eq!(report.observations[0].git_after_dirty, None);
    }

    #[test]
    fn missing_source_boundary_keeps_only_in_run_change_evidence() {
        let tools = tool_history("cnt-v1-after");
        let mut versions = source_versions("ver-v1-before", "ver-v1-after");
        versions.versions.clear();
        let report = build_source_freshness_evidence_report(
            &tools,
            &versions,
            &git_history(
                Some("git-v1-head"),
                Some("git-v1-head"),
                WorkspaceGitProvenanceCoverage::Observed,
            ),
        )
        .expect("report");

        assert_eq!(
            report.observations[0].evidence_observation,
            FreshnessEvidenceObservation::ChangeAndRereadObserved
        );
        assert_eq!(
            report.observations[0].source_boundary_comparison,
            SourceBoundaryComparison::Unknown
        );
        assert_eq!(
            report.observations[0].freshness_status,
            FreshnessInferenceStatus::NotProven
        );
    }

    #[test]
    fn unsupported_git_history_schema_fails_closed() {
        let tools = tool_history("cnt-v1-after");
        let versions = source_versions("ver-v1-before", "ver-v1-after");
        let mut git = git_history(
            Some("git-v1-head"),
            Some("git-v1-head"),
            WorkspaceGitProvenanceCoverage::Observed,
        );
        git.schema_version = 99;
        let error = build_source_freshness_evidence_report(&tools, &versions, &git)
            .expect_err("future Git history schema");
        assert!(matches!(
            error,
            SourceFreshnessEvidenceBuildError::UnsupportedWorkspaceGitProvenanceHistorySchema {
                actual: 99,
                ..
            }
        ));
    }

    #[test]
    fn mismatched_input_scope_fails_closed() {
        let tools = tool_history("cnt-v1-after");
        let mut versions = source_versions("ver-v1-before", "ver-v1-after");
        versions.run_limit = 25;
        let error = build_source_freshness_evidence_report(
            &tools,
            &versions,
            &git_history(
                Some("git-v1-head"),
                Some("git-v1-head"),
                WorkspaceGitProvenanceCoverage::Observed,
            ),
        )
        .expect_err("scope mismatch");
        assert_eq!(error, SourceFreshnessEvidenceBuildError::InputScopeMismatch);
    }

    #[test]
    fn duplicate_git_boundary_fails_closed() {
        let tools = tool_history("cnt-v1-after");
        let versions = source_versions("ver-v1-before", "ver-v1-after");
        let mut git = git_history(
            Some("git-v1-head"),
            Some("git-v1-head"),
            WorkspaceGitProvenanceCoverage::Observed,
        );
        git.observations.push(git.observations[0].clone());
        let error = build_source_freshness_evidence_report(&tools, &versions, &git)
            .expect_err("duplicate Git boundary");
        assert!(matches!(
            error,
            SourceFreshnessEvidenceBuildError::DuplicateWorkspaceGitBoundary { .. }
        ));
    }
}
