param(
    [string]$ExperimentRoot,
    [Parameter(Mandatory = $true)][string]$ProjectRoot,
    [Parameter(Mandatory = $true)][string]$WorkspaceWatchRoot,
    [string]$ExpectedOutputWorkspace
)

$ErrorActionPreference = 'Stop'
$ToolRoot = (Resolve-Path (Join-Path $PSScriptRoot '..\..')).Path
$CanonicalRoot = Join-Path $PSScriptRoot '002'
$OuterRoot = Split-Path $ToolRoot -Parent
$LauncherRoot = Join-Path $OuterRoot 'launchers'
if ([string]::IsNullOrWhiteSpace($ExperimentRoot)) {
    $ExperimentRoot = Join-Path $OuterRoot 'experiments\002-instrumentation-validation'
}
if ([string]::IsNullOrWhiteSpace($ExpectedOutputWorkspace)) {
    $ExpectedOutputWorkspace = $ProjectRoot
}

$dirty = (& git -C $ToolRoot status --porcelain | Out-String).Trim()
if ($dirty) {
    throw 'Experiment 002 harness may only be provisioned from a clean Git worktree.'
}
$head = (& git -C $ToolRoot rev-parse HEAD).Trim()
$short = (& git -C $ToolRoot rev-parse --short HEAD).Trim()
if ([string]::IsNullOrWhiteSpace($head)) { throw 'Unable to resolve Tokn Git HEAD.' }

$active = Join-Path $ExperimentRoot 'ACTIVE-RUN.json'
if (Test-Path -LiteralPath $active) {
    throw 'Refusing to replace the Experiment 002 harness while ACTIVE-RUN.json exists.'
}
New-Item -ItemType Directory -Force -Path $ExperimentRoot | Out-Null
New-Item -ItemType Directory -Force -Path (Join-Path $ExperimentRoot 'runs') | Out-Null
New-Item -ItemType Directory -Force -Path $LauncherRoot | Out-Null

foreach ($name in @('START-EXP002.ps1','FINISH-EXP002.ps1','RECOVER-EXP002.ps1','TASK.md')) {
    Copy-Item -LiteralPath (Join-Path $CanonicalRoot $name) -Destination (Join-Path $ExperimentRoot $name) -Force
}

$config = Get-Content -LiteralPath (Join-Path $CanonicalRoot 'experiment.template.json') -Raw | ConvertFrom-Json
$config.required_tokn_commit = $head
$config.tokn_repo = $ToolRoot
$config.project_root = $ProjectRoot
$config.workspace_watch_root = $WorkspaceWatchRoot
$config.expected_output_workspace = $ExpectedOutputWorkspace
$config | ConvertTo-Json -Depth 10 | Set-Content -LiteralPath (Join-Path $ExperimentRoot 'experiment.json') -Encoding UTF8

$precheck = Get-Content -LiteralPath (Join-Path $CanonicalRoot 'PRECHECK.template.md') -Raw
$precheck = $precheck.Replace('__DATE__', (Get-Date -Format 'yyyy-MM-dd'))
$precheck = $precheck.Replace('__TOKN_COMMIT__', $head)
$precheck = $precheck.Replace('__TOKN_COMMIT_SHORT__', $short)
$precheck = $precheck.Replace('__PROJECT_ROOT__', $ProjectRoot)
$precheck = $precheck.Replace('__WORKSPACE_WATCH_ROOT__', $WorkspaceWatchRoot)
$precheck | Set-Content -LiteralPath (Join-Path $ExperimentRoot 'PRECHECK.md') -Encoding UTF8
$launchers = @{
    'START-EXP002.cmd' = 'START-EXP002.ps1'
    'FINISH-EXP002.cmd' = 'FINISH-EXP002.ps1'
    'RECOVER-EXP002.cmd' = 'RECOVER-EXP002.ps1'
}
foreach ($entry in $launchers.GetEnumerator()) {
    $script = Join-Path $ExperimentRoot $entry.Value
    $body = "@echo off`r`npowershell.exe -NoProfile -ExecutionPolicy Bypass -File `"$script`"`r`npause`r`n"
    Set-Content -LiteralPath (Join-Path $LauncherRoot $entry.Key) -Value $body -NoNewline -Encoding ASCII
}

$owned = @(
    'START-EXP002.ps1',
    'FINISH-EXP002.ps1',
    'RECOVER-EXP002.ps1',
    'TASK.md',
    'PRECHECK.md',
    'experiment.json'
)
$hashes = [ordered]@{}
foreach ($name in $owned) {
    $hashes[$name] = (Get-FileHash -LiteralPath (Join-Path $ExperimentRoot $name) -Algorithm SHA256).Hash.ToLowerInvariant()
}
$manifest = [ordered]@{
    schema_version = 1
    experiment_id = '002-instrumentation-validation'
    generated_at = (Get-Date).ToString('o')
    source_commit = $head
    source_branch = (& git -C $ToolRoot branch --show-current).Trim()
    files = $hashes
}
$manifestPath = Join-Path $ExperimentRoot 'HARNESS-MANIFEST.json'
$manifest | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath $manifestPath -Encoding UTF8

Write-Host 'Experiment 002 harness provisioned from clean Git.'
Write-Host "Commit: $head"
Write-Host "Harness: $ExperimentRoot"
Write-Host "Manifest: $manifestPath"
