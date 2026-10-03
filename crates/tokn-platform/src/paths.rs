use std::path::PathBuf;

pub fn user_home() -> Option<PathBuf> {
    std::env::var_os("USERPROFILE")
        .or_else(|| std::env::var_os("HOME"))
        .map(PathBuf::from)
}

pub fn codex_home() -> Option<PathBuf> {
    if let Some(path) = std::env::var_os("CODEX_HOME") {
        return Some(PathBuf::from(path));
    }
    user_home().map(|p| p.join(".codex"))
}

pub fn observer_data_root() -> Option<PathBuf> {
    std::env::var_os("LOCALAPPDATA")
        .map(PathBuf::from)
        .map(|p| p.join("Tokn").join("Observer"))
}

pub fn observer_database_path() -> Option<PathBuf> {
    observer_data_root().map(|root| root.join("db").join("observer.sqlite3"))
}

pub fn observer_shadow_index_root() -> Option<PathBuf> {
    observer_data_root().map(|root| root.join("shadow-index"))
}

pub fn observer_shadow_project_root(project_id: &str) -> Option<PathBuf> {
    if !is_safe_cache_segment(project_id) {
        return None;
    }
    observer_shadow_index_root().map(|root| root.join(project_id))
}

fn is_safe_cache_segment(value: &str) -> bool {
    !value.is_empty()
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shadow_project_segment_accepts_private_ids_and_rejects_path_material() {
        assert!(is_safe_cache_segment("prj-0123456789abcdef01234567"));
        assert!(is_safe_cache_segment("fixture_project-1"));

        for value in [
            "",
            ".",
            "..",
            "../escape",
            r"..\escape",
            "project/child",
            r"C:\private\project",
            "project name",
            "project:stream",
        ] {
            assert!(
                !is_safe_cache_segment(value),
                "unexpectedly accepted {value:?}"
            );
        }
    }
}
