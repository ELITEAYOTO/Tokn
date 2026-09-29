$ErrorActionPreference = "Stop"
$Root = Resolve-Path (Join-Path $PSScriptRoot "..")
Set-Location $Root

& "$PSScriptRoot\with-msvc.cmd" cargo build --release --locked -p tokn-observe
if ($LASTEXITCODE -ne 0) { throw "release build failed" }

Write-Host "Release build: PASS"
Write-Host (Join-Path $Root "target\release\tokn-observe.exe")
