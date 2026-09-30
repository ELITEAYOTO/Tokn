$ErrorActionPreference = "Stop"

$ToolRoot = Resolve-Path (Join-Path $PSScriptRoot "..\..")
$V0Root = Split-Path $ToolRoot -Parent
$ExperimentRoot = Join-Path $V0Root "experiments\001-runtime-output-caps"
$ActivePath = Join-Path $ExperimentRoot "ACTIVE-RUN.json"
$SnapshotScript = Join-Path $PSScriptRoot "snapshot-project.ps1"
$InventoryScript = Join-Path $PSScriptRoot "inventory-workspaces.ps1"
$PolicyPlacementScript = Join-Path $PSScriptRoot "policy-placement.ps1"
$DiffScript = Join-Path $PSScriptRoot "diff-snapshots.ps1"
. $PolicyPlacementScript
$FinishTrace = Join-Path $ToolRoot "scripts\trace\finish-latest-trace.ps1"
$Observer = Join-Path $ToolRoot "target\release\tokn-observe.exe"

if (-not (Test-Path $ActivePath)) {
    Write-Host ""
    Write-Host "TOKN: no active Experiment 001 run found." -ForegroundColor Yellow
    Write-Host "Active file expected at: $ActivePath"
    exit 40
}

$runningDesktop = Get-Process ChatGPT -ErrorAction SilentlyContinue
$runningCodex = Get-Process codex -ErrorAction SilentlyContinue |
    Where-Object { $_.Path -like "*\AppData\Local\OpenAI\Codex\bin\*\codex.exe" }

if ($runningDesktop -or $runningCodex) {
    Write-Host ""
    Write-Host "TOKN: Codex Desktop is still running." -ForegroundColor Yellow
    Write-Host "Close Codex Desktop completely, then run launchers\FINISH-EXP001-CANDIDATE.cmd again."
    exit 41
}

$run = Get-Content -LiteralPath $ActivePath -Raw | ConvertFrom-Json
$RunRoot = [string]$run.run_root
$TraceRoot = [string]$run.trace_root
$ProjectRoot = [string]$run.project_root
$WorkspaceWatchRoot = $(if ($run.workspace_watch_root) {
    [string]$run.workspace_watch_root
} else {
    Split-Path (Split-Path $ProjectRoot -Parent) -Parent
})
$WorkspaceMaxDepth = $(if ($run.workspace_inventory_max_depth) {
    [int]$run.workspace_inventory_max_depth
} else {
    2
})
$BeforeWorkspaceInventory = [string]$run.workspace_before_inventory
$AgentsPath = Join-Path $ProjectRoot "AGENTS.md"
Write-Host ""
Write-Host "Restoring Tokn policy placements..." -ForegroundColor Cyan

if ($run.policy_placements) {
    $placementsRestored = Restore-ToknPolicyPlacements -Placements $run.policy_placements -PreserveDirectory $RunRoot -Prefix "finish-policy"
    if (-not $placementsRestored) {
        Write-Warning "At least one policy placement could not be restored safely; preserved state remains in the run folder."
    }
} elseif ([bool]$run.original_agents_present) {
    if (Test-Path $AgentsPath) {
        Copy-Item -LiteralPath $AgentsPath -Destination (Join-Path $RunRoot "AGENTS-at-finish-experimental.md") -Force
    }
    Copy-Item -LiteralPath $run.original_agents_backup -Destination $AgentsPath -Force
} elseif (Test-Path $AgentsPath) {
    $currentHash = (Get-FileHash -LiteralPath $AgentsPath -Algorithm SHA256).Hash.ToLowerInvariant()
    if ($currentHash -eq [string]$run.policy_sha256) {
        Remove-Item -LiteralPath $AgentsPath -Force
    } else {
        Copy-Item -LiteralPath $AgentsPath -Destination (Join-Path $RunRoot "AGENTS-at-finish-unexpected.md") -Force
        Write-Warning "Legacy AGENTS.md changed during the run; preserved and left in place."
    }
}

Write-Host "Pre-run policy placement state restored where safe." -ForegroundColor Green

if (-not (Test-Path $Observer)) {
    throw "Tokn release binary not found: $Observer"
}

Write-Host ""
Write-Host "Finalizing diagnostic trace..." -ForegroundColor Cyan
& powershell.exe -NoProfile -ExecutionPolicy Bypass -File $FinishTrace -TraceRoot $TraceRoot
$finishExit = $LASTEXITCODE
if ($finishExit -ne 0) {
    Write-Warning "Trace finalization returned exit code $finishExit."
    Write-Warning "ACTIVE-RUN.json is being kept so FINISH can be retried."
    exit $finishExit
}
$bundle = Get-ChildItem $TraceRoot -Filter manifest.json -File -Recurse |
    ForEach-Object {
        $dir = $_.Directory.FullName
        if (Test-Path (Join-Path $dir "trace.jsonl")) { $dir }
    } |
    Select-Object -First 1

if (-not $bundle) {
    Write-Warning "No diagnostic bundle found after trace finalization."
    Write-Warning "ACTIVE-RUN.json is being kept for recovery."
    exit 42
}

Write-Host ""
Write-Host "Resolving the workspace actually used by Astra..." -ForegroundColor Cyan

$AfterWorkspaceInventory = Join-Path $RunRoot "workspace-after.json"
& $InventoryScript -WatchRoot $WorkspaceWatchRoot -OutputPath $AfterWorkspaceInventory -MaxDepth $WorkspaceMaxDepth

$WorkspaceResolutionJson = Join-Path $RunRoot "workspace-resolution.json"
$WorkspaceResolutionText = Join-Path $RunRoot "TOKN_WORKSPACE_RESOLUTION.txt"
$resolveArgs = @(
    "resolve-workspace",
    $bundle,
    "--inventory", $AfterWorkspaceInventory,
    "--source-root", $ProjectRoot,
    "--output-json", $WorkspaceResolutionJson
)
if ($BeforeWorkspaceInventory -and (Test-Path -LiteralPath $BeforeWorkspaceInventory)) {
    $resolveArgs += @("--before-inventory", $BeforeWorkspaceInventory)
}

$workspaceOutput = & $Observer @resolveArgs 2>&1 | Out-String
$workspaceResolveExit = $LASTEXITCODE
$workspaceOutput | Set-Content -LiteralPath $WorkspaceResolutionText -Encoding UTF8

if ($workspaceResolveExit -ne 0 -or -not (Test-Path -LiteralPath $WorkspaceResolutionJson)) {
    Write-Warning "Workspace resolution failed. ACTIVE-RUN.json is being kept for retry."
    exit 43
}

$workspaceResolution = Get-Content -LiteralPath $WorkspaceResolutionJson -Raw | ConvertFrom-Json
$workspaceStatus = [string]$workspaceResolution.status
$ResolvedProjectRoot = [string]$workspaceResolution.selected_root

if ($workspaceStatus -ne "SELECTED" -or [string]::IsNullOrWhiteSpace($ResolvedProjectRoot) -or -not (Test-Path -LiteralPath $ResolvedProjectRoot)) {
    $run | Add-Member -NotePropertyName workspace_after_inventory -NotePropertyValue $AfterWorkspaceInventory -Force
    $run | Add-Member -NotePropertyName workspace_resolution_json -NotePropertyValue $WorkspaceResolutionJson -Force
    $run | Add-Member -NotePropertyName workspace_resolution_output -NotePropertyValue $WorkspaceResolutionText -Force
    $run | Add-Member -NotePropertyName workspace_resolution_status -NotePropertyValue $workspaceStatus -Force
    $run | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath (Join-Path $RunRoot "run.json") -Encoding UTF8
    Write-Warning "Tokn could not select one workspace safely (status: $workspaceStatus)."
    Write-Warning "No project quality gate was run. ACTIVE-RUN.json is being kept for retry."
    exit 43
}

Write-Host "Resolved project: $ResolvedProjectRoot" -ForegroundColor Green

if (-not $ResolvedProjectRoot.Equals($ProjectRoot, [System.StringComparison]::OrdinalIgnoreCase)) {
    $ResolvedAgentsPath = Join-Path $ResolvedProjectRoot "AGENTS.md"
    if (Test-Path -LiteralPath $ResolvedAgentsPath) {
        $resolvedAgentsHash = (Get-FileHash -LiteralPath $ResolvedAgentsPath -Algorithm SHA256).Hash.ToLowerInvariant()
        $sourcePlacement = $null
        if ($run.policy_placements) {
            $sourcePlacement = @($run.policy_placements) |
                Where-Object { ([string]$_.path).Equals($AgentsPath, [System.StringComparison]::OrdinalIgnoreCase) } |
                Select-Object -First 1
        }

        if ($sourcePlacement -and $resolvedAgentsHash -eq [string]$sourcePlacement.installed_sha256) {
            if ([bool]$sourcePlacement.had_original -and (Test-Path -LiteralPath ([string]$sourcePlacement.backup_path))) {
                Copy-Item -LiteralPath ([string]$sourcePlacement.backup_path) -Destination $ResolvedAgentsPath -Force
                Write-Host "Resolved workspace AGENTS.md restored from source placement backup." -ForegroundColor Green
            } elseif (-not [bool]$sourcePlacement.had_original) {
                Remove-Item -LiteralPath $ResolvedAgentsPath -Force
                Write-Host "Temporary Tokn AGENTS.md removed from resolved workspace." -ForegroundColor Green
            }
        } elseif (-not $run.policy_placements -and [bool]$run.original_agents_present) {
            $legacyExperimental = Join-Path $RunRoot "AGENTS-at-finish-experimental.md"
            if ((Test-Path -LiteralPath $legacyExperimental) -and
                $resolvedAgentsHash -eq (Get-FileHash -LiteralPath $legacyExperimental -Algorithm SHA256).Hash.ToLowerInvariant()) {
                Copy-Item -LiteralPath $run.original_agents_backup -Destination $ResolvedAgentsPath -Force
                Write-Host "Legacy resolved workspace AGENTS.md restored from pre-run backup." -ForegroundColor Green
            } else {
                Copy-Item -LiteralPath $ResolvedAgentsPath -Destination (Join-Path $RunRoot "AGENTS-at-resolved-workspace-unexpected.md") -Force
                Write-Warning "Legacy resolved workspace AGENTS.md could not be matched exactly; preserved and not overwritten."
            }
        } elseif (-not $run.policy_placements -and -not [bool]$run.original_agents_present -and
            $resolvedAgentsHash -eq [string]$run.policy_sha256) {
            Remove-Item -LiteralPath $ResolvedAgentsPath -Force
            Write-Host "Legacy temporary Tokn AGENTS.md removed from resolved workspace." -ForegroundColor Green
        } else {
            Copy-Item -LiteralPath $ResolvedAgentsPath -Destination (Join-Path $RunRoot "AGENTS-at-resolved-workspace-unexpected.md") -Force
            Write-Warning "Resolved workspace AGENTS.md is not an exact known Tokn placement; it was preserved and not overwritten."
        }
    }
}

$AfterSnapshot = Join-Path $RunRoot "project-after.json"
& $SnapshotScript -ProjectRoot $ResolvedProjectRoot -OutputPath $AfterSnapshot

$DiffJson = Join-Path $RunRoot "project-diff.json"
$DiffMarkdown = Join-Path $RunRoot "PROJECT_DIFF.md"
$diffArgs = @{
    BeforePath = $run.before_snapshot
    AfterPath = $AfterSnapshot
    OutputJson = $DiffJson
    OutputMarkdown = $DiffMarkdown
}
& $DiffScript @diffArgs

$fileReadCap = [int]$run.file_read_cap
$searchCap = [int]$run.search_cap

Write-Host ""
Write-Host "Running JEM automated quality gate (verify:local)..." -ForegroundColor Cyan
$VerifyPath = Join-Path $RunRoot "JEM_VERIFY_LOCAL.txt"
$escapedProject = $ResolvedProjectRoot.Replace('"', '""')
$escapedVerify = $VerifyPath.Replace('"', '""')
$verifyCommand = 'cd /d "' + $escapedProject + '" && npm run verify:local > "' + $escapedVerify + '" 2>&1'
& cmd.exe /d /s /c $verifyCommand
$verifyExit = $LASTEXITCODE
Write-Host "JEM verify:local exit code: $verifyExit"

function Save-CommandOutput {
    param(
        [string]$Path,
        [scriptblock]$Command
    )
    $output = & $Command 2>&1 | Out-String
    $code = $LASTEXITCODE
    $output | Set-Content -LiteralPath $Path -Encoding UTF8
    return $code
}

$AttributionPath = Join-Path $RunRoot "TOKN_ATTRIBUTION.txt"
$RunGroupPath = Join-Path $RunRoot "TOKN_RUN_GROUP.txt"
$RunGroupJson = Join-Path $RunRoot "tokn-run-group.json"
$PolicyCheckPath = Join-Path $RunRoot "TOKN_CAP_POLICY_CHECK.txt"
$PolicyEvidencePath = Join-Path $RunRoot "TOKN_POLICY_EVIDENCE.txt"
$PolicyEvidenceJson = Join-Path $RunRoot "tokn-policy-evidence.json"
$SimulationPath = Join-Path $RunRoot "TOKN_CAP_SIMULATION.txt"
$ComparePath = Join-Path $RunRoot "TOKN_COMPARE_TO_OLD_BASELINE.txt"

[void](Save-CommandOutput -Path $AttributionPath -Command {
    & $Observer analyze $bundle --top 20
})

$runGroupExit = Save-CommandOutput -Path $RunGroupPath -Command {
    & $Observer analyze-run $bundle --output-json $RunGroupJson
}
if ($runGroupExit -ne 0 -or -not (Test-Path -LiteralPath $RunGroupJson)) {
    Write-Warning "Tokn run-group analysis is incomplete (exit: $runGroupExit)."
}

$policyExit = Save-CommandOutput -Path $PolicyCheckPath -Command {
    & $Observer check-caps $bundle --cap "file_read=$fileReadCap" --cap "search=$searchCap"
}

$policyEvidenceArgs = @(
    "inspect-policy", $bundle,
    "--policy-id", "exp001-runtime-output-caps",
    "--marker", "Tokn Experiment 001 - Runtime Output Policy",
    "--cap", "file_read=$fileReadCap",
    "--cap", "search=$searchCap",
    "--enforcement", "supported-insufficient-input",
    "--output-json", $PolicyEvidenceJson
)
$policyEvidencePaths = $(if ($run.policy_placements) {
    @($run.policy_placements) | ForEach-Object { [string]$_.path }
} else { @($AgentsPath) })
$policyEvidencePaths += (Join-Path $ResolvedProjectRoot "AGENTS.md")
foreach ($path in ($policyEvidencePaths | Sort-Object -Unique)) {
    $policyEvidenceArgs += @("--policy-path", $path)
}
$policyEvidenceExit = Save-CommandOutput -Path $PolicyEvidencePath -Command {
    & $Observer @policyEvidenceArgs
}
if ($policyEvidenceExit -ne 0 -or -not (Test-Path -LiteralPath $PolicyEvidenceJson)) {
    Write-Warning "Tokn policy evidence analysis is incomplete (exit: $policyEvidenceExit)."
}

[void](Save-CommandOutput -Path $SimulationPath -Command {
    & $Observer simulate-caps $bundle --cap "file_read=$fileReadCap" --cap "search=$searchCap"
})

$nl = [Environment]::NewLine
$compareHeader = "IMPORTANT: this comparison uses the older JEM baseline from a different development task." + $nl +
    "Treat token deltas as descriptive only, not causal A/B evidence." + $nl + $nl
$compareBody = & $Observer compare $run.baseline_bundle $bundle 2>&1 | Out-String
($compareHeader + $compareBody) | Set-Content -LiteralPath $ComparePath -Encoding UTF8
$SessionEvidencePath = Join-Path $RunRoot "session-evidence.json"
$sessionEvidence = @()
$runGroup = $null

if (Test-Path -LiteralPath $RunGroupJson) {
    $runGroup = Get-Content -LiteralPath $RunGroupJson -Raw | ConvertFrom-Json

    foreach ($agent in @($runGroup.agents)) {
        $sessionPath = [string]$agent.source_path
        if ([string]::IsNullOrWhiteSpace($sessionPath) -or -not (Test-Path -LiteralPath $sessionPath)) {
            continue
        }

        $sessionFile = Get-Item -LiteralPath $sessionPath
        $sessionEvidence += [pscustomobject][ordered]@{
            path = $sessionPath
            bytes = [int64]$sessionFile.Length
            last_write = $sessionFile.LastWriteTime.ToString("o")
            sha256 = (Get-FileHash -LiteralPath $sessionPath -Algorithm SHA256).Hash.ToLowerInvariant()
            thread_id = [string]$agent.thread_id
            depth = [int]$agent.depth
            terminal = [string]$agent.terminal.status
        }
    }
}

@{
    captured_at = (Get-Date).ToString("o")
    root_thread_id = $(if ($runGroup) { [string]$runGroup.root_thread_id } else { $null })
    session_id = $(if ($runGroup) { [string]$runGroup.session_id } else { $null })
    files = @($sessionEvidence)
} | ConvertTo-Json -Depth 6 | Set-Content -LiteralPath $SessionEvidencePath -Encoding UTF8
$diff = Get-Content -LiteralPath $DiffJson -Raw | ConvertFrom-Json
$finishedAt = (Get-Date).ToString("o")

$run | Add-Member -NotePropertyName finished_at -NotePropertyValue $finishedAt -Force
$run | Add-Member -NotePropertyName bundle_path -NotePropertyValue $bundle -Force
$run | Add-Member -NotePropertyName workspace_after_inventory -NotePropertyValue $AfterWorkspaceInventory -Force
$run | Add-Member -NotePropertyName workspace_resolution_json -NotePropertyValue $WorkspaceResolutionJson -Force
$run | Add-Member -NotePropertyName workspace_resolution_output -NotePropertyValue $WorkspaceResolutionText -Force
$run | Add-Member -NotePropertyName workspace_resolution_status -NotePropertyValue $workspaceStatus -Force
$run | Add-Member -NotePropertyName resolved_project_root -NotePropertyValue $ResolvedProjectRoot -Force
$run | Add-Member -NotePropertyName after_snapshot -NotePropertyValue $AfterSnapshot -Force
$run | Add-Member -NotePropertyName project_diff_json -NotePropertyValue $DiffJson -Force
$run | Add-Member -NotePropertyName project_diff_markdown -NotePropertyValue $DiffMarkdown -Force
$run | Add-Member -NotePropertyName policy_check_exit_code -NotePropertyValue $policyExit -Force
$run | Add-Member -NotePropertyName policy_evidence_exit_code -NotePropertyValue $policyEvidenceExit -Force
$run | Add-Member -NotePropertyName policy_evidence_output -NotePropertyValue $PolicyEvidencePath -Force
$run | Add-Member -NotePropertyName policy_evidence_json -NotePropertyValue $PolicyEvidenceJson -Force
$run | Add-Member -NotePropertyName verify_local_exit_code -NotePropertyValue $verifyExit -Force
$run | Add-Member -NotePropertyName verify_local_output -NotePropertyValue $VerifyPath -Force
$run | Add-Member -NotePropertyName run_group_output -NotePropertyValue $RunGroupPath -Force
$run | Add-Member -NotePropertyName run_group_json -NotePropertyValue $RunGroupJson -Force
$run | Add-Member -NotePropertyName run_group_exit_code -NotePropertyValue $runGroupExit -Force
$run | Add-Member -NotePropertyName session_evidence -NotePropertyValue $SessionEvidencePath -Force
$run | Add-Member -NotePropertyName added_files -NotePropertyValue $diff.added_count -Force
$run | Add-Member -NotePropertyName modified_files -NotePropertyValue $diff.modified_count -Force
$run | Add-Member -NotePropertyName removed_files -NotePropertyValue $diff.removed_count -Force

$run | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath (Join-Path $RunRoot "run.json") -Encoding UTF8
$summaryLines = @(
    "# Tokn Experiment 001 - Candidate Run Complete",
    "",
    "- Finished: $finishedAt",
    "- Bundle: $bundle",
    "- Workspace resolution: $workspaceStatus",
    "- Resolved project: $ResolvedProjectRoot",
    "- Project files added: $($diff.added_count)",
    "- Project files modified: $($diff.modified_count)",
    "- Project files removed: $($diff.removed_count)",
    "- Cap policy check exit code: $policyExit",
    "- Policy evidence exit code: $policyEvidenceExit",
    "- JEM verify:local exit code: $verifyExit",
    "",
    "## Read next",
    "",
    "1. QUALITY_AFTER_RUN.md - fill this manually after checking JEM.",
    "2. TOKN_WORKSPACE_RESOLUTION.txt - why Tokn selected the final workspace.",
    "3. PROJECT_DIFF.md - source-level file changes against the resolved workspace.",
    "4. TOKN_RUN_GROUP.txt - parent/subagent token totals and terminal state.",
    "5. TOKN_POLICY_EVIDENCE.txt - policy hint, observed behavior and enforcement evidence.",
    "6. TOKN_CAP_POLICY_CHECK.txt - detailed cap compliance evidence.",
    "7. JEM_VERIFY_LOCAL.txt - automated quality-gate output on the resolved workspace.",
    "8. TOKN_ATTRIBUTION.txt - diagnostic attribution when available.",
    "9. TOKN_COMPARE_TO_OLD_BASELINE.txt - descriptive only; tasks differ.",
    "10. session-evidence.json - local Codex rollout files touched in the run window.",
    "",
    "Do not judge the experiment by speed. Quality and completed work are required."
)
$summaryLines | Set-Content -LiteralPath (Join-Path $RunRoot "RUN_COMPLETE.md") -Encoding UTF8

Remove-Item -LiteralPath $ActivePath -Force

Write-Host ""
Write-Host "TOKN EXPERIMENT 001 - CAPTURE COMPLETE" -ForegroundColor Green
Write-Host "Run folder: $RunRoot"
Write-Host "Bundle:     $bundle"
Write-Host ""
Write-Host "Next: open QUALITY_AFTER_RUN.md and evaluate the result." -ForegroundColor Cyan
Write-Host "Then give ChatGPT the run folder path; Tokn has captured the rest."
Write-Host ""
