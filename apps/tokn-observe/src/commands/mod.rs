mod activity_timeline;
mod analyze;
mod analyze_run;
mod check_caps;
mod common;
mod compare;
mod context_ledger;
mod cross_agent_evidence;
mod cross_run_comparison;
mod doctor;
mod evaluate_validity;
mod health;
mod hook_pre_tool_use;
mod hook_probe_pre_tool_use;
mod import;
mod inspect_policy;
mod inspect_schema;
mod rate_limit_history;
mod report;
mod resolve_workspace;
mod runner;
mod sessions;
mod shadow_index_capabilities;
mod simulate_caps;
mod source_freshness_evidence;
mod source_identity_history;
mod source_mutation_history;
mod source_mutation_window_history;
mod source_reread_evidence;
mod source_version_history;
mod store_evidence;
mod task_input_history;
mod workspace_git_provenance_history;

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
        Command::Runner { request } => runner::run(&request),
        Command::ContextLedger {
            project_id,
            workspace_id,
            limit,
            db,
            output_json,
        } => context_ledger::run(
            project_id.as_deref(),
            workspace_id.as_deref(),
            limit,
            db.as_deref(),
            output_json.as_deref(),
        ),
        Command::ActivityTimeline {
            project_id,
            workspace_id,
            limit,
            db,
            output_json,
        } => activity_timeline::run(
            project_id.as_deref(),
            workspace_id.as_deref(),
            limit,
            db.as_deref(),
            output_json.as_deref(),
        ),
        Command::CrossRunComparison {
            baseline_run_id,
            candidate_run_id,
            project_id,
            limit,
            db,
            output_json,
        } => cross_run_comparison::run(
            &baseline_run_id,
            &candidate_run_id,
            project_id.as_deref(),
            limit,
            db.as_deref(),
            output_json.as_deref(),
        ),
        Command::CrossAgentEvidence {
            project_id,
            workspace_id,
            limit,
            db,
            output_json,
        } => cross_agent_evidence::run(
            project_id.as_deref(),
            workspace_id.as_deref(),
            limit,
            db.as_deref(),
            output_json.as_deref(),
        ),
        Command::RateLimitHistory {
            project_id,
            workspace_id,
            limit,
            db,
            output_json,
        } => rate_limit_history::run(
            project_id.as_deref(),
            workspace_id.as_deref(),
            limit,
            db.as_deref(),
            output_json.as_deref(),
        ),
        Command::SourceFreshnessEvidence {
            project_id,
            workspace_id,
            limit,
            db,
            output_json,
        } => source_freshness_evidence::run(
            project_id.as_deref(),
            workspace_id.as_deref(),
            limit,
            db.as_deref(),
            output_json.as_deref(),
        ),
        Command::SourceIdentityHistory {
            project_id,
            workspace_id,
            limit,
            db,
            output_json,
        } => source_identity_history::run(
            project_id.as_deref(),
            workspace_id.as_deref(),
            limit,
            db.as_deref(),
            output_json.as_deref(),
        ),
        Command::SourceMutationHistory {
            project_id,
            workspace_id,
            limit,
            db,
            output_json,
        } => source_mutation_history::run(
            project_id.as_deref(),
            workspace_id.as_deref(),
            limit,
            db.as_deref(),
            output_json.as_deref(),
        ),
        Command::SourceMutationWindowHistory {
            project_id,
            workspace_id,
            limit,
            db,
            output_json,
        } => source_mutation_window_history::run(
            project_id.as_deref(),
            workspace_id.as_deref(),
            limit,
            db.as_deref(),
            output_json.as_deref(),
        ),
        Command::SourceRereadEvidence {
            project_id,
            workspace_id,
            limit,
            db,
            output_json,
        } => source_reread_evidence::run(
            project_id.as_deref(),
            workspace_id.as_deref(),
            limit,
            db.as_deref(),
            output_json.as_deref(),
        ),
        Command::SourceVersionHistory {
            project_id,
            workspace_id,
            limit,
            db,
            output_json,
        } => source_version_history::run(
            project_id.as_deref(),
            workspace_id.as_deref(),
            limit,
            db.as_deref(),
            output_json.as_deref(),
        ),
        Command::TaskInputHistory {
            project_id,
            workspace_id,
            limit,
            db,
            output_json,
        } => task_input_history::run(
            project_id.as_deref(),
            workspace_id.as_deref(),
            limit,
            db.as_deref(),
            output_json.as_deref(),
        ),
        Command::WorkspaceGitProvenanceHistory {
            project_id,
            workspace_id,
            limit,
            db,
            output_json,
        } => workspace_git_provenance_history::run(
            project_id.as_deref(),
            workspace_id.as_deref(),
            limit,
            db.as_deref(),
            output_json.as_deref(),
        ),
        Command::ShadowIndexCapabilities { output_json } => {
            shadow_index_capabilities::run(output_json.as_deref())
        }
        Command::StoreEvidence {
            evidence_dir,
            project_key,
            workspace_key,
            parent_workspace_key,
            runtime_profile,
            task_input,
            db,
        } => store_evidence::run(
            &evidence_dir,
            &project_key,
            &workspace_key,
            parent_workspace_key.as_deref(),
            runtime_profile.as_deref(),
            task_input.as_deref(),
            db.as_deref(),
        ),
        Command::EvaluateValidity { input, output_json } => {
            evaluate_validity::run(&input, output_json.as_deref())
        }
        Command::InspectPolicy {
            source,
            policy_id,
            marker,
            policy_paths,
            caps,
            enforcement,
            output_json,
        } => inspect_policy::run(
            &source,
            &policy_id,
            &marker,
            &policy_paths,
            &caps,
            &enforcement,
            output_json.as_deref(),
        ),
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
        Command::HookPreToolUse { caps, audit_jsonl } => {
            hook_pre_tool_use::run(&caps, audit_jsonl.as_deref())
        }
        Command::HookProbePreToolUse { audit_jsonl } => hook_probe_pre_tool_use::run(&audit_jsonl),
        Command::Report => report::run(),
        Command::DbPath => {
            println!("{}", common::db_path()?.display());
            Ok(())
        }
        Command::ImportPath { path } => import::run_path(&path),
    }
}
