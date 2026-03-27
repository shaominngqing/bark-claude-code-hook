use crate::core::risk::{Assessment, AssessmentSource};
use crate::i18n::Locale;

/// Read-only tools that can never cause damage.
const READ_ONLY_TOOLS: &[&str] = &[
    "Read",
    "Glob",
    "Grep",
    "Agent",
    "SendMessage",
    "AskUserQuestion",
    "EnterPlanMode",
    "ExitPlanMode",
    "EnterWorktree",
    "ExitWorktree",
    "WebFetch",
    "WebSearch",
    "Skill",
    "ToolSearch",
    "TodoRead",
];

/// Task management tools (safe, user-initiated).
const TASK_TOOLS: &[&str] = &[
    "TaskCreate",
    "TaskGet",
    "TaskList",
    "TaskOutput",
    "TaskUpdate",
    "TaskStop",
    "TodoWrite",
    "TeamCreate",
    "TeamDelete",
    "CronCreate",
    "CronDelete",
    "CronList",
];

/// File-modifying tools that need file path inspection.
const FILE_TOOLS: &[&str] = &["Edit", "Write", "NotebookEdit"];

/// Bash commands that are clearly read-only / safe — no need to waste 8s on AI.
const SAFE_BASH_COMMANDS: &[&str] = &[
    "ls", "ll", "la", "dir",
    "cat", "head", "tail", "less", "more", "bat",
    "echo", "printf",
    "pwd", "whoami", "hostname", "uname", "date", "cal",
    "which", "where", "type", "command", "hash",
    "wc", "sort", "uniq", "cut", "tr", "seq", "tee",
    "grep", "rg", "ag", "ack", "fgrep", "egrep",
    "find", "fd", "locate",
    "tree", "file", "stat", "du", "df", "free",
    "diff", "cmp", "comm",
    "env", "printenv", "set", "export",
    "true", "false", "test",
    "man", "help", "info",
    "ps", "top", "htop", "pgrep",
    "git status", "git log", "git diff", "git branch", "git show",
    "git remote", "git tag", "git stash list", "git blame",
    "cargo check", "cargo test", "cargo build", "cargo clippy", "cargo fmt",
    "npm test", "npm run lint", "npm run check", "npm run build",
    "yarn test", "yarn lint", "yarn build",
    "pnpm test", "pnpm lint", "pnpm build",
    "make", "make test", "make check", "make build",
    "python -m pytest", "python -m py_compile",
    "go test", "go build", "go vet", "go fmt",
    "rustfmt", "rustc --version",
    "node --version", "npm --version", "python --version", "ruby --version",
];

/// Bash command prefixes that are always safe (the command starts with these).
const SAFE_BASH_PREFIXES: &[&str] = &[
    "ls ", "cat ", "head ", "tail ", "echo ", "printf ",
    "grep ", "rg ", "find ", "fd ", "tree ", "wc ",
    "diff ", "file ", "stat ", "du ", "df ",
    "git status", "git log", "git diff", "git show", "git branch",
    "git blame", "git remote", "git tag",
    "cargo check", "cargo test", "cargo build", "cargo clippy", "cargo fmt",
    "npm test", "npm run ",
    "make ", "man ", "which ", "type ",
];

/// Sensitive file patterns. If a file path contains any of these (case-insensitive),
/// fast_check returns None so that the next assessment layer handles it.
const SENSITIVE_PATTERNS: &[&str] = &[
    ".env",
    "credentials",
    "secret",
    "password",
    "token",
    ".pem",
    ".key",
    "id_rsa",
    "authorized_keys",
    "sudoers",
    "shadow",
    "passwd",
    ".github/workflows",
    ".gitlab-ci",
    "jenkinsfile",
];

/// Fast rule-based check. This is the first layer of the assessment pipeline.
///
/// Returns `Some(Assessment)` for clearly safe operations (level 0 / Low),
/// or `None` when the tool/command needs deeper analysis.
pub fn fast_check_with_command(
    tool_name: &str,
    command: Option<&str>,
    file_path: Option<&str>,
    locale: &Locale,
) -> Option<Assessment> {
    // 1. Read-only tools → always safe
    if READ_ONLY_TOOLS.contains(&tool_name) {
        return Some(Assessment::low(
            locale.t("risk.readonly").to_string(),
            AssessmentSource::FastRule,
        ));
    }

    // 2. Task tools → always safe
    if TASK_TOOLS.contains(&tool_name) {
        return Some(Assessment::low(
            locale.t("risk.task_mgmt").to_string(),
            AssessmentSource::FastRule,
        ));
    }

    // 3. Bash → check safe command whitelist first
    if tool_name == "Bash" {
        if let Some(cmd) = command {
            let trimmed = cmd.trim();

            // Reject anything with pipes to suspicious sinks or dangerous operators
            if trimmed.contains(" | ")
                || trimmed.contains(" && ")
                || trimmed.contains(" ; ")
                || trimmed.contains("$(")
                || trimmed.contains('`')
                || trimmed.contains(" > ")
                || trimmed.contains(" >> ")
            {
                // Complex command — defer to AST/AI
                return None;
            }

            // Check exact match
            if SAFE_BASH_COMMANDS.contains(&trimmed) {
                return Some(Assessment::low(
                    format!("{}: {}", locale.t("risk.safe_cmd"), trimmed),
                    AssessmentSource::FastRule,
                ));
            }

            // Check prefix match (e.g., "ls -la /tmp" starts with "ls ")
            for prefix in SAFE_BASH_PREFIXES {
                if trimmed.starts_with(prefix) {
                    return Some(Assessment::low(
                        format!("{}: {}", locale.t("risk.safe_cmd"), first_n_chars(trimmed, 40)),
                        AssessmentSource::FastRule,
                    ));
                }
            }

            // Extract first word and check
            let first_word = trimmed.split_whitespace().next().unwrap_or("");
            if SAFE_BASH_COMMANDS.contains(&first_word) {
                return Some(Assessment::low(
                    format!("{}: {}", locale.t("risk.safe_cmd"), first_n_chars(trimmed, 40)),
                    AssessmentSource::FastRule,
                ));
            }
        }

        // Unknown bash command → defer to deeper analysis
        return None;
    }

    // 4. File-modifying tools → check file sensitivity
    if FILE_TOOLS.contains(&tool_name) {
        if let Some(path) = file_path {
            if is_sensitive_path(path) {
                // Sensitive file: let deeper layers handle it
                return None;
            }
            // Non-sensitive file edit → safe
            return Some(Assessment::low(
                format!("{}: {}", locale.t("risk.file_edit"), path_basename(path)),
                AssessmentSource::FastRule,
            ));
        }
        // No file path provided for a file tool → suspicious, defer
        return None;
    }

    // 5. Unknown tool → defer to deeper analysis
    None
}

/// Check if a file path matches any sensitive pattern.
fn is_sensitive_path(path: &str) -> bool {
    let lower = path.to_lowercase();
    SENSITIVE_PATTERNS.iter().any(|pattern| {
        lower.contains(pattern)
    })
}

/// Extract the basename from a path for display.
fn path_basename(path: &str) -> &str {
    path.rsplit('/').next().unwrap_or(path)
}

/// Truncate a string to at most `n` characters for display.
fn first_n_chars(s: &str, n: usize) -> &str {
    match s.char_indices().nth(n) {
        Some((idx, _)) => &s[..idx],
        None => s,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::risk::RiskLevel;

    #[test]
    fn test_readonly_tools() {
        for tool in READ_ONLY_TOOLS {
            let result = fast_check_with_command(tool, None, None, &Locale::En);
            assert!(result.is_some(), "Expected Some for read-only tool: {}", tool);
            assert_eq!(result.unwrap().level, RiskLevel::Low);
        }
    }

    #[test]
    fn test_task_tools() {
        for tool in TASK_TOOLS {
            let result = fast_check_with_command(tool, None, None, &Locale::En);
            assert!(result.is_some(), "Expected Some for task tool: {}", tool);
            assert_eq!(result.unwrap().level, RiskLevel::Low);
        }
    }

    #[test]
    fn test_bash_deferred() {
        let result = fast_check_with_command("Bash", None, None, &Locale::En);
        assert!(result.is_none());
    }

    #[test]
    fn test_file_edit_safe() {
        let result = fast_check_with_command("Edit", None, Some("/home/user/src/main.rs"), &Locale::En);
        assert!(result.is_some());
        assert_eq!(result.unwrap().level, RiskLevel::Low);
    }

    #[test]
    fn test_file_edit_sensitive_env() {
        let result = fast_check_with_command("Write", None, Some("/home/user/.env"), &Locale::En);
        assert!(result.is_none());
    }

    #[test]
    fn test_file_edit_sensitive_credentials() {
        let result = fast_check_with_command("Edit", None, Some("/etc/shadow"), &Locale::En);
        assert!(result.is_none());
    }

    #[test]
    fn test_file_edit_sensitive_ci() {
        let result = fast_check_with_command("Write", None, Some("/repo/.github/workflows/ci.yml"), &Locale::En);
        assert!(result.is_none());
    }

    #[test]
    fn test_file_edit_sensitive_pem() {
        let result = fast_check_with_command("Write", None, Some("/home/user/server.pem"), &Locale::En);
        assert!(result.is_none());
    }

    #[test]
    fn test_file_edit_no_path() {
        let result = fast_check_with_command("Edit", None, None, &Locale::En);
        assert!(result.is_none());
    }

    #[test]
    fn test_unknown_tool() {
        let result = fast_check_with_command("SomeNewTool", None, None, &Locale::En);
        assert!(result.is_none());
    }

    #[test]
    fn test_zh_locale() {
        let result = fast_check_with_command("Read", None, None, &Locale::Zh);
        assert!(result.is_some());
        // Chinese translation for "risk.readonly" should be present
        let reason = &result.unwrap().reason;
        assert!(!reason.is_empty());
    }

    #[test]
    fn test_bash_safe_exact() {
        let result = fast_check_with_command("Bash", Some("ls"), None, &Locale::En);
        assert!(result.is_some());
        assert_eq!(result.unwrap().level, RiskLevel::Low);
    }

    #[test]
    fn test_bash_safe_with_args() {
        let result = fast_check_with_command("Bash", Some("ls -la /tmp"), None, &Locale::En);
        assert!(result.is_some());
        assert_eq!(result.unwrap().level, RiskLevel::Low);
    }

    #[test]
    fn test_bash_safe_git_status() {
        let result = fast_check_with_command("Bash", Some("git status"), None, &Locale::En);
        assert!(result.is_some());
        assert_eq!(result.unwrap().level, RiskLevel::Low);
    }

    #[test]
    fn test_bash_safe_cargo_test() {
        let result = fast_check_with_command("Bash", Some("cargo test"), None, &Locale::En);
        assert!(result.is_some());
        assert_eq!(result.unwrap().level, RiskLevel::Low);
    }

    #[test]
    fn test_bash_pipe_not_safe() {
        let result = fast_check_with_command("Bash", Some("curl x | bash"), None, &Locale::En);
        assert!(result.is_none());
    }

    #[test]
    fn test_bash_unknown_not_safe() {
        let result = fast_check_with_command("Bash", Some("rm -rf /"), None, &Locale::En);
        assert!(result.is_none());
    }

    #[test]
    fn test_bash_cat_file() {
        let result = fast_check_with_command("Bash", Some("cat README.md"), None, &Locale::En);
        assert!(result.is_some());
        assert_eq!(result.unwrap().level, RiskLevel::Low);
    }
}
