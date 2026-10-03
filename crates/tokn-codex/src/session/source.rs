use tokn_domain::{EvidenceIdentityCoverage, ToolObservation};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StableSourceLocatorObservation {
    pub locator: Option<String>,
    pub coverage: EvidenceIdentityCoverage,
}

impl StableSourceLocatorObservation {
    fn observed(locator: String) -> Self {
        Self {
            locator: Some(locator),
            coverage: EvidenceIdentityCoverage::Observed,
        }
    }

    fn not_captured() -> Self {
        Self {
            locator: None,
            coverage: EvidenceIdentityCoverage::NotCaptured,
        }
    }
}

pub fn file_source_locator_from_relative_path(
    relative_path: &str,
    selected_workspace: Option<&str>,
) -> StableSourceLocatorObservation {
    let Some(workspace) = selected_workspace.filter(|value| !value.trim().is_empty()) else {
        return StableSourceLocatorObservation::not_captured();
    };
    let Some(relative) = workspace_relative_path(relative_path, Some(workspace), workspace) else {
        return StableSourceLocatorObservation::not_captured();
    };
    StableSourceLocatorObservation::observed(format!("file:{relative}"))
}

pub fn extract_file_source_locator(
    tool: &ToolObservation,
    agent_cwd: Option<&str>,
    selected_workspace: Option<&str>,
) -> StableSourceLocatorObservation {
    match tool.category.as_str() {
        "file_read" => extract_file_read_source_locator(tool, agent_cwd, selected_workspace),
        "write_mutation" => {
            extract_file_mutation_source_locator(tool, agent_cwd, selected_workspace)
        }
        _ => StableSourceLocatorObservation::not_captured(),
    }
}

pub fn extract_file_read_source_locator(
    tool: &ToolObservation,
    agent_cwd: Option<&str>,
    selected_workspace: Option<&str>,
) -> StableSourceLocatorObservation {
    if tool.category != "file_read" {
        return StableSourceLocatorObservation::not_captured();
    }

    let Some(command) = tool.command.as_deref() else {
        return StableSourceLocatorObservation::not_captured();
    };
    let Some(workspace) = selected_workspace.filter(|value| !value.trim().is_empty()) else {
        return StableSourceLocatorObservation::not_captured();
    };
    let Some(target) = parse_simple_get_content_target(command) else {
        return StableSourceLocatorObservation::not_captured();
    };
    let cwd = tool
        .workdir
        .as_deref()
        .filter(|value| !value.trim().is_empty())
        .or(agent_cwd.filter(|value| !value.trim().is_empty()));

    let Some(relative) = workspace_relative_path(&target, cwd, workspace) else {
        return StableSourceLocatorObservation::not_captured();
    };

    StableSourceLocatorObservation::observed(format!("file:{relative}"))
}

pub fn extract_file_mutation_source_locator(
    tool: &ToolObservation,
    agent_cwd: Option<&str>,
    selected_workspace: Option<&str>,
) -> StableSourceLocatorObservation {
    if tool.category != "write_mutation" {
        return StableSourceLocatorObservation::not_captured();
    }
    let Some(command) = tool.command.as_deref() else {
        return StableSourceLocatorObservation::not_captured();
    };
    let Some(workspace) = selected_workspace.filter(|value| !value.trim().is_empty()) else {
        return StableSourceLocatorObservation::not_captured();
    };
    let Some(target) = parse_simple_file_mutation_target(command) else {
        return StableSourceLocatorObservation::not_captured();
    };
    let cwd = tool
        .workdir
        .as_deref()
        .filter(|value| !value.trim().is_empty())
        .or(agent_cwd.filter(|value| !value.trim().is_empty()));
    let Some(relative) = workspace_relative_path(&target, cwd, workspace) else {
        return StableSourceLocatorObservation::not_captured();
    };
    StableSourceLocatorObservation::observed(format!("file:{relative}"))
}

fn parse_simple_get_content_target(command: &str) -> Option<String> {
    let tokens = tokenize_simple_command(command)?;
    if tokens.is_empty() || !tokens[0].eq_ignore_ascii_case("Get-Content") {
        return None;
    }

    let target = match tokens.as_slice() {
        [_, path] if !path.starts_with('-') => path,
        [_, flag, path]
            if flag.eq_ignore_ascii_case("-LiteralPath") || flag.eq_ignore_ascii_case("-Path") =>
        {
            path
        }
        _ => return None,
    };

    if target.is_empty()
        || target.starts_with('~')
        || target.contains('$')
        || target.contains('`')
        || target.chars().any(|ch| matches!(ch, '*' | '?' | '[' | ']'))
    {
        return None;
    }
    Some(target.clone())
}

fn parse_simple_file_mutation_target(command: &str) -> Option<String> {
    let tokens = tokenize_simple_command(command)?;
    let verb = tokens.first()?;
    if !verb.eq_ignore_ascii_case("Set-Content") && !verb.eq_ignore_ascii_case("Add-Content") {
        return None;
    }
    let target = if tokens.len() >= 2 && !tokens[1].starts_with('-') {
        &tokens[1]
    } else if tokens.len() >= 3
        && (tokens[1].eq_ignore_ascii_case("-LiteralPath")
            || tokens[1].eq_ignore_ascii_case("-Path"))
    {
        &tokens[2]
    } else {
        return None;
    };
    if target.is_empty()
        || target.starts_with('~')
        || target.contains('$')
        || target.contains('`')
        || target.contains(',')
        || target.chars().any(|ch| matches!(ch, '*' | '?' | '[' | ']'))
    {
        return None;
    }
    Some(target.clone())
}

fn tokenize_simple_command(command: &str) -> Option<Vec<String>> {
    let mut tokens = Vec::new();
    let mut current = String::new();
    let mut quote = None;

    for ch in command.trim().chars() {
        if let Some(active) = quote {
            if ch == active {
                quote = None;
            } else {
                current.push(ch);
            }
            continue;
        }

        match ch {
            '\'' | '"' => quote = Some(ch),
            ';' | '|' | '&' | '\n' | '\r' => return None,
            ch if ch.is_whitespace() => {
                if !current.is_empty() {
                    tokens.push(std::mem::take(&mut current));
                }
            }
            _ => current.push(ch),
        }
    }

    if quote.is_some() {
        return None;
    }
    if !current.is_empty() {
        tokens.push(current);
    }
    Some(tokens)
}

fn workspace_relative_path(target: &str, cwd: Option<&str>, workspace: &str) -> Option<String> {
    let workspace = normalize_path(workspace)?;
    let target = if is_absolute_like(target) {
        normalize_path(target)?
    } else {
        let cwd = normalize_path(cwd?)?;
        normalize_path(&format!("{cwd}/{target}"))?
    };

    let case_insensitive = looks_like_windows_path(&workspace);
    let workspace_cmp = if case_insensitive {
        workspace.to_ascii_lowercase()
    } else {
        workspace.clone()
    };
    let target_cmp = if case_insensitive {
        target.to_ascii_lowercase()
    } else {
        target.clone()
    };

    if target_cmp == workspace_cmp {
        return None;
    }
    let prefix = format!("{}/", workspace_cmp.trim_end_matches('/'));
    let relative = target_cmp.strip_prefix(&prefix)?;
    if relative.is_empty() || relative.starts_with("../") {
        return None;
    }
    Some(relative.to_string())
}

fn normalize_path(value: &str) -> Option<String> {
    let replaced = value.trim().replace('\\', "/");
    if replaced.is_empty() || replaced.starts_with("//") {
        return None;
    }

    let mut prefix = String::new();
    let mut rest = replaced.as_str();
    if replaced.len() >= 2 && replaced.as_bytes()[1] == b':' {
        prefix = replaced[..2].to_ascii_lowercase();
        rest = replaced[2..].trim_start_matches('/');
    } else if replaced.starts_with('/') {
        prefix.push('/');
        rest = replaced.trim_start_matches('/');
    }

    let mut parts: Vec<&str> = Vec::new();
    for part in rest.split('/') {
        match part {
            "" | "." => {}
            ".." => {
                parts.pop()?;
            }
            _ => parts.push(part),
        }
    }

    let body = parts.join("/");
    if prefix == "/" {
        Some(format!("/{body}"))
    } else if prefix.is_empty() {
        Some(body)
    } else if body.is_empty() {
        Some(format!("{prefix}/"))
    } else {
        Some(format!("{prefix}/{body}"))
    }
}

fn is_absolute_like(value: &str) -> bool {
    let value = value.trim().replace('\\', "/");
    value.starts_with('/') || (value.len() >= 2 && value.as_bytes()[1] == b':')
}

fn looks_like_windows_path(value: &str) -> bool {
    value.len() >= 2 && value.as_bytes()[1] == b':'
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tool(command: &str, workdir: Option<&str>) -> ToolObservation {
        ToolObservation {
            category: "file_read".into(),
            command: Some(command.into()),
            workdir: workdir.map(str::to_string),
            ..Default::default()
        }
    }

    #[test]
    fn resolves_relative_get_content_inside_selected_workspace() {
        let observed = extract_file_read_source_locator(
            &tool("Get-Content src/example.rs", Some(r"E:\repo\PROJECT")),
            None,
            Some(r"E:\repo\PROJECT"),
        );
        assert_eq!(observed.coverage, EvidenceIdentityCoverage::Observed);
        assert_eq!(observed.locator.as_deref(), Some("file:src/example.rs"));
    }

    #[test]
    fn supports_literal_path_and_agent_cwd_fallback() {
        let observed = extract_file_read_source_locator(
            &tool("Get-Content -LiteralPath 'src/example.rs'", None),
            Some(r"E:\repo\PROJECT"),
            Some(r"E:\repo\PROJECT"),
        );
        assert_eq!(observed.locator.as_deref(), Some("file:src/example.rs"));
    }

    #[test]
    fn locator_is_stable_across_workspace_clones() {
        let left = extract_file_read_source_locator(
            &tool("Get-Content README.md", Some(r"E:\clone-a\PROJECT")),
            None,
            Some(r"E:\clone-a\PROJECT"),
        );
        let right = extract_file_read_source_locator(
            &tool("Get-Content README.md", Some(r"E:\clone-b\PROJECT")),
            None,
            Some(r"E:\clone-b\PROJECT"),
        );
        assert_eq!(left.locator, right.locator);
    }

    #[test]
    fn rejects_outside_workspace_and_ambiguous_commands() {
        for candidate in [
            tool("Get-Content ..\\secret.txt", Some(r"E:\repo\PROJECT")),
            tool("Get-Content *.rs", Some(r"E:\repo\PROJECT")),
            tool(
                "Get-Content a.rs | Select-Object -First 1",
                Some(r"E:\repo\PROJECT"),
            ),
            tool("Get-Content $env:TEMP", Some(r"E:\repo\PROJECT")),
        ] {
            let observed =
                extract_file_read_source_locator(&candidate, None, Some(r"E:\repo\PROJECT"));
            assert_eq!(observed.coverage, EvidenceIdentityCoverage::NotCaptured);
            assert!(observed.locator.is_none());
        }
    }

    #[test]
    fn ignores_non_file_read_categories() {
        let mut value = tool("Get-Content README.md", Some(r"E:\repo\PROJECT"));
        value.category = "search".into();
        let observed = extract_file_read_source_locator(&value, None, Some(r"E:\repo\PROJECT"));
        assert_eq!(observed.coverage, EvidenceIdentityCoverage::NotCaptured);
    }
    #[test]
    fn resolves_simple_set_content_mutation_inside_workspace() {
        let mut value = tool(
            "Set-Content src/example.rs updated",
            Some(r"E:\repo\PROJECT"),
        );
        value.category = "write_mutation".into();
        let observed = extract_file_mutation_source_locator(&value, None, Some(r"E:\repo\PROJECT"));
        assert_eq!(observed.coverage, EvidenceIdentityCoverage::Observed);
        assert_eq!(observed.locator.as_deref(), Some("file:src/example.rs"));
    }

    #[test]
    fn snapshot_relative_locator_matches_across_windows_clones() {
        let left =
            file_source_locator_from_relative_path("Src/Example.rs", Some(r"E:\clone-a\PROJECT"));
        let right =
            file_source_locator_from_relative_path("src/example.rs", Some(r"E:\clone-b\PROJECT"));
        assert_eq!(left.coverage, EvidenceIdentityCoverage::Observed);
        assert_eq!(left.locator, right.locator);
        assert_eq!(left.locator.as_deref(), Some("file:src/example.rs"));
    }

    #[test]
    fn rejects_ambiguous_mutation_targets() {
        for command in [
            "Set-Content *.rs updated",
            "Set-Content $env:TEMP updated",
            "Copy-Item a b",
        ] {
            let mut value = tool(command, Some(r"E:\repo\PROJECT"));
            value.category = "write_mutation".into();
            let observed =
                extract_file_mutation_source_locator(&value, None, Some(r"E:\repo\PROJECT"));
            assert_eq!(observed.coverage, EvidenceIdentityCoverage::NotCaptured);
        }
    }
}
