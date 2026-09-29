use std::path::PathBuf;

use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(
    name = "tokn-observe",
    version,
    about = "Tokn / Tokn passive Codex observer"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    Doctor {
        #[arg(long)]
        dev: bool,
    },
    Sessions {
        #[arg(long, default_value_t = 20)]
        limit: usize,
    },
    InspectSchema {
        source: String,
    },
    Health {
        source: String,
    },
    Import {
        source: String,
    },
    Analyze {
        source: String,
        #[arg(long, default_value_t = 10)]
        top: usize,
    },
    AnalyzeRun {
        source: String,
        #[arg(long)]
        output_json: Option<PathBuf>,
    },
    InspectPolicy {
        source: String,
        #[arg(long, default_value = "policy")]
        policy_id: String,
        #[arg(long)]
        marker: String,
        #[arg(long = "policy-path")]
        policy_paths: Vec<PathBuf>,
        #[arg(long = "cap", value_name = "CATEGORY=TOKENS")]
        caps: Vec<String>,
        #[arg(
            long,
            default_value = "not-proven",
            value_parser = ["not-proven", "unavailable", "enforced"]
        )]
        enforcement: String,
        #[arg(long)]
        output_json: Option<PathBuf>,
    },
    ResolveWorkspace {
        source: String,
        #[arg(long)]
        inventory: PathBuf,
        #[arg(long)]
        before_inventory: Option<PathBuf>,
        #[arg(long)]
        source_root: PathBuf,
        #[arg(long = "expected-output")]
        expected_outputs: Vec<PathBuf>,
        #[arg(long)]
        output_json: Option<PathBuf>,
    },
    Compare {
        baseline: String,
        candidate: String,
    },
    SimulateCaps {
        source: String,
        #[arg(long = "cap", value_name = "CATEGORY=TOKENS")]
        caps: Vec<String>,
    },
    CheckCaps {
        source: String,
        #[arg(long = "cap", value_name = "CATEGORY=TOKENS")]
        caps: Vec<String>,
    },
    Report,
    DbPath,
    #[command(hide = true)]
    ImportPath {
        path: PathBuf,
    },
}
