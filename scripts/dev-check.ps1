param(
    [switch]$Full
)

$ErrorActionPreference = "Stop"
$Root = (Resolve-Path (Join-Path $PSScriptRoot ".." )).Path
Set-Location $Root

function Invoke-CheckedStep {
    param(
        [Parameter(Mandatory = $true)][string]$Name,
        [Parameter(Mandatory = $true)][scriptblock]$Action
    )
    Write-Host "`n== $Name ==" -ForegroundColor Cyan
    & $Action
    if ($LASTEXITCODE -ne 0) {
        throw "$Name failed with exit code $LASTEXITCODE"
    }
}

Invoke-CheckedStep "Repository privacy" { ./scripts/validation/check-repo-publication.ps1 }
Invoke-CheckedStep "PowerShell syntax" { ./scripts/validation/check-powershell-syntax.ps1 }
Invoke-CheckedStep "Benchmark manifest" { ./scripts/validation/check-benchmark-manifest.ps1 }
Invoke-CheckedStep "Rust fmt / Clippy / tests" { ./scripts/test.ps1 }
Invoke-CheckedStep "Documentation consistency" { ./scripts/validation/check-doc-consistency.ps1 }

if ($Full) {
    Invoke-CheckedStep "Release build + provenance" { ./scripts/build-release.ps1 -AllowDirty }
    Invoke-CheckedStep "MCP prototype smoke" { ./scripts/validation/check-mcp-prototype.ps1 }
    Invoke-CheckedStep "Experiment 001 golden replay" { ./scripts/validation/replay-exp001-golden.ps1 }
    Invoke-CheckedStep "Experiment 002 source selection" { ./scripts/validation/check-exp002-source-selection.ps1 }
}

Write-Host "`nTokn developer check: PASS" -ForegroundColor Green
if (-not $Full) {
    Write-Host "Tip: use ./scripts/dev-check.ps1 -Full before opening a PR."
}
