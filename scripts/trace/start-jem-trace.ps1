param(
    [string]$Label = "jem-ultimate",
    [switch]$DryRun
)

$ErrorActionPreference = "Stop"
$ToolRoot = Resolve-Path (Join-Path $PSScriptRoot "..\..")
$V0Root = Split-Path $ToolRoot -Parent
$TraceRoot = Join-Path $V0Root "traces"

$runningDesktop = Get-Process ChatGPT -ErrorAction SilentlyContinue
$runningCodex = Get-Process codex -ErrorAction SilentlyContinue |
    Where-Object { $_.Path -like "*\AppData\Local\OpenAI\Codex\bin\*\codex.exe" }

if (-not $DryRun -and ($runningDesktop -or $runningCodex)) {
    Write-Host ""
    Write-Host "TOKN: Codex Desktop is still running." -ForegroundColor Yellow
    Write-Host "Close Codex Desktop completely, then launch START-JEM-TRACE.cmd again."
    exit 20
}

$app = Get-StartApps |
    Where-Object { $_.AppID -eq "OpenAI.Codex_2p2nqsd0c76g0!App" } |
    Select-Object -First 1

if (-not $app) {
    throw "Codex Windows AppID not found."
}

$stamp = Get-Date -Format "yyyyMMdd-HHmmss"
$safeLabel = ($Label -replace '[^A-Za-z0-9._-]', '-')
$runRoot = Join-Path $TraceRoot "$stamp-$safeLabel"
New-Item -ItemType Directory -Force -Path $runRoot | Out-Null

$meta = [ordered]@{
    label = $Label
    started_at = (Get-Date).ToString("o")
    app_id = $app.AppID
    trace_root = $runRoot
    purpose = "Tokn real-world token efficiency baseline"
}
$meta | ConvertTo-Json -Depth 4 |
    Set-Content -Encoding UTF8 (Join-Path $runRoot "tokn-run.json")

Add-Type @"
using System;
using System.Runtime.InteropServices;
public static class ToknEnvBroadcast {
    [DllImport("user32.dll", CharSet = CharSet.Auto, SetLastError = true)]
    public static extern IntPtr SendMessageTimeout(
        IntPtr hWnd, uint Msg, UIntPtr wParam, string lParam,
        uint fuFlags, uint uTimeout, out UIntPtr lpdwResult);
}
"@

function Publish-EnvironmentChange {
    $result = [UIntPtr]::Zero
    [void][ToknEnvBroadcast]::SendMessageTimeout(
        [IntPtr]0xffff, 0x001A, [UIntPtr]::Zero, "Environment",
        2, 5000, [ref]$result)
}

[Environment]::SetEnvironmentVariable(
    "CODEX_ROLLOUT_TRACE_ROOT", $runRoot, "User")
$env:CODEX_ROLLOUT_TRACE_ROOT = $runRoot
Publish-EnvironmentChange

Write-Host ""
Write-Host "TOKN TRACE READY" -ForegroundColor Green
Write-Host "Trace root: $runRoot"
Write-Host "Label:      $Label"
Write-Host ""

if ($DryRun) {
    Write-Host "Dry-run only: Codex was NOT launched."
    [Environment]::SetEnvironmentVariable(
        "CODEX_ROLLOUT_TRACE_ROOT", $null, "User")
    Publish-EnvironmentChange
    exit 0
}

try {
    Write-Host "Launching Codex Desktop with diagnostic tracing enabled..."
    Start-Process "explorer.exe" "shell:AppsFolder\$($app.AppID)"
}
catch {
    [Environment]::SetEnvironmentVariable(
        "CODEX_ROLLOUT_TRACE_ROOT", $null, "User")
    Publish-EnvironmentChange
    throw
}

Write-Host ""
Write-Host "Now use Astra normally on JEM Ultimate."
Write-Host "When Astra finishes:"
Write-Host "1. Close Codex Desktop completely."
Write-Host "2. Double-click FINISH-JEM-TRACE.cmd"
Write-Host ""
