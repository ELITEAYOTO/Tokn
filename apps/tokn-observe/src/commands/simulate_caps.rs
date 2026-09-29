use std::path::Path;

use tokn_analysis::simulate_caps;
use tokn_codex::diagnostic::{DiagnosticBundle, read_diagnostic_observations};
use tokn_report::render_cap_simulation_text;

use super::common::parse_cap_overrides;

pub fn run(source: &str, caps: &[String]) -> anyhow::Result<()> {
    let overrides = parse_cap_overrides(caps)?;
    if overrides.is_empty() {
        anyhow::bail!("at least one --cap CATEGORY=TOKENS override is required");
    }

    let path = Path::new(source);
    let bundle = DiagnosticBundle::detect(path).ok_or_else(|| {
        anyhow::anyhow!("not a valid diagnostic trace bundle: {}", path.display())
    })?;
    let observations = read_diagnostic_observations(&bundle)?;
    let report = simulate_caps(&observations, &overrides);
    print!("{}", render_cap_simulation_text(&report));
    Ok(())
}
