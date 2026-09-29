pub fn classify_command_text(command: &str) -> String {
    let command = command.to_ascii_lowercase();

    if contains_any(
        &command,
        &[
            "cargo test",
            "cargo clippy",
            "cargo check",
            "cargo build",
            "npm test",
            "npm run test",
            "npm run build",
            "npm run lint",
            "pnpm test",
            "pnpm build",
            "pnpm lint",
            "pytest",
            "dotnet test",
        ],
    ) {
        "test_build".into()
    } else if contains_any(
        &command,
        &[
            "set-content",
            "add-content",
            "out-file",
            "copy-item",
            "move-item",
            "remove-item",
            "new-item",
            "mkdir",
            "write_text(",
            "write_bytes(",
        ],
    ) {
        "write_mutation".into()
    } else if contains_any(
        &command,
        &[" rg ", "rg ", "grep ", "findstr", "select-string"],
    ) {
        "search".into()
    } else if contains_any(
        &command,
        &[
            "get-content",
            " cat ",
            "cat ",
            " head ",
            "head ",
            " tail ",
            "tail ",
            "type ",
        ],
    ) {
        "file_read".into()
    } else if contains_any(&command, &["get-childitem", " dir ", "dir ", " ls ", "ls "]) {
        "directory_list".into()
    } else if contains_any(&command, &["git ", " git "]) {
        "git".into()
    } else if contains_any(
        &command,
        &["start-process", "stop-process", "taskkill", "kill-process"],
    ) {
        "process_control".into()
    } else {
        "command_other".into()
    }
}

fn contains_any(text: &str, needles: &[&str]) -> bool {
    needles.iter().any(|needle| text.contains(needle))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classifies_shared_command_families() {
        assert_eq!(classify_command_text("Get-Content README.md"), "file_read");
        assert_eq!(classify_command_text("rg TODO src"), "search");
        assert_eq!(
            classify_command_text("cargo test --workspace"),
            "test_build"
        );
        assert_eq!(
            classify_command_text("$x=Get-Content a; Set-Content b $x"),
            "write_mutation"
        );
    }
}
