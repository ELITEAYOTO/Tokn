use std::path::Path;

use anyhow::Context;
use tokn_analysis::build_cross_run_comparison;
use tokn_storage::Database;

use super::common::{open_db, validate_private_filter_id};

const MAX_HISTORY_RUNS: usize = 1_000;

pub fn run(
    baseline_run_id: &str,
    candidate_run_id: &str,
    project_id: Option<&str>,
    limit: usize,
    db_override: Option<&Path>,
    output_json: Option<&Path>,
) -> anyhow::Result<()> {
    validate_private_filter_id(project_id, "project", "prj-")?;
    if baseline_run_id.trim().is_empty() || candidate_run_id.trim().is_empty() {
        anyhow::bail!("baseline and candidate run ids must be non-empty");
    }
    if limit == 0 || limit > MAX_HISTORY_RUNS {
        anyhow::bail!("limit must be between 1 and {MAX_HISTORY_RUNS}");
    }

    let db = match db_override {
        Some(path) => Database::open(path)?,
        None => open_db()?,
    };
    let snapshot = db.historical_snapshot(project_id, None, limit)?;
    let source_versions = db.source_version_history(project_id, None, limit)?;
    let workspace_git = db.workspace_git_provenance_history(project_id, None, limit)?;
    let report = build_cross_run_comparison(
        &snapshot,
        &source_versions,
        &workspace_git,
        baseline_run_id,
        candidate_run_id,
    )?;
    let json = serde_json::to_string_pretty(&report)?;

    if let Some(path) = output_json {
        if let Some(parent) = path.parent()
            && !parent.as_os_str().is_empty()
        {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("create output directory {}", parent.display()))?;
        }
        std::fs::write(path, format!("{json}\n"))
            .with_context(|| format!("write {}", path.display()))?;
    } else {
        println!("{json}");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_invalid_arguments_before_store_lookup() {
        assert!(run("", "candidate", None, 50, None, None).is_err());
        assert!(run("baseline", "candidate", None, 0, None, None).is_err());
        assert!(
            run(
                "baseline",
                "candidate",
                None,
                MAX_HISTORY_RUNS + 1,
                None,
                None
            )
            .is_err()
        );
    }

    #[test]
    fn empty_store_reports_missing_run() {
        let path = std::env::temp_dir().join(format!(
            "tokn-cross-run-comparison-empty-{}.sqlite3",
            std::process::id()
        ));
        let _ = std::fs::remove_file(&path);
        let result = run("baseline", "candidate", None, 50, Some(&path), None);
        assert!(result.is_err());
        let _ = std::fs::remove_file(path);
    }
}
