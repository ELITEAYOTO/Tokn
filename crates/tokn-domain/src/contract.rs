use serde::{Deserialize, Serialize};

pub const MEASUREMENT_CONTRACT_ID: &str = "tokn.measurement.v0.1";
pub const MEASUREMENT_CONTRACT_MANIFEST_SCHEMA_VERSION: u64 = 1;
pub const MEASUREMENT_CONTRACT_VERSION: u64 = 1;
pub const EVIDENCE_LAYOUT_VERSION: u64 = 1;
pub const RUNNER_REQUEST_SCHEMA_VERSION: u64 = 1;
pub const RUNNER_RESULT_SCHEMA_VERSION: u64 = 1;
pub const SESSION_EVIDENCE_SCHEMA_VERSION: u64 = 1;
pub const RUN_GROUP_SCHEMA_VERSION: u64 = 1;
pub const TOKEN_ACCOUNTING_SEMANTICS_VERSION: u64 = 1;
pub const SOURCE_HEALTH_SCHEMA_VERSION: u64 = 1;
pub const TERMINAL_STATUS_SEMANTICS_VERSION: u64 = 1;
pub const WORKSPACE_INVENTORY_SCHEMA_VERSION: u64 = 1;
pub const WORKSPACE_RESOLUTION_SCHEMA_VERSION: u64 = 1;
pub const PROJECT_SNAPSHOT_SCHEMA_VERSION: u64 = 1;
pub const WORKSPACE_DIFF_SCHEMA_VERSION: u64 = 1;
pub const POLICY_EVIDENCE_SCHEMA_VERSION: u64 = 1;
pub const QUALITY_GATE_SCHEMA_VERSION: u64 = 1;
pub const RECOVERY_REPORT_SCHEMA_VERSION: u64 = 1;
pub const EXPERIMENT_VALIDITY_SCHEMA_VERSION: u64 = 1;
pub const MODEL_RUNTIME_PROFILE_SCHEMA_VERSION: u64 = 1;
pub const ANALYZER_SEMANTICS_VERSION: u64 = 1;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MeasurementContractManifest {
    pub schema_version: u64,
    pub contract_id: String,
    pub measurement_contract_version: u64,
    pub evidence_layout_version: u64,
    pub runner_request_schema_version: u64,
    pub runner_result_schema_version: u64,
    pub session_evidence_schema_version: u64,
    pub run_group_schema_version: u64,
    pub token_accounting_semantics_version: u64,
    pub source_health_schema_version: u64,
    pub terminal_status_semantics_version: u64,
    pub workspace_inventory_schema_version: u64,
    pub workspace_resolution_schema_version: u64,
    pub project_snapshot_schema_version: u64,
    pub workspace_diff_schema_version: u64,
    pub policy_evidence_schema_version: u64,
    pub quality_gate_schema_version: u64,
    pub recovery_report_schema_version: u64,
    pub experiment_validity_schema_version: u64,
    pub model_runtime_profile_schema_version: u64,
    pub analyzer_semantics_version: u64,
}

impl Default for MeasurementContractManifest {
    fn default() -> Self {
        Self {
            schema_version: MEASUREMENT_CONTRACT_MANIFEST_SCHEMA_VERSION,
            contract_id: MEASUREMENT_CONTRACT_ID.into(),
            measurement_contract_version: MEASUREMENT_CONTRACT_VERSION,
            evidence_layout_version: EVIDENCE_LAYOUT_VERSION,
            runner_request_schema_version: RUNNER_REQUEST_SCHEMA_VERSION,
            runner_result_schema_version: RUNNER_RESULT_SCHEMA_VERSION,
            session_evidence_schema_version: SESSION_EVIDENCE_SCHEMA_VERSION,
            run_group_schema_version: RUN_GROUP_SCHEMA_VERSION,
            token_accounting_semantics_version: TOKEN_ACCOUNTING_SEMANTICS_VERSION,
            source_health_schema_version: SOURCE_HEALTH_SCHEMA_VERSION,
            terminal_status_semantics_version: TERMINAL_STATUS_SEMANTICS_VERSION,
            workspace_inventory_schema_version: WORKSPACE_INVENTORY_SCHEMA_VERSION,
            workspace_resolution_schema_version: WORKSPACE_RESOLUTION_SCHEMA_VERSION,
            project_snapshot_schema_version: PROJECT_SNAPSHOT_SCHEMA_VERSION,
            workspace_diff_schema_version: WORKSPACE_DIFF_SCHEMA_VERSION,
            policy_evidence_schema_version: POLICY_EVIDENCE_SCHEMA_VERSION,
            quality_gate_schema_version: QUALITY_GATE_SCHEMA_VERSION,
            recovery_report_schema_version: RECOVERY_REPORT_SCHEMA_VERSION,
            experiment_validity_schema_version: EXPERIMENT_VALIDITY_SCHEMA_VERSION,
            model_runtime_profile_schema_version: MODEL_RUNTIME_PROFILE_SCHEMA_VERSION,
            analyzer_semantics_version: ANALYZER_SEMANTICS_VERSION,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn v0_1_contract_versions_are_frozen_to_one() {
        let manifest = MeasurementContractManifest::default();
        assert_eq!(manifest.schema_version, 1);
        assert_eq!(manifest.contract_id, "tokn.measurement.v0.1");
        assert_eq!(manifest.measurement_contract_version, 1);
        assert_eq!(manifest.evidence_layout_version, 1);
        assert_eq!(manifest.runner_request_schema_version, 1);
        assert_eq!(manifest.runner_result_schema_version, 1);
        assert_eq!(manifest.session_evidence_schema_version, 1);
        assert_eq!(manifest.run_group_schema_version, 1);
        assert_eq!(manifest.token_accounting_semantics_version, 1);
        assert_eq!(manifest.source_health_schema_version, 1);
        assert_eq!(manifest.terminal_status_semantics_version, 1);
        assert_eq!(manifest.workspace_inventory_schema_version, 1);
        assert_eq!(manifest.workspace_resolution_schema_version, 1);
        assert_eq!(manifest.project_snapshot_schema_version, 1);
        assert_eq!(manifest.workspace_diff_schema_version, 1);
        assert_eq!(manifest.policy_evidence_schema_version, 1);
        assert_eq!(manifest.quality_gate_schema_version, 1);
        assert_eq!(manifest.recovery_report_schema_version, 1);
        assert_eq!(manifest.experiment_validity_schema_version, 1);
        assert_eq!(manifest.model_runtime_profile_schema_version, 1);
        assert_eq!(manifest.analyzer_semantics_version, 1);
    }

    #[test]
    fn contract_manifest_round_trip_is_stable() {
        let original = MeasurementContractManifest::default();
        let json = serde_json::to_string_pretty(&original).expect("serialize contract");
        let parsed: MeasurementContractManifest =
            serde_json::from_str(&json).expect("parse contract");
        assert_eq!(parsed, original);
    }
}
