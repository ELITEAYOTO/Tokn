use std::path::Path;
use std::process::Command;

pub fn run_trace_reduce(codex_bin: &Path, bundle: &Path, output: &Path) -> std::io::Result<bool> {
    let status = Command::new(codex_bin)
        .arg("debug")
        .arg("trace-reduce")
        .arg(bundle)
        .arg("--output")
        .arg(output)
        .status()?;

    Ok(status.success())
}
