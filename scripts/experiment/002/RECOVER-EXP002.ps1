$ErrorActionPreference = 'Stop'
$ExperimentRoot = $PSScriptRoot
$Outer = (Resolve-Path (Join-Path $ExperimentRoot '..\..')).Path
$ActivePath = Join-Path $ExperimentRoot 'ACTIVE-RUN.json'

$runningDesktop = Get-Process ChatGPT -ErrorAction SilentlyContinue
$runningCodex = Get-Process codex -ErrorAction SilentlyContinue
if ($runningDesktop -or $runningCodex) {
    throw 'Close Codex/ChatGPT completely before RECOVER-EXP002.'
}

$run = $null
if (Test-Path -LiteralPath $ActivePath) {
    $run = Get-Content -LiteralPath $ActivePath -Raw | ConvertFrom-Json
}

Add-Type @"
using System;
using System.Runtime.InteropServices;
public static class ToknExp002RecoverBroadcast {
    [DllImport("user32.dll", CharSet = CharSet.Auto, SetLastError = true)]
    public static extern IntPtr SendMessageTimeout(
        IntPtr hWnd, uint Msg, UIntPtr wParam, string lParam,
        uint fuFlags, uint uTimeout, out UIntPtr lpdwResult);
}
"@ -ErrorAction SilentlyContinue
[Environment]::SetEnvironmentVariable('CODEX_ROLLOUT_TRACE_ROOT', $null, 'User')
$result = [UIntPtr]::Zero
[void][ToknExp002RecoverBroadcast]::SendMessageTimeout(
    [IntPtr]0xffff, 0x001A, [UIntPtr]::Zero, 'Environment', 2, 5000, [ref]$result)

if ($run) {
    $run | Add-Member -NotePropertyName recovered_at -NotePropertyValue ((Get-Date).ToString('o')) -Force
    $run | Add-Member -NotePropertyName recovery_note -NotePropertyValue 'Experiment 002 harness recovery executed. No policy placements exist for this experiment.' -Force
    $run | ConvertTo-Json -Depth 10 | Set-Content -LiteralPath (Join-Path $run.run_root 'run.json') -Encoding UTF8
}
if (Test-Path -LiteralPath $ActivePath) { Remove-Item -LiteralPath $ActivePath -Force }

Write-Host ''
Write-Host 'TOKN EXPERIMENT 002 - RECOVERY COMPLETE' -ForegroundColor Green
if ($run) {
    Write-Host "Run folder preserved: $($run.run_root)"
    Write-Host "Trace folder preserved: $($run.trace_root)"
} else {
    Write-Host 'No ACTIVE-RUN.json was present. Trace environment was still cleared.'
}
Write-Host 'No project or policy file was modified by recovery.'
