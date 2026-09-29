use std::fs::File;
use std::path::Path;

use tokn_analysis::build_run_group;
use tokn_codex::diagnostic::{DiagnosticBundle, assess_diagnostic_health};
use tokn_codex::session::collect_session_group;
use tokn_report::{render_run_group_text, render_source_health_text};

use super::common::{resolve_session_root_from_source, resolve_source};

pub fn run(source: &str, output_json: Option<&Path>) -> anyhow::Result<()> {
    let source_path = resolve_source(source)?;

    if source_path.is_dir() {
        let bundle = DiagnosticBundle::detect(&source_path)
            .ok_or_else(|| anyhow::anyhow!("not a valid diagnostic trace bundle"))?;
        let health = assess_diagnostic_health(&bundle)?;
        println!(
            "{}",
            render_source_health_text(&source_path.to_string_lossy(), &health)
        );

        if health.is_usable_for_token_accounting() {
            anyhow::bail!(
                "diagnostic source is healthy, but run-group analysis currently requires a session root"
            );
        }
    }

    let root_session = resolve_session_root_from_source(&source_path)?;
    let grouped = collect_session_group(&root_session)?;
    let group = build_run_group(&grouped.members, &grouped.root_thread_id)
        .ok_or_else(|| anyhow::anyhow!("failed to build run group"))?;

    println!("EVIDENCE ROOT\n  {}\n", root_session.display());
    print!("{}", render_run_group_text(&group));

    if let Some(path) = output_json {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        serde_json::to_writer_pretty(File::create(path)?, &group)?;
        println!("\nJSON\n  {}", path.display());
    }

    Ok(())
}
