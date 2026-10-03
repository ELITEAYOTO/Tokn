param()

$ErrorActionPreference = "Stop"
$Root = (Resolve-Path (Join-Path $PSScriptRoot "..\.." )).Path
$errors = New-Object System.Collections.Generic.List[string]
$files = Get-ChildItem (Join-Path $Root "scripts") -Recurse -Filter "*.ps1" -File

foreach ($file in $files) {
    try {
        [void][scriptblock]::Create((Get-Content -Raw -LiteralPath $file.FullName))
    }
    catch {
        $relative = if ($file.FullName.StartsWith($Root, [StringComparison]::OrdinalIgnoreCase)) { $file.FullName.Substring($Root.Length).TrimStart("\\", "/") } else { $file.FullName }
        $errors.Add("$relative : $($_.Exception.Message)")
    }
}

if ($errors.Count -gt 0) {
    Write-Host "PowerShell syntax: FAIL" -ForegroundColor Red
    $errors | ForEach-Object { Write-Host "  $_" }
    throw "PowerShell syntax validation failed."
}

Write-Host "PowerShell syntax: PASS ($($files.Count) scripts)"
