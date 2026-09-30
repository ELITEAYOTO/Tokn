$ErrorActionPreference = "Stop"
$Root = Resolve-Path (Join-Path $PSScriptRoot "..")
$OutDir = Join-Path (Split-Path $Root -Parent) "artifacts"
$Stage = Join-Path $env:TEMP "tokn-observer-v01-stage"
$Zip = Join-Path $OutDir "Tokn-Observer-V0.1-dev.zip"

if (Test-Path $Stage) { Remove-Item $Stage -Recurse -Force }
New-Item -ItemType Directory -Force -Path $Stage | Out-Null
New-Item -ItemType Directory -Force -Path $OutDir | Out-Null

$Exclude = @("target", "artifacts", ".git")
Get-ChildItem $Root -Force | Where-Object { $Exclude -notcontains $_.Name } | ForEach-Object {
    Copy-Item $_.FullName -Destination $Stage -Recurse -Force
}

$ReleaseBinary = Join-Path $Root "target\release\tokn-observe.exe"
if (-not (Test-Path -LiteralPath $ReleaseBinary)) {
    throw "Release binary missing: $ReleaseBinary"
}
$BinDir = Join-Path $Stage "bin"
New-Item -ItemType Directory -Force -Path $BinDir | Out-Null
Copy-Item -LiteralPath $ReleaseBinary -Destination (Join-Path $BinDir "tokn-observe.exe") -Force

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
    if (Test-Path $source) {
        Copy-Item -LiteralPath $source -Destination $KitRoot -Force
    }
}

$experimentSource = Join-Path $V0Root "experiments\001-runtime-output-caps"
$experimentDest = Join-Path $KitRoot "001-runtime-output-caps"
if (Test-Path $experimentSource) {
    New-Item -ItemType Directory -Force -Path $experimentDest | Out-Null
    foreach ($name in @("experiment.json", "TASK.md", "QUALITY_AFTER_RUN.md")) {
        $source = Join-Path $experimentSource $name
        if (Test-Path $source) {
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
Remove-Item $Stage -Recurse -Force
Write-Host $Zip
