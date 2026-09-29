use std::process::{Command, Output};

pub fn run_capture(program: &str, args: &[&str]) -> std::io::Result<Output> {
    Command::new(program).args(args).output()
}
