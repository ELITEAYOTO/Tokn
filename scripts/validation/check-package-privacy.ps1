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
        "bin/tokn-observe.provenance.json",
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
    $forbiddenExtensions = @(
        ".sqlite", ".sqlite3", ".db", ".log", ".wal", ".shm",
        ".pem", ".key", ".p12", ".pfx"
    )
    foreach ($name in $names) {
        $parts = @($name.Split("/") | ForEach-Object { $_.ToLowerInvariant() })
        foreach ($part in $forbiddenParts) {
            if ($parts -contains $part) {
                throw "package contains forbidden path: $name"
            }
        }
        $leaf = [IO.Path]::GetFileName($name).ToLowerInvariant()
        if ($leaf -eq ".env" -or $leaf.StartsWith(".env.") -or $leaf -eq "auth.json") {
            throw "package contains forbidden sensitive file: $name"
        }
        if ($leaf -match '^(credentials?|secrets?)(\..*)?$') {
            throw "package contains forbidden sensitive file: $name"
        }
        $extension = [IO.Path]::GetExtension($name).ToLowerInvariant()
        if ($forbiddenExtensions -contains $extension) {
            throw "package contains forbidden runtime/sensitive file: $name"
        }
        $wrapped = "/" + $name.ToLowerInvariant().Trim("/") + "/"
        if ($wrapped.Contains("/traces/") -or $wrapped.Contains("/runs/") -or $wrapped.Contains("/benchmarks/raw/") -or $wrapped.Contains("/benchmarks/runs/")) {
            throw "package contains raw runtime evidence path: $name"
        }
    }

    $secretPatterns = [ordered]@{
        github_token = '(?:ghp|gho|ghu|ghs|ghr)_[A-Za-z0-9]{20,}|github_pat_[A-Za-z0-9_]{20,}'
        openai_key = '\bsk-[A-Za-z0-9_-]{20,}\b'
        aws_key = '\bAKIA[0-9A-Z]{16}\b'
        private_key = '-----BEGIN (?:RSA |EC |OPENSSH |DSA )?PRIVATE KEY-----'
        jwt = '\beyJ[A-Za-z0-9_-]{10,}\.[A-Za-z0-9_-]{10,}\.[A-Za-z0-9_-]{10,}\b'
        bearer = '(?i)authorization\s*[:=]\s*bearer\s+[A-Za-z0-9._~+/=-]{16,}'
        secret_assignment = '(?i)\b(?:password|passwd|api[_-]?key|access[_-]?token|client[_-]?secret)\s*[:=]\s*["''][^"'']{8,}["'']'
        user_home = '(?i)C:\\Users\\[^\\\r\n]+\\'
    }
    $binaryExtensions = @(
        ".exe", ".dll", ".pdb", ".zip", ".png", ".jpg", ".jpeg",
        ".gif", ".webp", ".ico", ".woff", ".woff2", ".ttf"
    )

    foreach ($entry in $entries) {
        $name = $entry.FullName.Replace("\", "/")
        $extension = [IO.Path]::GetExtension($name).ToLowerInvariant()
        if ($binaryExtensions -contains $extension) { continue }

        $reader = [IO.StreamReader]::new($entry.Open(), [Text.Encoding]::UTF8, $true)
        try {
            while (($line = $reader.ReadLine()) -ne $null) {
                foreach ($rule in $secretPatterns.GetEnumerator()) {
                    if ([regex]::IsMatch($line, [string]$rule.Value)) {
                        throw "package sensitive content '$($rule.Key)' in $name"
                    }
                }
            }
        }
        finally { $reader.Dispose() }
    }

    $totalBytes = ($entries | Measure-Object -Property Length -Sum).Sum
    Write-Host "Package privacy: PASS"
    Write-Host "Files: $($entries.Count)"
    Write-Host "Uncompressed bytes: $totalBytes"
}
finally {
    $archive.Dispose()
}
