$ErrorActionPreference = "Stop"

$ToolRoot = Resolve-Path (Join-Path $PSScriptRoot "..\..")
$V0Root = Split-Path $ToolRoot -Parent
$ProbeRoot = Join-Path $V0Root "experiments\p6-hook-probe"
$ActivePath = Join-Path $ProbeRoot "ACTIVE-PROBE.json"

if (-not (Test-Path -LiteralPath $ActivePath)) {
    throw "No active P6 hook probe found."
}

$runningDesktop = Get-Process ChatGPT -ErrorAction SilentlyContinue
$runningCodex = Get-Process codex -ErrorAction SilentlyContinue |
    Where-Object { $_.Path -like "*\AppData\Local\OpenAI\Codex\bin\*\codex.exe" }
if ($runningDesktop -or $runningCodex) {
    Write-Host ""
    Write-Host "TOKN P6 PROBE: Codex Desktop is still running." -ForegroundColor Yellow
    Write-Host "Close Codex completely, then run FINISH-P6-HOOK-PROBE.cmd again."
    exit 61
}

$state = Get-Content -LiteralPath $ActivePath -Raw | ConvertFrom-Json
$RunRoot = [string]$state.run_root
$HooksPath = [string]$state.hooks_path
$AuditPath = [string]$state.audit_jsonl
$SummaryPath = [string]$state.summary_json

if (Test-Path -LiteralPath $HooksPath) {
    $currentHash = (Get-FileHash -LiteralPath $HooksPath -Algorithm SHA256).Hash.ToLowerInvariant()
    if ($currentHash -ne [string]$state.installed_hooks_sha256) {
        $unexpected = Join-Path $RunRoot "hooks-at-finish-unexpected.json"
        Copy-Item -LiteralPath $HooksPath -Destination $unexpected -Force
        Write-Warning "hooks.json changed during the probe. Current file preserved at $unexpected and was NOT overwritten."
        Write-Warning "Run RECOVER-P6-HOOK-PROBE.cmd after reviewing it."
        exit 62
    }
}

if ([bool]$state.had_hooks_before) {
    if (-not (Test-Path -LiteralPath ([string]$state.hooks_backup))) {
        throw "Original hooks.json backup is missing. Refusing to overwrite current hook state."
    }
    Copy-Item -LiteralPath ([string]$state.hooks_backup) -Destination $HooksPath -Force
} else {
    Remove-Item -LiteralPath $HooksPath -Force -ErrorAction SilentlyContinue
}

$entries = @()
if (Test-Path -LiteralPath $AuditPath) {
    foreach ($line in Get-Content -LiteralPath $AuditPath) {
        if (-not [string]::IsNullOrWhiteSpace($line)) {
            $entries += ($line | ConvertFrom-Json)
        }
    }
}

$allKeys = @($entries | ForEach-Object { @($_.tool_input_keys) } | ForEach-Object { $_ } | Sort-Object -Unique)
$withMax = @($entries | Where-Object { [bool]$_.max_output_tokens_present }).Count
$sessionStartCallbacks = @($entries | Where-Object { $_.hook_event_name -eq "SessionStart" }).Count
$preToolCallbacks = @($entries | Where-Object { $_.hook_event_name -eq "PreToolUse" }).Count
$sessions = @($entries | ForEach-Object { $_.session_id } | Where-Object { $_ } | Sort-Object -Unique)
$categories = @(
    $entries |
        Group-Object command_category |
        ForEach-Object {
            [pscustomobject]@{
                category = $(if ($_.Name) { $_.Name } else { "UNKNOWN" })
                count = $_.Count
            }
        }
)

$summary = [ordered]@{
    schema_version = 1
    finished_at = (Get-Date).ToString("o")
    callbacks = $entries.Count
    session_start_callbacks = $sessionStartCallbacks
    pre_tool_use_callbacks = $preToolCallbacks
    sessions = $sessions
    tool_input_keys = $allKeys
    max_output_tokens_present_callbacks = $withMax
    max_output_tokens_observed = ($withMax -gt 0)
    categories = $categories
    audit_jsonl = $AuditPath
}
$summary | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath $SummaryPath -Encoding UTF8

Remove-Item -LiteralPath $ActivePath -Force

Write-Host ""
Write-Host "TOKN P6 HOOK PROBE FINISHED" -ForegroundColor Green
Write-Host "Temporary hooks.json restored/removed." -ForegroundColor Green
Write-Host "Callbacks captured: $($entries.Count)"
Write-Host "SessionStart callbacks: $sessionStartCallbacks"
Write-Host "PreToolUse/Bash callbacks: $preToolCallbacks"
Write-Host "tool_input keys: $($allKeys -join ', ')"
Write-Host "max_output_tokens observed in callback: $($withMax -gt 0)"
Write-Host "Summary: $SummaryPath"
Write-Host "Audit:   $AuditPath"
if ($entries.Count -eq 0) {
    Write-Warning "No Tokn audit rows were captured. Do not infer hook inactivity from this alone; inspect Codex lifecycle logs."
} elseif ($sessionStartCallbacks -gt 0 -and $preToolCallbacks -eq 0) {
    Write-Warning "Hook activation is proven by SessionStart, but no PreToolUse/Bash callback was captured."
}
