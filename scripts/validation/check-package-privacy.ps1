param(
    [Parameter(Mandatory = $true)]
    [string]$ZipPath
)

$ErrorActionPreference = "Stop"
$ZipPath = (Resolve-Path -LiteralPath $ZipPath).Path
Add-Type -AssemblyName System.IO.Compression.FileSystem

$archive = [System.IO.Compression.ZipFile]::OpenRead($ZipPath)
try {
    $entries = @($archive.Entries | Where-Object { -not [string]::IsNullOrEmpty($_.Name) })
    $names = @($entries | ForEach-Object { $_.FullName.Replace("\", "/") })
    $lowerNames = @($names | ForEach-Object { $_.ToLowerInvariant() })

    $required = @(
        "bin/tokn-observe.exe",
        "fixtures/experiments/001-runner-golden/expected.json",
        "scripts/validation/replay-exp001-golden.ps1",
        "scripts/validation/check-doc-consistency.ps1"
    )
    foreach ($requiredPath in $required) {
        if ($lowerNames -notcontains $requiredPath.ToLowerInvariant()) {
            throw "package missing required file: $requiredPath"
        }
    }

    $forbiddenParts = @("target", ".git", "artifacts", ".codex")
    $forbiddenExtensions = @(".sqlite", ".db", ".log")
    foreach ($name in $names) {
        $parts = @($name.Split("/") | ForEach-Object { $_.ToLowerInvariant() })
        foreach ($part in $forbiddenParts) {
            if ($parts -contains $part) {
                throw "package contains forbidden path: $name"
            }
        }
        $extension = [System.IO.Path]::GetExtension($name).ToLowerInvariant()
        if ($forbiddenExtensions -contains $extension) {
            throw "package contains forbidden runtime file: $name"
        }
        $wrapped = "/" + $name.ToLowerInvariant().Trim("/") + "/"
        if ($wrapped.Contains("/traces/") -or $wrapped.Contains("/runs/")) {
            throw "package contains raw runtime evidence path: $name"
        }
    }

    $fixturePrefix = "fixtures/experiments/001-runner-golden/"
    $fixtureForbidden = @(
        "timot",
        "renou",
        "blockbench-plugin",
        "jem_ultimate",
        "01a0e3ed",
        "18eff7c4",
        "@gmail",
        "c:\users\",
        "c:/users/"
    )

    foreach ($entry in $entries) {
        $name = $entry.FullName.Replace("\", "/")
        if (-not $name.ToLowerInvariant().StartsWith($fixturePrefix)) {
            continue
        }
        $reader = [System.IO.StreamReader]::new($entry.Open())
        try {
            $text = $reader.ReadToEnd().ToLowerInvariant()
        }
        finally {
            $reader.Dispose()
        }
        foreach ($marker in $fixtureForbidden) {
            if ($text.Contains($marker)) {
                throw "golden fixture contains forbidden privacy marker '$marker' in $name"
            }
        }
    }

    $totalBytes = ($entries | Measure-Object -Property Length -Sum).Sum
    Write-Host "Package privacy: PASS"
    Write-Host "Files: $($entries.Count)"
    Write-Host "Uncompressed bytes: $totalBytes"
}
finally {
    $archive.Dispose()
}
