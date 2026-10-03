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
    Runner {
        request: PathBuf,
    },
    ContextLedger {
        #[arg(long)]
        project_id: Option<String>,
        #[arg(long)]
        workspace_id: Option<String>,
        #[arg(long, default_value_t = 50)]
        limit: usize,
        #[arg(long)]
        db: Option<PathBuf>,
        #[arg(long)]
        output_json: Option<PathBuf>,
    },
    ActivityTimeline {
        #[arg(long)]
        project_id: Option<String>,
        #[arg(long)]
        workspace_id: Option<String>,
        #[arg(long, default_value_t = 50)]
        limit: usize,
        #[arg(long)]
        db: Option<PathBuf>,
        #[arg(long)]
        output_json: Option<PathBuf>,
    },
    CrossRunComparison {
        #[arg(long)]
        baseline_run_id: String,
        #[arg(long)]
        candidate_run_id: String,
        #[arg(long)]
        project_id: Option<String>,
        #[arg(long, default_value_t = 50)]
        limit: usize,
        #[arg(long)]
        db: Option<PathBuf>,
        #[arg(long)]
        output_json: Option<PathBuf>,
    },
    CrossAgentEvidence {
        #[arg(long)]
        project_id: Option<String>,
        #[arg(long)]
        workspace_id: Option<String>,
        #[arg(long, default_value_t = 50)]
        limit: usize,
        #[arg(long)]
        db: Option<PathBuf>,
        #[arg(long)]
        output_json: Option<PathBuf>,
    },
    RateLimitHistory {
        #[arg(long)]
        project_id: Option<String>,
        #[arg(long)]
        workspace_id: Option<String>,
        #[arg(long, default_value_t = 50)]
        limit: usize,
        #[arg(long)]
        db: Option<PathBuf>,
        #[arg(long)]
        output_json: Option<PathBuf>,
    },
    SourceFreshnessEvidence {
        #[arg(long)]
        project_id: Option<String>,
        #[arg(long)]
        workspace_id: Option<String>,
        #[arg(long, default_value_t = 50)]
        limit: usize,
        #[arg(long)]
        db: Option<PathBuf>,
        #[arg(long)]
        output_json: Option<PathBuf>,
    },
    SourceIdentityHistory {
        #[arg(long)]
        project_id: Option<String>,
        #[arg(long)]
        workspace_id: Option<String>,
        #[arg(long, default_value_t = 50)]
        limit: usize,
        #[arg(long)]
        db: Option<PathBuf>,
        #[arg(long)]
        output_json: Option<PathBuf>,
    },
    SourceMutationHistory {
        #[arg(long)]
        project_id: Option<String>,
        #[arg(long)]
        workspace_id: Option<String>,
        #[arg(long, default_value_t = 50)]
        limit: usize,
        #[arg(long)]
        db: Option<PathBuf>,
        #[arg(long)]
        output_json: Option<PathBuf>,
    },
    SourceMutationWindowHistory {
        #[arg(long)]
        project_id: Option<String>,
        #[arg(long)]
        workspace_id: Option<String>,
        #[arg(long, default_value_t = 50)]
        limit: usize,
        #[arg(long)]
        db: Option<PathBuf>,
        #[arg(long)]
        output_json: Option<PathBuf>,
    },
    SourceVersionHistory {
        #[arg(long)]
        project_id: Option<String>,
        #[arg(long)]
        workspace_id: Option<String>,
        #[arg(long, default_value_t = 50)]
        limit: usize,
        #[arg(long)]
        db: Option<PathBuf>,
        #[arg(long)]
        output_json: Option<PathBuf>,
    },
    WorkspaceGitProvenanceHistory {
        #[arg(long)]
        project_id: Option<String>,
        #[arg(long)]
        workspace_id: Option<String>,
        #[arg(long, default_value_t = 50)]
        limit: usize,
        #[arg(long)]
        db: Option<PathBuf>,
        #[arg(long)]
        output_json: Option<PathBuf>,
    },
    StoreEvidence {
        evidence_dir: PathBuf,
        #[arg(long)]
        project_key: String,
        #[arg(long)]
        workspace_key: String,
        #[arg(long)]
        parent_workspace_key: Option<String>,
        #[arg(long)]
        runtime_profile: Option<PathBuf>,
        #[arg(long)]
        db: Option<PathBuf>,
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
            value_parser = ["not-proven", "unavailable", "supported-unverified", "supported-insufficient-input", "supported-not-active", "enforced"]
        )]
        enforcement: String,
        #[arg(long)]
        output_json: Option<PathBuf>,
    },
    EvaluateValidity {
        input: PathBuf,
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
    #[command(hide = true)]
    HookPreToolUse {
        #[arg(long = "cap", value_name = "CATEGORY=TOKENS")]
        caps: Vec<String>,
        #[arg(long)]
        audit_jsonl: Option<PathBuf>,
    },
    #[command(hide = true)]
    HookProbePreToolUse {
        #[arg(long)]
        audit_jsonl: PathBuf,
    },
    Report,
    DbPath,
    #[command(hide = true)]
    ImportPath {
        path: PathBuf,
    },
}
