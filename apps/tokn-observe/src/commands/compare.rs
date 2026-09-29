use std::path::Path;

use tokn_analysis::compare_runs;
use tokn_codex::diagnostic::{DiagnosticBundle, read_diagnostic_observations};
use tokn_report::render_comparison_text;

pub fn run(baseline: &str, candidate: &str) -> anyhow::Result<()> {
    let baseline = load_bundle(Path::new(baseline))?;
    let candidate = load_bundle(Path::new(candidate))?;
    let comparison = compare_runs(&baseline, &candidate);
    print!("{}", render_comparison_text(&comparison));
    Ok(())
}

fn load_bundle(path: &Path) -> anyhow::Result<tokn_domain::DiagnosticObservations> {
    let bundle = DiagnosticBundle::detect(path).ok_or_else(|| {
        anyhow::anyhow!("not a valid diagnostic trace bundle: {}", path.display())
    })?;
    read_diagnostic_observations(&bundle)
}
