param(
    [string]$BinaryPath = "target\release\tokn-mcp.exe"
)

$ErrorActionPreference = "Stop"
$Repo = (Resolve-Path (Join-Path $PSScriptRoot "..\..")).Path
Set-Location $Repo

$CodexCommand = Get-Command codex -ErrorAction SilentlyContinue
if ($null -eq $CodexCommand) {
    throw "Codex CLI is unavailable; target-runtime MCP validation cannot run."
}
if (-not (Test-Path -LiteralPath $BinaryPath)) {
    throw "Tokn MCP binary not found: $BinaryPath"
}

$CodexVersion = (& codex --version | Out-String).Trim()
$CodexHomePath = Join-Path $Repo "target\codex-home-mcp-runtime-check"
$DbPath = Join-Path $Repo "target\codex-mcp-runtime-check.sqlite3"
$ResolvedBinary = (Resolve-Path $BinaryPath).Path

Remove-Item $CodexHomePath -Recurse -Force -ErrorAction SilentlyContinue
Remove-Item $DbPath -Force -ErrorAction SilentlyContinue
New-Item -ItemType Directory -Force -Path $CodexHomePath | Out-Null

$OldCodexHome = $env:CODEX_HOME
$env:CODEX_HOME = $CodexHomePath

try {
    & codex mcp add tokn -- $ResolvedBinary --db $DbPath | Out-Null
    if ($LASTEXITCODE -ne 0) {
        throw "codex mcp add failed with exit code $LASTEXITCODE"
    }

    $Config = (& codex mcp get tokn --json | Out-String) | ConvertFrom-Json
    if ($Config.enabled -ne $true) {
        throw "Tokn MCP registration is not enabled"
    }
    if ($Config.transport.type -ne "stdio") {
        throw "Tokn MCP transport is not stdio"
    }
    if ([string]$Config.transport.command -ne $ResolvedBinary) {
        throw "Tokn MCP registered command mismatch"
    }
    $Args = @($Config.transport.args)
    if ($Args.Count -ne 2 -or $Args[0] -ne "--db" -or $Args[1] -ne $DbPath) {
        throw "Tokn MCP registered arguments mismatch"
    }

    $psi = [System.Diagnostics.ProcessStartInfo]::new()
    $psi.FileName = "cmd.exe"
    $psi.Arguments = '/d /s /c "codex app-server --stdio"'
    $psi.UseShellExecute = $false
    $psi.CreateNoWindow = $true
    $psi.RedirectStandardInput = $true
    $psi.RedirectStandardOutput = $true
    $psi.RedirectStandardError = $true
    $psi.Environment["CODEX_HOME"] = $CodexHomePath

    $Process = [System.Diagnostics.Process]::new()
    $Process.StartInfo = $psi
    if (-not $Process.Start()) {
        throw "Failed to start Codex app-server"
    }

    $Initialize = '{"id":1,"method":"initialize","params":{"clientInfo":{"name":"tokn-runtime-check","version":"1"},"capabilities":{}}}'
    $Inventory = '{"id":2,"method":"mcpServerStatus/list","params":{"serverName":"tokn","detail":"full","limit":10}}'

    $Process.StandardInput.WriteLine($Initialize)
    $Process.StandardInput.Flush()
    Start-Sleep -Milliseconds 700
    $Process.StandardInput.WriteLine($Inventory)
    $Process.StandardInput.Flush()
    Start-Sleep -Seconds 4
    $Process.StandardInput.Close()

    if (-not $Process.WaitForExit(12000)) {
        try { $Process.Kill() } catch {}
        throw "Codex app-server did not exit after stdin closed"
    }

    $Stdout = $Process.StandardOutput.ReadToEnd()
    $Stderr = $Process.StandardError.ReadToEnd()
    if ($Process.ExitCode -ne 0) {
        throw "Codex app-server exited with code $($Process.ExitCode): $Stderr"
    }
    if (-not [string]::IsNullOrWhiteSpace($Stderr)) {
        throw "Codex app-server wrote unexpected stderr: $Stderr"
    }

    $Responses = @()
    foreach ($Line in @($Stdout -split "\r?\n")) {
        if ([string]::IsNullOrWhiteSpace($Line)) {
            continue
        }
        try {
            $Message = $Line | ConvertFrom-Json
        }
        catch {
            continue
        }
        if ($null -ne $Message.id) {
            $Responses += $Message
        }
    }

    $InitializeResponse = @($Responses | Where-Object { $_.id -eq 1 })[0]
    if ($null -eq $InitializeResponse -or $null -ne $InitializeResponse.error) {
        throw "Codex app-server initialize failed"
    }

    $InventoryResponse = @($Responses | Where-Object { $_.id -eq 2 })[0]
    if ($null -eq $InventoryResponse) {
        throw "Codex MCP inventory response is missing"
    }
    if ($null -ne $InventoryResponse.error) {
        throw "Codex MCP inventory returned an error: $($InventoryResponse.error | ConvertTo-Json -Compress)"
    }

    $Servers = @($InventoryResponse.result.data)
    if ($Servers.Count -ne 1) {
        throw "Expected exactly one Tokn MCP server, got $($Servers.Count)"
    }
    $Server = $Servers[0]
    if ($Server.name -ne "tokn") {
        throw "Codex MCP inventory server name mismatch"
    }
    if ($Server.serverInfo.name -ne "tokn-mcp") {
        throw "Tokn MCP serverInfo name mismatch"
    }
    if ([string]::IsNullOrWhiteSpace([string]$Server.serverInfo.version)) {
        throw "Tokn MCP serverInfo version is missing"
    }
    if ($null -eq $Server.serverCapabilities.tools) {
        throw "Tokn MCP tool capability was not advertised"
    }
    if ($null -ne $Server.toolsError) {
        throw "Tokn MCP tool discovery returned an error: $($Server.toolsError)"
    }

    $ToolNames = @($Server.tools.PSObject.Properties.Name)
    if ($ToolNames.Count -ne 2) {
        throw "Expected exactly two Tokn MCP tools, got $($ToolNames.Count)"
    }
    if ($ToolNames -notcontains "tokn_status") {
        throw "Codex did not discover tokn_status"
    }
    if ($ToolNames -notcontains "tokn_recent_runs") {
        throw "Codex did not discover tokn_recent_runs"
    }

    if ($Stdout.Contains($DbPath)) {
        throw "Codex MCP inventory output leaked the local database path"
    }

    Write-Host "Codex MCP runtime discovery: PASS"
    Write-Host "Codex: $CodexVersion"
    Write-Host "Server: $($Server.serverInfo.name) $($Server.serverInfo.version)"
    Write-Host "Tools: $($ToolNames -join ', ')"
}
finally {
    $env:CODEX_HOME = $OldCodexHome
    if ($null -ne $Process -and -not $Process.HasExited) {
        try { $Process.Kill() } catch {}
    }
    if ($null -ne $Process) {
        $Process.Dispose()
    }
    Remove-Item $CodexHomePath -Recurse -Force -ErrorAction SilentlyContinue
    Remove-Item $DbPath -Force -ErrorAction SilentlyContinue
}
