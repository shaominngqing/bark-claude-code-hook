pub mod settings;
pub mod toml_rules;

use std::path::PathBuf;

pub use settings::{has_hook, enable_hook, disable_hook};

/// Get the bark data directory (`~/.claude/hooks/`).
pub fn bark_dir() -> PathBuf {
    let home = dirs::home_dir().expect("Could not determine home directory");
    let dir = home.join(".claude").join("hooks");
    std::fs::create_dir_all(&dir).ok();
    dir
}

/// Get the bark SQLite database path.
pub fn bark_db_path() -> PathBuf {
    bark_dir().join("bark.db")
}

/// Get the bark log file path.
pub fn bark_log_path() -> PathBuf {
    bark_dir().join("bark.log")
}

/// Get the bark TOML config path (`~/.claude/bark.toml`).
pub fn bark_toml_path() -> PathBuf {
    let home = dirs::home_dir().expect("Could not determine home directory");
    home.join(".claude").join("bark.toml")
}

/// Get the Claude Code `settings.json` path (`~/.claude/settings.json`).
pub fn settings_path() -> PathBuf {
    let home = dirs::home_dir().expect("Could not determine home directory");
    home.join(".claude").join("settings.json")
}

/// Get the daemon socket path (`~/.claude/bark.sock`).
pub fn socket_path() -> PathBuf {
    let home = dirs::home_dir().expect("Could not determine home directory");
    home.join(".claude").join("bark.sock")
}

/// Get the daemon PID file path (`~/.claude/bark.pid`).
pub fn pid_path() -> PathBuf {
    let home = dirs::home_dir().expect("Could not determine home directory");
    home.join(".claude").join("bark.pid")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_paths_are_under_home() {
        let home = dirs::home_dir().unwrap();
        assert!(bark_toml_path().starts_with(&home));
        assert!(settings_path().starts_with(&home));
        assert!(socket_path().starts_with(&home));
        assert!(pid_path().starts_with(&home));
    }

    #[test]
    fn test_socket_path_ends_with_sock() {
        let path = socket_path();
        assert_eq!(path.file_name().unwrap(), "bark.sock");
    }

    #[test]
    fn test_pid_path_ends_with_pid() {
        let path = pid_path();
        assert_eq!(path.file_name().unwrap(), "bark.pid");
    }
}
