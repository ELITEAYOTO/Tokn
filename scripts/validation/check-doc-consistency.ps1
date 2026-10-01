param()

$ErrorActionPreference = "Stop"
$Repo = (Resolve-Path (Join-Path $PSScriptRoot "..\..")).Path

function Read-RepoFile {
    param([string]$RelativePath)
    $path = Join-Path $Repo $RelativePath
    if (-not (Test-Path -LiteralPath $path)) {
        throw "required documentation file missing: $RelativePath"
    }
    Get-Content -LiteralPath $path -Raw
}

function Assert-Contains {
    param([string]$File, [string]$Text)
    $content = Read-RepoFile $File
    if (-not $content.Contains($Text)) {
        throw "$File missing expected text: $Text"
    }
    Write-Host "[PASS] $File contains: $Text"
}

function Assert-NotContains {
    param([string]$File, [string]$Text)
    $content = Read-RepoFile $File
    if ($content.Contains($Text)) {
        throw "$File contains stale text: $Text"
    }
}

$Cargo = Read-RepoFile "Cargo.toml"
if ($Cargo -notmatch '(?m)^version = "0\.1\.0"\r?$') {
    throw "Cargo workspace version is not 0.1.0"
}
Write-Host "[PASS] Cargo workspace version = 0.1.0"

Assert-Contains "README.md" "**P0-P9 DONE ; Experiment 002 ACCEPTED ; Measurement Contract Freeze NEXT.**"
Assert-Contains "STATUS.md" "**P0-P9 DONE. Experiment 002 est ACCEPTED. Measurement Contract Freeze est NEXT.**"
Assert-Contains "ROADMAP.md" "P9 golden replay/release validation : DONE"
Assert-Contains "docs/plans/V0.1-IMPLEMENTATION-PLAN.md" "Status: **DONE - P0-P9 DONE**"
Assert-Contains "docs/plans/V0.1-TEST-MATRIX.md" "Status: **P0-P9 PASS / V0.1 DONE**"
Assert-Contains "docs/plans/NEXT-SESSION-CHECKLIST.md" "P0-P9 : DONE"
Assert-Contains "docs/plans/NEXT-SESSION-CHECKLIST.md" "Measurement Contract Freeze : NEXT"
Assert-Contains "docs/plans/IMPLEMENTATION-PATH.md" "Current state: P0-P9 DONE / Experiment 002 ACCEPTED / Measurement Contract Freeze NEXT"
Assert-Contains "docs/experiments/002-INSTRUMENTATION-VALIDATION.md" "Status: DONE / ACCEPTED"
Assert-Contains "docs/INDEX.md" "**P0-P9 DONE ; Experiment 002 ACCEPTED ; Measurement Contract Freeze NEXT.**"

$activeDocs = @(
    "README.md",
    "STATUS.md",
    "ROADMAP.md",
    "docs/plans/V0.1-IMPLEMENTATION-PLAN.md",
    "docs/plans/V0.1-TEST-MATRIX.md",
    "docs/plans/NEXT-SESSION-CHECKLIST.md",
    "docs/plans/IMPLEMENTATION-PATH.md",
    "docs/experiments/002-INSTRUMENTATION-VALIDATION.md",
    "docs/INDEX.md"
)
$stale = @(
    "P9 est IN PROGRESS",
    "P9 : IN PROGRESS",
    "P9 : TODO",
    "P8 : IN PROGRESS",
    "BLOCKED BY P8-P9",
    "BLOCKED BY V0.1 P7-P9",
    "P0-P7 DONE ; P8 NEXT",
    "P0-P8 DONE / P9 IN PROGRESS"
)

foreach ($file in $activeDocs) {
    foreach ($needle in $stale) {
        Assert-NotContains $file $needle
    }
}

Write-Host "Documentation consistency: PASS"
