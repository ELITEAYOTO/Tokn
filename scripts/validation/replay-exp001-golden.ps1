param(
    [string]$BinaryPath = "target\release\tokn-observe.exe"
)

$ErrorActionPreference = "Stop"
$Repo = (Resolve-Path (Join-Path $PSScriptRoot "..\..")).Path
$Fixture = Join-Path $Repo "fixtures\experiments\001-runner-golden"
$Evidence = Join-Path $Repo "target\p9-exp001-golden"

function Assert-Equal {
    param([string]$Name, $Actual, $Expected)
    if ([string]$Actual -ne [string]$Expected) {
        throw "$Name mismatch: actual=$Actual expected=$Expected"
    }
    Write-Host "[PASS] $Name = $Actual"
}

Push-Location $Repo
try {
    if (-not (Test-Path -LiteralPath $BinaryPath)) {
        throw "Tokn binary not found: $BinaryPath"
    }
    if (Test-Path -LiteralPath $Evidence) {
        Remove-Item -LiteralPath $Evidence -Recurse -Force
    }

    & $BinaryPath runner (Join-Path $Fixture "runner-request.json")
    if ($LASTEXITCODE -ne 0) {
        throw "Runner exited with code $LASTEXITCODE"
    }

    $Expected = (Get-Content (Join-Path $Fixture "expected.json") -Raw | ConvertFrom-Json).expected
    $Result = Get-Content (Join-Path $Evidence "runner-result.json") -Raw | ConvertFrom-Json
    $Contract = Get-Content (Join-Path $Evidence "measurement-contract.json") -Raw | ConvertFrom-Json
    $Group = Get-Content (Join-Path $Evidence "run-group.json") -Raw | ConvertFrom-Json
    $Policy = Get-Content (Join-Path $Evidence "policy-evidence.json") -Raw | ConvertFrom-Json
    $Diff = Get-Content (Join-Path $Evidence "workspace-diff.json") -Raw | ConvertFrom-Json
    $Validity = Get-Content (Join-Path $Evidence "validity-report.json") -Raw | ConvertFrom-Json

    $Logical = [uint64]$Group.totals.input_tokens + [uint64]$Group.totals.output_tokens
    $Uncached = [uint64]$Group.totals.input_tokens - [uint64]$Group.totals.cached_input_tokens
    $Parent = @($Group.agents | Where-Object { $_.depth -eq 0 })[0]
    $ParentLogical = [uint64]$Parent.totals.input_tokens + [uint64]$Parent.totals.output_tokens
    $DescendantLogical = 0
    foreach ($Agent in @($Group.agents | Where-Object { $_.depth -gt 0 })) {
        $DescendantLogical += [uint64]$Agent.totals.input_tokens + [uint64]$Agent.totals.output_tokens
    }

    Assert-Equal "measurement_contract_id" $Contract.contract_id "tokn.measurement.v0.1"
    foreach ($property in @(
        "schema_version",
        "measurement_contract_version",
        "evidence_layout_version",
        "runner_request_schema_version",
        "runner_result_schema_version",
        "session_evidence_schema_version",
        "run_group_schema_version",
        "token_accounting_semantics_version",
        "source_health_schema_version",
        "terminal_status_semantics_version",
        "workspace_inventory_schema_version",
        "workspace_resolution_schema_version",
        "project_snapshot_schema_version",
        "workspace_diff_schema_version",
        "policy_evidence_schema_version",
        "quality_gate_schema_version",
        "recovery_report_schema_version",
        "experiment_validity_schema_version",
        "model_runtime_profile_schema_version",
        "analyzer_semantics_version"
    )) {
        Assert-Equal ("contract_" + $property) $Contract.$property 1
    }
    if ([string]::IsNullOrWhiteSpace([string]$Result.artifacts.measurement_contract)) {
        throw "runner result does not reference measurement-contract.json"
    }

    Assert-Equal "pipeline_status" $Result.pipeline_status $Expected.pipeline_status
    Assert-Equal "agents" $Result.agent_count $Expected.agents
    Assert-Equal "usage_records" $Group.totals.usage_records $Expected.usage_records
    Assert-Equal "input_tokens" $Group.totals.input_tokens $Expected.input_tokens
    Assert-Equal "cached_input_tokens" $Group.totals.cached_input_tokens $Expected.cached_input_tokens
    Assert-Equal "uncached_input_tokens" $Uncached $Expected.uncached_input_tokens
    Assert-Equal "output_tokens" $Group.totals.output_tokens $Expected.output_tokens
    Assert-Equal "reasoning_output_tokens" $Group.totals.reasoning_output_tokens $Expected.reasoning_output_tokens
    Assert-Equal "logical_total_tokens" $Logical $Expected.logical_total_tokens
    Assert-Equal "parent_logical_total_tokens" $ParentLogical $Expected.parent_logical_total_tokens
    Assert-Equal "descendant_logical_total_tokens" $DescendantLogical $Expected.descendant_logical_total_tokens
    Assert-Equal "root_terminal" $Result.root_terminal $Expected.root_terminal

    Assert-Equal "policy_observation" $Policy.observed.status $Expected.policy_observation
    Assert-Equal "policy_targeted" $Policy.observed.targeted $Expected.policy_targeted
    Assert-Equal "policy_compliant" $Policy.observed.compliant $Expected.policy_compliant
    Assert-Equal "policy_violations" $Policy.observed.violations $Expected.policy_violations
    Assert-Equal "policy_unknown" $Policy.observed.unknown $Expected.policy_unknown
    Assert-Equal "policy_parse_failures" $Policy.observed.parse_failures $Expected.policy_parse_failures
    Assert-Equal "policy_enforcement" $Policy.enforcement $Expected.policy_enforcement

    Assert-Equal "selected_workspace" $Result.selected_workspace $Expected.selected_workspace
    Assert-Equal "diff_added" $Diff.added_count $Expected.diff_added
    Assert-Equal "diff_modified" $Diff.modified_count $Expected.diff_modified
    Assert-Equal "diff_removed" $Diff.removed_count $Expected.diff_removed
    Assert-Equal "diff_targets_selected_workspace" $Result.workspace_diff_targets_selected_workspace $true

    Assert-Equal "validity_verdict" $Validity.verdict $Expected.validity_verdict
    Assert-Equal "causal_claims_allowed" $Validity.causal_claims_allowed $Expected.causal_claims_allowed
    Assert-Equal "descriptive_metrics_allowed" $Validity.descriptive_metrics_allowed $Expected.descriptive_metrics_allowed
    Assert-Equal "quality_status" $Result.quality_status $Expected.quality_status
    Assert-Equal "pending_steps" @($Result.pending_steps).Count 0

    Write-Host "EXP001 GOLDEN REPLAY PASS"
}
finally {
    Pop-Location
}
