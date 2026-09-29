param(
    [string]$TraceRoot,
    [switch]$SkipReducer
)

$ErrorActionPreference = "Stop"
$ToolRoot = Resolve-Path (Join-Path $PSScriptRoot "..\..")
$V0Root = Split-Path $ToolRoot -Parent
$AllTracesRoot = Join-Path $V0Root "traces"
$Observer = Join-Path $ToolRoot "target\release\tokn-observe.exe"

Add-Type @"
using System;
using System.Runtime.InteropServices;
public static class ToknEnvBroadcast {
    [DllImport("user32.dll", CharSet = CharSet.Auto, SetLastError = true)]
    public static extern IntPtr SendMessageTimeout(
        IntPtr hWnd, uint Msg, UIntPtr wParam, string lParam,
        uint fuFlags, uint uTimeout, out UIntPtr lpdwResult);
}
"@

function Publish-EnvironmentChange {
    $result = [UIntPtr]::Zero
    [void][ToknEnvBroadcast]::SendMessageTimeout(
        [IntPtr]0xffff, 0x001A, [UIntPtr]::Zero, "Environment",
        2, 5000, [ref]$result)
}

[Environment]::SetEnvironmentVariable(
    "CODEX_ROLLOUT_TRACE_ROOT", $null, "User")
Publish-EnvironmentChange

if (-not (Test-Path $Observer)) {
    throw "Tokn Observer release binary not found. Run scripts\build-release.ps1 first."
}

if (-not $TraceRoot) {
    $TraceRoot = Get-ChildItem $AllTracesRoot -Directory |
        Sort-Object LastWriteTime -Descending |
        Select-Object -First 1 -ExpandProperty FullName
}

if (-not $TraceRoot -or -not (Test-Path $TraceRoot)) {
    throw "No trace run directory found."
}

$bundle = Get-ChildItem $TraceRoot -Filter manifest.json -File -Recurse |
    ForEach-Object {
        $dir = $_.Directory.FullName
        if (Test-Path (Join-Path $dir "trace.jsonl")) { $dir }
    } |
    Select-Object -First 1

if (-not $bundle) {
    Write-Host ""
    Write-Host "No complete diagnostic bundle found yet." -ForegroundColor Yellow
    Write-Host "Expected manifest.json + trace.jsonl under:"
    Write-Host $TraceRoot
    Write-Host ""
    Write-Host "Keep this folder. Tokn can inspect it if Codex used another layout."
    exit 21
}

Write-Host ""
Write-Host "TOKN TRACE FOUND" -ForegroundColor Green
Write-Host "Bundle: $bundle"
Write-Host ""

& $Observer import $bundle
if ($LASTEXITCODE -ne 0) { throw "Tokn import failed." }

& $Observer report
if ($LASTEXITCODE -ne 0) { throw "Tokn report failed." }

if (-not $SkipReducer) {
    $codex = Get-ChildItem "$env:LOCALAPPDATA\OpenAI\Codex\bin" -Filter codex.exe -File -Recurse |
        Sort-Object LastWriteTime -Descending |
        Select-Object -First 1 -ExpandProperty FullName

    if ($codex) {
        $cacheRoot = Join-Path $env:LOCALAPPDATA "Tokn\Observer\cache\oracle"
        New-Item -ItemType Directory -Force -Path $cacheRoot | Out-Null
        $bundleName = Split-Path $bundle -Leaf
        $stateOut = Join-Path $cacheRoot "$bundleName-state.json"

        Write-Host ""
        Write-Host "Running official trace-reduce oracle..."
        & $codex debug trace-reduce $bundle -o $stateOut

        if ($LASTEXITCODE -eq 0) {
            Write-Host "Reducer state: $stateOut"
        } else {
            Write-Warning "trace-reduce failed; original bundle was not modified."
        }
    } else {
        Write-Warning "Embedded codex.exe not found; reducer skipped."
    }
}

$metaPath = Join-Path $TraceRoot "tokn-run.json"
if (Test-Path $metaPath) {
    try {
        $meta = Get-Content $metaPath -Raw | ConvertFrom-Json
        $meta | Add-Member -NotePropertyName finished_at -NotePropertyValue ((Get-Date).ToString("o")) -Force
        $meta | Add-Member -NotePropertyName bundle_path -NotePropertyValue $bundle -Force
        $meta | ConvertTo-Json -Depth 5 | Set-Content -Encoding UTF8 $metaPath
    } catch {
        Write-Warning "Could not update tokn-run.json: $($_.Exception.Message)"
    }
}

Write-Host ""
Write-Host "Tokn trace analysis finished." -ForegroundColor Green
Write-Host "Original trace remains untouched."
