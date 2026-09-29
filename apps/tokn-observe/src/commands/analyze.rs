use std::path::Path;

use tokn_analysis::build_attribution;
use tokn_codex::diagnostic::{DiagnosticBundle, read_diagnostic_observations};
use tokn_report::render_attribution_text;

pub fn run(source: &str, top: usize) -> anyhow::Result<()> {
    let path = Path::new(source);
    let bundle = DiagnosticBundle::detect(path)
        .ok_or_else(|| anyhow::anyhow!("not a valid diagnostic trace bundle"))?;
    let observations = read_diagnostic_observations(&bundle)?;
    let report = build_attribution(&observations);
    print!("{}", render_attribution_text(&report, top));
    Ok(())
}
