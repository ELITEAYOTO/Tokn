mod server;

use std::path::PathBuf;

use anyhow::Context;
use clap::Parser;
use tokn_platform::observer_database_path;
use tokn_storage::Database;

#[derive(Debug, Parser)]
#[command(name = "tokn-mcp", version, about = "Tokn local read-only MCP adapter")]
struct Cli {
    #[arg(long)]
    db: Option<PathBuf>,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let db_path = cli
        .db
        .or_else(observer_database_path)
        .context("LOCALAPPDATA is unavailable and --db was not provided")?;
    let db = Database::open_read_only(&db_path)?;
    server::run_stdio(db)
}
