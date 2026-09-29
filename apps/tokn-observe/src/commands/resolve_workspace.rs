use std::fs::File;
use std::path::{Path, PathBuf};

use tokn_analysis::{WorkspaceInventory, resolve_workspace};
use tokn_codex::session::collect_session_group;
use tokn_report::render_workspace_resolution_text;

use super::common::{resolve_session_root_from_source, resolve_source};

pub fn run(
    source: &str,
    inventory: &Path,
    before_inventory: Option<&Path>,
    source_root: &Path,
    expected_outputs: &[PathBuf],
    output_json: Option<&Path>,
) -> anyhow::Result<()> {
    let source_path = resolve_source(source)?;
    let root_session = resolve_session_root_from_source(&source_path)?;
    let group = collect_session_group(&root_session)?;

    let after = read_inventory(inventory)?;
    let before = before_inventory.map(read_inventory).transpose()?;
    let expected = expected_outputs
        .iter()
        .map(|path| path.to_string_lossy().to_string())
        .collect::<Vec<_>>();

    let resolution = resolve_workspace(
        before.as_ref(),
        &after,
        &source_root.to_string_lossy(),
        &expected,
        &group.members,
    );

    print!("{}", render_workspace_resolution_text(&resolution));

    if let Some(path) = output_json {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        serde_json::to_writer_pretty(File::create(path)?, &resolution)?;
        println!("\nJSON\n  {}", path.display());
    }

    Ok(())
}

fn read_inventory(path: &Path) -> anyhow::Result<WorkspaceInventory> {
    let bytes = std::fs::read(path)?;
    let bytes = bytes.strip_prefix(&[0xEF, 0xBB, 0xBF]).unwrap_or(&bytes);
    Ok(serde_json::from_slice(bytes)?)
}
