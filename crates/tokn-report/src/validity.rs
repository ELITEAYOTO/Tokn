use tokn_domain::ExperimentValidityReport;

pub fn render_experiment_validity_text(report: &ExperimentValidityReport) -> String {
    let mut out = String::new();
    out.push_str("TOKN EXPERIMENT VALIDITY V0.1\n\n");
    out.push_str("SUMMARY\n");
    out.push_str(&format!("  experiment_id: {}\n", report.experiment_id));
    out.push_str(&format!("  intent: {}\n", report.intent.as_str()));
    out.push_str(&format!("  verdict: {}\n", report.verdict.as_str()));
    out.push_str(&format!(
        "  causal_claims_allowed: {}\n",
        report.causal_claims_allowed
    ));
    out.push_str(&format!(
        "  descriptive_metrics_allowed: {}\n",
        report.descriptive_metrics_allowed
    ));

    out.push_str("\nREASONS\n");
    if report.reasons.is_empty() {
        out.push_str("  none\n");
    } else {
        for item in &report.reasons {
            out.push_str(&format!(
                "  [{}] {} {}: {}\n",
                item.dimension,
                item.code,
                item.status.as_str(),
                item.message
            ));
        }
    }

    out.push_str("\nCLAIM GATE\n");
    if report.causal_claims_allowed {
        out.push_str("  causal winner/savings claims: ALLOWED\n");
    } else {
        out.push_str("  causal winner/savings claims: BLOCKED\n");
        out.push_str("  descriptive metrics may still be reported when allowed above.\n");
    }
    out
}

#[cfg(test)]
mod tests {
    use tokn_domain::{ExperimentIntent, ExperimentValidityVerdict};

    use super::*;

    #[test]
    fn non_causal_verdict_explicitly_blocks_causal_claims() {
        let report = ExperimentValidityReport {
            experiment_id: "001".into(),
            intent: ExperimentIntent::Instrumentation,
            verdict: ExperimentValidityVerdict::InstrumentationOnly,
            causal_claims_allowed: false,
            descriptive_metrics_allowed: true,
            reasons: Vec::new(),
        };

        let text = render_experiment_validity_text(&report);
        assert!(text.contains("verdict: INSTRUMENTATION_ONLY"));
        assert!(text.contains("causal winner/savings claims: BLOCKED"));
    }
}
