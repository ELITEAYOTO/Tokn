use std::path::Path;

use anyhow::{Context, bail};
use tokn_shadow::{
    ShadowDecisionVerdict, ShadowIndexPilotConfig, run_sanitized_shadow_index_pilot,
};

pub fn run(
    config_path: &Path,
    output_dir: &Path,
    observed_at: &str,
    tokn_commit: &str,
) -> anyhow::Result<()> {
    let raw = std::fs::read(config_path)
        .with_context(|| format!("read shadow index pilot config {}", config_path.display()))?;
    let config: ShadowIndexPilotConfig = serde_json::from_slice(&raw)
        .with_context(|| format!("parse shadow index pilot config {}", config_path.display()))?;
    let pair = run_sanitized_shadow_index_pilot(
        &config,
        observed_at,
        tokn_commit,
        env!("CARGO_PKG_VERSION"),
    )?;
    if pair.candidate.decision_gate.verdict == ShadowDecisionVerdict::EligibleForImplementation {
        bail!("shadow index sanitized pilot must never emit ELIGIBLE_FOR_IMPLEMENTATION");
    }

    std::fs::create_dir_all(output_dir)
        .with_context(|| format!("create shadow index pilot output {}", output_dir.display()))?;
    let baseline_path = output_dir.join("direct-scan.json");
    let candidate_path = output_dir.join("unicode61.json");
    write_json(&baseline_path, &pair.baseline)?;
    write_json(&candidate_path, &pair.candidate)?;

    println!("Shadow index sanitized pilot complete");
    println!("  baseline: {}", baseline_path.display());
    println!("  candidate: {}", candidate_path.display());
    println!(
        "  candidate_verdict: {:?}",
        pair.candidate.decision_gate.verdict
    );
    println!(
        "  quality_gate: {:?}",
        pair.candidate.decision_gate.quality_gate_status
    );
    println!(
        "  correctness_gate: {:?}",
        pair.candidate.correctness.gate_status
    );
    println!(
        "  resource_gate: {:?}",
        pair.candidate.decision_gate.resource_gate_status
    );
    Ok(())
}

fn write_json(path: &Path, value: &impl serde::Serialize) -> anyhow::Result<()> {
    let json = serde_json::to_string_pretty(value)?;
    std::fs::write(path, format!("{json}\n"))
        .with_context(|| format!("write shadow index pilot output {}", path.display()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn command_writes_two_private_schema_v1_measurements_without_fixture_paths() {
        let config = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../benchmarks/shadow-index-sanitized-pilot-v1.json");
        let output = std::env::temp_dir().join(format!(
            "tokn-shadow-index-pilot-command-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&output);

        run(
            &config,
            &output,
            "2026-10-04T00:00:00Z",
            "1111111111111111111111111111111111111111",
        )
        .expect("run sanitized pilot command");

        let baseline_raw =
            std::fs::read_to_string(output.join("direct-scan.json")).expect("read baseline output");
        let candidate_raw =
            std::fs::read_to_string(output.join("unicode61.json")).expect("read candidate output");
        let baseline: serde_json::Value =
            serde_json::from_str(&baseline_raw).expect("parse baseline output");
        let candidate: serde_json::Value =
            serde_json::from_str(&candidate_raw).expect("parse candidate output");

        assert_eq!(baseline["schema_version"].as_u64(), Some(1));
        assert_eq!(
            baseline["condition"]["backend_id"].as_str(),
            Some("DIRECT_SCAN_V0")
        );
        assert_eq!(
            baseline["decision_gate"]["verdict"].as_str(),
            Some("BASELINE_ONLY")
        );
        assert_eq!(
            candidate["condition"]["backend_id"].as_str(),
            Some("SQLITE_FTS5_UNICODE61_V0")
        );
        assert_eq!(
            candidate["decision_gate"]["quality_gate_status"].as_str(),
            Some("FAIL")
        );
        assert_eq!(
            candidate["decision_gate"]["verdict"].as_str(),
            Some("REJECTED")
        );
        assert_eq!(
            candidate["correctness"]["gate_status"].as_str(),
            Some("UNKNOWN")
        );
        assert_eq!(
            candidate["decision_gate"]["resource_gate_status"].as_str(),
            Some("UNKNOWN")
        );
        for raw in [&baseline_raw, &candidate_raw] {
            assert!(!raw.contains("src/alpha.rs"));
            assert!(!raw.contains("AlphaWidget"));
            assert!(!raw.contains("tokn-shadow-index-pilot-command"));
        }

        let _ = std::fs::remove_dir_all(output);
    }
}
