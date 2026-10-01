param(
    [Parameter(Mandatory = $true)][string]$EvidenceRoot,
    [Parameter(Mandatory = $true)][string]$OutputPath,
    [string]$CodexCliVersion,
    [string]$CodexAppVersion
)

$ErrorActionPreference = 'Stop'
if (-not (Test-Path -LiteralPath $EvidenceRoot -PathType Container)) {
    throw "Evidence root missing: $EvidenceRoot"
}

$wanted = @(
    'model',
    'model_context_window',
    'reasoning_effort',
    'effort',
    'approval_policy',
    'sandbox_policy',
    'collaboration_mode'
)
$values = @{}
foreach ($key in $wanted) {
    $values[$key] = [System.Collections.Generic.HashSet[string]]::new(
        [System.StringComparer]::OrdinalIgnoreCase
    )
}

function Visit-JsonValue {
    param($Value)
    if ($null -eq $Value) { return }
    if ($Value -is [System.Management.Automation.PSCustomObject]) {
        foreach ($property in $Value.PSObject.Properties) {
            if ($wanted -contains $property.Name -and
                $null -ne $property.Value -and
                ($property.Value -is [string] -or $property.Value -is [System.ValueType])) {
                [void]$values[$property.Name].Add([string]$property.Value)
            }
            Visit-JsonValue $property.Value
        }
        return
    }
    if ($Value -is [System.Collections.IDictionary]) {
        foreach ($key in $Value.Keys) {
            Visit-JsonValue $Value[$key]
        }
        return
    }
    if ($Value -is [System.Collections.IEnumerable] -and $Value -isnot [string]) {
        foreach ($item in $Value) { Visit-JsonValue $item }
    }
}

$files = @(
    Get-ChildItem -LiteralPath $EvidenceRoot -Recurse -File |
        Where-Object { $_.Extension -in @('.json', '.jsonl') }
)
$parsedRecords = 0
$parseFailures = 0
foreach ($file in $files) {
    if ($file.Extension -eq '.json') {
        try {
            $obj = Get-Content -LiteralPath $file.FullName -Raw | ConvertFrom-Json
            Visit-JsonValue $obj
            $parsedRecords++
        } catch {
            $parseFailures++
        }
        continue
    }
    foreach ($line in [System.IO.File]::ReadLines($file.FullName)) {
        if ([string]::IsNullOrWhiteSpace($line)) { continue }
        try {
            $obj = $line | ConvertFrom-Json
            Visit-JsonValue $obj
            $parsedRecords++
        } catch {
            $parseFailures++
        }
    }
}
$observed = [ordered]@{
    models = @($values['model'] | Sort-Object)
    model_context_windows = @($values['model_context_window'] | Sort-Object)
    reasoning_efforts = @($values['reasoning_effort'] | Sort-Object)
    efforts = @($values['effort'] | Sort-Object)
    approval_policies = @($values['approval_policy'] | Sort-Object)
    sandbox_policies = @($values['sandbox_policy'] | Sort-Object)
    collaboration_modes = @($values['collaboration_mode'] | Sort-Object)
}
$modelRecorded = $(if ($observed.models.Count -gt 0) { 'PASS' } else { 'UNKNOWN' })

$report = [ordered]@{
    schema_version = 1
    observed_at = (Get-Date).ToString('o')
    source_kind = 'RUN_SCOPED_DIAGNOSTIC_TRACE'
    source_file_count = $files.Count
    parsed_records = $parsedRecords
    parse_failures = $parseFailures
    runtime = [ordered]@{
        codex_cli_version = $CodexCliVersion
        codex_app_version = $CodexAppVersion
    }
    observed = $observed
    model_recorded = $modelRecorded
    configuration_recorded = 'UNKNOWN'
    configuration_note = 'Observed values are retained, but completeness stays UNKNOWN until the post-Experiment-002 ModelRuntimeProfile contract is frozen.'
}
if ($OutputPath | Split-Path -Parent) {
    New-Item -ItemType Directory -Force -Path (Split-Path $OutputPath -Parent) | Out-Null
}
$report | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath $OutputPath -Encoding UTF8
Write-Host "Tokn runtime observation: $modelRecorded"
Write-Host "Models observed: $($observed.models -join ', ')"
Write-Host "Output: $OutputPath"
