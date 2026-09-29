$ErrorActionPreference = "Stop"

$ToolRoot = Resolve-Path (Join-Path $PSScriptRoot "..\..")
$V0Root = Split-Path $ToolRoot -Parent
$ProbeRoot = Join-Path $V0Root "experiments\p6-hook-probe"
$ActivePath = Join-Path $ProbeRoot "ACTIVE-PROBE.json"

$runningDesktop = Get-Process ChatGPT -ErrorAction SilentlyContinue
$runningCodex = Get-Process codex -ErrorAction SilentlyContinue |
    Where-Object { $_.Path -like "*\AppData\Local\OpenAI\Codex\bin\*\codex.exe" }
if ($runningDesktop -or $runningCodex) {
    Write-Host ""
    Write-Host "TOKN P6 RECOVERY: Codex Desktop is still running." -ForegroundColor Yellow
    Write-Host "Close Codex completely, then run recovery again."
    exit 63
}

if (-not (Test-Path -LiteralPath $ActivePath)) {
    Write-Host "No active P6 hook probe. Nothing to recover."
    exit 0
}

$state = Get-Content -LiteralPath $ActivePath -Raw | ConvertFrom-Json
$HooksPath = [string]$state.hooks_path

if ([bool]$state.had_hooks_before) {
    if (Test-Path -LiteralPath ([string]$state.hooks_backup)) {
        Copy-Item -LiteralPath ([string]$state.hooks_backup) -Destination $HooksPath -Force
    } else {
        Write-Warning "Original hooks.json backup missing. Current hook file was left untouched."
        exit 64
    }
} else {
    if (Test-Path -LiteralPath $HooksPath) {
        $currentHash = (Get-FileHash -LiteralPath $HooksPath -Algorithm SHA256).Hash.ToLowerInvariant()
        if ($currentHash -eq [string]$state.installed_hooks_sha256) {
            Remove-Item -LiteralPath $HooksPath -Force
        } else {
            $preserved = Join-Path ([string]$state.run_root) "hooks-at-recovery-unexpected.json"
            Copy-Item -LiteralPath $HooksPath -Destination $preserved -Force
            Write-Warning "hooks.json changed after Tokn installed it. It was preserved and left in place: $preserved"
            exit 65
        }
    }
}

Remove-Item -LiteralPath $ActivePath -Force
Write-Host ""
Write-Host "TOKN P6 HOOK PROBE RECOVERY COMPLETE" -ForegroundColor Green
Write-Host "Temporary hook state restored."
Write-Host "Probe run artifacts were preserved at: $($state.run_root)"
