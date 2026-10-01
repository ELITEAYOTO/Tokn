param([switch]$AllowDirty)

$ErrorActionPreference = "Stop"
$Root = Resolve-Path (Join-Path $PSScriptRoot "..")
Set-Location $Root

$SourceCommit = (& git rev-parse HEAD).Trim()
$SourceBranch = (& git branch --show-current).Trim()
$Dirty = (& git status --porcelain | Out-String).Trim()
if ($Dirty -and -not $AllowDirty) {
    throw "Release provenance requires a clean Git worktree. Commit or stash changes first."
}

& "$PSScriptRoot\with-msvc.cmd" cargo build --release --locked -p tokn-observe -p tokn-mcp
if ($LASTEXITCODE -ne 0) { throw "release build failed" }

$Binary = Join-Path $Root "target\release\tokn-observe.exe"
$BinaryHash = (Get-FileHash -LiteralPath $Binary -Algorithm SHA256).Hash.ToLowerInvariant()
$BinaryVersion = (& $Binary --version | Out-String).Trim()
$ProvenancePath = Join-Path $Root "target\release\tokn-observe.provenance.json"
$Provenance = [ordered]@{
    schema_version = 1
    built_at = (Get-Date).ToString("o")
    source_commit = $SourceCommit
    source_branch = $SourceBranch
    worktree_clean = (-not [bool]$Dirty)
    binary = "target/release/tokn-observe.exe"
    binary_sha256 = $BinaryHash
    binary_version = $BinaryVersion
}
$Provenance | ConvertTo-Json -Depth 4 | Set-Content -LiteralPath $ProvenancePath -Encoding UTF8

$McpBinary = Join-Path $Root "target\release\tokn-mcp.exe"
$McpBinaryHash = (Get-FileHash -LiteralPath $McpBinary -Algorithm SHA256).Hash.ToLowerInvariant()
$McpBinaryVersion = (& $McpBinary --version | Out-String).Trim()
$McpProvenancePath = Join-Path $Root "target\release\tokn-mcp.provenance.json"
$McpProvenance = [ordered]@{
    schema_version = 1
    built_at = (Get-Date).ToString("o")
    source_commit = $SourceCommit
    source_branch = $SourceBranch
    worktree_clean = (-not [bool]$Dirty)
    binary = "target/release/tokn-mcp.exe"
    binary_sha256 = $McpBinaryHash
    binary_version = $McpBinaryVersion
    transport = "stdio"
    read_only = $true
}
$McpProvenance | ConvertTo-Json -Depth 4 | Set-Content -LiteralPath $McpProvenancePath -Encoding UTF8

Write-Host "Release build: PASS"
Write-Host $Binary
Write-Host "Provenance: $ProvenancePath"
Write-Host "SHA-256: $BinaryHash"
Write-Host $McpBinary
Write-Host "MCP provenance: $McpProvenancePath"
Write-Host "MCP SHA-256: $McpBinaryHash"
Write-Host "Source commit: $SourceCommit"
