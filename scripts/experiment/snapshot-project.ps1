param(
    [Parameter(Mandatory = $true)]
    [string]$ProjectRoot,
    [Parameter(Mandatory = $true)]
    [string]$OutputPath
)

$ErrorActionPreference = "Stop"

$project = (Resolve-Path $ProjectRoot).Path.TrimEnd('\')
$excludedTop = @(
    "node_modules",
    ".test-build",
    ".browser-build",
    "dist",
    ".git"
)

function Get-RelativePath {
    param([string]$Base, [string]$Full)
    $prefix = $Base.TrimEnd('\') + "\"
    if (-not $Full.StartsWith($prefix, [System.StringComparison]::OrdinalIgnoreCase)) {
        throw "Path is outside project root: $Full"
    }
    return $Full.Substring($prefix.Length)
}
$files = New-Object System.Collections.Generic.List[object]
$totalBytes = [int64]0

Get-ChildItem -LiteralPath $project -File -Recurse -Force |
    Sort-Object FullName |
    ForEach-Object {
        $relative = Get-RelativePath -Base $project -Full $_.FullName
        $top = ($relative -split '[\\/]', 2)[0]

        if ($excludedTop -contains $top) {
            return
        }

        $hash = (Get-FileHash -LiteralPath $_.FullName -Algorithm SHA256).Hash.ToLowerInvariant()
        $totalBytes += $_.Length
        $files.Add([ordered]@{
            path = ($relative -replace '\\', '/')
            sha256 = $hash
            bytes = [int64]$_.Length
            last_write_utc = $_.LastWriteTimeUtc.ToString("o")
        })
    }
$snapshot = [ordered]@{
    schema_version = 1
    created_at = (Get-Date).ToString("o")
    project_root = $project
    excluded_top_level = $excludedTop
    file_count = $files.Count
    total_bytes = $totalBytes
    files = $files
}

$parent = Split-Path $OutputPath -Parent
if ($parent) {
    New-Item -ItemType Directory -Force -Path $parent | Out-Null
}

$snapshot | ConvertTo-Json -Depth 6 |
    Set-Content -LiteralPath $OutputPath -Encoding UTF8

Write-Host "TOKN PROJECT SNAPSHOT" -ForegroundColor Green
Write-Host "Project: $project"
Write-Host "Files:   $($files.Count)"
Write-Host "Bytes:   $totalBytes"
Write-Host "Output:  $OutputPath"
