param([switch]$DryRun)

$ErrorActionPreference = "Stop"
$ToolRoot = Resolve-Path (Join-Path $PSScriptRoot "..\..")
$V0Root = Split-Path $ToolRoot -Parent
$ProbeRoot = Join-Path $V0Root "experiments\p6-hook-probe"
$Workspace = Join-Path $ProbeRoot "workspace"
$ActivePath = Join-Path $ProbeRoot "ACTIVE-PROBE.json"
$Observer = Join-Path $ToolRoot "target\release\tokn-observe.exe"
$CodexHome = Join-Path $env:USERPROFILE ".codex"
$HooksPath = Join-Path $CodexHome "hooks.json"

if (Test-Path -LiteralPath $ActivePath) {
    throw "A P6 hook probe is already active. Finish or recover it first: $ActivePath"
}
if (-not (Test-Path -LiteralPath $Observer)) {
    throw "Tokn release binary not found: $Observer"
}
if (-not (Test-Path -LiteralPath $Workspace)) {
    throw "Probe workspace not found: $Workspace"
}

$runningDesktop = Get-Process ChatGPT -ErrorAction SilentlyContinue
$runningCodex = Get-Process codex -ErrorAction SilentlyContinue |
    Where-Object { $_.Path -like "*\AppData\Local\OpenAI\Codex\bin\*\codex.exe" }
if (-not $DryRun -and ($runningDesktop -or $runningCodex)) {
    Write-Host ""
    Write-Host "TOKN P6 PROBE: Codex Desktop is still running." -ForegroundColor Yellow
    Write-Host "Close Codex completely, then run START-P6-HOOK-PROBE.cmd again."
    exit 60
}

New-Item -ItemType Directory -Force -Path $ProbeRoot | Out-Null
New-Item -ItemType Directory -Force -Path $CodexHome | Out-Null

$HadHooks = Test-Path -LiteralPath $HooksPath
if ($HadHooks) {
    throw "Existing ~/.codex/hooks.json detected. Tokn refuses to overwrite or merge it automatically."
}

$stamp = Get-Date -Format "yyyyMMdd-HHmmss"
$RunRoot = Join-Path $ProbeRoot "runs\$stamp"
New-Item -ItemType Directory -Force -Path $RunRoot | Out-Null
$AuditPath = Join-Path $RunRoot "pretooluse-audit.jsonl"
$SummaryPath = Join-Path $RunRoot "summary.json"
$BackupPath = Join-Path $RunRoot "hooks-before.json"

$observerEscaped = $Observer.Replace('"', '\"')
$auditEscaped = $AuditPath.Replace('"', '\"')
$hookCommand = '"' + $observerEscaped + '" hook-probe-pre-tool-use --audit-jsonl "' + $auditEscaped + '"'

$hookConfig = [ordered]@{
    description = "Temporary Tokn P6 PreToolUse schema probe. Observation only."
    hooks = [ordered]@{
        SessionStart = @(
            [ordered]@{
                matcher = "^startup$"
                hooks = @(
                    [ordered]@{
                        type = "command"
                        command = $hookCommand
                        commandWindows = $hookCommand
                        timeout = 5
                        statusMessage = "Tokn P6 checking hook activation"
                    }
                )
            }
        )
        PreToolUse = @(
            [ordered]@{
                matcher = "^Bash$"
                hooks = @(
                    [ordered]@{
                        type = "command"
                        command = $hookCommand
                        commandWindows = $hookCommand
                        timeout = 5
                        statusMessage = "Tokn P6 observing PreToolUse schema"
                    }
                )
            }
        )
    }
}

$state = [ordered]@{
    schema_version = 1
    started_at = (Get-Date).ToString("o")
    run_root = $RunRoot
    workspace = $Workspace
    hooks_path = $HooksPath
    had_hooks_before = $HadHooks
    hooks_backup = $(if ($HadHooks) { $BackupPath } else { $null })
    audit_jsonl = $AuditPath
    summary_json = $SummaryPath
}
$state | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath $ActivePath -Encoding UTF8

$hookConfig | ConvertTo-Json -Depth 10 | Set-Content -LiteralPath $HooksPath -Encoding UTF8
$InstalledHash = (Get-FileHash -LiteralPath $HooksPath -Algorithm SHA256).Hash.ToLowerInvariant()
$state.installed_hooks_sha256 = $InstalledHash
$state | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath $ActivePath -Encoding UTF8

$prompt = @"
Fais uniquement une inspection en lecture seule de ce petit workspace.
Ne modifie aucun fichier.

1. Lis README.md et notes.txt avec tes outils locaux.
2. Recherche dans tout le workspace les chaines TODO et TOKN_P6_PROBE_2026.
3. Donne-moi ensuite un resume tres court de ce que tu as trouve.

Utilise ton fonctionnement normal d'Astra pour inspecter les fichiers, sans implementation ni modification.
"@
Set-Clipboard -Value $prompt

if ($DryRun) {
    Remove-Item -LiteralPath $HooksPath -Force
    Remove-Item -LiteralPath $ActivePath -Force
    Remove-Item -LiteralPath $RunRoot -Recurse -Force
    Write-Host "TOKN P6 HOOK PROBE DRY-RUN: PASS" -ForegroundColor Green
    exit 0
}

$codexCommand = Get-Command "codex.cmd" -ErrorAction SilentlyContinue
if (-not $codexCommand) {
    throw "Packaged Codex CLI not found in PATH. Install it with: npm install -g @openai/codex@alpha"
}
$codexCli = $codexCommand.Source

$cliCommand = '"' + $codexCli + '"'
Start-Process "cmd.exe" -WorkingDirectory $Workspace -ArgumentList "/k", $cliCommand

Write-Host ""
Write-Host "TOKN P6 HOOK PROBE READY" -ForegroundColor Green
Write-Host "Workspace: $Workspace"
Write-Host "Audit:     $AuditPath"
Write-Host ""
Write-Host "The probe prompt is already in your clipboard." -ForegroundColor Cyan
Write-Host "A Codex CLI window has been opened for hook trust." -ForegroundColor Yellow
Write-Host "1. In that CLI window, type: /hooks"
Write-Host "2. Review and trust the temporary Tokn hook."
Write-Host "3. Exit/close the Codex CLI window."
Write-Host "4. Open Codex Desktop normally."
Write-Host "5. Open the probe workspace and create ONE new Astra chat."
Write-Host "6. Paste the clipboard prompt, send it, and wait for completion."
Write-Host "7. Close Codex completely and run FINISH-P6-HOOK-PROBE.cmd."
