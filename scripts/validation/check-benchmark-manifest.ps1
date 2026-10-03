param()

$ErrorActionPreference = "Stop"
$Root = (Resolve-Path (Join-Path $PSScriptRoot "..\.." )).Path
$SchemaPath = Join-Path $Root "benchmarks\manifest.schema.json"
$ExamplePath = Join-Path $Root "benchmarks\example-manifest.json"
$ShadowSchemaPath = Join-Path $Root "benchmarks\shadow-index-measurement.schema.json"
$ShadowExamplePath = Join-Path $Root "benchmarks\example-shadow-index-measurement.json"

foreach ($path in @($SchemaPath, $ExamplePath, $ShadowSchemaPath, $ShadowExamplePath)) {
    if (-not (Test-Path -LiteralPath $path -PathType Leaf)) {
        throw "benchmark manifest file missing: $path"
    }
}

$schema = Get-Content -Raw -LiteralPath $SchemaPath | ConvertFrom-Json
$example = Get-Content -Raw -LiteralPath $ExamplePath | ConvertFrom-Json
$shadowSchema = Get-Content -Raw -LiteralPath $ShadowSchemaPath | ConvertFrom-Json
$shadowRaw = Get-Content -Raw -LiteralPath $ShadowExamplePath
$shadowExample = $shadowRaw | ConvertFrom-Json

if ($schema.title -ne "Tokn BenchmarkManifest V1") {
    throw "unexpected benchmark schema title"
}
if ($example.schema_version -ne 1) {
    throw "example benchmark schema_version must be 1"
}
if ($example.condition -ne "NATIVE_BASELINE") {
    throw "example condition must stay descriptive NATIVE_BASELINE"
}

if ([string]$example.repository.commit -notmatch '^[0-9a-fA-F]{40}$') {
    throw "repository commit must be a 40-character hex SHA"
}
if ([string]$example.prompt.sha256 -notmatch '^[0-9a-fA-F]{64}$') {
    throw "prompt sha256 must be a 64-character hex digest"
}
if ([string]$example.tokn.commit -notmatch '^[0-9a-fA-F]{40}$') {
    throw "Tokn commit must be a 40-character hex SHA"
}
if ([int]$example.run_order -lt 1) {
    throw "run_order must be >= 1"
}
if (-not [DateTimeOffset]::TryParse([string]$example.observed_at, [ref]([DateTimeOffset]$parsed = [DateTimeOffset]::MinValue))) {
    throw "observed_at must be an offset-aware date-time"
}

$allowedCache = @("UNKNOWN", "RECORDED_NOT_RESET", "CONTROLLED_COLD", "CONTROLLED_WARM")
if ($allowedCache -notcontains [string]$example.cache_condition) {
    throw "invalid cache_condition"
}

if ($shadowSchema.title -ne "Tokn ShadowIndexMeasurement V1") {
    throw "unexpected shadow index measurement schema title"
}
if ($shadowExample.schema_version -ne 1) {
    throw "shadow index example schema_version must be 1"
}
if ([string]$shadowExample.condition.mode -ne "DIRECT_SCAN") {
    throw "shadow index example must remain DIRECT_SCAN baseline"
}
if ([string]$shadowExample.condition.backend_id -ne "DIRECT_SCAN_V0") {
    throw "shadow index example backend_id must remain DIRECT_SCAN_V0"
}
if ([string]$shadowExample.decision_gate.verdict -ne "BASELINE_ONLY") {
    throw "shadow index example verdict must remain BASELINE_ONLY"
}
if ([string]$shadowExample.tokn.commit -notmatch '^[0-9a-fA-F]{40}$') {
    throw "shadow index Tokn commit must be a 40-character hex SHA"
}
if ([string]$shadowExample.corpus.commit -notmatch '^[0-9a-fA-F]{40}$') {
    throw "shadow index corpus commit must be a 40-character hex SHA"
}
if (-not [DateTimeOffset]::TryParse([string]$shadowExample.observed_at, [ref]([DateTimeOffset]$shadowParsed = [DateTimeOffset]::MinValue))) {
    throw "shadow index observed_at must be an offset-aware date-time"
}
if ([int64]$shadowExample.corpus.discovered_files -ne ([int64]$shadowExample.corpus.eligible_files + [int64]$shadowExample.corpus.skipped_files)) {
    throw "shadow index corpus discovered_files must equal eligible_files + skipped_files"
}
if ([int]$shadowExample.run_order -lt 1) {
    throw "shadow index run_order must be >= 1"
}
foreach ($requiredId in @(
    [string]$shadowExample.corpus.policy_id,
    [string]$shadowExample.query_config.query_set_id,
    [string]$shadowExample.query_config.gold_set_id,
    [string]$shadowExample.correctness.suite_id
)) {
    if ([string]::IsNullOrWhiteSpace($requiredId)) {
        throw "shadow index policy/query/gold/correctness identifiers must not be empty"
    }
}
if ([int64]$shadowExample.build.files_processed -ne [int64]$shadowExample.corpus.eligible_files) {
    throw "shadow index build files_processed must match eligible_files in the baseline example"
}
if ([int64]$shadowExample.build.files_skipped -ne [int64]$shadowExample.corpus.skipped_files) {
    throw "shadow index build files_skipped must match skipped_files in the baseline example"
}
if ([int64]$shadowExample.query_config.query_count -ne @($shadowExample.queries).Count) {
    throw "shadow index query_count must match unique queries array length"
}
$timingRepetitions = [int64]$shadowExample.query_config.timing_repetitions_per_query
if ($timingRepetitions -lt 1) {
    throw "shadow index timing_repetitions_per_query must be >= 1"
}
$timingSampleCount = 0
foreach ($query in @($shadowExample.queries)) {
    $samples = @($query.latency_samples_ms)
    if ($samples.Count -ne $timingRepetitions) {
        throw "each shadow index query must carry the predeclared timing repetition count"
    }
    foreach ($sample in $samples) {
        if ([double]$sample -lt 0) {
            throw "shadow index latency samples must be >= 0"
        }
    }
    $timingSampleCount += $samples.Count
}
if ([int64]$shadowExample.aggregate.sample_count -ne $timingSampleCount) {
    throw "shadow index aggregate sample_count must equal total latency samples"
}
if (@($shadowExample.query_config.k_values).Count -lt 1) {
    throw "shadow index k_values must not be empty"
}
foreach ($k in @($shadowExample.query_config.k_values)) {
    if ([int]$k -lt 1) {
        throw "shadow index k_values must be >= 1"
    }
}
if ([int64]$shadowExample.build.index_bytes -ne 0) {
    throw "DIRECT_SCAN_V0 example must not report persistent index bytes"
}
$expectedCorrectnessCases = @(
    "TRACKED_CONTENT_EDIT",
    "DIRTY_UNCHANGED_HEAD",
    "ADDED_TRACKED_FILE",
    "NON_IGNORED_UNTRACKED_FILE",
    "FILE_DELETION",
    "FILE_RENAME",
    "QUERY_AFTER_REFRESH",
    "HASH_MISMATCH_QUERY_VERIFICATION",
    "INDEX_DELETE_REBUILD",
    "GIT_PROVENANCE_FALLBACK"
)
$observedCorrectnessCases = @($shadowExample.correctness.cases | ForEach-Object { [string]$_.case_id })
if ($observedCorrectnessCases.Count -ne $expectedCorrectnessCases.Count) {
    throw "shadow index example must contain the complete V0 correctness case set"
}
foreach ($caseId in $expectedCorrectnessCases) {
    if (($observedCorrectnessCases | Where-Object { $_ -eq $caseId }).Count -ne 1) {
        throw "shadow index correctness case missing or duplicated: $caseId"
    }
}
if ([string]$shadowExample.correctness.gate_status -ne "PASS") {
    throw "DIRECT_SCAN_V0 example correctness gate must PASS"
}
if ([string]$shadowExample.decision_gate.quality_gate_status -ne "NOT_APPLICABLE" -or
    [string]$shadowExample.decision_gate.resource_gate_status -ne "NOT_APPLICABLE") {
    throw "DIRECT_SCAN_V0 BASELINE_ONLY example must keep candidate decision gates NOT_APPLICABLE"
}
if (-not $shadowSchema.properties.correctness -or -not $shadowSchema.properties.run_order -or
    -not $shadowSchema.properties.build.properties.files_processed -or -not $shadowSchema.allOf) {
    throw "shadow index schema is missing required V1 correctness/repetition guards"
}
if ($shadowRaw -match '(?i)[A-Z]:\\') {
    throw "shadow index published example must not contain an absolute Windows drive path"
}
if (-not [bool]$shadowExample.decision_gate.quality_floor_predeclared -or -not [bool]$shadowExample.decision_gate.resource_budgets_predeclared) {
    throw "shadow index example decision gates must be predeclared"
}

Write-Host "Benchmark manifest validation: PASS"
