use tokn_codex::session::list_sessions;

pub fn run(limit: usize) -> anyhow::Result<()> {
    let sessions = list_sessions();
    println!("TOKN SESSIONS");
    println!("found: {}", sessions.len());
    println!();

    for session in sessions.into_iter().take(limit) {
        println!("{:>10} bytes  {}", session.size, session.path.display());
    }

    Ok(())
}
