use std::fs::File;
use std::path::Path;

use tokn_analysis::reduce_experiment_validity;
use tokn_domain::ExperimentValidityInput;
use tokn_report::render_experiment_validity_text;

pub fn run(input: &Path, output_json: Option<&Path>) -> anyhow::Result<()> {
    let evidence: ExperimentValidityInput = serde_json::from_reader(File::open(input)?)?;
    let report = reduce_experiment_validity(&evidence);

    print!("{}", render_experiment_validity_text(&report));

    if let Some(path) = output_json {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        serde_json::to_writer_pretty(File::create(path)?, &report)?;
        println!("\nJSON\n  {}", path.display());
    }

    Ok(())
}
