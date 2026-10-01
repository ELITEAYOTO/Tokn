param([string]$BinaryPath = 'target\release\tokn-observe.exe')

$ErrorActionPreference = 'Stop'
$Repo = (Resolve-Path (Join-Path $PSScriptRoot '..\..')).Path
$TraceRoot = Join-Path $Repo 'fixtures\experiments\002-source-selection\trace-root'
$Sessions = Join-Path $Repo 'fixtures\experiments\001-runner-golden\sessions'
$After = Join-Path $Repo 'fixtures\experiments\001-runner-golden\workspace-after-inventory.json'
$Before = Join-Path $Repo 'fixtures\experiments\001-runner-golden\workspace-before-inventory.json'
$OutDir = Join-Path $Repo 'target\exp002-source-selection'
$Out = Join-Path $OutDir 'source-selection.json'
$Bin = Join-Path $Repo $BinaryPath
$Script = Join-Path $Repo 'scripts\experiment\resolve-exp002-source.ps1'

if (-not (Test-Path -LiteralPath $Bin -PathType Leaf)) { throw "Tokn binary missing: $Bin" }
Remove-Item -LiteralPath $OutDir -Recurse -Force -ErrorAction SilentlyContinue
New-Item -ItemType Directory -Force -Path $OutDir | Out-Null

& $Script -TraceRoot $TraceRoot -ToknBin $Bin -AfterInventory $After -BeforeInventory $Before -SourceRoot 'E:\fixture\B07-B_WORKING\PROJECT' -ExpectedOutput 'E:\fixture\B07-C_WORKING\PROJECT' -OutputPath $Out -SessionRoots @($Sessions)
if (-not (Test-Path -LiteralPath $Out -PathType Leaf)) { throw 'Experiment 002 source-selection report missing.' }
$report = Get-Content -LiteralPath $Out -Raw | ConvertFrom-Json

if ([string]$report.source_kind -ne 'codex-session') { throw "Expected codex-session fallback, got $($report.source_kind)" }
if (-not [bool]$report.fallback_used) { throw 'Expected fallback_used=true for empty diagnostic fixture.' }
if ((Split-Path ([string]$report.selected_source) -Leaf) -ne 'parent.jsonl') { throw "Unexpected parent source: $($report.selected_source)" }
if (@($report.session_candidates).Count -ne 3) { throw "Expected 3 subagent candidates, got $(@($report.session_candidates).Count)" }
if ([string]$report.selected_workspace -ne 'E:\fixture\B07-C_WORKING\PROJECT') { throw "Unexpected selected workspace: $($report.selected_workspace)" }

Write-Host 'Experiment 002 source selection: PASS'
Remove-Item -LiteralPath $OutDir -Recurse -Force -ErrorAction SilentlyContinue
