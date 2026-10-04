use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Component, Path, PathBuf},
    process::Command,
    sync::atomic::{AtomicU64, Ordering},
    time::Instant,
};

use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};
use tokn_domain::parse_rfc3339_unix_ms;
use tokn_platform::{
    DirectScanManifest, DirectScanPolicy, build_direct_scan_manifest, query_direct_scan_manifest,
};

use crate::{
    SQLITE_FTS5_UNICODE61_BACKEND_ID, SQLITE_FTS5_UNICODE61_TOKENIZER_ID, ShadowCapabilityState,
    SqliteFts5Unicode61Candidate, probe_sqlite_fts5_capabilities,
};

pub const SHADOW_INDEX_MEASUREMENT_SCHEMA_VERSION: u64 = 1;
pub const SHADOW_INDEX_PILOT_CONFIG_SCHEMA_VERSION: u64 = 1;
pub const SHADOW_INDEX_PILOT_MAX_QUERIES: usize = 64;
pub const SHADOW_INDEX_PILOT_MAX_TIMING_REPETITIONS: usize = 100;

static TEMP_COUNTER: AtomicU64 = AtomicU64::new(0);

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ShadowIndexPilotConfig {
    pub schema_version: u64,
    pub pilot_id: String,
    pub corpus: ShadowIndexPilotCorpusConfig,
    pub query_set_id: String,
    pub gold_set_id: String,
    pub correctness_suite_id: String,
    pub k_values: Vec<usize>,
    pub timing_repetitions_per_query: usize,
    pub queries: Vec<ShadowIndexPilotQuery>,
    pub thresholds: ShadowIndexPilotThresholds,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ShadowIndexPilotCorpusConfig {
    pub corpus_id: String,
    pub repository_ref: String,
    pub policy_id: String,
    pub max_file_bytes: u64,
    pub max_files: usize,
    pub tracked_files: Vec<ShadowIndexPilotFile>,
    pub untracked_files: Vec<ShadowIndexPilotFile>,
    pub ignored_files: Vec<ShadowIndexPilotFile>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShadowIndexPilotFile {
    pub path: String,
    pub content: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ShadowIndexPilotQuery {
    pub query_id: String,
    pub query_kind: ShadowQueryKind,
    pub text: String,
    pub gold_paths: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ShadowIndexPilotThresholds {
    pub thresholds_ref: String,
    pub quality_floor_predeclared: bool,
    pub resource_budgets_predeclared: bool,
    pub min_recall_at_5_ratio_to_direct: f64,
    pub min_mrr_ratio_to_direct: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ShadowQueryKind {
    Identifier,
    PathOrFilename,
    Concept,
    Substring,
    NoMatch,
    Other,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ShadowConditionMode {
    DirectScan,
    IndexedCandidate,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ShadowWorktreeStatus {
    Clean,
    Dirty,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ShadowCacheState {
    Unknown,
    Cold,
    Warm,
    Mixed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ShadowGateStatus {
    Pass,
    Fail,
    NotApplicable,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ShadowDecisionVerdict {
    BaselineOnly,
    Rejected,
    EligibleForImplementation,
    Inconclusive,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ShadowCorrectnessCaseId {
    TrackedContentEdit,
    DirtyUnchangedHead,
    AddedTrackedFile,
    NonIgnoredUntrackedFile,
    FileDeletion,
    FileRename,
    QueryAfterRefresh,
    HashMismatchQueryVerification,
    IndexDeleteRebuild,
    GitProvenanceFallback,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ShadowIndexMeasurementV1 {
    pub schema_version: u64,
    pub measurement_id: String,
    pub run_order: u64,
    pub observed_at: String,
    pub tokn: ShadowToknMeasurementIdentity,
    pub corpus: ShadowCorpusMeasurement,
    pub condition: ShadowConditionMeasurement,
    pub capability: ShadowCapabilityMeasurement,
    pub build: ShadowBuildMeasurement,
    pub refresh: Option<ShadowRefreshMeasurement>,
    pub query_config: ShadowQueryConfigMeasurement,
    pub queries: Vec<ShadowQueryMeasurement>,
    pub aggregate: ShadowAggregateMeasurement,
    pub correctness: ShadowCorrectnessMeasurement,
    pub decision_gate: ShadowDecisionGateMeasurement,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShadowToknMeasurementIdentity {
    pub commit: String,
    pub version: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShadowCorpusMeasurement {
    pub corpus_id: String,
    pub repository_ref: String,
    pub policy_id: String,
    pub commit: String,
    pub worktree_status: ShadowWorktreeStatus,
    pub discovered_files: u64,
    pub eligible_files: u64,
    pub skipped_files: u64,
    pub eligible_bytes: u64,
    pub source_bytes_read: u64,
    pub skipped_by_reason: BTreeMap<String, u64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShadowConditionMeasurement {
    pub mode: ShadowConditionMode,
    pub backend_id: String,
    pub tokenizer_id: Option<String>,
    pub configuration_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShadowCapabilityMeasurement {
    pub fts5: ShadowCapabilityState,
    pub contentless_delete: ShadowCapabilityState,
    pub integrity_check: ShadowGateStatus,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ShadowBuildMeasurement {
    pub elapsed_ms: f64,
    pub source_bytes_read: u64,
    pub index_bytes: u64,
    pub peak_rss_bytes: Option<u64>,
    pub cpu_ms: Option<f64>,
    pub files_processed: u64,
    pub files_skipped: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ShadowRefreshMeasurement {
    pub changed_files: u64,
    pub added_files: u64,
    pub deleted_files: u64,
    pub renamed_files: u64,
    pub files_reread: u64,
    pub bytes_reread: u64,
    pub elapsed_ms: f64,
    pub peak_rss_bytes: Option<u64>,
    pub fallback_full_verification: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShadowQueryConfigMeasurement {
    pub query_count: u64,
    pub k_values: Vec<usize>,
    pub query_set_id: String,
    pub gold_set_id: String,
    pub timing_repetitions_per_query: usize,
    pub cache_state: ShadowCacheState,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ShadowQueryMeasurement {
    pub query_id: String,
    pub query_kind: ShadowQueryKind,
    pub requested_k: usize,
    pub latency_samples_ms: Vec<f64>,
    pub expected_relevant_count: u64,
    pub relevant_returned_count: u64,
    pub first_relevant_rank: Option<u64>,
    pub reciprocal_rank: f64,
    pub no_match_correct: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ShadowAggregateMeasurement {
    pub sample_count: u64,
    pub latency_ms: ShadowLatencyPercentiles,
    pub recall_at_k: Vec<ShadowRecallAtK>,
    pub mrr: f64,
    pub no_match_accuracy: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ShadowLatencyPercentiles {
    pub p25: f64,
    pub p50: f64,
    pub p75: f64,
    pub p95: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ShadowRecallAtK {
    pub k: usize,
    pub recall: f64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShadowCorrectnessMeasurement {
    pub suite_id: String,
    pub cases: Vec<ShadowCorrectnessCaseMeasurement>,
    pub gate_status: ShadowGateStatus,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShadowCorrectnessCaseMeasurement {
    pub case_id: ShadowCorrectnessCaseId,
    pub status: ShadowGateStatus,
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ShadowDecisionGateMeasurement {
    pub quality_floor_predeclared: bool,
    pub resource_budgets_predeclared: bool,
    pub thresholds_ref: String,
    pub resource_gate_status: ShadowGateStatus,
    pub quality_gate_status: ShadowGateStatus,
    pub verdict: ShadowDecisionVerdict,
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ShadowIndexPilotPair {
    pub baseline: ShadowIndexMeasurementV1,
    pub candidate: ShadowIndexMeasurementV1,
}

#[derive(Debug, Clone)]
struct ObservedQuery {
    measurement: ShadowQueryMeasurement,
    ranked_paths: Vec<String>,
    gold_paths: BTreeSet<String>,
}

#[derive(Debug, Clone)]
struct QueryBatch {
    queries: Vec<ObservedQuery>,
    aggregate: ShadowAggregateMeasurement,
    source_bytes_read: u64,
}

struct TempPilotRepo {
    root: PathBuf,
    commit: String,
    worktree_status: ShadowWorktreeStatus,
}

struct MeasurementContext<'a> {
    config: &'a ShadowIndexPilotConfig,
    repo: &'a TempPilotRepo,
    tokn_commit: &'a str,
    tokn_version: &'a str,
    observed_at: &'a str,
}

impl Drop for TempPilotRepo {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

pub fn run_sanitized_shadow_index_pilot(
    config: &ShadowIndexPilotConfig,
    observed_at: &str,
    tokn_commit: &str,
    tokn_version: &str,
) -> Result<ShadowIndexPilotPair> {
    validate_pilot_config(config, observed_at, tokn_commit, tokn_version)?;
    let repo = TempPilotRepo::create(config)?;
    let project_scope_key = format!(
        "shadow-pilot-scope-{}-{}",
        std::process::id(),
        TEMP_COUNTER.fetch_add(1, Ordering::Relaxed)
    );
    let policy = DirectScanPolicy {
        max_file_bytes: config.corpus.max_file_bytes,
        max_files: config.corpus.max_files,
    };

    let baseline_build_start = Instant::now();
    let baseline_manifest = build_direct_scan_manifest(&repo.root, &project_scope_key, policy)?;
    let baseline_build_elapsed_ms = elapsed_ms(baseline_build_start);
    let baseline_build_source_bytes = baseline_manifest.eligible_bytes();
    let baseline_batch = run_query_batch(config, |query, limit| {
        let report =
            query_direct_scan_manifest(&baseline_manifest, &project_scope_key, query, limit)?;
        Ok((
            report.total_matching_files,
            report
                .hits
                .into_iter()
                .map(|hit| hit.relative_path)
                .collect(),
            report.source_bytes_read,
        ))
    })?;
    let measurement_context = MeasurementContext {
        config,
        repo: &repo,
        tokn_commit,
        tokn_version,
        observed_at,
    };
    let baseline = build_baseline_measurement(
        &measurement_context,
        &baseline_manifest,
        baseline_build_elapsed_ms,
        baseline_build_source_bytes,
        baseline_batch,
    );

    let candidate_build_start = Instant::now();
    let candidate_manifest = build_direct_scan_manifest(&repo.root, &project_scope_key, policy)?;
    ensure_same_corpus(&baseline_manifest, &candidate_manifest)?;
    let candidate_manifest_bytes = candidate_manifest.eligible_bytes();
    let candidate = SqliteFts5Unicode61Candidate::build(candidate_manifest, &project_scope_key)?;
    let candidate_build_elapsed_ms = elapsed_ms(candidate_build_start);
    let candidate_build_source_bytes =
        candidate_manifest_bytes.saturating_add(candidate.build_report().source_bytes_read);
    let candidate_batch = run_query_batch(config, |query, limit| {
        let report = candidate.query(&project_scope_key, query, limit)?;
        Ok((
            report.total_matching_files,
            report
                .hits
                .into_iter()
                .map(|hit| hit.relative_path)
                .collect(),
            report.source_bytes_read,
        ))
    })?;
    let candidate_measurement = build_candidate_measurement(
        &measurement_context,
        &baseline,
        candidate.build_report().documents_indexed,
        candidate_build_elapsed_ms,
        candidate_build_source_bytes,
        candidate_batch,
    )?;

    Ok(ShadowIndexPilotPair {
        baseline,
        candidate: candidate_measurement,
    })
}

fn validate_pilot_config(
    config: &ShadowIndexPilotConfig,
    observed_at: &str,
    tokn_commit: &str,
    tokn_version: &str,
) -> Result<()> {
    if config.schema_version != SHADOW_INDEX_PILOT_CONFIG_SCHEMA_VERSION {
        bail!("shadow index pilot config schema version is unsupported");
    }
    for value in [
        config.pilot_id.as_str(),
        config.corpus.corpus_id.as_str(),
        config.corpus.repository_ref.as_str(),
        config.corpus.policy_id.as_str(),
        config.query_set_id.as_str(),
        config.gold_set_id.as_str(),
        config.correctness_suite_id.as_str(),
        config.thresholds.thresholds_ref.as_str(),
        tokn_version,
    ] {
        if value.trim().is_empty() {
            bail!("shadow index pilot identifiers must not be empty");
        }
    }
    if parse_rfc3339_unix_ms(observed_at).is_none() {
        bail!("shadow index pilot observed_at must be valid RFC3339");
    }
    if tokn_commit.len() != 40 || !tokn_commit.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        bail!("shadow index pilot Tokn commit must be a 40-character hex SHA");
    }
    DirectScanPolicy {
        max_file_bytes: config.corpus.max_file_bytes,
        max_files: config.corpus.max_files,
    }
    .validate()?;
    if config.queries.is_empty() || config.queries.len() > SHADOW_INDEX_PILOT_MAX_QUERIES {
        bail!("shadow index pilot query count is outside the bounded limit");
    }
    if config.timing_repetitions_per_query == 0
        || config.timing_repetitions_per_query > SHADOW_INDEX_PILOT_MAX_TIMING_REPETITIONS
    {
        bail!("shadow index pilot timing repetition count is outside the bounded limit");
    }
    if config.k_values.is_empty() || !config.k_values.contains(&5) {
        bail!("shadow index pilot k_values must be non-empty and include 5");
    }
    let mut k_values = BTreeSet::new();
    for k in &config.k_values {
        if *k == 0 || !k_values.insert(*k) {
            bail!("shadow index pilot k_values must be unique positive integers");
        }
    }
    for ratio in [
        config.thresholds.min_recall_at_5_ratio_to_direct,
        config.thresholds.min_mrr_ratio_to_direct,
    ] {
        if !(0.0..=1.0).contains(&ratio) || ratio == 0.0 {
            bail!("shadow index pilot quality ratios must be in (0, 1]");
        }
    }
    if !config.thresholds.quality_floor_predeclared
        || !config.thresholds.resource_budgets_predeclared
    {
        bail!("shadow index pilot decision thresholds must be predeclared");
    }

    let mut all_paths = BTreeSet::new();
    let mut eligible_paths = BTreeSet::new();
    for file in config
        .corpus
        .tracked_files
        .iter()
        .chain(&config.corpus.untracked_files)
        .chain(&config.corpus.ignored_files)
    {
        validate_fixture_relative_path(&file.path)?;
        if !all_paths.insert(file.path.clone()) {
            bail!("shadow index pilot fixture path is duplicated");
        }
    }
    for file in config
        .corpus
        .tracked_files
        .iter()
        .chain(&config.corpus.untracked_files)
    {
        eligible_paths.insert(file.path.clone());
    }

    let mut query_ids = BTreeSet::new();
    for query in &config.queries {
        if query.query_id.trim().is_empty()
            || query.text.trim().is_empty()
            || !query_ids.insert(query.query_id.clone())
        {
            bail!("shadow index pilot queries require unique non-empty IDs and text");
        }
        let mut gold = BTreeSet::new();
        for path in &query.gold_paths {
            if !eligible_paths.contains(path) || !gold.insert(path.clone()) {
                bail!("shadow index pilot gold path is missing or duplicated");
            }
        }
        match query.query_kind {
            ShadowQueryKind::NoMatch if !query.gold_paths.is_empty() => {
                bail!("NO_MATCH pilot query must have an empty gold set");
            }
            ShadowQueryKind::NoMatch => {}
            _ if query.gold_paths.is_empty() => {
                bail!("positive pilot query must declare at least one gold path");
            }
            _ => {}
        }
    }
    Ok(())
}

impl TempPilotRepo {
    fn create(config: &ShadowIndexPilotConfig) -> Result<Self> {
        let counter = TEMP_COUNTER.fetch_add(1, Ordering::Relaxed);
        let root = std::env::temp_dir().join(format!(
            "tokn-shadow-index-pilot-{}-{counter}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).context("create shadow index pilot temp repository")?;
        run_git(&root, &["init", "--quiet"])?;
        run_git(&root, &["config", "core.autocrlf", "false"])?;

        for file in &config.corpus.tracked_files {
            write_fixture_file(&root, file)?;
        }
        let tracked_paths = config
            .corpus
            .tracked_files
            .iter()
            .map(|file| file.path.as_str())
            .collect::<Vec<_>>();
        let mut add_args = vec!["add", "--"];
        add_args.extend(tracked_paths);
        run_git(&root, &add_args)?;

        let mut commit = Command::new("git");
        commit
            .arg("-C")
            .arg(&root)
            .args([
                "-c",
                "user.name=Tokn Sanitized Fixture",
                "-c",
                "user.email=fixture@tokn.invalid",
                "commit",
                "--quiet",
                "-m",
                "shadow-index-sanitized-pilot-v1",
            ])
            .env("GIT_AUTHOR_DATE", "2026-10-04T00:00:00Z")
            .env("GIT_COMMITTER_DATE", "2026-10-04T00:00:00Z");
        let status = commit
            .status()
            .context("commit shadow index pilot fixture")?;
        if !status.success() {
            bail!("shadow index pilot fixture commit failed");
        }

        for file in &config.corpus.untracked_files {
            write_fixture_file(&root, file)?;
        }
        for file in &config.corpus.ignored_files {
            write_fixture_file(&root, file)?;
        }

        let commit = run_git(&root, &["rev-parse", "HEAD"])?;
        let commit = commit.trim().to_owned();
        if commit.len() != 40 || !commit.bytes().all(|byte| byte.is_ascii_hexdigit()) {
            bail!("shadow index pilot fixture commit is not a 40-character SHA");
        }
        let status = run_git(&root, &["status", "--porcelain"])?;
        let worktree_status = if status.trim().is_empty() {
            ShadowWorktreeStatus::Clean
        } else {
            ShadowWorktreeStatus::Dirty
        };

        Ok(Self {
            root,
            commit,
            worktree_status,
        })
    }
}

fn run_git(root: &Path, args: &[&str]) -> Result<String> {
    let output = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(args)
        .output()
        .context("execute git for shadow index sanitized pilot")?;
    if !output.status.success() {
        bail!("git command failed for shadow index sanitized pilot");
    }
    String::from_utf8(output.stdout).context("shadow index pilot git output is not UTF-8")
}

fn write_fixture_file(root: &Path, file: &ShadowIndexPilotFile) -> Result<()> {
    validate_fixture_relative_path(&file.path)?;
    let destination = root.join(&file.path);
    if let Some(parent) = destination.parent() {
        fs::create_dir_all(parent).context("create sanitized pilot fixture parent")?;
    }
    fs::write(destination, file.content.as_bytes()).context("write sanitized pilot fixture file")
}

fn validate_fixture_relative_path(path: &str) -> Result<()> {
    if path.is_empty()
        || path.contains('\\')
        || path.chars().any(char::is_control)
        || path.starts_with("//")
        || (path.len() >= 2 && path.as_bytes()[1] == b':')
        || !Path::new(path)
            .components()
            .all(|component| matches!(component, Component::Normal(_)))
    {
        bail!("shadow index pilot fixture path is unsafe");
    }
    Ok(())
}

fn ensure_same_corpus(left: &DirectScanManifest, right: &DirectScanManifest) -> Result<()> {
    if left.discovered_files != right.discovered_files
        || left.documents != right.documents
        || left.skipped != right.skipped
    {
        bail!("shadow index pilot conditions did not observe the same corpus");
    }
    Ok(())
}

fn run_query_batch<F>(config: &ShadowIndexPilotConfig, mut query_fn: F) -> Result<QueryBatch>
where
    F: FnMut(&str, usize) -> Result<(u64, Vec<String>, u64)>,
{
    let requested_k = *config
        .k_values
        .iter()
        .max()
        .context("shadow index pilot k_values missing")?;
    let mut observed_queries = Vec::with_capacity(config.queries.len());
    let mut all_latency_samples = Vec::new();
    let mut total_source_bytes_read = 0_u64;

    for query in &config.queries {
        let mut latency_samples_ms = Vec::with_capacity(config.timing_repetitions_per_query);
        let mut stable_result: Option<(u64, Vec<String>)> = None;
        for _ in 0..config.timing_repetitions_per_query {
            let started = Instant::now();
            let (total_matching_files, ranked_paths, source_bytes_read) =
                query_fn(&query.text, requested_k)?;
            let elapsed = elapsed_ms(started);
            total_source_bytes_read = total_source_bytes_read.saturating_add(source_bytes_read);
            latency_samples_ms.push(elapsed);
            all_latency_samples.push(elapsed);
            if let Some((expected_total, expected_paths)) = &stable_result {
                if *expected_total != total_matching_files || *expected_paths != ranked_paths {
                    bail!(
                        "shadow index pilot query results are not deterministic across repetitions"
                    );
                }
            } else {
                stable_result = Some((total_matching_files, ranked_paths));
            }
        }
        let (total_matching_files, ranked_paths) =
            stable_result.context("shadow index pilot query produced no timing observations")?;
        let gold_paths = query.gold_paths.iter().cloned().collect::<BTreeSet<_>>();
        let relevant_returned_count = ranked_paths
            .iter()
            .filter(|path| gold_paths.contains(*path))
            .count() as u64;
        let first_relevant_rank = ranked_paths
            .iter()
            .position(|path| gold_paths.contains(path))
            .map(|index| (index + 1) as u64);
        let reciprocal_rank = first_relevant_rank.map_or(0.0, |rank| 1.0 / rank as f64);
        let no_match_correct = match query.query_kind {
            ShadowQueryKind::NoMatch => Some(total_matching_files == 0),
            _ => None,
        };
        observed_queries.push(ObservedQuery {
            measurement: ShadowQueryMeasurement {
                query_id: query.query_id.clone(),
                query_kind: query.query_kind,
                requested_k,
                latency_samples_ms,
                expected_relevant_count: gold_paths.len() as u64,
                relevant_returned_count,
                first_relevant_rank,
                reciprocal_rank,
                no_match_correct,
            },
            ranked_paths,
            gold_paths,
        });
    }

    let aggregate = aggregate_query_measurements(config, &observed_queries, &all_latency_samples)?;
    Ok(QueryBatch {
        queries: observed_queries,
        aggregate,
        source_bytes_read: total_source_bytes_read,
    })
}

fn aggregate_query_measurements(
    config: &ShadowIndexPilotConfig,
    queries: &[ObservedQuery],
    latency_samples: &[f64],
) -> Result<ShadowAggregateMeasurement> {
    if latency_samples.is_empty() {
        bail!("shadow index pilot has no latency samples");
    }
    let positive = queries
        .iter()
        .filter(|query| query.measurement.query_kind != ShadowQueryKind::NoMatch)
        .collect::<Vec<_>>();
    if positive.is_empty() {
        bail!("shadow index pilot requires positive queries");
    }
    let mut recall_at_k = Vec::with_capacity(config.k_values.len());
    for k in &config.k_values {
        let macro_recall = positive
            .iter()
            .map(|query| {
                let relevant = query
                    .ranked_paths
                    .iter()
                    .take(*k)
                    .filter(|path| query.gold_paths.contains(*path))
                    .count();
                relevant as f64 / query.gold_paths.len() as f64
            })
            .sum::<f64>()
            / positive.len() as f64;
        recall_at_k.push(ShadowRecallAtK {
            k: *k,
            recall: macro_recall,
        });
    }
    let mrr = positive
        .iter()
        .map(|query| query.measurement.reciprocal_rank)
        .sum::<f64>()
        / positive.len() as f64;
    let no_match = queries
        .iter()
        .filter_map(|query| query.measurement.no_match_correct)
        .collect::<Vec<_>>();
    let no_match_accuracy = if no_match.is_empty() {
        None
    } else {
        Some(no_match.iter().filter(|value| **value).count() as f64 / no_match.len() as f64)
    };

    let mut sorted_latency = latency_samples.to_vec();
    sorted_latency.sort_by(f64::total_cmp);
    Ok(ShadowAggregateMeasurement {
        sample_count: latency_samples.len() as u64,
        latency_ms: ShadowLatencyPercentiles {
            p25: nearest_rank_percentile(&sorted_latency, 0.25),
            p50: nearest_rank_percentile(&sorted_latency, 0.50),
            p75: nearest_rank_percentile(&sorted_latency, 0.75),
            p95: nearest_rank_percentile(&sorted_latency, 0.95),
        },
        recall_at_k,
        mrr,
        no_match_accuracy,
    })
}

fn nearest_rank_percentile(sorted: &[f64], percentile: f64) -> f64 {
    let rank = (percentile * sorted.len() as f64).ceil() as usize;
    sorted[rank.saturating_sub(1).min(sorted.len() - 1)]
}

fn build_baseline_measurement(
    context: &MeasurementContext<'_>,
    manifest: &DirectScanManifest,
    build_elapsed_ms: f64,
    build_source_bytes: u64,
    batch: QueryBatch,
) -> ShadowIndexMeasurementV1 {
    let config = context.config;
    ShadowIndexMeasurementV1 {
        schema_version: SHADOW_INDEX_MEASUREMENT_SCHEMA_VERSION,
        measurement_id: format!("{}-direct-scan", config.pilot_id),
        run_order: 1,
        observed_at: context.observed_at.to_owned(),
        tokn: ShadowToknMeasurementIdentity {
            commit: context.tokn_commit.to_owned(),
            version: context.tokn_version.to_owned(),
        },
        corpus: corpus_measurement(config, context.repo, manifest, build_source_bytes),
        condition: ShadowConditionMeasurement {
            mode: ShadowConditionMode::DirectScan,
            backend_id: "DIRECT_SCAN_V0".to_owned(),
            tokenizer_id: None,
            configuration_id: config.corpus.policy_id.clone(),
        },
        capability: ShadowCapabilityMeasurement {
            fts5: ShadowCapabilityState::NotApplicable,
            contentless_delete: ShadowCapabilityState::NotApplicable,
            integrity_check: ShadowGateStatus::NotApplicable,
        },
        build: ShadowBuildMeasurement {
            elapsed_ms: build_elapsed_ms,
            source_bytes_read: build_source_bytes,
            index_bytes: 0,
            peak_rss_bytes: None,
            cpu_ms: None,
            files_processed: manifest.eligible_files(),
            files_skipped: manifest.skipped_files(),
        },
        refresh: None,
        query_config: query_config_measurement(config),
        queries: batch
            .queries
            .into_iter()
            .map(|query| query.measurement)
            .collect(),
        aggregate: batch.aggregate,
        correctness: incomplete_correctness(config, true),
        decision_gate: ShadowDecisionGateMeasurement {
            quality_floor_predeclared: config.thresholds.quality_floor_predeclared,
            resource_budgets_predeclared: config.thresholds.resource_budgets_predeclared,
            thresholds_ref: config.thresholds.thresholds_ref.clone(),
            resource_gate_status: ShadowGateStatus::NotApplicable,
            quality_gate_status: ShadowGateStatus::NotApplicable,
            verdict: ShadowDecisionVerdict::BaselineOnly,
            notes: vec![
                "Sanitized DIRECT_SCAN reference pilot; no backend-selection claim.".to_owned(),
                format!(
                    "Timed query samples read {} source bytes in total; V1 has no per-query byte field.",
                    batch.source_bytes_read
                ),
                "The mutation/refresh correctness suite is not executed by this pilot slice."
                    .to_owned(),
            ],
        },
    }
}

fn build_candidate_measurement(
    context: &MeasurementContext<'_>,
    baseline: &ShadowIndexMeasurementV1,
    documents_indexed: u64,
    build_elapsed_ms: f64,
    build_source_bytes: u64,
    batch: QueryBatch,
) -> Result<ShadowIndexMeasurementV1> {
    let config = context.config;
    let baseline_recall5 = recall_at_k(&baseline.aggregate, 5)?;
    let candidate_recall5 = recall_at_k(&batch.aggregate, 5)?;
    let quality_pass = candidate_recall5
        >= baseline_recall5 * config.thresholds.min_recall_at_5_ratio_to_direct
        && batch.aggregate.mrr
            >= baseline.aggregate.mrr * config.thresholds.min_mrr_ratio_to_direct;
    let quality_gate_status = if quality_pass {
        ShadowGateStatus::Pass
    } else {
        ShadowGateStatus::Fail
    };
    let verdict = if quality_pass {
        ShadowDecisionVerdict::Inconclusive
    } else {
        ShadowDecisionVerdict::Rejected
    };
    let capability = probe_sqlite_fts5_capabilities();

    let baseline_manifest = build_direct_scan_manifest(
        &context.repo.root,
        "shadow-pilot-corpus-accounting-only",
        DirectScanPolicy {
            max_file_bytes: config.corpus.max_file_bytes,
            max_files: config.corpus.max_files,
        },
    )?;
    if documents_indexed != baseline_manifest.eligible_files() {
        bail!("unicode61 pilot indexed document count does not match eligible corpus");
    }

    let mut notes = vec![
        "Sanitized unicode61 functional-candidate pilot; never ELIGIBLE_FOR_IMPLEMENTATION in this slice."
            .to_owned(),
        format!(
            "Timed candidate query samples re-verified {} source bytes in total before FTS lookup; V1 has no per-query byte field.",
            batch.source_bytes_read
        ),
        "Peak RSS/CPU, persistent index bytes and refresh cost are not captured; resource gate remains UNKNOWN."
            .to_owned(),
        "The full mutation/refresh correctness suite is not executed; correctness gate remains UNKNOWN."
            .to_owned(),
    ];
    if !quality_pass {
        notes.push(
            "The predeclared sanitized-pilot quality floor failed; this rejects this measured unicode61 condition, not every future lexical design."
                .to_owned(),
        );
    }

    Ok(ShadowIndexMeasurementV1 {
        schema_version: SHADOW_INDEX_MEASUREMENT_SCHEMA_VERSION,
        measurement_id: format!("{}-unicode61", config.pilot_id),
        run_order: 2,
        observed_at: context.observed_at.to_owned(),
        tokn: ShadowToknMeasurementIdentity {
            commit: context.tokn_commit.to_owned(),
            version: context.tokn_version.to_owned(),
        },
        corpus: corpus_measurement(config, context.repo, &baseline_manifest, build_source_bytes),
        condition: ShadowConditionMeasurement {
            mode: ShadowConditionMode::IndexedCandidate,
            backend_id: SQLITE_FTS5_UNICODE61_BACKEND_ID.to_owned(),
            tokenizer_id: Some(SQLITE_FTS5_UNICODE61_TOKENIZER_ID.to_owned()),
            configuration_id: "sqlite-fts5-unicode61-contentless-in-memory-v0".to_owned(),
        },
        capability: ShadowCapabilityMeasurement {
            fts5: capability.fts5,
            contentless_delete: capability.contentless_delete,
            integrity_check: ShadowGateStatus::Pass,
        },
        build: ShadowBuildMeasurement {
            elapsed_ms: build_elapsed_ms,
            source_bytes_read: build_source_bytes,
            index_bytes: 0,
            peak_rss_bytes: None,
            cpu_ms: None,
            files_processed: documents_indexed,
            files_skipped: baseline_manifest.skipped_files(),
        },
        refresh: None,
        query_config: query_config_measurement(config),
        queries: batch
            .queries
            .into_iter()
            .map(|query| query.measurement)
            .collect(),
        aggregate: batch.aggregate,
        correctness: incomplete_correctness(config, false),
        decision_gate: ShadowDecisionGateMeasurement {
            quality_floor_predeclared: config.thresholds.quality_floor_predeclared,
            resource_budgets_predeclared: config.thresholds.resource_budgets_predeclared,
            thresholds_ref: config.thresholds.thresholds_ref.clone(),
            resource_gate_status: ShadowGateStatus::Unknown,
            quality_gate_status,
            verdict,
            notes,
        },
    })
}

fn corpus_measurement(
    config: &ShadowIndexPilotConfig,
    repo: &TempPilotRepo,
    manifest: &DirectScanManifest,
    source_bytes_read: u64,
) -> ShadowCorpusMeasurement {
    let mut skipped_by_reason = BTreeMap::new();
    for skipped in &manifest.skipped {
        *skipped_by_reason
            .entry(skipped.reason.as_str().to_owned())
            .or_insert(0) += 1;
    }
    ShadowCorpusMeasurement {
        corpus_id: config.corpus.corpus_id.clone(),
        repository_ref: config.corpus.repository_ref.clone(),
        policy_id: config.corpus.policy_id.clone(),
        commit: repo.commit.clone(),
        worktree_status: repo.worktree_status,
        discovered_files: manifest.discovered_files,
        eligible_files: manifest.eligible_files(),
        skipped_files: manifest.skipped_files(),
        eligible_bytes: manifest.eligible_bytes(),
        source_bytes_read,
        skipped_by_reason,
    }
}

fn query_config_measurement(config: &ShadowIndexPilotConfig) -> ShadowQueryConfigMeasurement {
    ShadowQueryConfigMeasurement {
        query_count: config.queries.len() as u64,
        k_values: config.k_values.clone(),
        query_set_id: config.query_set_id.clone(),
        gold_set_id: config.gold_set_id.clone(),
        timing_repetitions_per_query: config.timing_repetitions_per_query,
        cache_state: ShadowCacheState::Mixed,
    }
}

fn incomplete_correctness(
    config: &ShadowIndexPilotConfig,
    direct_scan: bool,
) -> ShadowCorrectnessMeasurement {
    const CASES: [ShadowCorrectnessCaseId; 10] = [
        ShadowCorrectnessCaseId::TrackedContentEdit,
        ShadowCorrectnessCaseId::DirtyUnchangedHead,
        ShadowCorrectnessCaseId::AddedTrackedFile,
        ShadowCorrectnessCaseId::NonIgnoredUntrackedFile,
        ShadowCorrectnessCaseId::FileDeletion,
        ShadowCorrectnessCaseId::FileRename,
        ShadowCorrectnessCaseId::QueryAfterRefresh,
        ShadowCorrectnessCaseId::HashMismatchQueryVerification,
        ShadowCorrectnessCaseId::IndexDeleteRebuild,
        ShadowCorrectnessCaseId::GitProvenanceFallback,
    ];
    let cases = CASES
        .into_iter()
        .map(|case_id| {
            let index_delete = case_id == ShadowCorrectnessCaseId::IndexDeleteRebuild;
            ShadowCorrectnessCaseMeasurement {
                case_id,
                status: if index_delete {
                    ShadowGateStatus::NotApplicable
                } else {
                    ShadowGateStatus::Unknown
                },
                notes: if index_delete {
                    vec![if direct_scan {
                        "DIRECT_SCAN_V0 has no persistent index database.".to_owned()
                    } else {
                        "This unicode61 pilot is in-memory only and has no persistent index database."
                            .to_owned()
                    }]
                } else {
                    vec!["Not executed by the sanitized query-quality pilot slice.".to_owned()]
                },
            }
        })
        .collect();
    ShadowCorrectnessMeasurement {
        suite_id: config.correctness_suite_id.clone(),
        cases,
        gate_status: ShadowGateStatus::Unknown,
    }
}

fn recall_at_k(aggregate: &ShadowAggregateMeasurement, k: usize) -> Result<f64> {
    aggregate
        .recall_at_k
        .iter()
        .find(|row| row.k == k)
        .map(|row| row.recall)
        .context("shadow index pilot aggregate is missing required Recall@K")
}

fn elapsed_ms(started: Instant) -> f64 {
    started.elapsed().as_secs_f64() * 1_000.0
}

#[cfg(test)]
mod tests {
    use super::*;

    fn config() -> ShadowIndexPilotConfig {
        serde_json::from_str(include_str!(
            "../../../benchmarks/shadow-index-sanitized-pilot-v1.json"
        ))
        .expect("parse sanitized pilot config")
    }

    #[test]
    fn sanitized_pilot_emits_schema_v1_baseline_and_conservative_candidate_verdict() {
        let pair = run_sanitized_shadow_index_pilot(
            &config(),
            "2026-10-04T00:00:00Z",
            "1111111111111111111111111111111111111111",
            "0.1.0",
        )
        .expect("run sanitized pilot");

        assert_eq!(pair.baseline.schema_version, 1);
        assert_eq!(
            pair.baseline.condition.mode,
            ShadowConditionMode::DirectScan
        );
        assert_eq!(
            pair.baseline.decision_gate.verdict,
            ShadowDecisionVerdict::BaselineOnly
        );
        assert_eq!(
            pair.baseline.corpus.worktree_status,
            ShadowWorktreeStatus::Dirty
        );
        assert_eq!(pair.baseline.corpus.commit.len(), 40);
        assert_eq!(
            pair.baseline.corpus.discovered_files,
            pair.baseline.corpus.eligible_files + pair.baseline.corpus.skipped_files
        );

        assert_eq!(
            pair.candidate.condition.backend_id,
            SQLITE_FTS5_UNICODE61_BACKEND_ID
        );
        assert_eq!(
            pair.candidate.correctness.gate_status,
            ShadowGateStatus::Unknown
        );
        assert_eq!(
            pair.candidate.decision_gate.resource_gate_status,
            ShadowGateStatus::Unknown
        );
        assert_ne!(
            pair.candidate.decision_gate.verdict,
            ShadowDecisionVerdict::EligibleForImplementation
        );
        assert!(
            recall_at_k(&pair.candidate.aggregate, 5).expect("candidate recall")
                < recall_at_k(&pair.baseline.aggregate, 5).expect("baseline recall")
        );
        assert_eq!(
            pair.candidate.decision_gate.quality_gate_status,
            ShadowGateStatus::Fail
        );
        assert_eq!(
            pair.candidate.decision_gate.verdict,
            ShadowDecisionVerdict::Rejected
        );
    }

    #[test]
    fn pilot_config_rejects_unsafe_paths_and_unfrozen_thresholds() {
        let mut unsafe_config = config();
        unsafe_config.corpus.tracked_files[0].path = "../escape.txt".to_owned();
        assert!(
            validate_pilot_config(
                &unsafe_config,
                "2026-10-04T00:00:00Z",
                "1111111111111111111111111111111111111111",
                "0.1.0"
            )
            .is_err()
        );

        let mut thresholds = config();
        thresholds.thresholds.quality_floor_predeclared = false;
        assert!(
            validate_pilot_config(
                &thresholds,
                "2026-10-04T00:00:00Z",
                "1111111111111111111111111111111111111111",
                "0.1.0"
            )
            .is_err()
        );
    }

    #[test]
    fn measurement_serialization_uses_frozen_schema_labels() {
        assert_eq!(
            serde_json::to_string(&ShadowDecisionVerdict::BaselineOnly).expect("serialize verdict"),
            "\"BASELINE_ONLY\""
        );
        assert_eq!(
            serde_json::to_string(&ShadowConditionMode::IndexedCandidate)
                .expect("serialize condition"),
            "\"INDEXED_CANDIDATE\""
        );
        assert_eq!(
            serde_json::to_string(&ShadowQueryKind::PathOrFilename).expect("serialize query kind"),
            "\"PATH_OR_FILENAME\""
        );
    }
}
