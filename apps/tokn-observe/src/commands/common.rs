use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::time::Duration;

use anyhow::{Context, anyhow};
use tokn_analysis::CapPolicyStatus;
use tokn_codex::diagnostic::{DiagnosticBundle, FallbackStatus, discover_session_fallback};
use tokn_codex::session::{SessionFile, list_sessions, resolve_root_session};
use tokn_domain::PolicyObservationStatus;
use tokn_platform::observer_database_path;
use tokn_storage::Database;

pub fn resolve_source(value: &str) -> anyhow::Result<PathBuf> {
    if value.eq_ignore_ascii_case("latest") {
        return latest_session().map(|s| s.path);
    }

    let path = PathBuf::from(value);
    if path.exists() {
        Ok(path)
    } else {
        Err(anyhow!("source not found: {}", path.display()))
    }
}

pub fn latest_session() -> anyhow::Result<SessionFile> {
    list_sessions()
        .into_iter()
        .next()
        .ok_or_else(|| anyhow!("no Codex session rollouts found"))
}

pub fn resolve_session_root_from_source(path: &Path) -> anyhow::Result<PathBuf> {
    const FALLBACK_HORIZON: Duration = Duration::from_secs(2 * 60 * 60);

    if !path.is_dir() {
        return resolve_root_session(path);
    }

    let bundle = DiagnosticBundle::detect(path)
        .ok_or_else(|| anyhow!("not a valid diagnostic trace bundle: {}", path.display()))?;
    let fallback = discover_session_fallback(&bundle, FALLBACK_HORIZON)?;

    match fallback.status {
        FallbackStatus::Unique => fallback
            .selected
            .ok_or_else(|| anyhow!("unique fallback has no selected path")),
        FallbackStatus::None => anyhow::bail!("no safe session fallback candidate found"),
        FallbackStatus::Ambiguous => {
            anyhow::bail!("session fallback is ambiguous; refusing to choose automatically")
        }
    }
}

pub fn db_path() -> anyhow::Result<PathBuf> {
    observer_database_path().context("LOCALAPPDATA is unavailable")
}

pub fn open_db() -> anyhow::Result<Database> {
    let path = db_path()?;
    Database::open(Path::new(&path)).map_err(Into::into)
}

pub fn map_policy_observation_status(status: CapPolicyStatus) -> PolicyObservationStatus {
    match status {
        CapPolicyStatus::Pass => PolicyObservationStatus::Pass,
        CapPolicyStatus::Fail => PolicyObservationStatus::Fail,
        CapPolicyStatus::NoEvidence => PolicyObservationStatus::NoEvidence,
        CapPolicyStatus::IncompleteEvidence => PolicyObservationStatus::IncompleteEvidence,
    }
}

pub fn validate_private_filter_id(
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

pub fn parse_cap_overrides(values: &[String]) -> anyhow::Result<BTreeMap<String, u64>> {
    let mut out = BTreeMap::new();
    for value in values {
        let Some((category, tokens)) = value.split_once('=') else {
            anyhow::bail!("invalid cap '{value}', expected CATEGORY=TOKENS");
        };
        let category = category.trim();
        if category.is_empty() {
            anyhow::bail!("cap category cannot be empty");
        }
        let tokens: u64 = tokens
            .trim()
            .parse()
            .map_err(|_| anyhow::anyhow!("invalid token cap in '{value}'"))?;
        if tokens == 0 {
            anyhow::bail!("token cap must be greater than zero");
        }
        out.insert(category.to_string(), tokens);
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn private_filter_ids_accept_only_canonical_hashed_ids() {
        assert!(
            validate_private_filter_id(
                Some("prj-0123456789abcdef01234567"),
                "project",
                "prj-",
            )
            .is_ok()
        );
        assert!(
            validate_private_filter_id(
                Some("wsp-abcdef0123456789abcdef01"),
                "workspace",
                "wsp-",
            )
            .is_ok()
        );

        let raw = format!("C:\\{}\\someone\\project", "Users");
        assert!(validate_private_filter_id(Some(&raw), "project", "prj-").is_err());
        assert!(validate_private_filter_id(Some("prj-short"), "project", "prj-").is_err());
        assert!(
            validate_private_filter_id(
                Some("wsp-0123456789abcdef0123456z"),
                "workspace",
                "wsp-",
            )
            .is_err()
        );
    }

    #[test]
    fn parses_repeatable_category_caps() {
        let values = vec!["file_read=5000".to_string(), "search=3000".to_string()];
        let parsed = parse_cap_overrides(&values).unwrap();
        assert_eq!(parsed.get("file_read"), Some(&5_000));
        assert_eq!(parsed.get("search"), Some(&3_000));
    }
}
