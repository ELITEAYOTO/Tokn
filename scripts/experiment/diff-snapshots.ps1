param(
    [Parameter(Mandatory = $true)]
    [string]$BeforePath,
    [Parameter(Mandatory = $true)]
    [string]$AfterPath,
    [Parameter(Mandatory = $true)]
    [string]$OutputJson,
    [Parameter(Mandatory = $true)]
    [string]$OutputMarkdown
)

$ErrorActionPreference = "Stop"

$before = Get-Content -LiteralPath $BeforePath -Raw | ConvertFrom-Json
$after = Get-Content -LiteralPath $AfterPath -Raw | ConvertFrom-Json

$beforeMap = @{}
$afterMap = @{}

foreach ($file in $before.files) { $beforeMap[$file.path] = $file }
foreach ($file in $after.files) { $afterMap[$file.path] = $file }

$added = New-Object System.Collections.Generic.List[object]
$removed = New-Object System.Collections.Generic.List[object]
$modified = New-Object System.Collections.Generic.List[object]
foreach ($path in $afterMap.Keys) {
    if (-not $beforeMap.ContainsKey($path)) {
        $added.Add($afterMap[$path])
        continue
    }

    if ($beforeMap[$path].sha256 -ne $afterMap[$path].sha256) {
        $modified.Add([ordered]@{
            path = $path
            before_sha256 = $beforeMap[$path].sha256
            after_sha256 = $afterMap[$path].sha256
            before_bytes = [int64]$beforeMap[$path].bytes
            after_bytes = [int64]$afterMap[$path].bytes
            byte_delta = [int64]$afterMap[$path].bytes - [int64]$beforeMap[$path].bytes
        })
    }
}

foreach ($path in $beforeMap.Keys) {
    if (-not $afterMap.ContainsKey($path)) {
        $removed.Add($beforeMap[$path])
    }
}
$result = [ordered]@{
    schema_version = 1
    created_at = (Get-Date).ToString("o")
    before = $BeforePath
    after = $AfterPath
    added_count = $added.Count
    modified_count = $modified.Count
    removed_count = $removed.Count
    added = @($added | Sort-Object path)
    modified = @($modified | Sort-Object path)
    removed = @($removed | Sort-Object path)
}

$result | ConvertTo-Json -Depth 6 |
    Set-Content -LiteralPath $OutputJson -Encoding UTF8

$lines = New-Object System.Collections.Generic.List[string]
$lines.Add("# Tokn Project Diff")
$lines.Add("")
$lines.Add("- Added: **$($added.Count)**")
$lines.Add("- Modified: **$($modified.Count)**")
$lines.Add("- Removed: **$($removed.Count)**")
$lines.Add("")
foreach ($section in @(
    @{ Name = "Modified"; Items = @($modified | Sort-Object path) },
    @{ Name = "Added"; Items = @($added | Sort-Object path) },
    @{ Name = "Removed"; Items = @($removed | Sort-Object path) }
)) {
    $lines.Add("## $($section.Name)")
    $lines.Add("")
    if ($section.Items.Count -eq 0) {
        $lines.Add("_None._")
    } else {
        foreach ($item in $section.Items) {
            $lines.Add("- " + $item.path)
        }
    }
    $lines.Add("")
}

$lines | Set-Content -LiteralPath $OutputMarkdown -Encoding UTF8

Write-Host "TOKN PROJECT DIFF" -ForegroundColor Green
Write-Host "Added:    $($added.Count)"
Write-Host "Modified: $($modified.Count)"
Write-Host "Removed:  $($removed.Count)"
Write-Host "JSON:     $OutputJson"
Write-Host "Markdown: $OutputMarkdown"
