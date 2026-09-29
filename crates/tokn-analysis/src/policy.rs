use tokn_domain::{
    PolicyEnforcementStatus, PolicyEvidenceReport, PolicyObservationSummary, PolicyPlacement,
    SessionPolicyEvidence,
};

pub fn build_policy_evidence_report(
    policy_id: impl Into<String>,
    marker: impl Into<String>,
    placements: Vec<PolicyPlacement>,
    sessions: Vec<SessionPolicyEvidence>,
    observed: PolicyObservationSummary,
    enforcement: PolicyEnforcementStatus,
) -> PolicyEvidenceReport {
    let instruction_observed_threads = sessions
        .iter()
        .filter(|session| session.instruction_observed)
        .count() as u64;
    let repository_read_threads = sessions
        .iter()
        .filter(|session| session.repository_read_observed)
        .count() as u64;
    let repository_read_count = sessions
        .iter()
        .map(|session| session.repository_read_count)
        .sum();
    let parse_failures = sessions.iter().map(|session| session.parse_failures).sum();

    PolicyEvidenceReport {
        policy_id: policy_id.into(),
        marker: marker.into(),
        hint_placed: !placements.is_empty(),
        placements,
        sessions,
        instruction_observed_threads,
        repository_read_threads,
        repository_read_count,
        parse_failures,
        observed,
        enforcement,
    }
}

#[cfg(test)]
mod tests {
    use tokn_domain::{PolicyObservationStatus, SessionPolicyEvidence};

    use super::*;

    #[test]
    fn aggregates_policy_evidence_without_inventing_enforcement() {
        let sessions = vec![
            SessionPolicyEvidence {
                instruction_observed: true,
                repository_read_observed: true,
                repository_read_count: 2,
                ..Default::default()
            },
            SessionPolicyEvidence {
                parse_failures: 1,
                ..Default::default()
            },
        ];
        let observed = PolicyObservationSummary {
            status: PolicyObservationStatus::Fail,
            targeted: 3,
            compliant: 2,
            violations: 1,
            ..Default::default()
        };

        let report = build_policy_evidence_report(
            "test-policy",
            "marker",
            vec![PolicyPlacement {
                path: "C:/repo/AGENTS.md".into(),
                sha256: None,
            }],
            sessions,
            observed,
            PolicyEnforcementStatus::NotProven,
        );

        assert!(report.hint_placed);
        assert_eq!(report.instruction_observed_threads, 1);
        assert_eq!(report.repository_read_threads, 1);
        assert_eq!(report.repository_read_count, 2);
        assert_eq!(report.parse_failures, 1);
        assert_eq!(report.observed.status, PolicyObservationStatus::Fail);
        assert_eq!(report.enforcement, PolicyEnforcementStatus::NotProven);
    }
}
