param(
    [Parameter(Mandatory = $true)]
    [ValidateRange(1, 999)]
    [int]$PackNumber
)

$ErrorActionPreference = "Stop"

$RepoRoot = Resolve-Path (Join-Path $PSScriptRoot "..\..")
$WorkspaceRoot = Split-Path $RepoRoot -Parent
$OutRoot = Join-Path $WorkspaceRoot "context-packs"

Push-Location $RepoRoot
try {
    $Commit = (git rev-parse HEAD).Trim()
    $ShortCommit = (git rev-parse --short=7 HEAD).Trim()
    $Branch = (git branch --show-current).Trim()
    $Dirty = git status --porcelain
    if ($Dirty) {
        throw "Worktree is not clean; refusing to build a canonical Context Pack."
    }

    $PackId = "TOKN-CONTEXT-PACK-{0:D3}" -f $PackNumber
    $Date = Get-Date -Format "yyyyMMdd"
    $PackName = "$PackId-$Date-$ShortCommit"
    $Stage = Join-Path $OutRoot $PackName
    $Zip = Join-Path $OutRoot ($PackName + ".zip")
    $ShaFile = Join-Path $OutRoot ($PackName + ".sha256")

    New-Item -ItemType Directory -Force -Path $OutRoot | Out-Null
    foreach ($path in @($Stage, $Zip, $ShaFile)) {
        if (Test-Path $path) {
            Remove-Item $path -Recurse -Force
        }
    }
    New-Item -ItemType Directory -Force -Path $Stage | Out-Null

    foreach ($name in @("README.md", "STATUS.md", "ROADMAP.md", "CHANGELOG.md", "SECURITY.md")) {
        Copy-Item (Join-Path $RepoRoot $name) $Stage
    }
    Copy-Item (Join-Path $RepoRoot "docs") (Join-Path $Stage "docs") -Recurse

    $Files = Get-ChildItem -Path $Stage -Recurse -File | Sort-Object FullName
    $Manifest = @(
        "# Tokn Context Pack Manifest",
        "",
        "Pack ID: $PackId",
        "Generated at: $(Get-Date -Format 'yyyy-MM-dd')",
        "Source branch: $Branch",
        "Source commit: $Commit",
        "Worktree state: clean",
        "Project state: P0-P7 DONE / P8 NEXT / P9 TODO",
        "Next milestone: P8 - Runner V0.1 self-contained",
        "",
        "## Canonical source",
        "",
        "The Tokn Git repository is the source of truth.",
        "This ZIP is a portable documentation snapshot derived from that exact commit.",
        "It never overrides a newer canonical repository state.",
        "",
        "## Evidence vocabulary",
        "",
        "- ACCEPTED: project decision.",
        "- VERIFIED: external fact confirmed by an authoritative current source.",
        "- OBSERVED: direct runtime/fixture/experiment observation.",
        "- RESEARCH: researched input not yet fully verified/adopted.",
        "- HYPOTHESIS: testable explanation or proposal.",
        "- UNKNOWN: unavailable or unverified.",
        "- SUPERSEDED: replaced by newer evidence or decision.",
        "",
        "UNKNOWN is never silently converted to zero, false, unsupported or PASS.",
        "",
        "## Inventory",
        "",
        "| Path | Bytes | SHA-256 |",
        "| --- | ---: | --- |"
    )

    foreach ($File in $Files) {
        $Relative = $File.FullName.Substring($Stage.Length + 1).Replace("\", "/")
        $Hash = (Get-FileHash -Algorithm SHA256 -Path $File.FullName).Hash.ToLower()
        $Manifest += "| $Relative | $($File.Length) | $Hash |"
    }

    $Manifest += @(
        "",
        "## Re-integration rule",
        "",
        "Compare the source commit with the current repository.",
        "Merge deltas at canonical paths and reconcile STATUS, ROADMAP, INDEX, ADRs and plans.",
        "Review git diff and regenerate a new pack from the resulting clean commit."
    )

    $Manifest | Set-Content -Encoding UTF8 -Path (Join-Path $Stage "CONTEXT-PACK-MANIFEST.md")

    Compress-Archive -Path (Join-Path $Stage "*") -DestinationPath $Zip -CompressionLevel Optimal
    $ZipHash = (Get-FileHash -Algorithm SHA256 -Path $Zip).Hash.ToLower()
    ($ZipHash + "  " + [IO.Path]::GetFileName($Zip)) |
        Set-Content -NoNewline -Encoding ascii -Path $ShaFile

    Write-Host "PACK=$Stage"
    Write-Host "ZIP=$Zip"
    Write-Host "SHA256=$ZipHash"
    Write-Host "SOURCE_COMMIT=$Commit"
}
finally {
    Pop-Location
}
