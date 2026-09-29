use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use tokn_platform::codex_home;

use crate::{CodexCapabilities, CodexSurface};

#[derive(Debug, Clone)]
pub struct CodexInstall {
    pub path: PathBuf,
    pub surface: CodexSurface,
    pub version: Option<String>,
    pub capabilities: CodexCapabilities,
}

pub fn session_root() -> Option<PathBuf> {
    codex_home().map(|p| p.join("sessions"))
}

pub fn session_roots() -> Vec<PathBuf> {
    let Some(home) = codex_home() else {
        return Vec::new();
    };

    vec![home.join("sessions"), home.join("archived_sessions")]
}

pub fn discover_codex() -> Vec<CodexInstall> {
    let mut candidates = Vec::new();
    if let Ok(path) = std::env::var("TOKN_CODEX_BIN") {
        candidates.push((PathBuf::from(path), CodexSurface::Custom));
    }
    if let Ok(path) = std::env::var("CODEX_BIN") {
        candidates.push((PathBuf::from(path), CodexSurface::Custom));
    }

    if let Ok(local) = std::env::var("LOCALAPPDATA") {
        let root = PathBuf::from(local)
            .join("OpenAI")
            .join("Codex")
            .join("bin");
        collect_named(
            &root,
            "codex.exe",
            CodexSurface::Desktop,
            &mut candidates,
            3,
        );
    }

    if let Ok(home) = std::env::var("USERPROFILE") {
        let root = PathBuf::from(home).join(".vscode").join("extensions");
        collect_named(&root, "codex.exe", CodexSurface::VsCode, &mut candidates, 5);
    }

    let mut seen = std::collections::BTreeSet::new();
    let mut out = Vec::new();
    for (path, surface) in candidates {
        if !path.is_file() {
            continue;
        }
        let key = path.to_string_lossy().to_ascii_lowercase();
        if !seen.insert(key) {
            continue;
        }
        let version = run_text(&path, &["--version"]);
        let capabilities = probe_capabilities(&path);
        out.push(CodexInstall {
            path,
            surface,
            version,
            capabilities,
        });
    }
    out
}

fn probe_capabilities(path: &Path) -> CodexCapabilities {
    CodexCapabilities {
        session_rollouts: session_roots().iter().any(|path| path.is_dir()),
        trace_reduce: command_ok(path, &["debug", "trace-reduce", "--help"]),
        prompt_input_debug: command_ok(path, &["debug", "prompt-input", "--help"]),
        app_server_debug: command_ok(path, &["debug", "app-server", "--help"]),
    }
}

fn command_ok(path: &Path, args: &[&str]) -> bool {
    Command::new(path)
        .args(args)
        .output()
        .is_ok_and(|o| o.status.success())
}

fn run_text(path: &Path, args: &[&str]) -> Option<String> {
    let output = Command::new(path).args(args).output().ok()?;
    if !output.status.success() {
        return None;
    }
    Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
}
fn collect_named(
    root: &Path,
    name: &str,
    surface: CodexSurface,
    out: &mut Vec<(PathBuf, CodexSurface)>,
    depth: usize,
) {
    if depth == 0 || !root.is_dir() {
        return;
    }
    let Ok(entries) = fs::read_dir(root) else {
        return;
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_file()
            && path
                .file_name()
                .is_some_and(|n| n.eq_ignore_ascii_case(name))
        {
            out.push((path, surface));
        } else if path.is_dir() {
            collect_named(&path, name, surface, out, depth - 1);
        }
    }
}
