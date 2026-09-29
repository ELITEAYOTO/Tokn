use tokn_codex::diagnostic::{DiagnosticBundle, assess_diagnostic_health};
use tokn_codex::session::assess_session_health;
use tokn_report::render_source_health_text;

use super::common::resolve_source;

pub fn run(source: &str) -> anyhow::Result<()> {
    let path = resolve_source(source)?;

    let health = if path.is_dir() {
        let bundle = DiagnosticBundle::detect(&path)
            .ok_or_else(|| anyhow::anyhow!("not a valid diagnostic trace bundle"))?;
        assess_diagnostic_health(&bundle)?
    } else {
        assess_session_health(&path)?
    };

    print!(
        "{}",
        render_source_health_text(&path.to_string_lossy(), &health)
    );
    Ok(())
}
