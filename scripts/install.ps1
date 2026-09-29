$ErrorActionPreference = "Stop"
$Root = Resolve-Path (Join-Path $PSScriptRoot "..")
$Exe = Join-Path $Root "target\release\tokn-observe.exe"

if (-not (Test-Path $Exe)) {
    & "$PSScriptRoot\build-release.ps1"
}
if (-not (Test-Path $Exe)) {
    throw "tokn-observe.exe not found after build"
}

$Dest = Join-Path $env:LOCALAPPDATA "Tokn\bin"
New-Item -ItemType Directory -Force -Path $Dest | Out-Null
Copy-Item $Exe (Join-Path $Dest "tokn-observe.exe") -Force
Write-Host "Installed to $Dest"
