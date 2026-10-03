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

Assert-Contains "README.md" "**P0-P9 DONE ; Experiment 002 ACCEPTED ; Measurement Contract V1 FROZEN ; Store foundation DONE ; Local MCP transport prototype ACCEPTED ; Historical Analyzer + Context Ledger IN PROGRESS.**"
Assert-Contains "STATUS.md" "Historical Analyzer + Context Ledger est IN PROGRESS"
Assert-Contains "STATUS.md" 'GitHub governance : `main` est actuellement non protegee (`protected=false`)'
Assert-Contains "ROADMAP.md" "P9 golden replay/release validation : DONE"
Assert-Contains "docs/plans/V0.1-IMPLEMENTATION-PLAN.md" "Status: **DONE - P0-P9 DONE**"
Assert-Contains "docs/plans/V0.1-TEST-MATRIX.md" "Status: **P0-P9 PASS / V0.1 DONE**"
Assert-Contains "docs/plans/NEXT-SESSION-CHECKLIST.md" "P0-P9 : DONE"
Assert-Contains "docs/plans/NEXT-SESSION-CHECKLIST.md" "Measurement Contract V1 : DONE / FROZEN"
Assert-Contains "docs/plans/NEXT-SESSION-CHECKLIST.md" "Store foundation + ModelRuntimeProfile persistence : DONE"
Assert-Contains "docs/plans/NEXT-SESSION-CHECKLIST.md" "Local MCP transport prototype : DONE / ACCEPTED"
Assert-Contains "docs/plans/NEXT-SESSION-CHECKLIST.md" "Historical Analyzer + Context Ledger : IN PROGRESS"
Assert-Contains "docs/plans/IMPLEMENTATION-PATH.md" "Historical Analyzer + Context Ledger IN PROGRESS"
Assert-Contains "docs/plans/IMPLEMENTATION-PATH.md" "Status: DONE / TRANSPORT PROTOTYPE ACCEPTED"
Assert-Contains "docs/plans/IMPLEMENTATION-PATH.md" "Status: IN PROGRESS"
Assert-Contains "docs/experiments/002-INSTRUMENTATION-VALIDATION.md" "Status: DONE / ACCEPTED"
Assert-Contains "docs/design/MEASUREMENT-CONTRACT-V0.1.md" "Status: FROZEN V1 CONTRACT"
Assert-Contains "docs/design/MODEL-RUNTIME-PROFILE.md" "Status: FROZEN V1 DOMAIN CONTRACT / PERSISTENCE IMPLEMENTED"
Assert-Contains "docs/design/TOKN-STORE-V2.md" "Status: FOUNDATION IMPLEMENTED"
Assert-Contains "docs/design/LOCAL-MCP-PROTOTYPE.md" "Status: TRANSPORT PROTOTYPE ACCEPTED / DIRECT HOST TOOL CALL VALIDATED"
Assert-Contains "docs/design/LOCAL-MCP-PROTOTYPE.md" "tokn_context_ledger"
Assert-Contains "docs/design/LOCAL-MCP-PROTOTYPE.md" 'turn granularity `NOT_CAPTURED`'
Assert-Contains "docs/design/LOCAL-MCP-PROTOTYPE.md" 'retained-context status `UNKNOWN`'
Assert-Contains "docs/design/HISTORICAL-CONTEXT-LEDGER.md" "Status: M4 IN PROGRESS / CONTEXT LEDGER V1 CORE ACCEPTED"
Assert-Contains "docs/design/HISTORICAL-CONTEXT-LEDGER.md" "turn granularity is NOT_CAPTURED"
Assert-Contains "docs/design/HISTORICAL-CONTEXT-LEDGER.md" "retained-context status remains UNKNOWN"
Assert-Contains "docs/design/PLUGIN-ENGINE-INTEGRATION.md" "Status: LOCAL STDIO TRANSPORT PROTOTYPE ACCEPTED"
Assert-Contains "docs/INDEX.md" "**P0-P9 DONE ; Experiment 002 ACCEPTED ; Measurement Contract V1 FROZEN ; Store foundation DONE ; Local MCP transport prototype ACCEPTED ; Historical Analyzer + Context Ledger IN PROGRESS.**"
Assert-Contains "README.md" "Codex/Astra est le premier runtime valide, pas une dependance du domaine Tokn."
Assert-Contains "ROADMAP.md" "## Cross-cutting multi-runtime gates"
Assert-Contains "docs/decisions/ADR-006-MULTI-RUNTIME-CORE-ADAPTER-BOUNDARY.md" "Status: ACCEPTED"
Assert-Contains "docs/design/MULTI-RUNTIME-CORE-AND-DATA-ARCHITECTURE.md" "one analytical brain, many runtime translators"
Assert-Contains "docs/design/TARGET-ARCHITECTURE.md" "Runtime -> Runtime Adapter -> Normalized Contracts -> Tokn Core/Engine -> Tokn Store -> Query/Views"
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
Assert-Contains "docs/plans/NEXT-SESSION-CHECKLIST.md" "Cross-Run Source Re-read Evidence V0: ACCEPTED CHRONOLOGY FOUNDATION."
Assert-Contains "docs/plans/NEXT-SESSION-CHECKLIST.md" "Task Input Identity V0: ACCEPTED OBSERVATION FOUNDATION."
Assert-Contains "docs/plans/NEXT-SESSION-CHECKLIST.md" "Cross-Run Comparability V0 et Task Input Identity V0 sont ACCEPTED OBSERVATION FOUNDATIONS"
Assert-Contains "docs/plans/NEXT-SESSION-CHECKLIST.md" "ne construire une preuve de livraison runtime que depuis un vrai evenement rollout directement observe et fixtureable"
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
Assert-Contains "docs/MAINTENANCE.md" "Historical Analyzer + Context Ledger IN PROGRESS"
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
