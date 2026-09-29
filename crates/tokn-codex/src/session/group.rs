use std::collections::HashSet;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use tokn_domain::AgentEvidence;

use super::{
    SessionFile, SessionMetadata, list_sessions, read_session_evidence, read_session_metadata,
};

#[derive(Debug, Clone)]
pub struct SessionIndexEntry {
    pub file: SessionFile,
    pub metadata: SessionMetadata,
}

#[derive(Debug, Clone)]
pub struct SessionGroupEvidence {
    pub root_thread_id: String,
    pub session_id: Option<String>,
    pub members: Vec<AgentEvidence>,
}

pub fn index_sessions() -> Vec<SessionIndexEntry> {
    list_sessions()
        .into_iter()
        .filter_map(|file| {
            let metadata = read_session_metadata(&file.path).ok().flatten()?;
            Some(SessionIndexEntry { file, metadata })
        })
        .collect()
}

pub fn resolve_root_session(path: &Path) -> Result<PathBuf> {
    let metadata = read_session_metadata(path)?
        .ok_or_else(|| anyhow::anyhow!("session_meta not found in {}", path.display()))?;

    if metadata.is_root_candidate() {
        return Ok(path.to_path_buf());
    }

    let session_id = metadata
        .session_id
        .as_deref()
        .ok_or_else(|| anyhow::anyhow!("subagent session_id is missing"))?;

    index_sessions()
        .into_iter()
        .find(|entry| {
            entry.metadata.session_id.as_deref() == Some(session_id)
                && entry.metadata.is_root_candidate()
        })
        .map(|entry| entry.file.path)
        .ok_or_else(|| anyhow::anyhow!("root session not found for {}", path.display()))
}

pub fn collect_session_group(root_path: &Path) -> Result<SessionGroupEvidence> {
    let files = index_sessions()
        .into_iter()
        .map(|entry| entry.file.path)
        .collect::<Vec<_>>();
    collect_session_group_from_files(root_path, &files)
}

pub fn collect_session_group_from_files(
    root_path: &Path,
    candidates: &[PathBuf],
) -> Result<SessionGroupEvidence> {
    let root_metadata = read_session_metadata(root_path)?
        .ok_or_else(|| anyhow::anyhow!("session_meta not found in {}", root_path.display()))?;
    let root_thread_id = root_metadata
        .thread_id
        .clone()
        .ok_or_else(|| anyhow::anyhow!("root thread id is missing"))?;
    let session_id = root_metadata.session_id.clone();

    let mut paths = Vec::new();
    let mut seen = HashSet::new();
    paths.push(root_path.to_path_buf());
    seen.insert(root_path.to_path_buf());

    for path in candidates {
        if seen.contains(path) {
            continue;
        }
        let Some(metadata) = read_session_metadata(path).ok().flatten() else {
            continue;
        };
        if session_id.is_some() && metadata.session_id == session_id {
            paths.push(path.clone());
            seen.insert(path.clone());
        }
    }

    let mut members = Vec::new();
    for path in paths {
        members.push(
            read_session_evidence(&path)
                .with_context(|| format!("read grouped session {}", path.display()))?,
        );
    }

    Ok(SessionGroupEvidence {
        root_thread_id,
        session_id,
        members,
    })
}
