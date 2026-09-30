use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectSnapshotFile {
    pub path: String,
    pub sha256: String,
    pub bytes: u64,
    #[serde(default)]
    pub last_write_utc: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectSnapshot {
    pub schema_version: u64,
    #[serde(default)]
    pub created_at: Option<String>,
    pub project_root: String,
    #[serde(default)]
    pub excluded_top_level: Vec<String>,
    pub file_count: u64,
    pub total_bytes: u64,
    #[serde(default)]
    pub files: Vec<ProjectSnapshotFile>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModifiedSnapshotFile {
    pub path: String,
    pub before_sha256: String,
    pub after_sha256: String,
    pub before_bytes: u64,
    pub after_bytes: u64,
    pub byte_delta: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkspaceDiffReport {
    pub schema_version: u64,
    pub before: String,
    pub after: String,
    pub added_count: u64,
    pub modified_count: u64,
    pub removed_count: u64,
    pub added: Vec<ProjectSnapshotFile>,
    pub modified: Vec<ModifiedSnapshotFile>,
    pub removed: Vec<ProjectSnapshotFile>,
}

pub fn diff_project_snapshots(
    before: &ProjectSnapshot,
    after: &ProjectSnapshot,
    before_label: impl Into<String>,
    after_label: impl Into<String>,
) -> WorkspaceDiffReport {
    let before_map = before
        .files
        .iter()
        .map(|file| (file.path.as_str(), file))
        .collect::<BTreeMap<_, _>>();
    let after_map = after
        .files
        .iter()
        .map(|file| (file.path.as_str(), file))
        .collect::<BTreeMap<_, _>>();

    let mut added = Vec::new();
    let mut modified = Vec::new();
    let mut removed = Vec::new();

    for (path, after_file) in &after_map {
        let Some(before_file) = before_map.get(path) else {
            added.push((*after_file).clone());
            continue;
        };

        if before_file.sha256 != after_file.sha256 {
            modified.push(ModifiedSnapshotFile {
                path: (*path).to_string(),
                before_sha256: before_file.sha256.clone(),
                after_sha256: after_file.sha256.clone(),
                before_bytes: before_file.bytes,
                after_bytes: after_file.bytes,
                byte_delta: signed_delta(before_file.bytes, after_file.bytes),
            });
        }
    }

    for (path, before_file) in &before_map {
        if !after_map.contains_key(path) {
            removed.push((*before_file).clone());
        }
    }

    WorkspaceDiffReport {
        schema_version: 1,
        before: before_label.into(),
        after: after_label.into(),
        added_count: added.len() as u64,
        modified_count: modified.len() as u64,
        removed_count: removed.len() as u64,
        added,
        modified,
        removed,
    }
}

fn signed_delta(before: u64, after: u64) -> i64 {
    if after >= before {
        i64::try_from(after - before).unwrap_or(i64::MAX)
    } else {
        -i64::try_from(before - after).unwrap_or(i64::MAX)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn file(path: &str, sha256: &str, bytes: u64) -> ProjectSnapshotFile {
        ProjectSnapshotFile {
            path: path.into(),
            sha256: sha256.into(),
            bytes,
            last_write_utc: None,
        }
    }

    fn snapshot(files: Vec<ProjectSnapshotFile>) -> ProjectSnapshot {
        ProjectSnapshot {
            schema_version: 1,
            created_at: None,
            project_root: "E:/fixture".into(),
            excluded_top_level: vec![],
            file_count: files.len() as u64,

            total_bytes: files.iter().map(|file| file.bytes).sum(),
            files,
        }
    }

    #[test]
    fn reproduces_historical_snapshot_diff_semantics() {
        let before = snapshot(vec![
            file("a.txt", "same", 10),
            file("b.txt", "old", 20),
            file("c.txt", "gone", 30),
        ]);
        let after = snapshot(vec![
            file("a.txt", "same", 10),
            file("b.txt", "new", 25),
            file("d.txt", "added", 40),
        ]);

        let diff = diff_project_snapshots(&before, &after, "before.json", "after.json");

        assert_eq!(diff.added_count, 1);
        assert_eq!(diff.modified_count, 1);
        assert_eq!(diff.removed_count, 1);
        assert_eq!(diff.added[0].path, "d.txt");
        assert_eq!(diff.modified[0].path, "b.txt");
        assert_eq!(diff.modified[0].byte_delta, 5);
        assert_eq!(diff.removed[0].path, "c.txt");
    }
}
