param([switch]$DryRun)

$ErrorActionPreference = 'Stop'
$ExperimentRoot = $PSScriptRoot
$Outer = (Resolve-Path (Join-Path $ExperimentRoot '..\..')).Path
$ToolRoot = Join-Path $Outer 'tool'
$ConfigPath = Join-Path $ExperimentRoot 'experiment.json'
$TaskPath = Join-Path $ExperimentRoot 'TASK.md'
$ActivePath = Join-Path $ExperimentRoot 'ACTIVE-RUN.json'
$SnapshotScript = Join-Path $ToolRoot 'scripts\experiment\snapshot-project.ps1'
$InventoryScript = Join-Path $ToolRoot 'scripts\experiment\inventory-workspaces.ps1'
$StartTrace = Join-Path $ToolRoot 'scripts\trace\start-jem-trace.ps1'
$ToknBin = Join-Path $ToolRoot 'target\release\tokn-observe.exe'
$BuildProvenancePath = Join-Path $ToolRoot 'target\release\tokn-observe.provenance.json'
$HarnessManifestPath = Join-Path $ExperimentRoot 'HARNESS-MANIFEST.json'

if (Test-Path -LiteralPath $ActivePath) {
    throw "Experiment 002 already has ACTIVE-RUN.json. Finish or recover it first."
}

$runningDesktop = Get-Process ChatGPT -ErrorAction SilentlyContinue
$runningCodex = Get-Process codex -ErrorAction SilentlyContinue
if (-not $DryRun -and ($runningDesktop -or $runningCodex)) {
    throw 'Close Codex/ChatGPT completely before START-EXP002.'
}

$config = Get-Content -LiteralPath $ConfigPath -Raw | ConvertFrom-Json
$ProjectRoot = [string]$config.project_root
$WatchRoot = [string]$config.workspace_watch_root
$MaxDepth = [int]$config.workspace_inventory_max_depth

if (-not (Test-Path -LiteralPath $ProjectRoot -PathType Container)) { throw "Project root missing: $ProjectRoot" }
if (-not (Test-Path -LiteralPath $WatchRoot -PathType Container)) { throw "Watch root missing: $WatchRoot" }

$head = (& git -C $ToolRoot rev-parse HEAD).Trim()
if ($head -ne [string]$config.required_tokn_commit) {
    throw "Tokn commit mismatch. Expected $($config.required_tokn_commit), got $head"
}
$dirty = (& git -C $ToolRoot status --porcelain | Out-String).Trim()
if ($dirty) { throw 'Tokn repository must be clean before Experiment 002.' }

if (-not (Test-Path -LiteralPath $HarnessManifestPath -PathType Leaf)) {
    throw "Experiment 002 harness manifest missing: $HarnessManifestPath"
}
$harnessManifest = Get-Content -LiteralPath $HarnessManifestPath -Raw | ConvertFrom-Json
if ([string]$harnessManifest.source_commit -ne $head) {
    throw "Harness source commit mismatch. Expected $head, got $($harnessManifest.source_commit)"
}
foreach ($entry in $harnessManifest.files.PSObject.Properties) {
    $path = Join-Path $ExperimentRoot $entry.Name
    if (-not (Test-Path -LiteralPath $path -PathType Leaf)) { throw "Harness file missing: $path" }
    $actual = (Get-FileHash -LiteralPath $path -Algorithm SHA256).Hash.ToLowerInvariant()
    if ($actual -ne [string]$entry.Value) { throw "Harness file drift detected: $($entry.Name)" }
}

if (-not (Test-Path -LiteralPath $ToknBin -PathType Leaf)) { throw "Release binary missing: $ToknBin" }
if (-not (Test-Path -LiteralPath $BuildProvenancePath -PathType Leaf)) {
    throw 'Release provenance missing. Run scripts\build-release.ps1 from a clean commit first.'
}
$buildProvenance = Get-Content -LiteralPath $BuildProvenancePath -Raw | ConvertFrom-Json
if ([string]$buildProvenance.source_commit -ne $head) {
    throw "Release binary provenance commit mismatch. Expected $head, got $($buildProvenance.source_commit)"
}
if (-not [bool]$buildProvenance.worktree_clean) {
    throw 'Release binary provenance says it was built from a dirty worktree.'
}
$binaryHash = (Get-FileHash -LiteralPath $ToknBin -Algorithm SHA256).Hash.ToLowerInvariant()
if ($binaryHash -ne [string]$buildProvenance.binary_sha256) {
    throw 'Release binary SHA-256 no longer matches its provenance record.'
}
$binaryVersion = (& $ToknBin --version | Out-String).Trim()
if ($binaryVersion -ne ('tokn-observe ' + [string]$config.required_tokn_version)) {
    throw "Release binary version mismatch: $binaryVersion"
}

$taskDocument = Get-Content -LiteralPath $TaskPath -Raw
$match = [regex]::Match($taskDocument, '(?s)<!-- ASTRA_PROMPT_START -->(.*?)<!-- ASTRA_PROMPT_END -->')
if (-not $match.Success) { throw 'TASK.md prompt markers are missing.' }
$prompt = $match.Groups[1].Value.Trim()
if ([string]::IsNullOrWhiteSpace($prompt)) { throw 'Experiment 002 prompt is empty.' }

$stamp = Get-Date -Format 'yyyyMMdd-HHmmss'
$suffix = $(if ($DryRun) { '-dryrun' } else { '' })
$RunRoot = Join-Path $ExperimentRoot ("runs\$stamp$suffix")
New-Item -ItemType Directory -Force -Path $RunRoot | Out-Null
Copy-Item -LiteralPath $ConfigPath -Destination (Join-Path $RunRoot 'experiment.json')
Copy-Item -LiteralPath $TaskPath -Destination (Join-Path $RunRoot 'TASK.md')
Copy-Item -LiteralPath (Join-Path $ExperimentRoot 'PRECHECK.md') -Destination (Join-Path $RunRoot 'PRECHECK.md')

$expectedFailure = [string]$config.baseline.failure
$BaselineFullLog = Join-Path $RunRoot 'baseline-full-verify.log'
Push-Location $ProjectRoot
try {
    cmd.exe /d /s /c 'npm.cmd run verify:local' *>&1 |
        Tee-Object -FilePath $BaselineFullLog | Out-Host
    $baselineFullExit = $LASTEXITCODE
} finally {
    Pop-Location
}
$baselineFullText = Get-Content -LiteralPath $BaselineFullLog -Raw
if ($baselineFullExit -eq 0) {
    throw 'The frozen Experiment 002 task is stale: verify:local now passes.'
}
if (-not $baselineFullText.Contains($expectedFailure)) {
    throw 'verify:local no longer fails for the frozen missing Quick Fix workflow reason.'
}
foreach ($expectation in @(
    @{ label='tests'; value=[int]$config.baseline.test_total },
    @{ label='pass'; value=[int]$config.baseline.test_pass },
    @{ label='fail'; value=[int]$config.baseline.test_fail }
)) {
    $pattern = '(?m)^# ' + $expectation.label + '\s+' + $expectation.value + '\s*$'
    if (-not [regex]::IsMatch($baselineFullText, $pattern)) {
        throw "verify:local baseline count drift: expected $($expectation.label)=$($expectation.value)"
    }
}

$BaselineLog = Join-Path $RunRoot 'baseline-targeted-test.log'
Push-Location $ProjectRoot
try {
    cmd.exe /d /s /c 'npm.cmd run build:test && node --test tests/b07c-quick-fix-workflow.test.cjs' *>&1 |
        Tee-Object -FilePath $BaselineLog | Out-Host
    $baselineExit = $LASTEXITCODE
} finally {
    Pop-Location
}
$baselineText = Get-Content -LiteralPath $BaselineLog -Raw
if ($baselineExit -eq 0) {
    throw 'The frozen Experiment 002 task is stale: the targeted baseline now passes.'
}
if (-not $baselineText.Contains($expectedFailure)) {
    throw 'The targeted baseline failed differently from the frozen preflight. Refuse to launch Experiment 002.'
}

$BeforeSnapshot = Join-Path $RunRoot 'project-before.json'
& $SnapshotScript -ProjectRoot $ProjectRoot -OutputPath $BeforeSnapshot
$BeforeInventory = Join-Path $RunRoot 'workspace-before.json'
& $InventoryScript -WatchRoot $WatchRoot -OutputPath $BeforeInventory -MaxDepth $MaxDepth

$taskHash = (Get-FileHash -LiteralPath $TaskPath -Algorithm SHA256).Hash.ToLowerInvariant()
$appPackage = Get-AppxPackage OpenAI.Codex -ErrorAction SilentlyContinue
$appVersion = $(if ($appPackage) { $appPackage.Version.ToString() } else { $null })
$codexCli = Join-Path $env:APPDATA 'npm\codex.cmd'
$codexVersion = $(if (Test-Path $codexCli) { (& $codexCli --version 2>$null | Out-String).Trim() } else { $null })
$nodeVersion = $(& node --version 2>$null | Out-String).Trim()
$npmVersion = $(& npm.cmd --version 2>$null | Out-String).Trim()

$label = $(if ($DryRun) { 'exp002-instrumentation-dryrun' } else { [string]$config.trace_label })
$traceArgs = @('-NoProfile','-ExecutionPolicy','Bypass','-File',$StartTrace,'-Label',$label)
if ($DryRun) { $traceArgs += '-DryRun' }
& powershell.exe @traceArgs
if ($LASTEXITCODE -ne 0) { throw "Trace launcher failed with exit code $LASTEXITCODE" }

$TraceRoot = Get-ChildItem (Join-Path $Outer 'traces') -Directory |
    Where-Object { $_.Name -like "*-$label" } |
    Sort-Object LastWriteTime -Descending |
    Select-Object -First 1 -ExpandProperty FullName
if (-not $TraceRoot) { throw "Could not locate trace root for $label" }

$run = [ordered]@{
    schema_version = 1
    experiment_id = '002-instrumentation-validation'
    started_at = (Get-Date).ToString('o')
    mode = $(if ($DryRun) { 'dryrun' } else { 'real' })
    tokn_commit = $head
    tokn_version = [string]$config.required_tokn_version
    project_root = $ProjectRoot
    workspace_watch_root = $WatchRoot
    workspace_inventory_max_depth = $MaxDepth
    expected_output_workspace = [string]$config.expected_output_workspace
    run_root = $RunRoot
    trace_root = $TraceRoot
    before_snapshot = $BeforeSnapshot
    before_inventory = $BeforeInventory
    task_path = (Join-Path $RunRoot 'TASK.md')
    task_sha256 = $taskHash
    baseline_full_exit_code = $baselineFullExit
    baseline_targeted_exit_code = $baselineExit
    baseline_expected_failure = $expectedFailure
    tokn_binary_sha256 = $binaryHash
    tokn_binary_source_commit = [string]$buildProvenance.source_commit
    harness_source_commit = [string]$harnessManifest.source_commit
    harness_manifest_sha256 = (Get-FileHash -LiteralPath $HarnessManifestPath -Algorithm SHA256).Hash.ToLowerInvariant()
    codex_app_version = $appVersion
    codex_cli_version = $codexVersion
    node_version = $nodeVersion
    npm_version = $npmVersion
}
$runJson = $run | ConvertTo-Json -Depth 8
$runJson | Set-Content -LiteralPath (Join-Path $RunRoot 'run.json') -Encoding UTF8

if ($DryRun) {
    Remove-Item -LiteralPath $TraceRoot -Recurse -Force -ErrorAction SilentlyContinue
    Remove-Item -LiteralPath $RunRoot -Recurse -Force -ErrorAction SilentlyContinue
    Write-Host ''
    Write-Host 'TOKN EXPERIMENT 002 DRY-RUN: PASS' -ForegroundColor Green
    Write-Host 'Baseline matched the frozen single failure. Snapshot/inventory/trace lifecycle passed. No Codex launch.'
    exit 0
}

$runJson | Set-Content -LiteralPath $ActivePath -Encoding UTF8
Set-Clipboard -Value $prompt

Write-Host ''
Write-Host 'TOKN EXPERIMENT 002 READY' -ForegroundColor Green
Write-Host "Run root:   $RunRoot"
Write-Host "Trace root: $TraceRoot"
Write-Host "Project:    $ProjectRoot"
Write-Host ''
Write-Host 'The frozen Experiment 002 task is in the clipboard.' -ForegroundColor Cyan
Write-Host 'In Codex:'
Write-Host '1. Open the PROJECT folder shown above.'
Write-Host '2. Paste the clipboard as ONE new Astra prompt.'
Write-Host '3. Let Astra work normally; do not ask it to be shorter or faster.'
Write-Host '4. When it is completely finished, close Codex Desktop.'
Write-Host '5. Run FINISH-EXP002.ps1.'
