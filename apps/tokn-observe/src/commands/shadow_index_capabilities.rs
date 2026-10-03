use std::path::Path;

use anyhow::Context;
use tokn_shadow::probe_sqlite_fts5_capabilities;

pub fn run(output_json: Option<&Path>) -> anyhow::Result<()> {
    let report = probe_sqlite_fts5_capabilities();
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
    use tokn_shadow::{
        DIRECT_SCAN_FALLBACK_BACKEND_ID, SHADOW_SQLITE_CAPABILITY_SCHEMA_VERSION,
        ShadowCapabilityState,
    };

    #[test]
    fn capability_command_emits_runtime_observed_report() {
        let output = std::env::temp_dir().join(format!(
            "tokn-shadow-index-capabilities-{}.json",
            std::process::id()
        ));
        let _ = std::fs::remove_file(&output);

        run(Some(&output)).expect("shadow index capability probe");
        let report: serde_json::Value =
            serde_json::from_slice(&std::fs::read(&output).expect("read output"))
                .expect("parse output");

        assert_eq!(
            report["schema_version"].as_u64(),
            Some(SHADOW_SQLITE_CAPABILITY_SCHEMA_VERSION)
        );
        assert_eq!(
            report["fallback_backend_id"].as_str(),
            Some(DIRECT_SCAN_FALLBACK_BACKEND_ID)
        );
        assert_eq!(report["extension_loading_attempted"].as_bool(), Some(false));
        assert_eq!(report["fts5"].as_str(), Some("OBSERVED_AVAILABLE"));
        assert_eq!(report["trigram"].as_str(), Some("OBSERVED_AVAILABLE"));
        assert_eq!(report["unicode61"].as_str(), Some("OBSERVED_AVAILABLE"));

        let _ = std::fs::remove_file(output);
    }

    #[test]
    fn current_bundled_build_keeps_direct_scan_as_fallback() {
        let report = probe_sqlite_fts5_capabilities();
        assert_eq!(report.fts5, ShadowCapabilityState::ObservedAvailable);
        assert_eq!(report.fallback_backend_id, DIRECT_SCAN_FALLBACK_BACKEND_ID);
        assert!(!report.extension_loading_attempted);
    }
}
