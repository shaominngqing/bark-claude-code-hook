/// Commands that perform destructive or irreversible operations.
pub const DESTRUCTIVE_COMMANDS: &[&str] = &[
    "rm",
    "rmdir",
    "mkfs",
    "dd",
    "shred",
    "wipefs",
    "> /dev/sda",
];

/// Programs that can execute arbitrary code from stdin/arguments.
pub const REMOTE_EXECUTION_SINKS: &[&str] = &[
    "bash", "sh", "zsh", "eval", "source", "python", "perl", "ruby", "node",
];

/// Commands that fetch content from remote URLs.
pub const REMOTE_FETCH_COMMANDS: &[&str] = &["curl", "wget", "fetch"];

/// File paths that contain sensitive data (credentials, keys, configs).
pub const SENSITIVE_PATHS: &[&str] = &[
    "/etc/passwd",
    "/etc/shadow",
    ".ssh/",
    "id_rsa",
    "authorized_keys",
    ".env",
    ".pem",
    ".key",
];

/// Returns `true` if the given command name is a known destructive command.
pub fn is_destructive(cmd: &str) -> bool {
    let base = cmd
        .rsplit('/')
        .next()
        .unwrap_or(cmd)
        .trim();
    DESTRUCTIVE_COMMANDS.iter().any(|&d| base == d)
}

/// Returns `true` if the given command name is a known remote-fetch command.
pub fn is_remote_fetch(cmd: &str) -> bool {
    let base = cmd
        .rsplit('/')
        .next()
        .unwrap_or(cmd)
        .trim();
    REMOTE_FETCH_COMMANDS.iter().any(|&r| base == r)
}

/// Returns `true` if the given command name is a known execution sink.
pub fn is_execution_sink(cmd: &str) -> bool {
    let base = cmd
        .rsplit('/')
        .next()
        .unwrap_or(cmd)
        .trim();
    REMOTE_EXECUTION_SINKS.iter().any(|&s| base == s)
}

/// Returns `true` if the given path contains or matches a sensitive path pattern.
pub fn is_sensitive_path(path: &str) -> bool {
    SENSITIVE_PATHS.iter().any(|&s| path.contains(s))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_destructive() {
        assert!(is_destructive("rm"));
        assert!(is_destructive("/usr/bin/rm"));
        assert!(!is_destructive("ls"));
        assert!(!is_destructive("cat"));
        assert!(is_destructive("shred"));
    }

    #[test]
    fn test_is_remote_fetch() {
        assert!(is_remote_fetch("curl"));
        assert!(is_remote_fetch("wget"));
        assert!(is_remote_fetch("/usr/bin/curl"));
        assert!(!is_remote_fetch("cat"));
    }

    #[test]
    fn test_is_execution_sink() {
        assert!(is_execution_sink("bash"));
        assert!(is_execution_sink("sh"));
        assert!(is_execution_sink("python"));
        assert!(is_execution_sink("/bin/bash"));
        assert!(!is_execution_sink("cat"));
    }

    #[test]
    fn test_is_sensitive_path() {
        assert!(is_sensitive_path("/etc/passwd"));
        assert!(is_sensitive_path("/home/user/.ssh/id_rsa"));
        assert!(is_sensitive_path("/app/.env"));
        assert!(is_sensitive_path("server.pem"));
        assert!(!is_sensitive_path("/tmp/foo.txt"));
    }
}
