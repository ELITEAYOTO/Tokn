use std::path::PathBuf;
use std::time::{Duration, UNIX_EPOCH};

use anyhow::Result;

use crate::session::{SessionMetadata, index_sessions, parse_rfc3339_unix_ms};

use super::DiagnosticBundle;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FallbackStatus {
    None,
    Unique,
    Ambiguous,
}

impl FallbackStatus {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::None => "NONE",
            Self::Unique => "UNIQUE",
            Self::Ambiguous => "AMBIGUOUS",
        }
    }
}

#[derive(Debug, Clone)]
pub struct SessionFallbackCandidate {
    pub path: PathBuf,
    pub observed_unix_ms: u64,
    pub metadata: SessionMetadata,
}

#[derive(Debug, Clone)]
pub struct SessionFallbackDiscovery {
    pub status: FallbackStatus,
    pub candidates: Vec<SessionFallbackCandidate>,
    pub selected: Option<PathBuf>,
}

pub fn discover_session_fallback(
    bundle: &DiagnosticBundle,
    horizon: Duration,
) -> Result<SessionFallbackDiscovery> {
    let Some(started_at) = bundle.started_at_unix_ms() else {
        return Ok(SessionFallbackDiscovery {
            status: FallbackStatus::None,
            candidates: Vec::new(),
            selected: None,
        });
    };
    let horizon_ms = u64::try_from(horizon.as_millis()).unwrap_or(u64::MAX);
    let end = started_at.saturating_add(horizon_ms);

    let mut candidates = index_sessions()
        .into_iter()
        .filter_map(|entry| {
            if !entry.metadata.is_root_candidate() {
                return None;
            }
            if entry
                .metadata
                .originator
                .as_deref()
                .is_some_and(|originator| !originator.eq_ignore_ascii_case("Codex Desktop"))
            {
                return None;
            }
            let metadata_unix_ms = entry
                .metadata
                .timestamp
                .as_deref()
                .and_then(parse_rfc3339_unix_ms);
            let modified_unix_ms = entry.file.modified.and_then(|modified| {
                u64::try_from(modified.duration_since(UNIX_EPOCH).ok()?.as_millis()).ok()
            });
            let observed_unix_ms = metadata_unix_ms.or(modified_unix_ms)?;
            if observed_unix_ms < started_at || observed_unix_ms > end {
                return None;
            }

            Some(SessionFallbackCandidate {
                path: entry.file.path,
                observed_unix_ms,
                metadata: entry.metadata,
            })
        })
        .collect::<Vec<_>>();

    candidates.sort_by_key(|candidate| candidate.observed_unix_ms);

    let (status, selected) = match candidates.as_slice() {
        [] => (FallbackStatus::None, None),
        [candidate] => (FallbackStatus::Unique, Some(candidate.path.clone())),
        _ => (FallbackStatus::Ambiguous, None),
    };

    Ok(SessionFallbackDiscovery {
        status,
        candidates,
        selected,
    })
}
