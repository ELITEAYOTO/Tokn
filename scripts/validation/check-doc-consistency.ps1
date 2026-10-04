param()

$ErrorActionPreference = "Stop"
$Repo = (Resolve-Path (Join-Path $PSScriptRoot "..\..")).Path

function Read-RepoFile {
    param([string]$RelativePath)
    $path = Join-Path $Repo $RelativePath
    if (-not (Test-Path -LiteralPath $path)) {
        throw "required documentation file missing: $RelativePath"
    }
    Get-Content -LiteralPath $path -Raw
}

function Assert-Contains {
    param([string]$File, [string]$Text)
    $content = Read-RepoFile $File
    if (-not $content.Contains($Text)) {
        throw "$File missing expected text: $Text"
    }
    Write-Host "[PASS] $File contains: $Text"
}

function Assert-NotContains {
    param([string]$File, [string]$Text)
    $content = Read-RepoFile $File
    if ($content.Contains($Text)) {
        throw "$File contains stale text: $Text"
    }
}

$Cargo = Read-RepoFile "Cargo.toml"
if ($Cargo -notmatch '(?m)^version = "0\.1\.0"\r?$') {
    throw "Cargo workspace version is not 0.1.0"
}
Write-Host "[PASS] Cargo workspace version = 0.1.0"

Assert-Contains "README.md" "**P0-P9 DONE ; Experiment 002 ACCEPTED ; Measurement Contract V1 FROZEN ; Store foundation DONE ; Local MCP transport prototype ACCEPTED ; M4 Historical Analyzer + Context Ledger CORE ACCEPTED / EVIDENCE-GATED EXTENSIONS ; M4.5 Context Identity + Shadow Foundations STARTED / OBSERVATION-ONLY.**"
Assert-Contains "STATUS.md" "M4 Historical Analyzer + Context Ledger est CORE ACCEPTED avec EXTENSIONS EVIDENCE-GATED"
Assert-Contains "STATUS.md" 'GitHub governance : `main` est actuellement non protegee (`protected=false`)'
Assert-Contains "ROADMAP.md" "P9 golden replay/release validation : DONE"
Assert-Contains "docs/plans/V0.1-IMPLEMENTATION-PLAN.md" "Status: **DONE - P0-P9 DONE**"
Assert-Contains "docs/plans/V0.1-TEST-MATRIX.md" "Status: **P0-P9 PASS / V0.1 DONE**"
Assert-Contains "docs/plans/NEXT-SESSION-CHECKLIST.md" "P0-P9 : DONE"
Assert-Contains "docs/plans/NEXT-SESSION-CHECKLIST.md" "Measurement Contract V1 : DONE / FROZEN"
Assert-Contains "docs/plans/NEXT-SESSION-CHECKLIST.md" "Store foundation + ModelRuntimeProfile persistence : DONE"
Assert-Contains "docs/plans/NEXT-SESSION-CHECKLIST.md" "Local MCP transport prototype : DONE / ACCEPTED"
Assert-Contains "docs/plans/NEXT-SESSION-CHECKLIST.md" "Historical Analyzer + Context Ledger : CORE ACCEPTED / EVIDENCE-GATED EXTENSIONS"
Assert-Contains "docs/plans/IMPLEMENTATION-PATH.md" "M4 Historical Analyzer + Context Ledger core is **ACCEPTED**"
Assert-Contains "docs/plans/IMPLEMENTATION-PATH.md" "Status: DONE / TRANSPORT PROTOTYPE ACCEPTED"
Assert-Contains "ROADMAP.md" "Statut : CORE ACCEPTED / EVIDENCE-GATED EXTENSIONS"
Assert-Contains "docs/experiments/002-INSTRUMENTATION-VALIDATION.md" "Status: DONE / ACCEPTED"
Assert-Contains "docs/design/MEASUREMENT-CONTRACT-V0.1.md" "Status: FROZEN V1 CONTRACT"
Assert-Contains "docs/design/MODEL-RUNTIME-PROFILE.md" "Status: FROZEN V1 DOMAIN CONTRACT / PERSISTENCE IMPLEMENTED"
Assert-Contains "docs/design/TOKN-STORE-V2.md" "Status: FOUNDATION IMPLEMENTED"
Assert-Contains "docs/design/LOCAL-MCP-PROTOTYPE.md" "Status: TRANSPORT PROTOTYPE ACCEPTED / DIRECT HOST TOOL CALL VALIDATED"
Assert-Contains "docs/design/LOCAL-MCP-PROTOTYPE.md" "tokn_context_ledger"
Assert-Contains "docs/design/LOCAL-MCP-PROTOTYPE.md" 'turn granularity `NOT_CAPTURED`'
Assert-Contains "docs/design/LOCAL-MCP-PROTOTYPE.md" 'retained-context status `UNKNOWN`'
Assert-Contains "docs/design/HISTORICAL-CONTEXT-LEDGER.md" "Status: M4 CORE ACCEPTED / EVIDENCE-GATED EXTENSIONS"
Assert-Contains "docs/design/HISTORICAL-CONTEXT-LEDGER.md" "turn granularity is NOT_CAPTURED"
Assert-Contains "docs/design/HISTORICAL-CONTEXT-LEDGER.md" "retained-context status remains UNKNOWN"
Assert-Contains "docs/design/PLUGIN-ENGINE-INTEGRATION.md" "Status: LOCAL STDIO TRANSPORT PROTOTYPE ACCEPTED"
Assert-Contains "docs/INDEX.md" "**P0-P9 DONE ; Experiment 002 ACCEPTED ; Measurement Contract V1 FROZEN ; Store foundation DONE ; Local MCP transport prototype ACCEPTED ; M4 Historical Analyzer + Context Ledger CORE ACCEPTED / EVIDENCE-GATED EXTENSIONS ; M4.5 Context Identity + Shadow Foundations STARTED / OBSERVATION-ONLY.**"
Assert-Contains "README.md" "Codex/Astra est le premier runtime valide, pas une dependance du domaine Tokn."
Assert-Contains "ROADMAP.md" "## Cross-cutting multi-runtime gates"
Assert-Contains "docs/decisions/ADR-006-MULTI-RUNTIME-CORE-ADAPTER-BOUNDARY.md" "Status: ACCEPTED"
Assert-Contains "docs/design/MULTI-RUNTIME-CORE-AND-DATA-ARCHITECTURE.md" "one analytical brain, many runtime translators"
Assert-Contains "docs/design/TARGET-ARCHITECTURE.md" "Runtime -> Runtime Adapter -> Normalized Contracts -> Tokn Core/Engine -> Tokn Store -> Query/Views"
Assert-Contains "docs/design/ARCHITECTURE-OVERVIEW.md" "tokn-shadow : mecanique shadow DERIVED/rebuildable et probes backend, separes du Measurement Store"
Assert-Contains "docs/plans/NEXT-SESSION-CHECKLIST.md" "SourceStableId"
Assert-Contains "docs/design/MODEL-RUNTIME-PROFILE.md" "Token Semantics V2"
Assert-Contains "docs/design/TOKN-STORE-V2.md" "## Current next boundary"
Assert-Contains "docs/design/CONTEXT-RESULT-IDENTITY.md" "Status: M4 ACCEPTED FOUNDATION / M4.5 SEED"
Assert-Contains "docs/design/SOURCE-IDENTITY-CONTENT-EVOLUTION.md" "Status: M4 ACCEPTED FOUNDATION / M4.5 SEED"
Assert-Contains "docs/design/SOURCE-MUTATION-OBSERVATION.md" "## Source Mutation Window V0"
Assert-Contains "docs/design/SOURCE-MUTATION-OBSERVATION.md" 'Every window keeps `causality_status = NOT_PROVEN`.'
Assert-Contains "docs/design/WORKSPACE-GIT-PROVENANCE.md" "Status: ACCEPTED FOUNDATION / OBSERVATION-ONLY (2026-10-03)."
Assert-Contains "docs/design/SOURCE-FRESHNESS-EVIDENCE.md" "Status: M4 ACCEPTED CORROBORATION FOUNDATION / OBSERVATION-ONLY"
Assert-Contains "docs/design/SOURCE-FRESHNESS-EVIDENCE.md" 'freshness_status = NOT_PROVEN'
Assert-Contains "docs/design/SOURCE-FRESHNESS-EVIDENCE.md" 'invalidation_status = NOT_PROVEN'
Assert-Contains "docs/design/SOURCE-FRESHNESS-EVIDENCE.md" '`tokn-observe source-freshness-evidence`'
Assert-Contains "docs/design/CROSS-RUN-COMPARABILITY.md" "Status: M4 ACCEPTED OBSERVATION FOUNDATION / OBSERVATION-ONLY"
Assert-Contains "docs/design/CROSS-RUN-COMPARABILITY.md" '`causal_claims_status = NOT_ESTABLISHED`'
Assert-Contains "docs/design/CROSS-RUN-COMPARABILITY.md" '`tokn-observe cross-run-comparison`'
Assert-Contains "docs/design/CROSS-RUN-COMPARABILITY.md" 'Detailed compaction chronology remains `NOT_CAPTURED`'
Assert-Contains "docs/design/TASK-INPUT-IDENTITY.md" "Status: M4 ACCEPTED OBSERVATION FOUNDATION / ARTIFACT-IDENTITY ONLY"
Assert-Contains "docs/design/TASK-INPUT-IDENTITY.md" '`delivery_status = NOT_PROVEN`'
Assert-Contains "docs/design/TASK-INPUT-IDENTITY.md" '`tokn-observe task-input-history`'
Assert-Contains "docs/design/TASK-INPUT-IDENTITY.md" '`causal_claims_status = NOT_ESTABLISHED`'
Assert-Contains "docs/design/SOURCE-REREAD-EVIDENCE.md" "Status: M4 ACCEPTED CHRONOLOGY FOUNDATION / OBSERVATION-ONLY"
Assert-Contains "docs/design/SOURCE-REREAD-EVIDENCE.md" '`rediscovery_status = NOT_PROVEN`'
Assert-Contains "docs/design/SOURCE-REREAD-EVIDENCE.md" '`redundancy_status = NOT_PROVEN`'
Assert-Contains "docs/design/SOURCE-REREAD-EVIDENCE.md" '`freshness_status = NOT_PROVEN`'
Assert-Contains "docs/design/SOURCE-REREAD-EVIDENCE.md" '`tokn-observe source-reread-evidence`'
Assert-Contains "docs/design/SOURCE-REREAD-EVIDENCE.md" '`run_created_at_unix`'
Assert-Contains "docs/design/M4-EXIT-GATE.md" "Status: ACCEPTED / CORE COMPLETE WITH EVIDENCE-GATED EXTENSIONS"
Assert-Contains "docs/design/M4-EXIT-GATE.md" 'semantic phase timeline: `NOT_CAPTURED`'
Assert-Contains "docs/design/M4-EXIT-GATE.md" 'runtime task delivery: `NOT_PROVEN`'
Assert-Contains "docs/design/M4-EXIT-GATE.md" "Shadow Repository Index V0 design + measurement contract"
Assert-Contains "docs/design/SHADOW-REPOSITORY-INDEX.md" "Status: M4.5 ACCEPTED DESIGN / PREREQUISITES IMPLEMENTED / DIRECT_SCAN ACCEPTED REFERENCE FOUNDATION"
Assert-Contains "docs/design/SHADOW-REPOSITORY-INDEX.md" "DIRECT_SCAN_V0"
Assert-Contains "docs/design/SHADOW-REPOSITORY-INDEX.md" "Do not duplicate the derivation algorithm in a second crate."
Assert-Contains "docs/design/SHADOW-REPOSITORY-INDEX.md" "tokn-domain::identity"
Assert-Contains "docs/design/SHADOW-REPOSITORY-INDEX.md" "ixc-v1-*"
Assert-Contains "docs/design/SHADOW-REPOSITORY-INDEX.md" "observer_shadow_index_root()"
Assert-Contains "docs/design/SHADOW-REPOSITORY-INDEX.md" "contentless FTS"
Assert-Contains "docs/benchmarks/SHADOW-INDEX-PROTOCOL-V0.md" "Status: M4.5 ACCEPTED PRE-IMPLEMENTATION MEASUREMENT CONTRACT"
Assert-Contains "docs/benchmarks/SHADOW-INDEX-PROTOCOL-V0.md" "SourceStableId derivation reuse: **RESOLVED**"
Assert-Contains "docs/benchmarks/SHADOW-INDEX-PROTOCOL-V0.md" "direct-scan reference semantics + sanitized Git fixture: **ACCEPTED REFERENCE FOUNDATION / FULL GATES PASS**"
Assert-Contains "docs/benchmarks/SHADOW-INDEX-PROTOCOL-V0.md" "ELIGIBLE_FOR_IMPLEMENTATION"
Assert-Contains "benchmarks/shadow-index-measurement.schema.json" "Tokn ShadowIndexMeasurement V1"
Assert-Contains "ROADMAP.md" "Shadow Repository Index V0 : ACCEPTED DESIGN / PREREQUISITES IMPLEMENTED / DIRECT_SCAN ACCEPTED REFERENCE FOUNDATION"
Assert-Contains "docs/plans/IMPLEMENTATION-PATH.md" "## Phase G.5 - Context Identity + Shadow Foundations"
Assert-Contains "docs/plans/IMPLEMENTATION-PATH.md" "Status: STARTED / OBSERVATION-ONLY"
Assert-Contains "docs/plans/IMPLEMENTATION-PATH.md" "shared SourceStableId derivation: **DONE**"
Assert-Contains "docs/plans/NEXT-SESSION-CHECKLIST.md" "Premier scope M4.5 : **ACCEPTED DESIGN / DIRECT_SCAN ACCEPTED REFERENCE FOUNDATION**."
Assert-Contains "docs/plans/NEXT-SESSION-CHECKLIST.md" 'SourceStableId shared derivation outside `tokn-storage`: **DONE**'
Assert-Contains "docs/plans/NEXT-SESSION-CHECKLIST.md" "DIRECT_SCAN_V0 reference semantics: **ACCEPTED REFERENCE FOUNDATION / FULL GATES PASS**"
Assert-Contains "docs/design/SHADOW-REPOSITORY-INDEX.md" "No backend is selected by this design."
Assert-Contains "docs/design/SHADOW-REPOSITORY-INDEX.md" "DIRECT_SCAN ACCEPTED REFERENCE FOUNDATION"
Assert-Contains "docs/design/SHADOW-REPOSITORY-INDEX.md" "100,000 discovered files"
Assert-Contains "docs/design/SHADOW-REPOSITORY-INDEX.md" "REFRESH_REQUIRED"
Assert-Contains "docs/benchmarks/SHADOW-INDEX-PROTOCOL-V0.md" "tokn-platform::direct_scan"
Assert-Contains "docs/plans/NEXT-SESSION-CHECKLIST.md" "DIRECT_SCAN_V0 reference semantics: **ACCEPTED REFERENCE FOUNDATION / FULL GATES PASS**"
Assert-Contains "docs/plans/NEXT-SESSION-CHECKLIST.md" "Cross-Run Source Re-read Evidence V0: ACCEPTED CHRONOLOGY FOUNDATION."
Assert-Contains "docs/plans/NEXT-SESSION-CHECKLIST.md" "Task Input Identity V0: ACCEPTED OBSERVATION FOUNDATION."
Assert-Contains "docs/plans/NEXT-SESSION-CHECKLIST.md" "M4 exit boundary :"
Assert-Contains "docs/plans/NEXT-SESSION-CHECKLIST.md" "Premier scope M4.5 :"
Assert-Contains "docs/plans/NEXT-SESSION-CHECKLIST.md" "DIRECT_SCAN_V0 est **ACCEPTED REFERENCE FOUNDATION**"
Assert-Contains "STATUS.md" "capability probe SQLite/FTS5 est **ACCEPTED CAPABILITY FOUNDATION / FULL GATES PASS**"
Assert-Contains "docs/design/SHADOW-REPOSITORY-INDEX.md" 'Accepted capability implementation lives in the dedicated `tokn-shadow` crate'
Assert-Contains "docs/design/SHADOW-REPOSITORY-INDEX.md" '`extension_loading_attempted=false`'
Assert-Contains "docs/design/SHADOW-REPOSITORY-INDEX.md" '`fallback_backend_id=DIRECT_SCAN_V0`'
Assert-Contains "docs/benchmarks/SHADOW-INDEX-PROTOCOL-V0.md" '`tokn-observe shadow-index-capabilities`'
Assert-Contains "docs/plans/IMPLEMENTATION-PATH.md" "FTS5 capability probe: **ACCEPTED CAPABILITY FOUNDATION / FULL GATES PASS**"
Assert-Contains "docs/plans/NEXT-SESSION-CHECKLIST.md" "SQLite FTS5 capability probe: **ACCEPTED CAPABILITY FOUNDATION / FULL GATES PASS**"
Assert-Contains "docs/INDEX.md" 'probe SQLite/FTS5 est ACCEPTED CAPABILITY FOUNDATION'
Assert-Contains "STATUS.md" '`SQLITE_FTS5_UNICODE61_V0` reste **FUNCTIONAL BENCHMARK CANDIDATE / FULL GATES PASS**'
Assert-Contains "docs/design/SHADOW-REPOSITORY-INDEX.md" '`sdoc-v1-*`'
Assert-Contains "docs/design/SHADOW-REPOSITORY-INDEX.md" 'every query re-verifies the complete current manifest'
Assert-Contains "docs/design/SHADOW-REPOSITORY-INDEX.md" 'functional benchmark candidate, not an eligible backend'
Assert-Contains "docs/benchmarks/SHADOW-INDEX-PROTOCOL-V0.md" '`SQLITE_FTS5_UNICODE61_V0`: **FUNCTIONAL BENCHMARK CANDIDATE / FULL GATES PASS**'
Assert-Contains "docs/plans/IMPLEMENTATION-PATH.md" '`SQLITE_FTS5_UNICODE61_V0`: **FUNCTIONAL BENCHMARK CANDIDATE / FULL GATES PASS**'
Assert-Contains "docs/plans/NEXT-SESSION-CHECKLIST.md" '`SQLITE_FTS5_UNICODE61_V0`: **FUNCTIONAL BENCHMARK CANDIDATE / FULL GATES PASS**'
Assert-Contains "ROADMAP.md" '`SQLITE_FTS5_UNICODE61_V0`: FUNCTIONAL BENCHMARK CANDIDATE / FULL GATES PASS'
Assert-Contains "ROADMAP.md" "sanitized Shadow Index pilot: IMPLEMENTED / FULL GATES PASS"
Assert-Contains "STATUS.md" "unicode61**. Ce verdict ne selectionne aucun autre backend"
Assert-Contains "docs/benchmarks/SHADOW-INDEX-PROTOCOL-V0.md" "sanitized pilot harness: **IMPLEMENTED / FULL GATES PASS**"
Assert-Contains "docs/plans/NEXT-SESSION-CHECKLIST.md" "sanitized Shadow Index pilot: **IMPLEMENTED / FULL GATES PASS**"
Assert-Contains "docs/plans/IMPLEMENTATION-PATH.md" "sanitized pilot harness: **IMPLEMENTED / FULL GATES PASS**"
Assert-Contains "docs/design/SHADOW-REPOSITORY-INDEX.md" 'the measured condition is `REJECTED`'
Assert-Contains "docs/audits/2026-10-04-CLAUDE-TECHNICAL-AUDIT-RECONCILIATION.md" "NO REMEDIATION APPLIED"
Assert-Contains "STATUS.md" "Wave A remediation is in progress: R-01 JSONL early-EOF/shrink infinite-loop risk is fixed and merged"
Assert-Contains "STATUS.md" "D-01 SQLite"
Assert-Contains "SECURITY.md" "this is not a cryptographic-secrecy guarantee"
Assert-Contains "docs/plans/NEXT-SESSION-CHECKLIST.md" "Wave A integrity remediation is now in progress."
Assert-Contains "docs/INDEX.md" "2026-10-04-CLAUDE-TECHNICAL-AUDIT-RECONCILIATION.md"
Assert-Contains "ROADMAP.md" '`tokn_status` + `tokn_recent_runs` + `tokn_context_ledger`'
Assert-Contains "docs/plans/IMPLEMENTATION-PATH.md" "Task Input Identity V0 ACCEPTED OBSERVATION FOUNDATION"
Assert-NotContains "docs/plans/NEXT-SESSION-CHECKLIST.md" "Cross-Run Comparability V0 est le slice courant"
Assert-NotContains "ROADMAP.md" "decouvre les deux outils"
Assert-Contains "docs/plans/NEXT-SESSION-CHECKLIST.md" "Workspace Git Provenance V0 + WorkspaceGitProvenanceHistory V1: ACCEPTED FOUNDATION."
Assert-Contains "docs/plans/NEXT-SESSION-CHECKLIST.md" "Source Mutation Window V0: ACCEPTED CHRONOLOGY FOUNDATION."
Assert-Contains "docs/plans/NEXT-SESSION-CHECKLIST.md" "Source Freshness Evidence V0:"
Assert-Contains "docs/plans/NEXT-SESSION-CHECKLIST.md" "SourceStableId file V0 + SourceIdentityHistory V1: ACCEPTED FOUNDATION."
Assert-Contains "docs/design/CROSS-AGENT-EVIDENCE.md" "# Cross-Agent Evidence V2"
Assert-Contains "docs/plans/NEXT-SESSION-CHECKLIST.md" "Context/Result Identity Foundation V0 : ACCEPTED FOUNDATION."
Assert-Contains "docs/design/CONTEXT-RESULT-IDENTITY.md" "## Transient result identity capture hardening"
Assert-Contains "docs/strategy/CONTEXT-EFFICIENCY-STRATEGY.md" "## Buffer / replay architecture gates - 2026-10-03"
Assert-Contains "README.md" "## Quickstart developpeur"
Assert-Contains "SECURITY.md" "## Packaging"
Assert-Contains "docs/MAINTENANCE.md" "M4 Historical Analyzer + Context Ledger CORE ACCEPTED / EVIDENCE-GATED EXTENSIONS"
Assert-Contains "docs/reference/PRIVACY.md" "## Current durable boundary"
Assert-NotContains "docs/reference/PRIVACY.md" "V0.0 does not:"
Assert-NotContains "docs/MAINTENANCE.md" "Ne pas lancer Experiment 002 avant P9"
Assert-Contains "docs/operations/DEVELOPMENT-WORKFLOW.md" "## Merge rule"
Assert-Contains "docs/benchmarks/BASELINE-PROTOCOL-V1.md" "Status: ACCEPTED PROTOCOL / BASELINE-ONLY"
Assert-Contains "docs/benchmarks/BASELINE-PROTOCOL-V1.md" "Experiment 003 remains the first causal A/B optimization experiment."
Assert-Contains "benchmarks/manifest.schema.json" "Tokn BenchmarkManifest V1"

$activeDocs = @(
    "README.md",
    "STATUS.md",
    "ROADMAP.md",
    "docs/plans/V0.1-IMPLEMENTATION-PLAN.md",
    "docs/plans/V0.1-TEST-MATRIX.md",
    "docs/plans/NEXT-SESSION-CHECKLIST.md",
    "docs/plans/IMPLEMENTATION-PATH.md",
    "docs/experiments/002-INSTRUMENTATION-VALIDATION.md",
    "docs/design/MEASUREMENT-CONTRACT-V0.1.md",
    "docs/design/MODEL-RUNTIME-PROFILE.md",
    "docs/design/TOKN-STORE-V2.md",
    "docs/design/LOCAL-MCP-PROTOTYPE.md",
    "docs/design/HISTORICAL-CONTEXT-LEDGER.md",
    "docs/design/M4-EXIT-GATE.md",
    "docs/design/SHADOW-REPOSITORY-INDEX.md",
    "docs/benchmarks/SHADOW-INDEX-PROTOCOL-V0.md",
    "docs/design/SOURCE-REREAD-EVIDENCE.md",
    "docs/design/PLUGIN-ENGINE-INTEGRATION.md",
    "docs/design/TARGET-ARCHITECTURE.md",
    "docs/design/MULTI-RUNTIME-CORE-AND-DATA-ARCHITECTURE.md",
    "docs/design/WORKSPACE-GIT-PROVENANCE.md",
    "docs/design/SOURCE-FRESHNESS-EVIDENCE.md",
    "docs/design/CROSS-RUN-COMPARABILITY.md",
    "docs/design/TASK-INPUT-IDENTITY.md",
    "docs/design/CONTEXT-RESULT-IDENTITY.md",
    "docs/design/CROSS-AGENT-EVIDENCE.md",
    "docs/decisions/ADR-006-MULTI-RUNTIME-CORE-ADAPTER-BOUNDARY.md",
    "docs/MAINTENANCE.md",
    "docs/reference/PRIVACY.md",
    "docs/operations/DEVELOPMENT-WORKFLOW.md",
    "docs/benchmarks/BASELINE-PROTOCOL-V1.md",
    "SECURITY.md",
    "docs/INDEX.md"
)
$stale = @(
    "P9 est IN PROGRESS",
    "P9 : IN PROGRESS",
    "P9 : TODO",
    "P8 : IN PROGRESS",
    "BLOCKED BY P8-P9",
    "BLOCKED BY V0.1 P7-P9",
    "P0-P7 DONE ; P8 NEXT",
    "P0-P8 DONE / P9 IN PROGRESS",
    "Experiment 002 NEXT",
    "Measurement Contract Freeze NEXT",
    "Store + ModelRuntimeProfile : NEXT",
    "Store + ModelRuntimeProfile NEXT",
    "PERSISTENCE PENDING",
    "Local MCP prototype NEXT",
    "Local MCP integration prototype : NEXT",
    "DIRECT HOST TOOL CALL PENDING",
    "direct host tool-call execution is not claimed",
    "Historical Analyzer NEXT",
    "Historical Analyzer est NEXT",
    "Historical Analyzer + Context Ledger : NEXT",
    "Historical Analyzer + Context Ledger IN PROGRESS",
    "M4 IN PROGRESS / CONTEXT LEDGER V1 CORE ACCEPTED",
    "## Phase G - Historical Analyzer + Context Ledger`n`nStatus: IN PROGRESS",
    "Context Ledger is NEXT",
    "P0-P7 DONE / P8 NEXT",
    "Cross-Agent Evidence V1",
    'result identity remains NOT_CAPTURED',
    "result identity still NOT_CAPTURED"
)

foreach ($file in $activeDocs) {
    foreach ($needle in $stale) {
        Assert-NotContains $file $needle
    }
}

Write-Host "Documentation consistency: PASS"
