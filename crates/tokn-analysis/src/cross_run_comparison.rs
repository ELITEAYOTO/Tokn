use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use serde::{Deserialize, Serialize};
use tokn_domain::{
    EVIDENCE_LAYOUT_VERSION, HISTORICAL_SNAPSHOT_SCHEMA_VERSION, HistoricalRunRecord,
    HistoricalSnapshot, MEASUREMENT_CONTRACT_VERSION, SOURCE_VERSION_HISTORY_SCHEMA_VERSION,
    SourceVersionBoundary, SourceVersionHistory, ValidityCheckStatus,
    WORKSPACE_GIT_PROVENANCE_HISTORY_SCHEMA_VERSION, WorkspaceGitProvenanceCoverage,
    WorkspaceGitProvenanceHistory,
};

use crate::{RuntimeProfileCompatibilityReport, reduce_runtime_profile_compatibility};

pub const CROSS_RUN_COMPARISON_SCHEMA_VERSION: u64 = 1;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum CrossRunCausalClaimsStatus {
    #[default]
    NotEstablished,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum CrossRunSourceObservation {
    SameVersionObserved,
    DifferentVersionObserved,
    BaselineOnlyObserved,
    CandidateOnlyObserved,
    #[default]
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CrossRunSourceComparison {
    pub source_stable_id: String,
    pub baseline_before_version: Option<String>,
    pub candidate_before_version: Option<String>,
    pub observation: CrossRunSourceObservation,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CrossRunCheck {
    pub code: String,
    pub status: ValidityCheckStatus,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CrossRunComparisonReport {
    pub schema_version: u64,
    pub baseline_run_id: String,
    pub candidate_run_id: String,
    pub observed_scope_status: ValidityCheckStatus,
    pub observed_scope_compatible: bool,
    pub causal_claims_status: CrossRunCausalClaimsStatus,
    pub checks: Vec<CrossRunCheck>,
    pub runtime_profile: RuntimeProfileCompatibilityReport,
    pub sources_before: Vec<CrossRunSourceComparison>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CrossRunComparisonBuildError {
    UnsupportedHistoricalSnapshotSchema {
        actual: u64,
        expected: u64,
    },
    UnsupportedSourceVersionHistorySchema {
        actual: u64,
        expected: u64,
    },
    UnsupportedWorkspaceGitHistorySchema {
        actual: u64,
        expected: u64,
    },
    InputScopeMismatch,
    SameRunRequested,
    RunNotFound {
        run_id: String,
    },
    DuplicateSourceBefore {
        run_id: String,
        source_stable_id: String,
    },
    DuplicateGitBefore {
        run_id: String,
    },
}

impl fmt::Display for CrossRunComparisonBuildError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedHistoricalSnapshotSchema { actual, expected } => write!(
                formatter,
                "unsupported HistoricalSnapshot schema_version {actual}; expected {expected}"
            ),
            Self::UnsupportedSourceVersionHistorySchema { actual, expected } => write!(
                formatter,
                "unsupported SourceVersionHistory schema_version {actual}; expected {expected}"
            ),
            Self::UnsupportedWorkspaceGitHistorySchema { actual, expected } => write!(
                formatter,
                "unsupported WorkspaceGitProvenanceHistory schema_version {actual}; expected {expected}"
            ),
            Self::InputScopeMismatch => write!(
                formatter,
                "cross-run comparison inputs must use identical project/workspace filters"
            ),
            Self::SameRunRequested => {
                write!(formatter, "baseline and candidate run ids must differ")
            }
            Self::RunNotFound { run_id } => write!(
                formatter,
                "run not found in selected history window: {run_id}"
            ),
            Self::DuplicateSourceBefore {
                run_id,
                source_stable_id,
            } => write!(
                formatter,
                "duplicate BEFORE source version for run {run_id} source {source_stable_id}"
            ),
            Self::DuplicateGitBefore { run_id } => write!(
                formatter,
                "duplicate BEFORE workspace Git provenance for run {run_id}"
            ),
        }
    }
}

impl std::error::Error for CrossRunComparisonBuildError {}

pub fn build_cross_run_comparison(
    snapshot: &HistoricalSnapshot,
    source_versions: &SourceVersionHistory,
    workspace_git: &WorkspaceGitProvenanceHistory,
    baseline_run_id: &str,
    candidate_run_id: &str,
) -> Result<CrossRunComparisonReport, CrossRunComparisonBuildError> {
    validate_inputs(snapshot, source_versions, workspace_git)?;
    if baseline_run_id == candidate_run_id {
        return Err(CrossRunComparisonBuildError::SameRunRequested);
    }

    let baseline = find_run(snapshot, baseline_run_id)?;
    let candidate = find_run(snapshot, candidate_run_id)?;
    let baseline_profile = runtime_profile(snapshot, baseline);
    let candidate_profile = runtime_profile(snapshot, candidate);
    let runtime_profile = reduce_runtime_profile_compatibility(baseline_profile, candidate_profile);

    let mut checks = vec![
        equality_check(
            "PROJECT_SCOPE",
            &baseline.project_id,
            &candidate.project_id,
            "runs must belong to the same project scope",
        ),
        contract_check(
            "MEASUREMENT_CONTRACT",
            baseline.measurement_contract_version,
            candidate.measurement_contract_version,
            MEASUREMENT_CONTRACT_VERSION,
        ),
        contract_check(
            "EVIDENCE_LAYOUT",
            baseline.evidence_layout_version,
            candidate.evidence_layout_version,
            EVIDENCE_LAYOUT_VERSION,
        ),
        CrossRunCheck {
            code: "RUNTIME_PROFILE".into(),
            status: runtime_profile.status,
            message:
                "runtime/model/config comparability is delegated to RuntimeProfileCompatibility V1"
                    .into(),
        },
    ];

    let same_project = baseline.project_id == candidate.project_id;
    let (sources_before, source_status) = if same_project {
        compare_sources_before(source_versions, baseline_run_id, candidate_run_id)?
    } else {
        (Vec::new(), ValidityCheckStatus::Unknown)
    };
    checks.push(CrossRunCheck {
        code: "SOURCE_BEFORE_STATE".into(),
        status: source_status,
        message: "BEFORE source sets are comparable only within one project scope; PASS additionally requires non-empty identical source sets with equal exact versions".into(),
    });

    let git_status = if same_project {
        compare_git_before(workspace_git, baseline_run_id, candidate_run_id)?
    } else {
        ValidityCheckStatus::Unknown
    };
    checks.push(CrossRunCheck {
        code: "WORKSPACE_GIT_BEFORE".into(),
        status: git_status,
        message:
            "BEFORE Git state passes only for observed equal HEAD with both working trees clean"
                .into(),
    });

    let observed_scope_status = aggregate_status(&checks);
    Ok(CrossRunComparisonReport {
        schema_version: CROSS_RUN_COMPARISON_SCHEMA_VERSION,
        baseline_run_id: baseline_run_id.to_string(),
        candidate_run_id: candidate_run_id.to_string(),
        observed_scope_status,
        observed_scope_compatible: observed_scope_status == ValidityCheckStatus::Pass,
        causal_claims_status: CrossRunCausalClaimsStatus::NotEstablished,
        checks,
        runtime_profile,
        sources_before,
    })
}

fn validate_inputs(
    snapshot: &HistoricalSnapshot,
    source_versions: &SourceVersionHistory,
    workspace_git: &WorkspaceGitProvenanceHistory,
) -> Result<(), CrossRunComparisonBuildError> {
    if snapshot.schema_version != HISTORICAL_SNAPSHOT_SCHEMA_VERSION {
        return Err(
            CrossRunComparisonBuildError::UnsupportedHistoricalSnapshotSchema {
                actual: snapshot.schema_version,
                expected: HISTORICAL_SNAPSHOT_SCHEMA_VERSION,
            },
        );
    }
    if source_versions.schema_version != SOURCE_VERSION_HISTORY_SCHEMA_VERSION {
        return Err(
            CrossRunComparisonBuildError::UnsupportedSourceVersionHistorySchema {
                actual: source_versions.schema_version,
                expected: SOURCE_VERSION_HISTORY_SCHEMA_VERSION,
            },
        );
    }
    if workspace_git.schema_version != WORKSPACE_GIT_PROVENANCE_HISTORY_SCHEMA_VERSION {
        return Err(
            CrossRunComparisonBuildError::UnsupportedWorkspaceGitHistorySchema {
                actual: workspace_git.schema_version,
                expected: WORKSPACE_GIT_PROVENANCE_HISTORY_SCHEMA_VERSION,
            },
        );
    }
    let scope_matches = snapshot.project_filter == source_versions.project_filter
        && snapshot.project_filter == workspace_git.project_filter
        && snapshot.workspace_filter == source_versions.workspace_filter
        && snapshot.workspace_filter == workspace_git.workspace_filter;
    if !scope_matches {
        return Err(CrossRunComparisonBuildError::InputScopeMismatch);
    }
    Ok(())
}

fn find_run<'a>(
    snapshot: &'a HistoricalSnapshot,
    run_id: &str,
) -> Result<&'a HistoricalRunRecord, CrossRunComparisonBuildError> {
    snapshot
        .runs
        .iter()
        .find(|run| run.run_id == run_id)
        .ok_or_else(|| CrossRunComparisonBuildError::RunNotFound {
            run_id: run_id.to_string(),
        })
}

fn runtime_profile<'a>(
    snapshot: &'a HistoricalSnapshot,
    run: &HistoricalRunRecord,
) -> Option<&'a tokn_domain::ModelRuntimeProfile> {
    let profile_id = run.profile_id.as_deref()?;
    snapshot
        .runtime_profiles
        .iter()
        .find(|record| record.profile_id == profile_id)
        .map(|record| &record.profile)
}

fn equality_check(code: &str, baseline: &str, candidate: &str, message: &str) -> CrossRunCheck {
    CrossRunCheck {
        code: code.into(),
        status: if baseline == candidate {
            ValidityCheckStatus::Pass
        } else {
            ValidityCheckStatus::Fail
        },
        message: message.into(),
    }
}

fn contract_check(code: &str, baseline: u64, candidate: u64, supported: u64) -> CrossRunCheck {
    let status = if baseline != candidate {
        ValidityCheckStatus::Fail
    } else if baseline == supported {
        ValidityCheckStatus::Pass
    } else {
        ValidityCheckStatus::Unknown
    };
    CrossRunCheck {
        code: code.into(),
        status,
        message: format!("both runs must use the same supported version {supported}"),
    }
}

fn compare_sources_before(
    history: &SourceVersionHistory,
    baseline_run_id: &str,
    candidate_run_id: &str,
) -> Result<(Vec<CrossRunSourceComparison>, ValidityCheckStatus), CrossRunComparisonBuildError> {
    let baseline = source_before_map(history, baseline_run_id)?;
    let candidate = source_before_map(history, candidate_run_id)?;
    let ids = baseline
        .keys()
        .chain(candidate.keys())
        .cloned()
        .collect::<BTreeSet<_>>();

    let mut comparisons = Vec::with_capacity(ids.len());
    for source_stable_id in ids {
        let left = baseline.get(&source_stable_id).cloned();
        let right = candidate.get(&source_stable_id).cloned();
        let observation = match (&left, &right) {
            (Some(a), Some(b)) if a == b => CrossRunSourceObservation::SameVersionObserved,
            (Some(_), Some(_)) => CrossRunSourceObservation::DifferentVersionObserved,
            (Some(_), None) => CrossRunSourceObservation::BaselineOnlyObserved,
            (None, Some(_)) => CrossRunSourceObservation::CandidateOnlyObserved,
            (None, None) => CrossRunSourceObservation::Unknown,
        };
        comparisons.push(CrossRunSourceComparison {
            source_stable_id,
            baseline_before_version: left,
            candidate_before_version: right,
            observation,
        });
    }

    let status = if baseline.is_empty() || candidate.is_empty() {
        ValidityCheckStatus::Unknown
    } else if comparisons
        .iter()
        .any(|item| item.observation == CrossRunSourceObservation::DifferentVersionObserved)
    {
        ValidityCheckStatus::Fail
    } else if baseline.len() != candidate.len()
        || comparisons.iter().any(|item| {
            matches!(
                item.observation,
                CrossRunSourceObservation::BaselineOnlyObserved
                    | CrossRunSourceObservation::CandidateOnlyObserved
            )
        })
    {
        ValidityCheckStatus::Unknown
    } else {
        ValidityCheckStatus::Pass
    };
    Ok((comparisons, status))
}

fn source_before_map(
    history: &SourceVersionHistory,
    run_id: &str,
) -> Result<BTreeMap<String, String>, CrossRunComparisonBuildError> {
    let mut map = BTreeMap::new();
    for version in history.versions.iter().filter(|version| {
        version.run_id == run_id && version.boundary == SourceVersionBoundary::Before
    }) {
        if map
            .insert(
                version.source_stable_id.clone(),
                version.version_fingerprint.clone(),
            )
            .is_some()
        {
            return Err(CrossRunComparisonBuildError::DuplicateSourceBefore {
                run_id: run_id.to_string(),
                source_stable_id: version.source_stable_id.clone(),
            });
        }
    }
    Ok(map)
}

fn compare_git_before(
    history: &WorkspaceGitProvenanceHistory,
    baseline_run_id: &str,
    candidate_run_id: &str,
) -> Result<ValidityCheckStatus, CrossRunComparisonBuildError> {
    let baseline = git_before(history, baseline_run_id)?;
    let candidate = git_before(history, candidate_run_id)?;
    let (Some(left), Some(right)) = (baseline, candidate) else {
        return Ok(ValidityCheckStatus::Unknown);
    };
    if left.coverage != WorkspaceGitProvenanceCoverage::Observed
        || right.coverage != WorkspaceGitProvenanceCoverage::Observed
    {
        return Ok(ValidityCheckStatus::Unknown);
    }
    let (Some(left_head), Some(right_head)) = (&left.head_fingerprint, &right.head_fingerprint)
    else {
        return Ok(ValidityCheckStatus::Unknown);
    };
    if left_head != right_head {
        return Ok(ValidityCheckStatus::Fail);
    }
    match (left.dirty, right.dirty) {
        (Some(false), Some(false)) => Ok(ValidityCheckStatus::Pass),
        (Some(a), Some(b)) if a != b => Ok(ValidityCheckStatus::Fail),
        _ => Ok(ValidityCheckStatus::Unknown),
    }
}

fn git_before<'a>(
    history: &'a WorkspaceGitProvenanceHistory,
    run_id: &str,
) -> Result<
    Option<&'a tokn_domain::HistoricalWorkspaceGitProvenanceRecord>,
    CrossRunComparisonBuildError,
> {
    let mut matches = history.observations.iter().filter(|observation| {
        observation.run_id == run_id && observation.boundary == SourceVersionBoundary::Before
    });
    let first = matches.next();
    if matches.next().is_some() {
        return Err(CrossRunComparisonBuildError::DuplicateGitBefore {
            run_id: run_id.to_string(),
        });
    }
    Ok(first)
}

fn aggregate_status(checks: &[CrossRunCheck]) -> ValidityCheckStatus {
    if checks
        .iter()
        .any(|check| check.status == ValidityCheckStatus::Fail)
    {
        ValidityCheckStatus::Fail
    } else if checks
        .iter()
        .any(|check| check.status == ValidityCheckStatus::Unknown)
    {
        ValidityCheckStatus::Unknown
    } else {
        ValidityCheckStatus::Pass
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokn_domain::{
        HistoricalRuntimeProfileRecord, HistoricalSourceVersionRecord,
        HistoricalWorkspaceGitProvenanceRecord, ModelRuntimeProfile, RuntimeConfigurationProfile,
    };

    fn profile() -> ModelRuntimeProfile {
        ModelRuntimeProfile {
            schema_version: tokn_domain::MODEL_RUNTIME_PROFILE_SCHEMA_VERSION,
            observed_at: "2026-10-03T20:00:00+02:00".into(),
            runtime_kind: "codex".into(),
            runtime_version: Some("codex-cli fixture".into()),
            model: Some("fixture-model".into()),
            model_provider: Some("openai".into()),
            reasoning_effort: Some("high".into()),
            app_version: None,
            model_context_window: None,
            multi_agent_protocol_version: None,
            configuration: RuntimeConfigurationProfile {
                permission_profile: Some("workspace-write".into()),
                approval_policy: Some("on-request".into()),
                sandbox_policy: Some("workspace-write".into()),
                collaboration_mode: Some("multi-agent".into()),
                context_management_mode: Some("default".into()),
                ..Default::default()
            },
            configuration_complete: ValidityCheckStatus::Pass,
            capabilities: BTreeMap::new(),
        }
    }

    fn run(id: &str, project: &str, profile_id: &str) -> HistoricalRunRecord {
        HistoricalRunRecord {
            run_id: id.into(),
            project_id: project.into(),
            workspace_id: format!("wsp-{id}"),
            profile_id: Some(profile_id.into()),
            measurement_contract_version: MEASUREMENT_CONTRACT_VERSION,
            evidence_layout_version: EVIDENCE_LAYOUT_VERSION,
            ..Default::default()
        }
    }

    fn snapshot(project_candidate: &str) -> HistoricalSnapshot {
        HistoricalSnapshot {
            schema_version: HISTORICAL_SNAPSHOT_SCHEMA_VERSION,
            project_filter: Some("prj-fixture".into()),
            workspace_filter: None,
            runs: vec![
                run("baseline", "prj-fixture", "rtp-a"),
                run("candidate", project_candidate, "rtp-b"),
            ],
            runtime_profiles: vec![
                HistoricalRuntimeProfileRecord {
                    profile_id: "rtp-a".into(),
                    profile: profile(),
                },
                HistoricalRuntimeProfileRecord {
                    profile_id: "rtp-b".into(),
                    profile: profile(),
                },
            ],
            ..Default::default()
        }
    }

    fn versions(candidate_version: &str) -> SourceVersionHistory {
        SourceVersionHistory {
            schema_version: SOURCE_VERSION_HISTORY_SCHEMA_VERSION,
            project_filter: Some("prj-fixture".into()),
            workspace_filter: None,
            run_limit: 50,
            versions: vec![
                source_version("baseline", "src-v1-a", "ver-v1-a"),
                source_version("candidate", "src-v1-a", candidate_version),
            ],
        }
    }

    fn source_version(run_id: &str, source: &str, version: &str) -> HistoricalSourceVersionRecord {
        HistoricalSourceVersionRecord {
            version_id: format!("{run_id}-{source}"),
            run_id: run_id.into(),
            project_id: "prj-fixture".into(),
            workspace_id: format!("wsp-{run_id}"),
            source_stable_id: source.into(),
            boundary: SourceVersionBoundary::Before,
            version_fingerprint: version.into(),
            snapshot_observed_at: None,
            bytes: 10,
            run_created_at_unix: 1,
        }
    }

    fn git(
        candidate_head: &str,
        baseline_dirty: Option<bool>,
        candidate_dirty: Option<bool>,
    ) -> WorkspaceGitProvenanceHistory {
        WorkspaceGitProvenanceHistory {
            schema_version: WORKSPACE_GIT_PROVENANCE_HISTORY_SCHEMA_VERSION,
            project_filter: Some("prj-fixture".into()),
            workspace_filter: None,
            run_limit: 50,
            observations: vec![
                git_record("baseline", "git-v1-head", baseline_dirty),
                git_record("candidate", candidate_head, candidate_dirty),
            ],
        }
    }

    fn git_record(
        run_id: &str,
        head: &str,
        dirty: Option<bool>,
    ) -> HistoricalWorkspaceGitProvenanceRecord {
        HistoricalWorkspaceGitProvenanceRecord {
            provenance_id: format!("git-{run_id}"),
            run_id: run_id.into(),
            project_id: "prj-fixture".into(),
            workspace_id: format!("wsp-{run_id}"),
            boundary: SourceVersionBoundary::Before,
            coverage: WorkspaceGitProvenanceCoverage::Observed,
            head_fingerprint: Some(head.into()),
            dirty,
            snapshot_observed_at: None,
            run_created_at_unix: 1,
        }
    }

    #[test]
    fn identical_observed_starting_state_passes_scope_but_not_causality() {
        let report = build_cross_run_comparison(
            &snapshot("prj-fixture"),
            &versions("ver-v1-a"),
            &git("git-v1-head", Some(false), Some(false)),
            "baseline",
            "candidate",
        )
        .expect("report");

        assert_eq!(report.observed_scope_status, ValidityCheckStatus::Pass);
        assert!(report.observed_scope_compatible);
        assert_eq!(
            report.causal_claims_status,
            CrossRunCausalClaimsStatus::NotEstablished
        );
    }

    #[test]
    fn different_project_fails_without_comparing_project_scoped_fingerprints() {
        let report = build_cross_run_comparison(
            &snapshot("prj-other"),
            &versions("ver-v1-different"),
            &git("git-v1-other", Some(false), Some(false)),
            "baseline",
            "candidate",
        )
        .expect("report");

        assert_eq!(report.observed_scope_status, ValidityCheckStatus::Fail);
        assert!(!report.observed_scope_compatible);
        assert!(report.sources_before.is_empty());
        assert_eq!(
            report
                .checks
                .iter()
                .find(|check| check.code == "SOURCE_BEFORE_STATE")
                .expect("source check")
                .status,
            ValidityCheckStatus::Unknown
        );
        assert_eq!(
            report
                .checks
                .iter()
                .find(|check| check.code == "WORKSPACE_GIT_BEFORE")
                .expect("git check")
                .status,
            ValidityCheckStatus::Unknown
        );
    }

    #[test]
    fn different_source_version_fails_within_same_project() {
        let report = build_cross_run_comparison(
            &snapshot("prj-fixture"),
            &versions("ver-v1-different"),
            &git("git-v1-head", Some(false), Some(false)),
            "baseline",
            "candidate",
        )
        .expect("report");

        assert_eq!(report.observed_scope_status, ValidityCheckStatus::Fail);
        assert_eq!(
            report.sources_before[0].observation,
            CrossRunSourceObservation::DifferentVersionObserved
        );
    }

    #[test]
    fn asymmetric_source_set_is_unknown_not_fail() {
        let mut history = versions("ver-v1-a");
        history
            .versions
            .push(source_version("baseline", "src-v1-extra", "ver-v1-extra"));
        let report = build_cross_run_comparison(
            &snapshot("prj-fixture"),
            &history,
            &git("git-v1-head", Some(false), Some(false)),
            "baseline",
            "candidate",
        )
        .expect("report");

        assert_eq!(report.observed_scope_status, ValidityCheckStatus::Unknown);
    }

    #[test]
    fn same_git_head_with_dirty_worktrees_is_unknown() {
        let report = build_cross_run_comparison(
            &snapshot("prj-fixture"),
            &versions("ver-v1-a"),
            &git("git-v1-head", Some(true), Some(true)),
            "baseline",
            "candidate",
        )
        .expect("report");

        let git_check = report
            .checks
            .iter()
            .find(|check| check.code == "WORKSPACE_GIT_BEFORE")
            .expect("git check");
        assert_eq!(git_check.status, ValidityCheckStatus::Unknown);
    }

    #[test]
    fn changed_git_head_fails_observed_scope() {
        let report = build_cross_run_comparison(
            &snapshot("prj-fixture"),
            &versions("ver-v1-a"),
            &git("git-v1-other", Some(false), Some(false)),
            "baseline",
            "candidate",
        )
        .expect("report");
        assert_eq!(report.observed_scope_status, ValidityCheckStatus::Fail);
    }

    #[test]
    fn same_run_and_missing_run_are_rejected() {
        let snapshot = snapshot("prj-fixture");
        let versions = versions("ver-v1-a");
        let git = git("git-v1-head", Some(false), Some(false));
        assert_eq!(
            build_cross_run_comparison(&snapshot, &versions, &git, "baseline", "baseline")
                .expect_err("same run"),
            CrossRunComparisonBuildError::SameRunRequested
        );
        assert!(matches!(
            build_cross_run_comparison(&snapshot, &versions, &git, "baseline", "missing"),
            Err(CrossRunComparisonBuildError::RunNotFound { .. })
        ));
    }

    #[test]
    fn future_schema_and_scope_mismatch_fail_closed() {
        let mut unsupported_snapshot = snapshot("prj-fixture");
        unsupported_snapshot.schema_version = 99;
        assert!(matches!(
            build_cross_run_comparison(
                &unsupported_snapshot,
                &versions("ver-v1-a"),
                &git("git-v1-head", Some(false), Some(false)),
                "baseline",
                "candidate"
            ),
            Err(CrossRunComparisonBuildError::UnsupportedHistoricalSnapshotSchema { .. })
        ));

        let snapshot = snapshot("prj-fixture");
        let mut versions = versions("ver-v1-a");
        versions.project_filter = None;
        assert_eq!(
            build_cross_run_comparison(
                &snapshot,
                &versions,
                &git("git-v1-head", Some(false), Some(false)),
                "baseline",
                "candidate"
            )
            .expect_err("scope mismatch"),
            CrossRunComparisonBuildError::InputScopeMismatch
        );
    }
}
