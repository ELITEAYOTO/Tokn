function Install-ToknPolicyPlacement {
    [CmdletBinding()]
    param(
        [Parameter(Mandatory = $true)][string]$TargetPath,
        [Parameter(Mandatory = $true)][string]$PolicyPath,
        [Parameter(Mandatory = $true)][string]$BackupPath
    )

    $parent = Split-Path $TargetPath -Parent
    if (-not (Test-Path -LiteralPath $parent)) {
        throw "Policy placement parent not found: $parent"
    }

    $hadOriginal = Test-Path -LiteralPath $TargetPath
    if ($hadOriginal) {
        Copy-Item -LiteralPath $TargetPath -Destination $BackupPath -Force
        $original = Get-Content -LiteralPath $TargetPath -Raw
        $policy = Get-Content -LiteralPath $PolicyPath -Raw
        ($original.TrimEnd() + [Environment]::NewLine + [Environment]::NewLine +
            "# Tokn Temporary Experiment Overlay" + [Environment]::NewLine +
            $policy) | Set-Content -LiteralPath $TargetPath -Encoding UTF8
    } else {
        Copy-Item -LiteralPath $PolicyPath -Destination $TargetPath -Force
    }

    $installedHash = (Get-FileHash -LiteralPath $TargetPath -Algorithm SHA256).Hash.ToLowerInvariant()

    return [pscustomobject]@{
        path = $TargetPath
        had_original = $hadOriginal
        backup_path = $(if ($hadOriginal) { $BackupPath } else { $null })
        installed_sha256 = $installedHash
    }
}

function Restore-ToknPolicyPlacement {
    [CmdletBinding()]
    param(
        [Parameter(Mandatory = $true)]$Placement,
        [string]$PreserveUnexpectedPath
    )

    $target = [string]$Placement.path
    $hadOriginal = [bool]$Placement.had_original
    $installedHash = [string]$Placement.installed_sha256
    $exists = Test-Path -LiteralPath $target
    $currentHash = $null
    if ($exists) {
        $currentHash = (Get-FileHash -LiteralPath $target -Algorithm SHA256).Hash.ToLowerInvariant()
    }

    if ($hadOriginal) {
        $backup = [string]$Placement.backup_path
        if (-not $backup -or -not (Test-Path -LiteralPath $backup)) {
            Write-Warning "Policy backup missing for $target; refusing to overwrite current state."
            return $false
        }

        if ($exists -and $currentHash -ne $installedHash -and $PreserveUnexpectedPath) {
            Copy-Item -LiteralPath $target -Destination $PreserveUnexpectedPath -Force
            Write-Warning "Policy target changed during run; unexpected state preserved before restore: $target"
        }
        Copy-Item -LiteralPath $backup -Destination $target -Force
        return $true
    }

    if (-not $exists) {
        return $true
    }

    if ($currentHash -eq $installedHash) {
        Remove-Item -LiteralPath $target -Force
        return $true
    }

    if ($PreserveUnexpectedPath) {
        Copy-Item -LiteralPath $target -Destination $PreserveUnexpectedPath -Force
    }
    Write-Warning "Policy target changed during run and had no pre-run file; leaving it in place: $target"
    return $false
}

function Restore-ToknPolicyPlacements {
    [CmdletBinding()]
    param(
        [Parameter(Mandatory = $true)]$Placements,
        [Parameter(Mandatory = $true)][string]$PreserveDirectory,
        [string]$Prefix = "policy-unexpected"
    )

    $index = 0
    $allRestored = $true
    foreach ($placement in @($Placements)) {
        $preserve = Join-Path $PreserveDirectory ("{0}-{1}.md" -f $Prefix, $index)
        if (-not (Restore-ToknPolicyPlacement -Placement $placement -PreserveUnexpectedPath $preserve)) {
            $allRestored = $false
        }
        $index++
    }
    return $allRestored
}
