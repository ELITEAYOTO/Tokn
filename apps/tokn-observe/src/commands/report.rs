use tokn_report::render_text;

use super::common::open_db;

pub fn run() -> anyhow::Result<()> {
    let db = open_db()?;
    let Some(run) = db.latest_run()? else {
        println!("No imported runs.");
        return Ok(());
    };

    print!("{}", render_text(&run));
    Ok(())
}
