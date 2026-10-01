$ErrorActionPreference = 'Stop'
$ExperimentRoot = $PSScriptRoot
$Outer = (Resolve-Path (Join-Path $ExperimentRoot '..\..')).Path
$ToolRoot = Join-Path $Outer 'tool'
$ConfigPath = Join-Path $ExperimentRoot 'experiment.json'
$ActivePath = Join-Path $ExperimentRoot 'ACTIVE-RUN.json'
$InventoryScript = Join-Path $ToolRoot 'scripts\experiment\inventory-workspaces.ps1'
$SnapshotScript = Join-Path $ToolRoot 'scripts\experiment\snapshot-project.ps1'
$ToknBin = Join-Path $ToolRoot 'target\release\tokn-observe.exe'
$RuntimeObservationScript = Join-Path $ToolRoot 'scripts\experiment\capture-runtime-observation.ps1'

function Clear-ToknTraceEnvironment {
    Add-Type @"
using System;
using System.Runtime.InteropServices;
public static class ToknExp002EnvBroadcast {
    [DllImport("user32.dll", CharSet = CharSet.Auto, SetLastError = true)]
    public static extern IntPtr SendMessageTimeout(
        IntPtr hWnd, uint Msg, UIntPtr wParam, string lParam,
        uint fuFlags, uint uTimeout, out UIntPtr lpdwResult);
}
"@ -ErrorAction SilentlyContinue
    [Environment]::SetEnvironmentVariable('CODEX_ROLLOUT_TRACE_ROOT', $null, 'User')
    $result = [UIntPtr]::Zero
    [void][ToknExp002EnvBroadcast]::SendMessageTimeout(
        [IntPtr]0xffff, 0x001A, [UIntPtr]::Zero, 'Environment', 2, 5000, [ref]$result)
}

$runningDesktop = Get-Process ChatGPT -ErrorAction SilentlyContinue
$runningCodex = Get-Process codex -ErrorAction SilentlyContinue
if ($runningDesktop -or $runningCodex) {
    throw 'Close Codex/ChatGPT completely before FINISH-EXP002.'
}
if (-not (Test-Path -LiteralPath $ActivePath)) { throw 'No ACTIVE-RUN.json for Experiment 002.' }
if (-not (Test-Path -LiteralPath $ToknBin -PathType Leaf)) { throw "Tokn release binary missing: $ToknBin" }

$config = Get-Content -LiteralPath $ConfigPath -Raw | ConvertFrom-Json
$run = Get-Content -LiteralPath $ActivePath -Raw | ConvertFrom-Json
$RunRoot = [string]$run.run_root
$TraceRoot = [string]$run.trace_root
$ProjectRoot = [string]$run.project_root
$WatchRoot = [string]$run.workspace_watch_root
$MaxDepth = [int]$run.workspace_inventory_max_depth

Clear-ToknTraceEnvironment

$RuntimeObservationPath = Join-Path $RunRoot 'runtime-observation.json'
& $RuntimeObservationScript -EvidenceRoot $TraceRoot -OutputPath $RuntimeObservationPath -CodexCliVersion ([string]$run.codex_cli_version) -CodexAppVersion ([string]$run.codex_app_version)
if ($LASTEXITCODE -ne 0) { throw "Runtime observation failed with exit code $LASTEXITCODE" }
$runtimeObservation = Get-Content -LiteralPath $RuntimeObservationPath -Raw | ConvertFrom-Json

$AfterInventory = Join-Path $RunRoot 'workspace-after.json'
& $InventoryScript -WatchRoot $WatchRoot -OutputPath $AfterInventory -MaxDepth $MaxDepth

$ResolutionPath = Join-Path $RunRoot 'workspace-resolution-pre-runner.json'
& $ToknBin resolve-workspace $TraceRoot --inventory $AfterInventory --before-inventory $run.before_inventory --source-root $ProjectRoot --output-json $ResolutionPath
if ($LASTEXITCODE -ne 0) { throw "Workspace resolver failed with exit code $LASTEXITCODE" }
$resolution = Get-Content -LiteralPath $ResolutionPath -Raw | ConvertFrom-Json
if ([string]$resolution.status -ne 'SELECTED' -or [string]::IsNullOrWhiteSpace([string]$resolution.selected_root)) {
    throw "Workspace resolution did not select exactly one output. Status=$($resolution.status)"
}
$SelectedWorkspace = [string]$resolution.selected_root

$AfterSnapshot = Join-Path $RunRoot 'project-after.json'
& $SnapshotScript -ProjectRoot $SelectedWorkspace -OutputPath $AfterSnapshot

$EvidenceDir = Join-Path $RunRoot 'evidence'
$RunnerRequestPath = Join-Path $RunRoot 'runner-request.json'
$runnerRequest = [ordered]@{
    schema_version = 1
    run_id = ('exp002-' + (Split-Path $RunRoot -Leaf))
    source = $TraceRoot
    source_root = $ProjectRoot
    after_inventory = $AfterInventory
    before_inventory = [string]$run.before_inventory
    before_snapshot = [string]$run.before_snapshot
    after_snapshot = $AfterSnapshot
    expected_outputs = @()
    evidence_dir = $EvidenceDir
    experiment = [ordered]@{
        experiment_id = '002-instrumentation-validation'
        intent = 'INSTRUMENTATION'
        validity = [ordered]@{
            exact_task_captured = 'PASS'
            completion_required = $true
            tool_evidence_required = $true
            policy_enforcement_accurately_labeled = 'NOT_REQUIRED'
            model_recorded = [string]$runtimeObservation.model_recorded
            configuration_recorded = [string]$runtimeObservation.configuration_recorded
            comparable_to_baseline = 'UNKNOWN'
            human_or_host_gates = 'NOT_REQUIRED'
            baseline_available = 'FAIL'
            same_task = 'UNKNOWN'
            same_starting_workspace = 'UNKNOWN'
            same_runtime_model_config = 'UNKNOWN'
            single_primary_variable = 'FAIL'
        }
    }
    quality = [ordered]@{
        required = $true
        program = 'npm.cmd'
        args = @('run','verify:local')
    }
}
$runnerRequest | ConvertTo-Json -Depth 12 | Set-Content -LiteralPath $RunnerRequestPath -Encoding UTF8

Push-Location $ToolRoot
try {
    & $ToknBin runner $RunnerRequestPath
    $runnerExit = $LASTEXITCODE
} finally {
    Pop-Location
}
if ($runnerExit -ne 0) { throw "Tokn Runner failed with exit code $runnerExit" }

$RunnerResultPath = Join-Path $EvidenceDir 'runner-result.json'
$result = Get-Content -LiteralPath $RunnerResultPath -Raw | ConvertFrom-Json
$validityInput = Get-Content -LiteralPath (Join-Path $EvidenceDir 'validity-input.json') -Raw | ConvertFrom-Json
$accepted = (
    [string]$result.pipeline_status -eq 'COMPLETE' -and
    [string]$result.root_terminal -eq 'COMPLETED' -and
    [string]$result.quality_status -eq 'PASS' -and
    [string]$result.validity_verdict -eq 'INSTRUMENTATION_ONLY' -and
    -not [bool]$result.causal_claims_allowed -and
    [bool]$result.descriptive_metrics_allowed -and
    [string]$validityInput.runtime.runtime_recorded -eq 'PASS' -and
    [string]$runtimeObservation.model_recorded -eq 'PASS' -and
    @($result.pending_steps).Count -eq 0
)

$final = [ordered]@{
    schema_version = 1
    experiment_id = '002-instrumentation-validation'
    finished_at = (Get-Date).ToString('o')
    accepted = $accepted
    run_root = $RunRoot
    trace_root = $TraceRoot
    selected_workspace = [string]$result.selected_workspace
    pipeline_status = [string]$result.pipeline_status
    terminal_status = [string]$result.root_terminal
    quality_status = [string]$result.quality_status
    validity_verdict = [string]$result.validity_verdict
    causal_claims_allowed = [bool]$result.causal_claims_allowed
    descriptive_metrics_allowed = [bool]$result.descriptive_metrics_allowed
    agent_count = [int]$result.agent_count
    pending_steps = @($result.pending_steps)
    runner_result = $RunnerResultPath
    runtime_observation = $RuntimeObservationPath
    runtime_recorded = [string]$validityInput.runtime.runtime_recorded
    model_recorded = [string]$runtimeObservation.model_recorded
    configuration_recorded = [string]$runtimeObservation.configuration_recorded
    observed_models = @($runtimeObservation.observed.models)
}
$final | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath (Join-Path $RunRoot 'experiment-result.json') -Encoding UTF8

$summary = @(
    '# Experiment 002 - Finish Summary',
    '',
    ('Accepted: **' + $accepted + '**'),
    '',
    ('Selected workspace: `' + $result.selected_workspace + '`'),
    ('Pipeline: `' + $result.pipeline_status + '`'),
    ('Terminal: `' + $result.root_terminal + '`'),
    ('Quality: `' + $result.quality_status + '`'),
    ('Validity: `' + $result.validity_verdict + '`'),
    ('Agents: `' + $result.agent_count + '`'),
    ('Runtime recorded: `' + $validityInput.runtime.runtime_recorded + '`'),
    ('Model recorded: `' + $runtimeObservation.model_recorded + '`'),
    ('Configuration recorded: `' + $runtimeObservation.configuration_recorded + '`'),
    ('Observed models: `' + (@($runtimeObservation.observed.models) -join ', ') + '`'),
    ('Causal claims allowed: `' + $result.causal_claims_allowed + '`'),
    ('Descriptive metrics allowed: `' + $result.descriptive_metrics_allowed + '`')
)
$summary | Set-Content -LiteralPath (Join-Path $RunRoot 'FINISH-SUMMARY.md') -Encoding UTF8

$run | Add-Member -NotePropertyName finished_at -NotePropertyValue ((Get-Date).ToString('o')) -Force
$run | Add-Member -NotePropertyName selected_workspace -NotePropertyValue ([string]$result.selected_workspace) -Force
$run | Add-Member -NotePropertyName experiment_accepted -NotePropertyValue $accepted -Force
$run | ConvertTo-Json -Depth 10 | Set-Content -LiteralPath (Join-Path $RunRoot 'run.json') -Encoding UTF8
Remove-Item -LiteralPath $ActivePath -Force

Write-Host ''
if ($accepted) {
    Write-Host 'TOKN EXPERIMENT 002: ACCEPTED' -ForegroundColor Green
} else {
    Write-Host 'TOKN EXPERIMENT 002: NOT ACCEPTED' -ForegroundColor Yellow
}
Write-Host "Run root: $RunRoot"
Write-Host "Workspace: $($result.selected_workspace)"
Write-Host "Pipeline: $($result.pipeline_status)"
Write-Host "Terminal: $($result.root_terminal)"
Write-Host "Quality: $($result.quality_status)"
Write-Host "Validity: $($result.validity_verdict)"

if (-not $accepted) { exit 2 }
