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
$Doctor = (& codex doctor --json | Out-String) | ConvertFrom-Json
$CodexExe = [string]$Doctor.checks.'runtime.provenance'.details.'current executable'
if ([string]::IsNullOrWhiteSpace($CodexExe) -or -not (Test-Path -LiteralPath $CodexExe)) {
    throw "Unable to resolve the native Codex executable."
}

$CodexHomePath = Join-Path $Repo "target\codex-home-mcp-runtime-check"
$DbPath = Join-Path $Repo "target\codex-mcp-runtime-check.sqlite3"
$ResolvedBinary = (Resolve-Path $BinaryPath).Path

Remove-Item $CodexHomePath -Recurse -Force -ErrorAction SilentlyContinue
Remove-Item $DbPath -Force -ErrorAction SilentlyContinue
New-Item -ItemType Directory -Force -Path $CodexHomePath | Out-Null

$OldCodexHome = $env:CODEX_HOME
$env:CODEX_HOME = $CodexHomePath
$Process = $null

function Read-AppResponseById {
    param(
        [Parameter(Mandatory = $true)] [System.Diagnostics.Process]$Process,
        [Parameter(Mandatory = $true)] [int]$Id,
        [int]$TimeoutMs = 15000
    )

    $Deadline = [DateTime]::UtcNow.AddMilliseconds($TimeoutMs)
    while ([DateTime]::UtcNow -lt $Deadline) {
        $Task = $Process.StandardOutput.ReadLineAsync()
        $Remaining = [int][Math]::Max(1, ($Deadline - [DateTime]::UtcNow).TotalMilliseconds)
        if (-not $Task.Wait($Remaining)) {
            throw "Timed out waiting for Codex app-server response id=$Id"
        }

        $Line = $Task.Result
        if ($null -eq $Line) {
            throw "Codex app-server stdout closed while waiting for id=$Id"
        }
        if ([string]::IsNullOrWhiteSpace($Line)) {
            continue
        }

        try {
            $Message = $Line | ConvertFrom-Json
        }
        catch {
            continue
        }

        if ($Message.id -eq $Id) {
            return $Message
        }
    }

    throw "Timed out waiting for Codex app-server response id=$Id"
}

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

    $Psi = [System.Diagnostics.ProcessStartInfo]::new()
    $Psi.FileName = $CodexExe
    $Psi.Arguments = "app-server --stdio"
    $Psi.UseShellExecute = $false
    $Psi.CreateNoWindow = $true
    $Psi.RedirectStandardInput = $true
    $Psi.RedirectStandardOutput = $true
    $Psi.RedirectStandardError = $true
    $Psi.Environment["CODEX_HOME"] = $CodexHomePath

    $Process = [System.Diagnostics.Process]::new()
    $Process.StartInfo = $Psi
    if (-not $Process.Start()) {
        throw "Failed to start native Codex app-server"
    }

    $Initialize = @{
        id = 1
        method = "initialize"
        params = @{
            clientInfo = @{
                name = "tokn-runtime-check"
                version = "1"
            }
            capabilities = @{
                experimentalApi = $true
            }
        }
    } | ConvertTo-Json -Depth 8 -Compress

    $Process.StandardInput.WriteLine($Initialize)
    $Process.StandardInput.Flush()
    $InitializeResponse = Read-AppResponseById -Process $Process -Id 1 -TimeoutMs 10000
    if ($null -ne $InitializeResponse.error) {
        throw "Codex app-server initialize failed: $($InitializeResponse.error | ConvertTo-Json -Compress)"
    }

    $Process.StandardInput.WriteLine('{"method":"initialized"}')

    $Inventory = @{
        id = 2
        method = "mcpServerStatus/list"
        params = @{
            serverName = "tokn"
            detail = "full"
            limit = 10
        }
    } | ConvertTo-Json -Depth 6 -Compress
    $Process.StandardInput.WriteLine($Inventory)
    $Process.StandardInput.Flush()

    $InventoryResponse = Read-AppResponseById -Process $Process -Id 2 -TimeoutMs 20000
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

    $ThreadStart = @{
        id = 3
        method = "thread/start"
        params = @{
            ephemeral = $true
            cwd = $Repo
        }
    } | ConvertTo-Json -Depth 8 -Compress
    $Process.StandardInput.WriteLine($ThreadStart)
    $Process.StandardInput.Flush()

    $ThreadResponse = Read-AppResponseById -Process $Process -Id 3 -TimeoutMs 15000
    if ($null -ne $ThreadResponse.error) {
        throw "Codex thread/start failed: $($ThreadResponse.error | ConvertTo-Json -Compress)"
    }

    $ThreadId = [string]$ThreadResponse.result.thread.id
    if ([string]::IsNullOrWhiteSpace($ThreadId)) {
        throw "Codex thread/start returned no thread id"
    }
    if ($ThreadResponse.result.thread.status.type -ne "idle") {
        throw "Validation thread is not idle"
    }
    if (@($ThreadResponse.result.thread.turns).Count -ne 0) {
        throw "Validation thread unexpectedly contains model turns"
    }

    $ToolCall = @{
        id = 4
        method = "mcpServer/tool/call"
        params = @{
            server = "tokn"
            threadId = $ThreadId
            tool = "tokn_status"
            arguments = @{}
        }
    } | ConvertTo-Json -Depth 8 -Compress
    $Process.StandardInput.WriteLine($ToolCall)
    $Process.StandardInput.Flush()

    $ToolResponse = Read-AppResponseById -Process $Process -Id 4 -TimeoutMs 15000
    if ($null -ne $ToolResponse.error) {
        throw "Codex MCP tool call failed: $($ToolResponse.error | ConvertTo-Json -Compress)"
    }
    if ($ToolResponse.result.isError -eq $true) {
        throw "Tokn MCP tool returned isError=true"
    }

    $TextItems = @($ToolResponse.result.content | Where-Object { $_.type -eq "text" })
    if ($TextItems.Count -lt 1 -or [string]::IsNullOrWhiteSpace([string]$TextItems[0].text)) {
        throw "Tokn status returned no text content"
    }
    $StatusPayload = $TextItems[0].text | ConvertFrom-Json
    if ($StatusPayload.server.name -ne "tokn-mcp") {
        throw "Tokn status server name mismatch"
    }
    if ($StatusPayload.server.transport -ne "stdio") {
        throw "Tokn status transport mismatch"
    }
    if ($StatusPayload.server.read_only -ne $true) {
        throw "Tokn status read_only mismatch"
    }
    if ([string]$StatusPayload.store.schema_version -ne "2") {
        throw "Tokn status Store schema mismatch"
    }
    if (@($StatusPayload.tools) -notcontains "tokn_status" -or
        @($StatusPayload.tools) -notcontains "tokn_recent_runs") {
        throw "Tokn status tool list mismatch"
    }

    $SerializedStatus = $ToolResponse | ConvertTo-Json -Depth 20 -Compress
    if ($SerializedStatus.Contains($DbPath)) {
        throw "Tokn MCP tool output leaked the local database path"
    }

    Write-Host "Codex MCP runtime tool call: PASS"
    Write-Host "Codex: $CodexVersion"
    Write-Host "Server: $($Server.serverInfo.name) $($Server.serverInfo.version)"
    Write-Host "Tools: $($ToolNames -join ', ')"
    Write-Host "Thread: ephemeral / idle / 0 turns"
    Write-Host "Tool call: tokn_status via mcpServer/tool/call"
}
finally {
    $env:CODEX_HOME = $OldCodexHome
    if ($null -ne $Process -and -not $Process.HasExited) {
        try {
            $Process.StandardInput.Close()
        }
        catch {}
        if (-not $Process.WaitForExit(3000)) {
            try { $Process.Kill() } catch {}
        }
    }
    if ($null -ne $Process) {
        $Process.Dispose()
    }
    Remove-Item $CodexHomePath -Recurse -Force -ErrorAction SilentlyContinue
    Remove-Item $DbPath -Force -ErrorAction SilentlyContinue
}
