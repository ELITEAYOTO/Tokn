use std::path::Path;

use anyhow::Context;
use tokn_analysis::build_context_ledger;
use tokn_storage::Database;

use super::common::open_db;

const MAX_HISTORY_RUNS: usize = 1_000;

pub fn run(
    project_id: Option<&str>,
    workspace_id: Option<&str>,
    limit: usize,
    db_override: Option<&Path>,
    output_json: Option<&Path>,
) -> anyhow::Result<()> {
    validate_filter_id(project_id, "project", "prj-")?;
    validate_filter_id(workspace_id, "workspace", "wsp-")?;
    if limit == 0 || limit > MAX_HISTORY_RUNS {
        anyhow::bail!("limit must be between 1 and {MAX_HISTORY_RUNS}");
    }

    let db = match db_override {
        Some(path) => Database::open(path)?,
        None => open_db()?,
    };
    let snapshot = db.historical_snapshot(project_id, workspace_id, limit)?;
    let ledger = build_context_ledger(&snapshot)?;
    let json = serde_json::to_string_pretty(&ledger)?;

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

fn validate_filter_id(
    value: Option<&str>,
    label: &str,
    expected_prefix: &str,
) -> anyhow::Result<()> {
    let Some(value) = value else {
        return Ok(());
    };

    if value.trim() != value || value.is_empty() {
        anyhow::bail!("{label} id must be a non-empty canonical Store id");
    }
    let Some(hex) = value.strip_prefix(expected_prefix) else {
        anyhow::bail!(
            "{label} id must be a privacy-preserving Store id beginning with {expected_prefix}"
        );
    };
    if hex.len() != 24 || !hex.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        anyhow::bail!(
            "{label} id must contain exactly 24 hexadecimal characters after {expected_prefix}"
        );
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_canonical_private_ids() {
        assert!(
            validate_filter_id(Some("prj-0123456789abcdef01234567"), "project", "prj-").is_ok()
        );
        assert!(
            validate_filter_id(Some("wsp-abcdef0123456789abcdef01"), "workspace", "wsp-").is_ok()
        );
    }

    #[test]
    fn rejects_raw_paths_and_malformed_ids() {
        let raw = format!("C:\\{}\\someone\\project", "Users");
        assert!(validate_filter_id(Some(&raw), "project", "prj-").is_err());
        assert!(validate_filter_id(Some("prj-short"), "project", "prj-").is_err());
        assert!(
            validate_filter_id(Some("wsp-0123456789abcdef0123456z"), "workspace", "wsp-").is_err()
        );
    }

    #[test]
    fn empty_store_emits_explicit_unknown_and_not_captured_statuses() {
        let path = std::env::temp_dir().join(format!(
            "tokn-context-ledger-empty-{}.sqlite3",
            std::process::id()
        ));
        let output = std::env::temp_dir().join(format!(
            "tokn-context-ledger-empty-{}.json",
            std::process::id()
        ));
        let _ = std::fs::remove_file(&path);
        let _ = std::fs::remove_file(&output);

        run(None, None, 50, Some(&path), Some(&output)).expect("context ledger");
        let value: serde_json::Value =
            serde_json::from_slice(&std::fs::read(&output).expect("read output"))
                .expect("parse output");

        assert_eq!(value["schema_version"].as_u64(), Some(1));
        assert_eq!(
            value["turn_granularity_status"].as_str(),
            Some("NOT_CAPTURED")
        );
        assert_eq!(
            value["current_retained_context_status"].as_str(),
            Some("UNKNOWN")
        );
        assert_eq!(value["runs"].as_array().map(Vec::len), Some(0));

        let _ = std::fs::remove_file(path);
        let _ = std::fs::remove_file(output);
    }
}
