use std::path::{Path, PathBuf};

use serde_json::Value;

#[derive(Debug, Clone)]
pub struct DiagnosticBundle {
    pub root: PathBuf,
    pub manifest: PathBuf,
    pub trace: PathBuf,
    pub payloads: PathBuf,
}

impl DiagnosticBundle {
    pub fn detect(root: &Path) -> Option<Self> {
        let manifest = root.join("manifest.json");
        let trace = root.join("trace.jsonl");
        if !manifest.is_file() || !trace.is_file() {
            return None;
        }

        Some(Self {
            root: root.to_path_buf(),
            manifest,
            trace,
            payloads: root.join("payloads"),
        })
    }

    pub fn started_at_unix_ms(&self) -> Option<u64> {
        let manifest: Value =
            serde_json::from_reader(std::fs::File::open(&self.manifest).ok()?).ok()?;
        manifest.get("started_at_unix_ms").and_then(Value::as_u64)
    }
}
