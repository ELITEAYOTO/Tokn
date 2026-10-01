param(
    [string]$BinaryPath = "target\release\tokn-mcp.exe"
)

$ErrorActionPreference = "Stop"
$Repo = (Resolve-Path (Join-Path $PSScriptRoot "..\..")).Path
Set-Location $Repo

if (-not (Test-Path -LiteralPath $BinaryPath)) {
    throw "Tokn MCP binary not found: $BinaryPath"
}

$Db = Join-Path $Repo "target\mcp-prototype-smoke.sqlite3"
if (Test-Path -LiteralPath $Db) {
    Remove-Item -LiteralPath $Db -Force
}

$psi = [System.Diagnostics.ProcessStartInfo]::new()
$psi.FileName = (Resolve-Path $BinaryPath).Path
$psi.Arguments = '--db "' + $Db + '"'
$psi.UseShellExecute = $false
$psi.CreateNoWindow = $true
$psi.RedirectStandardInput = $true
$psi.RedirectStandardOutput = $true
$psi.RedirectStandardError = $true

$process = [System.Diagnostics.Process]::new()
$process.StartInfo = $psi

try {
    if (-not $process.Start()) {
        throw "Failed to start Tokn MCP process"
    }

    $messages = @(
        '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-06-18","clientInfo":{"name":"tokn-smoke","version":"1"},"capabilities":{}}}',
        '{"jsonrpc":"2.0","method":"notifications/initialized"}',
        '{"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}',
        '{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"tokn_status","arguments":{}}}',
        '{"jsonrpc":"2.0","id":4,"method":"tools/call","params":{"name":"tokn_recent_runs","arguments":{"limit":5}}}',
        '{"jsonrpc":"2.0","id":5,"method":"tools/call","params":{"name":"tokn_context_ledger","arguments":{}}}'
    )

    foreach ($message in $messages) {
        $process.StandardInput.WriteLine($message)
    }
    $process.StandardInput.Close()

    $stdout = $process.StandardOutput.ReadToEnd()
    $stderr = $process.StandardError.ReadToEnd()
    $process.WaitForExit()

    if ($process.ExitCode -ne 0) {
        throw "Tokn MCP exited with code $($process.ExitCode): $stderr"
    }
    if (-not [string]::IsNullOrWhiteSpace($stderr)) {
        throw "Tokn MCP wrote unexpected stderr: $stderr"
    }

    $lines = @($stdout -split "\r?\n" | Where-Object { -not [string]::IsNullOrWhiteSpace($_) })
    if ($lines.Count -ne 5) {
        throw "Expected 5 MCP responses, got $($lines.Count): $stdout"
    }

    $initialize = $lines[0] | ConvertFrom-Json
    if ($initialize.id -ne 1) { throw "Initialize response id mismatch" }
    if ($initialize.result.protocolVersion -ne "2025-06-18") {
        throw "Protocol negotiation mismatch"
    }
    if ($initialize.result.serverInfo.name -ne "tokn-mcp") {
        throw "Server name mismatch"
    }

    $tools = $lines[1] | ConvertFrom-Json
    if ($tools.id -ne 2) { throw "tools/list response id mismatch" }
    $toolNames = @($tools.result.tools | ForEach-Object { $_.name })
    if ($toolNames.Count -ne 3) { throw "Expected exactly 3 MCP tools" }
    if ($toolNames -notcontains "tokn_status") { throw "tokn_status tool missing" }
    if ($toolNames -notcontains "tokn_recent_runs") { throw "tokn_recent_runs tool missing" }
    if ($toolNames -notcontains "tokn_context_ledger") { throw "tokn_context_ledger tool missing" }

    $status = $lines[2] | ConvertFrom-Json
    if ($status.id -ne 3) { throw "tokn_status response id mismatch" }
    $statusPayload = $status.result.content[0].text | ConvertFrom-Json
    if ($statusPayload.store.schema_version -ne "2") {
        throw "Store schema mismatch"
    }
    if ([int]$statusPayload.store.runs -ne 0) {
        throw "Fresh smoke Store should have 0 runs"
    }
    if ($statusPayload.server.transport -ne "stdio") {
        throw "MCP transport mismatch"
    }
    if ($statusPayload.server.read_only -ne $true) {
        throw "MCP server must report read_only=true"
    }

    $history = $lines[3] | ConvertFrom-Json
    if ($history.id -ne 4) { throw "tokn_recent_runs response id mismatch" }
    $historyPayload = $history.result.content[0].text | ConvertFrom-Json
    if ([int]$historyPayload.count -ne 0) {
        throw "Fresh smoke Store history should be empty"
    }

    $ledger = $lines[4] | ConvertFrom-Json
    if ($ledger.id -ne 5) { throw "tokn_context_ledger response id mismatch" }
    $ledgerPayload = $ledger.result.content[0].text | ConvertFrom-Json
    if ([int]$ledgerPayload.schema_version -ne 1) {
        throw "Context Ledger schema mismatch"
    }
    if ($ledgerPayload.turn_granularity_status -ne "NOT_CAPTURED") {
        throw "Context Ledger turn granularity must remain NOT_CAPTURED"
    }
    if ($ledgerPayload.current_retained_context_status -ne "UNKNOWN") {
        throw "Context Ledger retained-context status must remain UNKNOWN"
    }
    if (@($ledgerPayload.runs).Count -ne 0) {
        throw "Fresh smoke Store Context Ledger should have no runs"
    }

    if ($stdout.Contains($Db)) {
        throw "MCP response leaked the local database path"
    }

    Write-Host "MCP prototype smoke: PASS"
}
finally {
    if (-not $process.HasExited) {
        try { $process.Kill() } catch {}
    }
    $process.Dispose()
    if (Test-Path -LiteralPath $Db) {
        Remove-Item -LiteralPath $Db -Force
    }
}
