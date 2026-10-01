use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use serde::{Deserialize, Serialize};
use tokn_domain::{
    HISTORICAL_SNAPSHOT_SCHEMA_VERSION, HistoricalAgentRecord, HistoricalProvenanceRecord,
    HistoricalRunRecord, HistoricalSnapshot, HistoricalWorkspaceRecord, ModelRuntimeProfile,
    TokenTotals,
};

pub const CONTEXT_LEDGER_SCHEMA_VERSION: u64 = 1;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum LedgerEvidenceStatus {
    Complete,
    Partial,
    NoEvidence,
    NotCaptured,
    #[default]
    Unknown,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct CoveredTokenMetric {
    pub observed_tokens: u64,
    pub known_records: u64,
    pub usage_records: u64,
    pub status: LedgerEvidenceStatus,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct DerivedTokenMetric {
    pub tokens: Option<u64>,
    pub status: LedgerEvidenceStatus,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContextTokenLedger {
    pub usage_records: u64,
    pub input: CoveredTokenMetric,
    pub cached_input: CoveredTokenMetric,
    pub cache_write_input: CoveredTokenMetric,
    pub output: CoveredTokenMetric,
    pub reasoning_output: CoveredTokenMetric,
    pub ordinary_uncached_input: DerivedTokenMetric,
    pub logical_total: DerivedTokenMetric,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentContextLedger {
    pub thread_id: String,
    pub parent_thread_id: Option<String>,
    pub depth: u32,
    pub terminal_status: String,
    pub duration_ms: Option<u64>,
    pub tokens: ContextTokenLedger,
    pub integrity_issues: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RunContextLedger {
    pub run_id: String,
    pub project_id: String,
    pub workspace_id: String,
    pub workspace_lineage: Vec<String>,
    pub profile_id: Option<String>,
    pub runtime_profile: Option<ModelRuntimeProfile>,
    pub measurement_contract_version: u64,
    pub evidence_layout_version: u64,
    pub root_thread_id: String,
    pub root_terminal: String,
    pub source_health: String,
    pub validity_verdict: Option<String>,
    pub quality_status: Option<String>,
    pub created_at_unix: i64,
    pub tokens: ContextTokenLedger,
    pub agents: Vec<AgentContextLedger>,
    pub provenance: Vec<HistoricalProvenanceRecord>,
    pub turn_ledger_status: LedgerEvidenceStatus,
    pub current_retained_context_tokens: Option<u64>,
    pub current_retained_context_status: LedgerEvidenceStatus,
    pub integrity_issues: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContextLedger {
    pub schema_version: u64,
    pub historical_snapshot_schema_version: u64,
    pub project_filter: Option<String>,
    pub workspace_filter: Option<String>,
    pub workspaces: Vec<HistoricalWorkspaceRecord>,
    pub runs: Vec<RunContextLedger>,
    pub turn_granularity_status: LedgerEvidenceStatus,
    pub current_retained_context_status: LedgerEvidenceStatus,
    pub integrity_issues: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ContextLedgerBuildError {
    UnsupportedHistoricalSnapshotSchema { actual: u64, expected: u64 },
}

impl fmt::Display for ContextLedgerBuildError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedHistoricalSnapshotSchema { actual, expected } => write!(
                formatter,
                "unsupported HistoricalSnapshot schema_version {actual}; expected {expected}"
            ),
        }
    }
}

impl std::error::Error for ContextLedgerBuildError {}

pub fn build_context_ledger(
    snapshot: &HistoricalSnapshot,
) -> Result<ContextLedger, ContextLedgerBuildError> {
    if snapshot.schema_version != HISTORICAL_SNAPSHOT_SCHEMA_VERSION {
        return Err(
            ContextLedgerBuildError::UnsupportedHistoricalSnapshotSchema {
                actual: snapshot.schema_version,
                expected: HISTORICAL_SNAPSHOT_SCHEMA_VERSION,
            },
        );
    }

    let workspaces = snapshot
        .workspaces
        .iter()
        .map(|workspace| (workspace.workspace_id.as_str(), workspace))
        .collect::<BTreeMap<_, _>>();
    let profiles = snapshot
        .runtime_profiles
        .iter()
        .map(|profile| (profile.profile_id.as_str(), &profile.profile))
        .collect::<BTreeMap<_, _>>();

    let mut agents_by_run = BTreeMap::<&str, Vec<&HistoricalAgentRecord>>::new();
    for agent in &snapshot.agents {
        agents_by_run
            .entry(agent.run_id.as_str())
            .or_default()
            .push(agent);
    }

    let mut provenance_by_run = BTreeMap::<&str, Vec<&HistoricalProvenanceRecord>>::new();
    for source in &snapshot.provenance {
        provenance_by_run
            .entry(source.run_id.as_str())
            .or_default()
            .push(source);
    }

    let mut runs = Vec::with_capacity(snapshot.runs.len());
    let mut ledger_issues = Vec::new();

    for run in &snapshot.runs {
        let mut issues = Vec::new();
        let tokens = token_ledger(
            &run.totals,
            run.logical_tokens,
            &format!("run {}", run.run_id),
            &mut issues,
        );

        let agent_records = agents_by_run
            .get(run.run_id.as_str())
            .cloned()
            .unwrap_or_default();
        if agent_records.len() as u64 != run.agent_count {
            issues.push(format!(
                "agent_count mismatch: persisted={} loaded={}",
                run.agent_count,
                agent_records.len()
            ));
        }

        let agent_ids = agent_records
            .iter()
            .map(|agent| agent.thread_id.as_str())
            .collect::<BTreeSet<_>>();
        if !agent_ids.contains(run.root_thread_id.as_str()) {
            issues.push(format!(
                "root thread {} is missing from loaded agents",
                run.root_thread_id
            ));
        }

        let mut agent_total = TokenTotals::default();
        let mut agents = Vec::with_capacity(agent_records.len());
        for agent in agent_records {
            let mut agent_issues = Vec::new();
            if let Some(parent) = agent.parent_thread_id.as_deref()
                && !agent_ids.contains(parent)
            {
                agent_issues.push(format!("parent thread {parent} is missing"));
            }

            let agent_tokens = token_ledger(
                &agent.totals,
                agent.logical_tokens,
                &format!("agent {}", agent.thread_id),
                &mut agent_issues,
            );
            agent_total.add_totals(&agent.totals);
            agents.push(AgentContextLedger {
                thread_id: agent.thread_id.clone(),
                parent_thread_id: agent.parent_thread_id.clone(),
                depth: agent.depth,
                terminal_status: agent.terminal_status.clone(),
                duration_ms: agent.duration_ms,
                tokens: agent_tokens,
                integrity_issues: agent_issues,
            });
        }

        if !agents.is_empty() && agent_total != run.totals {
            issues.push("run token totals do not equal the sum of loaded agent totals".into());
        }

        let workspace_lineage = workspace_lineage(run, &workspaces, &mut issues);

        let runtime_profile = match run.profile_id.as_deref() {
            Some(profile_id) => match profiles.get(profile_id) {
                Some(profile) => Some((*profile).clone()),
                None => {
                    issues.push(format!("runtime profile {profile_id} is missing"));
                    None
                }
            },
            None => None,
        };

        let provenance: Vec<HistoricalProvenanceRecord> = provenance_by_run
            .get(run.run_id.as_str())
            .map(|items| items.iter().map(|item| (*item).clone()).collect())
            .unwrap_or_default();
        if provenance.is_empty() {
            issues.push("run has no persisted provenance".into());
        }

        for agent in &agents {
            for issue in &agent.integrity_issues {
                issues.push(format!("agent {}: {issue}", agent.thread_id));
            }
        }

        for issue in &issues {
            ledger_issues.push(format!("run {}: {issue}", run.run_id));
        }

        runs.push(RunContextLedger {
            run_id: run.run_id.clone(),
            project_id: run.project_id.clone(),
            workspace_id: run.workspace_id.clone(),
            workspace_lineage,
            profile_id: run.profile_id.clone(),
            runtime_profile,
            measurement_contract_version: run.measurement_contract_version,
            evidence_layout_version: run.evidence_layout_version,
            root_thread_id: run.root_thread_id.clone(),
            root_terminal: run.root_terminal.clone(),
            source_health: run.source_health.clone(),
            validity_verdict: run.validity_verdict.clone(),
            quality_status: run.quality_status.clone(),
            created_at_unix: run.created_at_unix,
            tokens,
            agents,
            provenance,
            turn_ledger_status: LedgerEvidenceStatus::NotCaptured,
            current_retained_context_tokens: None,
            current_retained_context_status: LedgerEvidenceStatus::Unknown,
            integrity_issues: issues,
        });
    }

    Ok(ContextLedger {
        schema_version: CONTEXT_LEDGER_SCHEMA_VERSION,
        historical_snapshot_schema_version: snapshot.schema_version,
        project_filter: snapshot.project_filter.clone(),
        workspace_filter: snapshot.workspace_filter.clone(),
        workspaces: snapshot.workspaces.clone(),
        runs,
        turn_granularity_status: LedgerEvidenceStatus::NotCaptured,
        current_retained_context_status: LedgerEvidenceStatus::Unknown,
        integrity_issues: ledger_issues,
    })
}

fn token_ledger(
    totals: &TokenTotals,
    persisted_logical: Option<u64>,
    scope: &str,
    issues: &mut Vec<String>,
) -> ContextTokenLedger {
    let input = covered_metric(
        totals.input_tokens,
        totals.input_known,
        totals.usage_records,
    );
    let cached_input = covered_metric(
        totals.cached_input_tokens,
        totals.cached_input_known,
        totals.usage_records,
    );
    let cache_write_input = covered_metric(
        totals.cache_write_input_tokens,
        totals.cache_write_input_known,
        totals.usage_records,
    );
    let output = covered_metric(
        totals.output_tokens,
        totals.output_known,
        totals.usage_records,
    );
    let reasoning_output = covered_metric(
        totals.reasoning_output_tokens,
        totals.reasoning_output_known,
        totals.usage_records,
    );

    let uncached = if totals.usage_records == 0 {
        DerivedTokenMetric {
            tokens: None,
            status: LedgerEvidenceStatus::NoEvidence,
        }
    } else if input.status == LedgerEvidenceStatus::Complete
        && cached_input.status == LedgerEvidenceStatus::Complete
    {
        match totals.input_tokens.checked_sub(totals.cached_input_tokens) {
            Some(tokens) => DerivedTokenMetric {
                tokens: Some(tokens),
                status: LedgerEvidenceStatus::Complete,
            },
            None => {
                issues.push(format!(
                    "{scope} cached input exceeds input; uncached is unknown"
                ));
                DerivedTokenMetric {
                    tokens: None,
                    status: LedgerEvidenceStatus::Unknown,
                }
            }
        }
    } else {
        DerivedTokenMetric {
            tokens: None,
            status: combined_derived_status(input.status, cached_input.status),
        }
    };

    let derived_logical = totals.logical_total();
    let logical_status = if totals.usage_records == 0 {
        LedgerEvidenceStatus::NoEvidence
    } else if derived_logical.is_some() {
        LedgerEvidenceStatus::Complete
    } else {
        combined_derived_status(input.status, output.status)
    };
    if persisted_logical != derived_logical {
        issues.push(format!(
            "{scope} persisted logical total {:?} differs from derived {:?}",
            persisted_logical, derived_logical
        ));
    }

    ContextTokenLedger {
        usage_records: totals.usage_records,
        input,
        cached_input,
        cache_write_input,
        output,
        reasoning_output,
        ordinary_uncached_input: uncached,
        logical_total: DerivedTokenMetric {
            tokens: derived_logical,
            status: logical_status,
        },
    }
}

fn covered_metric(
    observed_tokens: u64,
    known_records: u64,
    usage_records: u64,
) -> CoveredTokenMetric {
    let status = if usage_records == 0 {
        LedgerEvidenceStatus::NoEvidence
    } else if known_records == usage_records {
        LedgerEvidenceStatus::Complete
    } else if known_records == 0 {
        LedgerEvidenceStatus::NotCaptured
    } else {
        LedgerEvidenceStatus::Partial
    };

    CoveredTokenMetric {
        observed_tokens,
        known_records,
        usage_records,
        status,
    }
}

fn combined_derived_status(
    left: LedgerEvidenceStatus,
    right: LedgerEvidenceStatus,
) -> LedgerEvidenceStatus {
    use LedgerEvidenceStatus::{Complete, NoEvidence, NotCaptured, Partial, Unknown};

    match (left, right) {
        (NoEvidence, _) | (_, NoEvidence) => NoEvidence,
        (Unknown, _) | (_, Unknown) => Unknown,
        (NotCaptured, _) | (_, NotCaptured) => NotCaptured,
        (Partial, _) | (_, Partial) => Partial,
        (Complete, Complete) => Complete,
    }
}

fn workspace_lineage(
    run: &HistoricalRunRecord,
    workspaces: &BTreeMap<&str, &HistoricalWorkspaceRecord>,
    issues: &mut Vec<String>,
) -> Vec<String> {
    let mut lineage = Vec::new();
    let mut seen = BTreeSet::new();
    let mut current = Some(run.workspace_id.as_str());

    while let Some(workspace_id) = current {
        if !seen.insert(workspace_id.to_string()) {
            issues.push(format!(
                "workspace lineage cycle detected at {workspace_id}"
            ));
            break;
        }

        lineage.push(workspace_id.to_string());
        let Some(workspace) = workspaces.get(workspace_id) else {
            issues.push(format!("workspace {workspace_id} is missing"));
            break;
        };
        if workspace.project_id != run.project_id {
            issues.push(format!(
                "workspace {workspace_id} belongs to project {}, expected {}",
                workspace.project_id, run.project_id
            ));
        }
        current = workspace.parent_workspace_id.as_deref();
    }

    lineage
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokn_domain::{
        HISTORICAL_SNAPSHOT_SCHEMA_VERSION, HistoricalRuntimeProfileRecord,
        MODEL_RUNTIME_PROFILE_SCHEMA_VERSION, RuntimeConfigurationProfile, ValidityCheckStatus,
    };

    fn complete_totals(input: u64, cached: u64, output: u64) -> TokenTotals {
        TokenTotals {
            usage_records: 1,
            input_tokens: input,
            cached_input_tokens: cached,
            cache_write_input_tokens: 10,
            output_tokens: output,
            reasoning_output_tokens: 5,
            input_known: 1,
            cached_input_known: 1,
            cache_write_input_known: 1,
            output_known: 1,
            reasoning_output_known: 1,
        }
    }

    fn run(totals: TokenTotals) -> HistoricalRunRecord {
        HistoricalRunRecord {
            run_id: "run-1".into(),
            project_id: "prj-1".into(),
            workspace_id: "wsp-child".into(),
            profile_id: Some("rtp-1".into()),
            measurement_contract_version: 1,
            evidence_layout_version: 1,
            root_thread_id: "thread-root".into(),
            root_terminal: "COMPLETED".into(),
            source_health: "HEALTHY".into(),
            validity_verdict: Some("INSTRUMENTATION_ONLY".into()),
            quality_status: Some("PASS".into()),
            agent_count: 1,
            logical_tokens: totals.logical_total(),
            totals,
            created_at_unix: 1,
        }
    }

    fn profile() -> ModelRuntimeProfile {
        ModelRuntimeProfile {
            schema_version: MODEL_RUNTIME_PROFILE_SCHEMA_VERSION,
            observed_at: "2026-10-01T20:00:00Z".into(),
            runtime_kind: "codex".into(),
            runtime_version: Some("fixture".into()),
            app_version: None,
            model: Some("fixture-model".into()),
            model_provider: Some("openai".into()),
            reasoning_effort: None,
            model_context_window: Some(200_000),
            multi_agent_protocol_version: None,
            configuration: RuntimeConfigurationProfile::default(),
            configuration_complete: ValidityCheckStatus::Unknown,
            capabilities: Default::default(),
        }
    }

    fn snapshot(totals: TokenTotals) -> HistoricalSnapshot {
        let run = run(totals.clone());
        HistoricalSnapshot {
            schema_version: HISTORICAL_SNAPSHOT_SCHEMA_VERSION,
            project_filter: Some("prj-1".into()),
            workspace_filter: None,
            workspaces: vec![
                HistoricalWorkspaceRecord {
                    workspace_id: "wsp-root".into(),
                    project_id: "prj-1".into(),
                    parent_workspace_id: None,
                    snapshot_fingerprint: Some("root".into()),
                    identity_version: 1,
                    created_at_unix: 0,
                },
                HistoricalWorkspaceRecord {
                    workspace_id: "wsp-child".into(),
                    project_id: "prj-1".into(),
                    parent_workspace_id: Some("wsp-root".into()),
                    snapshot_fingerprint: Some("child".into()),
                    identity_version: 1,
                    created_at_unix: 1,
                },
            ],
            runs: vec![run.clone()],
            agents: vec![HistoricalAgentRecord {
                run_id: run.run_id.clone(),
                thread_id: "thread-root".into(),
                parent_thread_id: None,
                depth: 0,
                terminal_status: "COMPLETED".into(),
                duration_ms: Some(500),
                logical_tokens: totals.logical_total(),
                totals,
            }],
            provenance: vec![HistoricalProvenanceRecord {
                source_id: "src-1".into(),
                run_id: run.run_id,
                source_kind: "codex-session".into(),
                source_fingerprint: "fingerprint".into(),
                snapshot_bytes: Some(100),
                adapter_name: Some("fixture".into()),
                adapter_version: Some("1".into()),
                created_at_unix: 1,
            }],
            runtime_profiles: vec![HistoricalRuntimeProfileRecord {
                profile_id: "rtp-1".into(),
                profile: profile(),
            }],
        }
    }

    #[test]
    fn complete_ledger_preserves_accounting_and_unknown_context() {
        let ledger = build_context_ledger(&snapshot(complete_totals(100, 70, 20))).unwrap();
        let run = &ledger.runs[0];

        assert_eq!(run.tokens.input.observed_tokens, 100);
        assert_eq!(run.tokens.cached_input.observed_tokens, 70);
        assert_eq!(run.tokens.ordinary_uncached_input.tokens, Some(30));
        assert_eq!(
            run.tokens.ordinary_uncached_input.status,
            LedgerEvidenceStatus::Complete
        );
        assert_eq!(run.tokens.logical_total.tokens, Some(120));
        assert_eq!(
            run.workspace_lineage,
            vec!["wsp-child".to_string(), "wsp-root".to_string()]
        );
        assert_eq!(run.turn_ledger_status, LedgerEvidenceStatus::NotCaptured);
        assert_eq!(run.current_retained_context_tokens, None);
        assert_eq!(
            run.current_retained_context_status,
            LedgerEvidenceStatus::Unknown
        );
        assert!(run.integrity_issues.is_empty());
        assert!(ledger.integrity_issues.is_empty());
    }

    #[test]
    fn partial_cache_coverage_does_not_invent_uncached_tokens() {
        let totals = TokenTotals {
            usage_records: 2,
            input_tokens: 200,
            cached_input_tokens: 70,
            output_tokens: 20,
            input_known: 2,
            cached_input_known: 1,
            output_known: 2,
            ..Default::default()
        };
        let mut snapshot = snapshot(totals.clone());
        snapshot.runs[0].logical_tokens = totals.logical_total();
        snapshot.agents[0].logical_tokens = totals.logical_total();

        let ledger = build_context_ledger(&snapshot).unwrap();
        let tokens = &ledger.runs[0].tokens;
        assert_eq!(tokens.cached_input.status, LedgerEvidenceStatus::Partial);
        assert_eq!(tokens.ordinary_uncached_input.tokens, None);
        assert_eq!(
            tokens.ordinary_uncached_input.status,
            LedgerEvidenceStatus::Partial
        );
    }

    #[test]
    fn integrity_mismatches_are_reported_not_repaired() {
        let mut snapshot = snapshot(complete_totals(100, 70, 20));
        snapshot.runs[0].agent_count = 2;
        snapshot.runs[0].logical_tokens = Some(999);

        let ledger = build_context_ledger(&snapshot).unwrap();
        let issues = &ledger.runs[0].integrity_issues;
        assert!(
            issues
                .iter()
                .any(|item| item.contains("agent_count mismatch"))
        );
        assert!(
            issues
                .iter()
                .any(|item| item.contains("persisted logical total"))
        );
        assert_eq!(ledger.runs[0].tokens.logical_total.tokens, Some(120));
    }

    #[test]
    fn unknown_snapshot_schema_fails_closed() {
        let mut snapshot = snapshot(complete_totals(100, 70, 20));
        snapshot.schema_version = 99;

        assert!(matches!(
            build_context_ledger(&snapshot),
            Err(ContextLedgerBuildError::UnsupportedHistoricalSnapshotSchema { actual: 99, .. })
        ));
    }
}
