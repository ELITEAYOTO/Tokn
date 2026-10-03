param()

$ErrorActionPreference = "Stop"
$Root = (Resolve-Path (Join-Path $PSScriptRoot "..\.." )).Path
$SchemaPath = Join-Path $Root "benchmarks\manifest.schema.json"
$ExamplePath = Join-Path $Root "benchmarks\example-manifest.json"

foreach ($path in @($SchemaPath, $ExamplePath)) {
    if (-not (Test-Path -LiteralPath $path -PathType Leaf)) {
        throw "benchmark manifest file missing: $path"
    }
}

$schema = Get-Content -Raw -LiteralPath $SchemaPath | ConvertFrom-Json
$example = Get-Content -Raw -LiteralPath $ExamplePath | ConvertFrom-Json

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

Write-Host "Benchmark manifest validation: PASS"
