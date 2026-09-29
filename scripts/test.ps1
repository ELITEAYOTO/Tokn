$ErrorActionPreference = "Stop"
$Root = Resolve-Path (Join-Path $PSScriptRoot "..")
Set-Location $Root

& "$PSScriptRoot\with-msvc.cmd" cargo fmt --all -- --check
if ($LASTEXITCODE -ne 0) { throw "cargo fmt --check failed" }

& "$PSScriptRoot\with-msvc.cmd" cargo clippy --workspace --all-targets -- -D warnings
if ($LASTEXITCODE -ne 0) { throw "cargo clippy failed" }

& "$PSScriptRoot\with-msvc.cmd" cargo test --workspace
if ($LASTEXITCODE -ne 0) { throw "cargo test failed" }

Write-Host "Tokn Observer tests: PASS"
