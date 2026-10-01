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
