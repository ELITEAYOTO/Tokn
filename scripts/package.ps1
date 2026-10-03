$ErrorActionPreference = "Stop"
$Root = (Resolve-Path (Join-Path $PSScriptRoot "..")).Path
$OutDir = Join-Path (Split-Path $Root -Parent) "artifacts"
$Stage = Join-Path $env:TEMP "tokn-observer-v01-stage"
$Zip = Join-Path $OutDir "Tokn-Observer-V0.1.zip"

Push-Location $Root
try {
    $dirty = (& git status --porcelain | Out-String).Trim()
    if ($LASTEXITCODE -ne 0) { throw "git status failed" }
    if ($dirty) { throw "Packaging requires a clean Git worktree." }

    $tracked = @(git ls-files)
    if ($LASTEXITCODE -ne 0) { throw "git ls-files failed" }

    if (Test-Path $Stage) { Remove-Item $Stage -Recurse -Force }
    New-Item -ItemType Directory -Force -Path $Stage | Out-Null
    New-Item -ItemType Directory -Force -Path $OutDir | Out-Null

    foreach ($relative in $tracked) {
        $source = Join-Path $Root $relative
        if (-not (Test-Path -LiteralPath $source -PathType Leaf)) { continue }
        $destination = Join-Path $Stage ($relative -replace '/', '\')
        $parent = Split-Path $destination -Parent
        if ($parent) { New-Item -ItemType Directory -Force -Path $parent | Out-Null }
        Copy-Item -LiteralPath $source -Destination $destination -Force
    }

    $ReleaseBinary = Join-Path $Root "target\release\tokn-observe.exe"
    $ReleaseProvenance = Join-Path $Root "target\release\tokn-observe.provenance.json"
    foreach ($required in @($ReleaseBinary, $ReleaseProvenance)) {
        if (-not (Test-Path -LiteralPath $required -PathType Leaf)) {
            throw "Release artifact missing: $required"
        }
    }
    $BinDir = Join-Path $Stage "bin"
    New-Item -ItemType Directory -Force -Path $BinDir | Out-Null
    Copy-Item -LiteralPath $ReleaseBinary -Destination (Join-Path $BinDir "tokn-observe.exe") -Force
    Copy-Item -LiteralPath $ReleaseProvenance -Destination (Join-Path $BinDir "tokn-observe.provenance.json") -Force

    $V0Root = Split-Path $Root -Parent
    $LauncherRoot = Join-Path $V0Root "launchers"
    $KitRoot = Join-Path $Stage "experiment-kit"
    New-Item -ItemType Directory -Force -Path $KitRoot | Out-Null

    $wrappers = @(
        "START-EXP001-CANDIDATE.cmd",
        "FINISH-EXP001-CANDIDATE.cmd",
        "RECOVER-EXP001.cmd",
        "EDIT-EXP001-TASK.cmd"
    )
    foreach ($wrapper in $wrappers) {
        $source = Join-Path $LauncherRoot $wrapper
        if (Test-Path -LiteralPath $source -PathType Leaf) {
            Copy-Item -LiteralPath $source -Destination $KitRoot -Force
        }
    }

    $experimentSource = Join-Path $V0Root "experiments\001-runtime-output-caps"
    $experimentDest = Join-Path $KitRoot "001-runtime-output-caps"
    if (Test-Path $experimentSource) {
        New-Item -ItemType Directory -Force -Path $experimentDest | Out-Null
        foreach ($name in @("experiment.json", "TASK.md", "QUALITY_AFTER_RUN.md")) {
            $source = Join-Path $experimentSource $name
            if (Test-Path -LiteralPath $source -PathType Leaf) {
                Copy-Item -LiteralPath $source -Destination $experimentDest -Force
            }
        }
        $policySource = Join-Path $experimentSource "policies"
        if (Test-Path $policySource) {
            Copy-Item -LiteralPath $policySource -Destination $experimentDest -Recurse -Force
        }
    }

    if (Test-Path $Zip) { Remove-Item $Zip -Force }
    Compress-Archive -Path (Join-Path $Stage "*") -DestinationPath $Zip -CompressionLevel Optimal
    try {
        & (Join-Path $Root "scripts\validation\check-package-privacy.ps1") -ZipPath $Zip
        if ($LASTEXITCODE -ne 0) { throw "Package privacy validation failed." }
    }
    catch {
        Remove-Item $Zip -Force -ErrorAction SilentlyContinue
        throw
    }

    Write-Host $Zip
}
finally {
    if (Test-Path $Stage) { Remove-Item $Stage -Recurse -Force -ErrorAction SilentlyContinue }
    Pop-Location
}
