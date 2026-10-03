use std::collections::BTreeMap;
use std::fmt;

use serde::{Deserialize, Serialize};
use tokn_domain::{
    SOURCE_VERSION_HISTORY_SCHEMA_VERSION, SourceVersionBoundary, SourceVersionHistory,
};

pub const SOURCE_BOUNDARY_VERSION_REPORT_SCHEMA_VERSION: u64 = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SourceBoundaryComparison {
    UnchangedObserved,
    ChangedObserved,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceBoundaryVersionObservation {
    pub run_id: String,
    pub project_id: String,
    pub workspace_id: String,
    pub source_stable_id: String,
    pub before_version_fingerprint: Option<String>,
    pub after_version_fingerprint: Option<String>,
    pub comparison: SourceBoundaryComparison,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceBoundaryVersionReport {
    pub schema_version: u64,
    pub source_version_history_schema_version: u64,
    pub project_filter: Option<String>,
    pub workspace_filter: Option<String>,
    pub run_limit: u64,
    pub sources: Vec<SourceBoundaryVersionObservation>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SourceBoundaryVersionBuildError {
    UnsupportedSourceVersionHistorySchema { actual: u64, expected: u64 },
    DuplicateBoundary {
        run_id: String,
        source_stable_id: String,
        boundary: SourceVersionBoundary,
    },
}

impl fmt::Display for SourceBoundaryVersionBuildError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedSourceVersionHistorySchema { actual, expected } => write!(
                formatter,
                "unsupported SourceVersionHistory schema_version {actual}; expected {expected}"
            ),
            Self::DuplicateBoundary {
                run_id,
                source_stable_id,
                boundary,
            } => write!(
                formatter,
                "duplicate {} source-version boundary for run {run_id} source {source_stable_id}",
                boundary.as_str()
            ),
        }
    }
}

impl std::error::Error for SourceBoundaryVersionBuildError {}

#[derive(Default)]
struct BoundaryPair {
    before: Option<String>,
    after: Option<String>,
}

pub fn build_source_boundary_version_report(
    history: &SourceVersionHistory,
) -> Result<SourceBoundaryVersionReport, SourceBoundaryVersionBuildError> {
    if history.schema_version != SOURCE_VERSION_HISTORY_SCHEMA_VERSION {
        return Err(
            SourceBoundaryVersionBuildError::UnsupportedSourceVersionHistorySchema {
                actual: history.schema_version,
                expected: SOURCE_VERSION_HISTORY_SCHEMA_VERSION,
            },
        );
    }

    let mut grouped = BTreeMap::<(String, String, String, String), BoundaryPair>::new();
    for version in &history.versions {
        let key = (
            version.run_id.clone(),
            version.project_id.clone(),
            version.workspace_id.clone(),
            version.source_stable_id.clone(),
        );
        let pair = grouped.entry(key).or_default();
        let slot = match version.boundary {
            SourceVersionBoundary::Before => &mut pair.before,
            SourceVersionBoundary::After => &mut pair.after,
        };
        if slot.is_some() {
            return Err(SourceBoundaryVersionBuildError::DuplicateBoundary {
                run_id: version.run_id.clone(),
                source_stable_id: version.source_stable_id.clone(),
                boundary: version.boundary,
            });
        }
        *slot = Some(version.version_fingerprint.clone());
    }

    let sources = grouped
        .into_iter()
        .map(
            |((run_id, project_id, workspace_id, source_stable_id), pair)| {
                let comparison = match (&pair.before, &pair.after) {
                    (Some(before), Some(after)) if before == after => {
                        SourceBoundaryComparison::UnchangedObserved
                    }
                    (Some(_), Some(_)) => SourceBoundaryComparison::ChangedObserved,
                    _ => SourceBoundaryComparison::Unknown,
                };
                SourceBoundaryVersionObservation {
                    run_id,
                    project_id,
                    workspace_id,
                    source_stable_id,
                    before_version_fingerprint: pair.before,
                    after_version_fingerprint: pair.after,
                    comparison,
                }
            },
        )
        .collect();

    Ok(SourceBoundaryVersionReport {
        schema_version: SOURCE_BOUNDARY_VERSION_REPORT_SCHEMA_VERSION,
        source_version_history_schema_version: history.schema_version,
        project_filter: history.project_filter.clone(),
        workspace_filter: history.workspace_filter.clone(),
        run_limit: history.run_limit,
        sources,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokn_domain::HistoricalSourceVersionRecord;

    fn version(boundary: SourceVersionBoundary, fingerprint: &str) -> HistoricalSourceVersionRecord {
        HistoricalSourceVersionRecord {
            version_id: format!("svr-{}", boundary.as_str()),
            run_id: "run-1".into(),
            project_id: "prj-fixture".into(),
            workspace_id: "wsp-fixture".into(),
            source_stable_id: "src-v1-fixture".into(),
            boundary,
            version_fingerprint: fingerprint.into(),
            snapshot_observed_at: None,
            bytes: 10,
            run_created_at_unix: 1,
        }
    }

    fn history(versions: Vec<HistoricalSourceVersionRecord>) -> SourceVersionHistory {
        SourceVersionHistory {
            schema_version: SOURCE_VERSION_HISTORY_SCHEMA_VERSION,
            project_filter: Some("prj-fixture".into()),
            workspace_filter: None,
            run_limit: 50,
            versions,
        }
    }

    #[test]
    fn identical_boundaries_are_unchanged_observed() {
        let report = build_source_boundary_version_report(&history(vec![
            version(SourceVersionBoundary::Before, "ver-v1-same"),
            version(SourceVersionBoundary::After, "ver-v1-same"),
        ]))
        .expect("report");
        assert_eq!(report.sources.len(), 1);
        assert_eq!(
            report.sources[0].comparison,
            SourceBoundaryComparison::UnchangedObserved
        );
    }

    #[test]
    fn different_boundaries_are_changed_observed() {
        let report = build_source_boundary_version_report(&history(vec![
            version(SourceVersionBoundary::Before, "ver-v1-before"),
            version(SourceVersionBoundary::After, "ver-v1-after"),
        ]))
        .expect("report");
        assert_eq!(
            report.sources[0].comparison,
            SourceBoundaryComparison::ChangedObserved
        );
    }

    #[test]
    fn missing_boundary_stays_unknown() {
        let report = build_source_boundary_version_report(&history(vec![version(
            SourceVersionBoundary::After,
            "ver-v1-after",
        )]))
        .expect("report");
        assert_eq!(report.sources[0].comparison, SourceBoundaryComparison::Unknown);
    }

    #[test]
    fn duplicate_boundary_fails_closed() {
        let error = build_source_boundary_version_report(&history(vec![
            version(SourceVersionBoundary::Before, "ver-v1-a"),
            version(SourceVersionBoundary::Before, "ver-v1-b"),
        ]))
        .expect_err("duplicate must fail");
        assert!(matches!(
            error,
            SourceBoundaryVersionBuildError::DuplicateBoundary { .. }
        ));
    }
}
