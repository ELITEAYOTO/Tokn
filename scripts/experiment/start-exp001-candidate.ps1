param(
    [switch]$DryRun
)

$ErrorActionPreference = "Stop"

$ToolRoot = Resolve-Path (Join-Path $PSScriptRoot "..\..")
$V0Root = Split-Path $ToolRoot -Parent
$ExperimentRoot = Join-Path $V0Root "experiments\001-runtime-output-caps"
$ConfigPath = Join-Path $ExperimentRoot "experiment.json"
$TaskPath = Join-Path $ExperimentRoot "TASK.md"
$PolicyPath = Join-Path $ExperimentRoot "policies\candidate-AGENTS.md"
$QualityTemplate = Join-Path $ExperimentRoot "QUALITY_AFTER_RUN.md"
$ActivePath = Join-Path $ExperimentRoot "ACTIVE-RUN.json"
$SnapshotScript = Join-Path $PSScriptRoot "snapshot-project.ps1"
$InventoryScript = Join-Path $PSScriptRoot "inventory-workspaces.ps1"
$StartTrace = Join-Path $ToolRoot "scripts\trace\start-jem-trace.ps1"

if (Test-Path $ActivePath) {
    Write-Host ""
    Write-Host "TOKN: an experiment run is already marked active." -ForegroundColor Yellow
    Write-Host "Finish it or run RECOVER-EXP001.cmd before starting another."
    Write-Host "Active file: $ActivePath"
    exit 30
}

$runningDesktop = Get-Process ChatGPT -ErrorAction SilentlyContinue
$runningCodex = Get-Process codex -ErrorAction SilentlyContinue |
    Where-Object { $_.Path -like "*\AppData\Local\OpenAI\Codex\bin\*\codex.exe" }

if (-not $DryRun -and ($runningDesktop -or $runningCodex)) {
    Write-Host ""
    Write-Host "TOKN: Codex Desktop is still running." -ForegroundColor Yellow
    Write-Host "Close Codex Desktop completely before starting Experiment 001."
    exit 32
}

$config = Get-Content -LiteralPath $ConfigPath -Raw | ConvertFrom-Json
$ProjectRoot = [string]$config.project_root
$WorkspaceWatchRoot = $(if ($config.workspace_watch_root) {
    [string]$config.workspace_watch_root
} else {
    Split-Path (Split-Path $ProjectRoot -Parent) -Parent
})
$WorkspaceMaxDepth = $(if ($config.workspace_inventory_max_depth) {
    [int]$config.workspace_inventory_max_depth
} else {
    2
})

if (-not (Test-Path $ProjectRoot)) {
    throw "JEM project root not found: $ProjectRoot"
}
if (-not (Test-Path $WorkspaceWatchRoot)) {
    throw "Workspace watch root not found: $WorkspaceWatchRoot"
}
$taskDocument = Get-Content -LiteralPath $TaskPath -Raw
$match = [regex]::Match(
    $taskDocument,
    '(?s)<!-- ASTRA_PROMPT_START -->(.*?)<!-- ASTRA_PROMPT_END -->'
)

if (-not $match.Success) {
    throw "TASK.md markers are missing."
}

$prompt = $match.Groups[1].Value.Trim()

if ([string]::IsNullOrWhiteSpace($prompt) -or
    $prompt -match 'REPLACE THIS PLACEHOLDER') {
    Write-Host ""
    Write-Host "TOKN: TASK.md is not ready." -ForegroundColor Yellow
    Write-Host "Edit this file first:"
    Write-Host $TaskPath
    exit 31
}

$stamp = Get-Date -Format "yyyyMMdd-HHmmss"
$RunRoot = Join-Path $ExperimentRoot "runs\$stamp-candidate"
New-Item -ItemType Directory -Force -Path $RunRoot | Out-Null

Copy-Item -LiteralPath $ConfigPath -Destination (Join-Path $RunRoot "experiment.json")
Copy-Item -LiteralPath $TaskPath -Destination (Join-Path $RunRoot "TASK.md")
Copy-Item -LiteralPath $PolicyPath -Destination (Join-Path $RunRoot "candidate-AGENTS.md")
Copy-Item -LiteralPath $QualityTemplate -Destination (Join-Path $RunRoot "QUALITY_AFTER_RUN.md")

$BeforeSnapshot = Join-Path $RunRoot "project-before.json"
& $SnapshotScript -ProjectRoot $ProjectRoot -OutputPath $BeforeSnapshot

$BeforeWorkspaceInventory = Join-Path $RunRoot "workspace-before.json"
& $InventoryScript -WatchRoot $WorkspaceWatchRoot -OutputPath $BeforeWorkspaceInventory -MaxDepth $WorkspaceMaxDepth

$AgentsPath = Join-Path $ProjectRoot "AGENTS.md"
$OriginalAgentsPath = Join-Path $RunRoot "original-AGENTS.md"
$HadOriginalAgents = Test-Path $AgentsPath

if ($HadOriginalAgents) {
    Copy-Item -LiteralPath $AgentsPath -Destination $OriginalAgentsPath
    $original = Get-Content -LiteralPath $AgentsPath -Raw
    $policy = Get-Content -LiteralPath $PolicyPath -Raw
    ($original.TrimEnd() + [Environment]::NewLine + [Environment]::NewLine +
        "# Tokn Temporary Experiment Overlay" + [Environment]::NewLine +
        $policy) |
        Set-Content -LiteralPath $AgentsPath -Encoding UTF8
} else {
    Copy-Item -LiteralPath $PolicyPath -Destination $AgentsPath
}

$taskHash = (Get-FileHash -LiteralPath $TaskPath -Algorithm SHA256).Hash.ToLowerInvariant()
$policyHash = (Get-FileHash -LiteralPath $PolicyPath -Algorithm SHA256).Hash.ToLowerInvariant()

$label = $(if ($DryRun) { "exp001-candidate-dryrun" } else { "exp001-candidate" })
$traceExit = 0
$traceArgs = @(
    "-NoProfile",
    "-ExecutionPolicy", "Bypass",
    "-File", $StartTrace,
    "-Label", $label
)
if ($DryRun) {
    $traceArgs += "-DryRun"
}

try {
    & powershell.exe @traceArgs
    $traceExit = $LASTEXITCODE
} catch {
    $traceExit = 99
    Write-Warning $_.Exception.Message
}

if ($traceExit -ne 0) {
    if ($HadOriginalAgents) {
        Copy-Item -LiteralPath $OriginalAgentsPath -Destination $AgentsPath -Force
    } else {
        Remove-Item -LiteralPath $AgentsPath -Force -ErrorAction SilentlyContinue
    }
    Remove-Item -LiteralPath $RunRoot -Recurse -Force -ErrorAction SilentlyContinue
    throw "Trace launcher failed with exit code $traceExit. JEM AGENTS.md was restored."
}
$TraceRoot = Get-ChildItem (Join-Path $V0Root "traces") -Directory |
    Where-Object { $_.Name -like "*-$label" } |
    Sort-Object LastWriteTime -Descending |
    Select-Object -First 1 -ExpandProperty FullName

if (-not $TraceRoot) {
    if ($HadOriginalAgents) {
        Copy-Item -LiteralPath $OriginalAgentsPath -Destination $AgentsPath -Force
    } else {
        Remove-Item -LiteralPath $AgentsPath -Force -ErrorAction SilentlyContinue
    }
    throw "Could not locate the trace directory created for $label."
}

if ($DryRun) {
    if ($HadOriginalAgents) {
        Copy-Item -LiteralPath $OriginalAgentsPath -Destination $AgentsPath -Force
    } else {
        Remove-Item -LiteralPath $AgentsPath -Force -ErrorAction SilentlyContinue
    }
    Remove-Item -LiteralPath $RunRoot -Recurse -Force -ErrorAction SilentlyContinue
    Remove-Item -LiteralPath $TraceRoot -Recurse -Force -ErrorAction SilentlyContinue
    Write-Host ""
    Write-Host "TOKN EXPERIMENT 001 DRY-RUN: PASS" -ForegroundColor Green
    Write-Host "No Codex launch, no active run, JEM instructions restored."
    exit 0
}

$appPackage = Get-AppxPackage OpenAI.Codex -ErrorAction SilentlyContinue
$appVersion = $(if ($appPackage) { $appPackage.Version.ToString() } else { $null })
$embeddedCodex = Get-ChildItem "$env:LOCALAPPDATA\OpenAI\Codex\bin" -Filter codex.exe -File -Recurse -ErrorAction SilentlyContinue |
    Sort-Object LastWriteTime -Descending |
    Select-Object -First 1 -ExpandProperty FullName
$codexVersion = $(if ($embeddedCodex) { (& $embeddedCodex --version 2>$null | Out-String).Trim() } else { $null })
$nodeVersion = $(& node --version 2>$null | Out-String).Trim()
$npmVersion = $(& npm --version 2>$null | Out-String).Trim()
$packageJson = Get-Content -LiteralPath (Join-Path $ProjectRoot "package.json") -Raw | ConvertFrom-Json

$run = [ordered]@{
    schema_version = 1
    experiment_id = "001-runtime-output-caps"
    mode = "candidate"
    started_at = (Get-Date).ToString("o")
    codex_desktop_version = $appVersion
    codex_binary_version = $codexVersion
    node_version = $nodeVersion
    npm_version = $npmVersion
    project_package_version = $packageJson.version
    run_root = $RunRoot
    trace_root = $TraceRoot
    project_root = $ProjectRoot
    workspace_watch_root = $WorkspaceWatchRoot
    workspace_inventory_max_depth = $WorkspaceMaxDepth
    workspace_before_inventory = $BeforeWorkspaceInventory
    task_path = (Join-Path $RunRoot "TASK.md")
    task_sha256 = $taskHash
    policy_path = (Join-Path $RunRoot "candidate-AGENTS.md")
    policy_sha256 = $policyHash
    before_snapshot = $BeforeSnapshot
    original_agents_present = $HadOriginalAgents
    original_agents_backup = $(if ($HadOriginalAgents) { $OriginalAgentsPath } else { $null })
    baseline_bundle = $config.baseline_bundle
    file_read_cap = [int]$config.candidate_policy.file_read_max_output_tokens
    search_cap = [int]$config.candidate_policy.search_max_output_tokens
}

$run | ConvertTo-Json -Depth 6 |
    Set-Content -LiteralPath (Join-Path $RunRoot "run.json") -Encoding UTF8
$run | ConvertTo-Json -Depth 6 |
    Set-Content -LiteralPath $ActivePath -Encoding UTF8

Set-Clipboard -Value $prompt
Write-Host ""
Write-Host "TOKN EXPERIMENT 001 - CANDIDATE READY" -ForegroundColor Green
Write-Host "Run root:   $RunRoot"
Write-Host "Trace root: $TraceRoot"
Write-Host "Project:    $ProjectRoot"
Write-Host ""
Write-Host "The exact Astra task is now in your clipboard." -ForegroundColor Cyan
Write-Host "In Codex:"
Write-Host "1. Open JEM Ultimate at the PROJECT folder."
Write-Host "2. Paste the clipboard contents as ONE new Astra prompt."
Write-Host "3. Let Astra work normally."
Write-Host "4. Avoid extra messages unless they are genuinely required."
Write-Host "5. When Astra is completely finished, close Codex Desktop."
Write-Host "6. Double-click FINISH-EXP001-CANDIDATE.cmd"
Write-Host ""
Write-Host "Quality is the invariant. The policy only bounds read/search tool output."
