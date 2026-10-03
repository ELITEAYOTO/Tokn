use std::path::Path;

use anyhow::Context;
use tokn_analysis::build_cross_agent_evidence;
use tokn_storage::Database;

use super::common::{open_db, validate_private_filter_id};

const MAX_HISTORY_RUNS: usize = 1_000;

pub fn run(
    project_id: Option<&str>,
    workspace_id: Option<&str>,
    limit: usize,
    db_override: Option<&Path>,
    output_json: Option<&Path>,
) -> anyhow::Result<()> {
    validate_private_filter_id(project_id, "project", "prj-")?;
    validate_private_filter_id(workspace_id, "workspace", "wsp-")?;
    if limit == 0 || limit > MAX_HISTORY_RUNS {
        anyhow::bail!("limit must be between 1 and {MAX_HISTORY_RUNS}");
    }

    let db = match db_override {
        Some(path) => Database::open(path)?,
        None => open_db()?,
    };

    let snapshot = db.historical_snapshot(project_id, workspace_id, limit)?;
    let activity_history = db.tool_activity_history(project_id, workspace_id, limit)?;
    let report = build_cross_agent_evidence(&snapshot, &activity_history)?;
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
    use tokn_domain::TOOL_ACTIVITY_HISTORY_SCHEMA_VERSION;

    #[test]
    fn empty_store_emits_empty_cross_agent_report() {
        let path = std::env::temp_dir().join(format!(
            "tokn-cross-agent-evidence-empty-{}.sqlite3",
            std::process::id()
        ));
        let output = std::env::temp_dir().join(format!(
            "tokn-cross-agent-evidence-empty-{}.json",
            std::process::id()
        ));
        let _ = std::fs::remove_file(&path);
        let _ = std::fs::remove_file(&output);

        run(None, None, 50, Some(&path), Some(&output)).expect("cross-agent evidence");
        let value: serde_json::Value =
            serde_json::from_slice(&std::fs::read(&output).expect("read output"))
                .expect("parse output");

        assert_eq!(value["schema_version"].as_u64(), Some(2));
        assert_eq!(value["source_snapshot_schema_version"].as_u64(), Some(1));
        assert_eq!(
            value["source_activity_history_schema_version"].as_u64(),
            Some(TOOL_ACTIVITY_HISTORY_SCHEMA_VERSION)
        );
        assert_eq!(value["overlaps"].as_array().map(Vec::len), Some(0));

        let _ = std::fs::remove_file(path);
        let _ = std::fs::remove_file(output);
    }

    #[test]
    fn rejects_invalid_limit_before_opening_store() {
        assert!(run(None, None, 0, None, None).is_err());
        assert!(run(None, None, MAX_HISTORY_RUNS + 1, None, None).is_err());
    }
}
