$ErrorActionPreference = "Stop"

$ToolRoot = Resolve-Path (Join-Path $PSScriptRoot "..\..")
$V0Root = Split-Path $ToolRoot -Parent
$ExperimentRoot = Join-Path $V0Root "experiments\001-runtime-output-caps"
$ActivePath = Join-Path $ExperimentRoot "ACTIVE-RUN.json"
$PolicyPlacementScript = Join-Path $PSScriptRoot "policy-placement.ps1"
. $PolicyPlacementScript

$runningDesktop = Get-Process ChatGPT -ErrorAction SilentlyContinue
$runningCodex = Get-Process codex -ErrorAction SilentlyContinue |
    Where-Object { $_.Path -like "*\AppData\Local\OpenAI\Codex\bin\*\codex.exe" }

if ($runningDesktop -or $runningCodex) {
    Write-Host ""
    Write-Host "TOKN RECOVERY: Codex Desktop is still running." -ForegroundColor Yellow
    Write-Host "Close it completely, then run RECOVER-EXP001.cmd again."
    exit 50
}

$run = $null
if (Test-Path $ActivePath) {
    $run = Get-Content -LiteralPath $ActivePath -Raw | ConvertFrom-Json
}
if ($run) {
    $AgentsPath = Join-Path $run.project_root "AGENTS.md"
    $RunRoot = $run.run_root

    if ($run.policy_placements) {
        $placementsRestored = Restore-ToknPolicyPlacements -Placements $run.policy_placements -PreserveDirectory $RunRoot -Prefix "recovery-policy"
        if (-not $placementsRestored) {
            Write-Warning "Recovery could not safely restore every known policy placement."
        }
    } elseif ([bool]$run.original_agents_present) {
        if (Test-Path $AgentsPath) {
            Copy-Item -LiteralPath $AgentsPath -Destination (Join-Path $RunRoot "AGENTS-at-recovery.md") -Force
        }
        if (Test-Path $run.original_agents_backup) {
            Copy-Item -LiteralPath $run.original_agents_backup -Destination $AgentsPath -Force
        } else {
            Write-Warning "Original AGENTS.md backup is missing. Current AGENTS.md was preserved in the run folder."
        }
    } elseif (Test-Path $AgentsPath) {
        $currentHash = (Get-FileHash -LiteralPath $AgentsPath -Algorithm SHA256).Hash.ToLowerInvariant()
        if ($currentHash -eq [string]$run.policy_sha256) {
            Remove-Item -LiteralPath $AgentsPath -Force
        } else {
            Copy-Item -LiteralPath $AgentsPath -Destination (Join-Path $RunRoot "AGENTS-at-recovery.md") -Force
            Write-Warning "Legacy AGENTS.md changed during the run; preserved and left in place."
        }
    }

    $run | Add-Member -NotePropertyName recovered_at -NotePropertyValue ((Get-Date).ToString("o")) -Force
    $run | Add-Member -NotePropertyName recovery_note -NotePropertyValue "Manual recovery executed; original trace/run files preserved." -Force
    $run | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath (Join-Path $RunRoot "run.json") -Encoding UTF8
}
Add-Type @"
using System;
using System.Runtime.InteropServices;
public static class ToknExperimentEnvBroadcast {
    [DllImport("user32.dll", CharSet = CharSet.Auto, SetLastError = true)]
    public static extern IntPtr SendMessageTimeout(
        IntPtr hWnd, uint Msg, UIntPtr wParam, string lParam,
        uint fuFlags, uint uTimeout, out UIntPtr lpdwResult);
}
"@

$result = [UIntPtr]::Zero
[Environment]::SetEnvironmentVariable("CODEX_ROLLOUT_TRACE_ROOT", $null, "User")
[void][ToknExperimentEnvBroadcast]::SendMessageTimeout(
    [IntPtr]0xffff,
    0x001A,
    [UIntPtr]::Zero,
    "Environment",
    2,
    5000,
    [ref]$result
)

if (Test-Path $ActivePath) {
    Remove-Item -LiteralPath $ActivePath -Force
}

Write-Host ""
Write-Host "TOKN EXPERIMENT 001 - RECOVERY COMPLETE" -ForegroundColor Green
if ($run) {
    Write-Host "Run folder preserved: $($run.run_root)"
    Write-Host "Trace folder preserved: $($run.trace_root)"
} else {
    Write-Host "No ACTIVE-RUN.json was present. Trace environment was still cleared."
}
Write-Host "JEM instructions were restored to the best known pre-run state."
Write-Host ""
