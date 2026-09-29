param(
    [Parameter(Mandatory = $true)]
    [string]$WatchRoot,

    [Parameter(Mandatory = $true)]
    [string]$OutputPath,

    [ValidateRange(0, 8)]
    [int]$MaxDepth = 3
)

$ErrorActionPreference = "Stop"

$root = (Resolve-Path -LiteralPath $WatchRoot).Path.TrimEnd('\')
$markers = @(
    "package.json",
    "Cargo.toml",
    "pyproject.toml",
    "go.mod",
    "pom.xml",
    "build.gradle",
    "settings.gradle"
)
$excludedDirectoryNames = @(
    ".git",
    "node_modules",
    "dist",
    "target",
    "vendor",
    ".test-build",
    ".browser-build"
)

function Get-RelativePath {
    param([string]$Base, [string]$Full)

    if ($Full.Equals($Base, [System.StringComparison]::OrdinalIgnoreCase)) {
        return "."
    }

    $prefix = $Base.TrimEnd('\') + "\"
    if (-not $Full.StartsWith($prefix, [System.StringComparison]::OrdinalIgnoreCase)) {
        throw "Path is outside workspace watch root: $Full"
    }

    return ($Full.Substring($prefix.Length) -replace '\\', '/')
}

$queue = New-Object System.Collections.Generic.Queue[object]
$queue.Enqueue([pscustomobject]@{
    path = $root
    depth = 0
})

$candidates = @()

while ($queue.Count -gt 0) {
    $item = $queue.Dequeue()
    $path = [string]$item.path
    $depth = [int]$item.depth

    $foundMarkers = @()

    foreach ($marker in $markers) {
        $markerPath = Join-Path $path $marker
        if (-not (Test-Path -LiteralPath $markerPath -PathType Leaf)) {
            continue
        }

        $file = Get-Item -LiteralPath $markerPath
        $foundMarkers += [pscustomobject][ordered]@{
            name = $marker
            bytes = [int64]$file.Length
            sha256 = (Get-FileHash -LiteralPath $markerPath -Algorithm SHA256).Hash.ToLowerInvariant()
        }
    }

    if ($foundMarkers.Count -gt 0) {
        $dirInfo = Get-Item -LiteralPath $path
        $relativePath = Get-RelativePath -Base $root -Full $path
        $candidates += [pscustomobject][ordered]@{
            root = $path
            relative = [string]$relativePath
            depth = $depth
            last_write_utc = $dirInfo.LastWriteTimeUtc.ToString("o")
            markers = @($foundMarkers)
        }
    }

    if ($depth -ge $MaxDepth) {
        continue
    }

    Get-ChildItem -LiteralPath $path -Directory -Force -ErrorAction SilentlyContinue |
        Sort-Object FullName |
        ForEach-Object {
            if ($excludedDirectoryNames -contains $_.Name) {
                return
            }

            $queue.Enqueue([pscustomobject]@{
                path = $_.FullName
                depth = $depth + 1
            })
        }
}

$result = [ordered]@{
    schema_version = 1
    created_at = (Get-Date).ToString("o")
    watch_root = $root
    max_depth = $MaxDepth
    marker_names = $markers
    excluded_directory_names = $excludedDirectoryNames
    candidate_count = $candidates.Count
    candidates = @($candidates | Sort-Object root)
}

$parent = Split-Path $OutputPath -Parent
if ($parent) {
    New-Item -ItemType Directory -Force -Path $parent | Out-Null
}

$json = $result | ConvertTo-Json -Depth 8
$utf8NoBom = New-Object System.Text.UTF8Encoding
[System.IO.File]::WriteAllText(
    $OutputPath,
    $json + [Environment]::NewLine,
    $utf8NoBom
)

Write-Host "TOKN WORKSPACE INVENTORY" -ForegroundColor Green
Write-Host "Watch root: $root"
Write-Host "Candidates: $($candidates.Count)"
Write-Host "Output:     $OutputPath"
