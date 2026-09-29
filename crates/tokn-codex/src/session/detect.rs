use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use crate::session_roots;

#[derive(Debug, Clone)]
pub struct SessionFile {
    pub path: PathBuf,
    pub size: u64,
    pub modified: Option<SystemTime>,
}

pub fn list_sessions() -> Vec<SessionFile> {
    let mut out = Vec::new();
    for root in session_roots() {
        walk(&root, &mut out, 8);
    }
    out.sort_by(|a, b| {
        b.path
            .file_name()
            .cmp(&a.path.file_name())
            .then_with(|| b.path.cmp(&a.path))
    });
    out
}

fn walk(root: &Path, out: &mut Vec<SessionFile>, depth: usize) {
    if depth == 0 || !root.is_dir() {
        return;
    }
    let Ok(entries) = fs::read_dir(root) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            walk(&path, out, depth - 1);
            continue;
        }
        let Some(name) = path.file_name().and_then(|v| v.to_str()) else {
            continue;
        };
        if !name.starts_with("rollout-") || !name.ends_with(".jsonl") {
            continue;
        }
        if let Ok(meta) = entry.metadata() {
            out.push(SessionFile {
                path,
                size: meta.len(),
                modified: meta.modified().ok(),
            });
        }
    }
}
