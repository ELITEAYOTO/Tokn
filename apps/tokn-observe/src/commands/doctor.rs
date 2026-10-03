use tokn_codex::{discover_codex, session_root};
use tokn_platform::{observer_data_root, platform_info};

pub fn run(dev: bool) -> anyhow::Result<()> {
    let p = platform_info();
    println!("TOKN OBSERVER {}", env!("CARGO_PKG_VERSION"));
    println!();
    println!("PLATFORM");
    println!("  OS              {}", p.os);
    println!("  architecture    {}", p.arch);
    if let Some(root) = observer_data_root() {
        println!("  runtime data    {}", root.display());
    }

    println!();
    println!("CODEX");
    let installs = discover_codex();
    if installs.is_empty() {
        println!("  install         NOT FOUND");
    } else {
        for install in installs {
            println!("  surface         {:?}", install.surface);
            println!("  path            {}", install.path.display());
            println!(
                "  version         {}",
                install.version.unwrap_or_else(|| "unknown".into())
            );
            println!(
                "  capabilities    {}",
                install.capabilities.names().join(", ")
            );
        }
    }

    println!();
    println!("SESSIONS");
    println!(
        "  root            {}",
        session_root()
            .map(|p| p.display().to_string())
            .unwrap_or_else(|| "not found".into())
    );

    if dev {
        println!();
        println!("DEVELOPMENT");
        println!("  rust target     x86_64-pc-windows-msvc");
    }

    Ok(())
}
