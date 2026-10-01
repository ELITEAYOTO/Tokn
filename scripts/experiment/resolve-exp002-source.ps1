param(
    [Parameter(Mandatory = $true)][string]$TraceRoot,
    [Parameter(Mandatory = $true)][string]$ToknBin,
    [Parameter(Mandatory = $true)][string]$AfterInventory,
    [Parameter(Mandatory = $true)][string]$BeforeInventory,
    [Parameter(Mandatory = $true)][string]$SourceRoot,
    [Parameter(Mandatory = $true)][string]$ExpectedOutput,
    [Parameter(Mandatory = $true)][string]$OutputPath,
    [string[]]$SessionRoots
)

$ErrorActionPreference = 'Stop'
if (-not $SessionRoots -or $SessionRoots.Count -eq 0) {
    $SessionRoots = @(
        (Join-Path $HOME '.codex\sessions'),
        (Join-Path $HOME '.codex\archived_sessions')
    )
}

function Invoke-WorkspaceResolution {
    param([string]$Source, [string]$JsonPath)
    Remove-Item -LiteralPath $JsonPath -Force -ErrorAction SilentlyContinue
    $args = @(
        'resolve-workspace', $Source,
        '--inventory', $AfterInventory,
        '--before-inventory', $BeforeInventory,
        '--source-root', $SourceRoot,
        '--expected-output', $ExpectedOutput,
        '--output-json', $JsonPath
    )
    $previousErrorAction = $ErrorActionPreference
    $ErrorActionPreference = 'Continue'
    try {
        & $ToknBin @args 2>$null | Out-Null
        $exitCode = $LASTEXITCODE
    }
    finally {
        $ErrorActionPreference = $previousErrorAction
    }
    if ($exitCode -ne 0 -or -not (Test-Path -LiteralPath $JsonPath -PathType Leaf)) { return $null }
    $resolution = Get-Content -LiteralPath $JsonPath -Raw | ConvertFrom-Json
    if ([string]$resolution.status -ne 'SELECTED' -or [string]::IsNullOrWhiteSpace([string]$resolution.selected_root)) { return $null }
    return $resolution
}

function Read-SessionMeta {
    param([string]$Path)
    try {
        $first = [System.IO.File]::ReadLines($Path) | Select-Object -First 1
        if ([string]::IsNullOrWhiteSpace($first)) { return $null }
        $record = $first | ConvertFrom-Json
        if ([string]$record.type -ne 'session_meta') { return $null }
        return $record.payload
    }
    catch { return $null }
}
$resolutionPath = Join-Path (Split-Path $OutputPath -Parent) 'workspace-resolution-pre-runner.json'
$diagnosticBundles = New-Object System.Collections.Generic.List[string]
if (Test-Path -LiteralPath (Join-Path $TraceRoot 'manifest.json') -PathType Leaf) { $diagnosticBundles.Add($TraceRoot) }
if (Test-Path -LiteralPath $TraceRoot -PathType Container) {
    foreach ($dir in @(Get-ChildItem -LiteralPath $TraceRoot -Directory -Filter 'trace-*' -ErrorAction SilentlyContinue)) {
        if (Test-Path -LiteralPath (Join-Path $dir.FullName 'manifest.json') -PathType Leaf) { $diagnosticBundles.Add($dir.FullName) }
    }
}

foreach ($bundle in @($diagnosticBundles | Sort-Object -Unique)) {
    $resolution = Invoke-WorkspaceResolution -Source $bundle -JsonPath $resolutionPath
    if ($null -ne $resolution) {
        $report = [ordered]@{
            schema_version = 1
            selected_source = $bundle
            source_kind = 'diagnostic'
            session_candidates = @()
            fallback_used = $false
            fallback_reason = $null
            diagnostic_trace_root = $TraceRoot
            diagnostic_bundle = $bundle
            workspace_resolution = $resolutionPath
            selected_workspace = [string]$resolution.selected_root
        }
        $report | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath $OutputPath -Encoding UTF8
        return
    }
}

$rootThreadIds = New-Object System.Collections.Generic.HashSet[string]
foreach ($bundle in @($diagnosticBundles | Sort-Object -Unique)) {
    try {
        $manifest = Get-Content -LiteralPath (Join-Path $bundle 'manifest.json') -Raw | ConvertFrom-Json
        $root = [string]$manifest.root_thread_id
        if (-not [string]::IsNullOrWhiteSpace($root)) { [void]$rootThreadIds.Add($root) }
    }
    catch {}
}
if ($rootThreadIds.Count -ne 1) { throw "Unable to derive exactly one root_thread_id from diagnostic evidence. Found $($rootThreadIds.Count)." }
$rootThreadId = @($rootThreadIds)[0]

$sessionFiles = New-Object System.Collections.Generic.List[string]
foreach ($root in $SessionRoots) {
    if (-not (Test-Path -LiteralPath $root -PathType Container)) { continue }
    foreach ($file in @(Get-ChildItem -LiteralPath $root -Recurse -File -Filter '*.jsonl' -ErrorAction SilentlyContinue)) { $sessionFiles.Add($file.FullName) }
}
$parents = New-Object System.Collections.Generic.List[string]
$candidates = New-Object System.Collections.Generic.List[string]
foreach ($file in @($sessionFiles | Sort-Object -Unique)) {
    $meta = Read-SessionMeta -Path $file
    if ($null -eq $meta) { continue }
    $id = [string]$meta.id
    $sessionId = [string]$meta.session_id
    $parentId = [string]$meta.parent_thread_id
    if ($id -eq $rootThreadId -and [string]::IsNullOrWhiteSpace($parentId)) {
        $parents.Add($file)
        continue
    }
    if ($parentId -eq $rootThreadId -or ($sessionId -eq $rootThreadId -and $id -ne $rootThreadId)) { $candidates.Add($file) }
}
if ($parents.Count -ne 1) { throw "Expected exactly one persisted parent rollout for $rootThreadId; found $($parents.Count)." }
$parent = $parents[0]
$resolution = Invoke-WorkspaceResolution -Source $parent -JsonPath $resolutionPath
if ($null -eq $resolution) { throw "Persisted session fallback could not resolve a safe output workspace for $rootThreadId." }

$bundleValue = $null
if ($diagnosticBundles.Count -eq 1) { $bundleValue = $diagnosticBundles[0] }
$report = [ordered]@{
    schema_version = 1
    selected_source = $parent
    source_kind = 'codex-session'
    session_candidates = @($candidates | Sort-Object -Unique)
    fallback_used = $true
    fallback_reason = 'Diagnostic evidence was present but not directly consumable by the V0.1 workspace resolver; a persisted healthy session source was used.'
    root_thread_id = $rootThreadId
    diagnostic_trace_root = $TraceRoot
    diagnostic_bundle = $bundleValue
    workspace_resolution = $resolutionPath
    selected_workspace = [string]$resolution.selected_root
}
$report | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath $OutputPath -Encoding UTF8
