mod analyze;
mod analyze_run;
mod check_caps;
mod common;
mod compare;
mod doctor;
mod health;
mod import;
mod inspect_schema;
mod report;
mod resolve_workspace;
mod sessions;
mod simulate_caps;

use crate::cli::{Cli, Command};

pub fn run(cli: Cli) -> anyhow::Result<()> {
    match cli.command {
        Command::Doctor { dev } => doctor::run(dev),
        Command::Sessions { limit } => sessions::run(limit),
        Command::InspectSchema { source } => inspect_schema::run(&source),
        Command::Health { source } => health::run(&source),
        Command::Import { source } => import::run(&source),
        Command::Analyze { source, top } => analyze::run(&source, top),
        Command::AnalyzeRun {
            source,
            output_json,
        } => analyze_run::run(&source, output_json.as_deref()),
        Command::ResolveWorkspace {
            source,
            inventory,
            before_inventory,
            source_root,
            expected_outputs,
            output_json,
        } => resolve_workspace::run(
            &source,
            &inventory,
            before_inventory.as_deref(),
            &source_root,
            &expected_outputs,
            output_json.as_deref(),
        ),
        Command::Compare {
            baseline,
            candidate,
        } => compare::run(&baseline, &candidate),
        Command::SimulateCaps { source, caps } => simulate_caps::run(&source, &caps),
        Command::CheckCaps { source, caps } => check_caps::run(&source, &caps),
        Command::Report => report::run(),
        Command::DbPath => {
            println!("{}", common::db_path()?.display());
            Ok(())
        }
        Command::ImportPath { path } => import::run_path(&path),
    }
}
