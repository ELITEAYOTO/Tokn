use tokn_domain::PolicyEvidenceReport;

pub fn render_policy_evidence_text(report: &PolicyEvidenceReport) -> String {
    let mut out = String::new();
    out.push_str("TOKN POLICY EVIDENCE V0.1\n\n");
    out.push_str("SUMMARY\n");
    out.push_str(&format!("  policy_id: {}\n", report.policy_id));
    out.push_str(&format!(
        "  policy_hint: {}\n",
        if report.hint_placed {
            "PRESENT"
        } else {
            "ABSENT"
        }
    ));
    out.push_str(&format!(
        "  instruction_observed_threads: {}\n",
        report.instruction_observed_threads
    ));
    out.push_str(&format!(
        "  repository_read_threads: {}\n",
        report.repository_read_threads
    ));
    out.push_str(&format!(
        "  repository_read_count: {}\n",
        report.repository_read_count
    ));
    out.push_str(&format!("  parse_failures: {}\n", report.parse_failures));
    out.push_str(&format!(
        "  policy_observed: {}\n",
        report.observed.status.as_str()
    ));
    out.push_str(&format!(
        "  observed_targets: {}\n  observed_compliant: {}\n  observed_violations: {}\n  observed_unknown: {}\n",
        report.observed.targeted,
        report.observed.compliant,
        report.observed.violations,
        report.observed.unknown
    ));
    out.push_str(&format!(
        "  policy_enforced: {}\n",
        report.enforcement.as_str()
    ));

    out.push_str("\nPLACEMENTS\n");
    if report.placements.is_empty() {
        out.push_str("  none\n");
    } else {
        for placement in &report.placements {
            let hash = placement.sha256.as_deref().unwrap_or("UNKNOWN");
            out.push_str(&format!("  {} sha256={}\n", placement.path, hash));
        }
    }

    out.push_str("\nSESSIONS\n");
    for session in &report.sessions {
        out.push_str(&format!(
            "  thread={} instructions={} repository_reads={} parse_failures={} source={}\n",
            session.thread_id.as_deref().unwrap_or("UNKNOWN"),
            session.instruction_observed,
            session.repository_read_count,
            session.parse_failures,
            session.source_path
        ));
        for path in &session.matched_policy_paths {
            out.push_str(&format!("    matched_policy={}\n", path));
        }
    }

    out
}

#[cfg(test)]
mod tests {
    use tokn_domain::{
        PolicyEnforcementStatus, PolicyEvidenceReport, PolicyObservationStatus,
        PolicyObservationSummary,
    };

    use super::*;

    #[test]
    fn renders_three_policy_axes_separately() {
        let report = PolicyEvidenceReport {
            policy_id: "p1".into(),
            hint_placed: true,
            observed: PolicyObservationSummary {
                status: PolicyObservationStatus::Fail,
                violations: 1,
                ..Default::default()
            },
            enforcement: PolicyEnforcementStatus::NotProven,
            ..Default::default()
        };

        let rendered = render_policy_evidence_text(&report);
        assert!(rendered.contains("policy_hint: PRESENT"));
        assert!(rendered.contains("policy_observed: FAIL"));
        assert!(rendered.contains("policy_enforced: NOT_PROVEN"));
    }
}
