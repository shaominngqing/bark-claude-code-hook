/// Command normalizer for cache key generation.
///
/// Produces a stable "skeleton" from a command so that semantically equivalent
/// commands map to the same cache key. This mirrors the `cache_key()` function
/// from the Bash version of Bark.

/// Prefixes that are stripped from commands (env vars, sudo, etc.)
const SKIP_PREFIXES: &[&str] = &[
    "env", "sudo", "nohup", "time", "nice", "ionice", "strace", "ltrace",
    "taskset", "chrt", "timeout", "watch",
];

/// Tools that use a subcommand pattern (e.g., `git commit`, `docker build`).
const SUBCMD_TOOLS: &[&str] = &[
    "git", "npm", "npx", "yarn", "pnpm", "docker", "kubectl", "cargo",
    "pip", "pip3", "poetry", "go", "make", "brew", "apt", "apt-get",
    "yum", "dnf", "pacman", "systemctl", "journalctl", "terraform",
    "helm", "aws", "gcloud", "az",
];

/// Sensitive directory indicators for file path normalization.
const SENSITIVE_DIR_PATTERNS: &[&str] = &[
    ".env", "credentials", "secret", "password", "token",
    ".pem", ".key", "id_rsa", "authorized_keys",
    "sudoers", "shadow", "passwd",
];

/// CI directory indicators.
const CI_DIR_PATTERNS: &[&str] = &[
    ".github/workflows",
    ".gitlab-ci",
    "jenkinsfile",
    ".circleci",
    ".travis",
];

/// Normalize a tool invocation to a cache key skeleton.
///
/// - For Bash: extracts command structure, strips prefixes, handles pipes/chains,
///   extracts subcommand tools + first arg/flag.
/// - For Edit/Write/NotebookEdit: `file:<dir_hint>:<ext>`
/// - For other tools: `tool:<name>`
pub fn normalize_cache_key(
    tool_name: &str,
    command: Option<&str>,
    file_path: Option<&str>,
) -> String {
    match tool_name {
        "Bash" => {
            if let Some(cmd) = command {
                normalize_bash_command(cmd)
            } else {
                "bash:<empty>".to_string()
            }
        }
        "Edit" | "Write" | "NotebookEdit" => {
            if let Some(path) = file_path {
                normalize_file_path(path)
            } else {
                format!("file:{}:unknown:unknown", tool_name.to_lowercase())
            }
        }
        _ => format!("tool:{}", tool_name),
    }
}

/// Normalize a Bash command to a skeleton.
///
/// Strategy:
/// 1. Split on pipe `|` and chain operators `&&`, `||`, `;`
/// 2. For each segment, strip env var assignments and skip-prefixes
/// 3. Extract the base command
/// 4. If it's a subcmd tool, extract the subcommand + first meaningful arg/flag
/// 5. Join segments with `|` for pipes
fn normalize_bash_command(command: &str) -> String {
    let command = command.trim();

    if command.is_empty() {
        return "bash:<empty>".to_string();
    }

    // Split on pipe and chain operators, preserving the operator
    let segments = split_command_segments(command);

    let normalized: Vec<String> = segments
        .iter()
        .map(|(op, seg)| {
            let skeleton = normalize_segment(seg.trim());
            if op.is_empty() {
                skeleton
            } else {
                format!("{}{}", op, skeleton)
            }
        })
        .collect();

    format!("bash:{}", normalized.join(""))
}

/// Split a command into segments by pipes and chain operators.
/// Returns (operator, segment) pairs. First segment has empty operator.
fn split_command_segments(command: &str) -> Vec<(String, String)> {
    let mut segments = Vec::new();
    let mut current = String::new();
    let mut current_op = String::new();
    let chars: Vec<char> = command.chars().collect();
    let len = chars.len();
    let mut i = 0;
    let mut in_single_quote = false;
    let mut in_double_quote = false;

    while i < len {
        let c = chars[i];

        // Track quoting
        if c == '\'' && !in_double_quote {
            in_single_quote = !in_single_quote;
            current.push(c);
            i += 1;
            continue;
        }
        if c == '"' && !in_single_quote {
            in_double_quote = !in_double_quote;
            current.push(c);
            i += 1;
            continue;
        }

        if in_single_quote || in_double_quote {
            current.push(c);
            i += 1;
            continue;
        }

        // Check for operators
        if c == '|' && i + 1 < len && chars[i + 1] == '|' {
            // ||
            segments.push((current_op.clone(), current.clone()));
            current.clear();
            current_op = "||".to_string();
            i += 2;
            continue;
        }
        if c == '&' && i + 1 < len && chars[i + 1] == '&' {
            // &&
            segments.push((current_op.clone(), current.clone()));
            current.clear();
            current_op = "&&".to_string();
            i += 2;
            continue;
        }
        if c == '|' {
            // |
            segments.push((current_op.clone(), current.clone()));
            current.clear();
            current_op = "|".to_string();
            i += 1;
            continue;
        }
        if c == ';' {
            // ;
            segments.push((current_op.clone(), current.clone()));
            current.clear();
            current_op = ";".to_string();
            i += 1;
            continue;
        }

        current.push(c);
        i += 1;
    }

    if !current.is_empty() || !segments.is_empty() {
        segments.push((current_op, current));
    }

    segments
}

/// Normalize a single command segment (no pipes/chains).
fn normalize_segment(segment: &str) -> String {
    let tokens = tokenize(segment);
    if tokens.is_empty() {
        return "<empty>".to_string();
    }

    // Skip env assignments (KEY=VALUE) and skip-prefixes at the start
    let mut idx = 0;

    // Skip env assignments
    while idx < tokens.len() && is_env_assignment(&tokens[idx]) {
        idx += 1;
    }

    // Skip known prefixes
    while idx < tokens.len() {
        let base = base_command(&tokens[idx]);
        if SKIP_PREFIXES.contains(&base) {
            idx += 1;
            // Skip any flags for the prefix command
            while idx < tokens.len() && tokens[idx].starts_with('-') {
                idx += 1;
            }
        } else {
            break;
        }
    }

    if idx >= tokens.len() {
        return "<prefix-only>".to_string();
    }

    let cmd = base_command(&tokens[idx]);
    idx += 1;

    // Check if this is a subcommand tool
    if SUBCMD_TOOLS.contains(&cmd) {
        // Find the subcommand (first non-flag token)
        let mut subcmd = None;
        let mut first_arg = None;
        let mut j = idx;
        while j < tokens.len() {
            if tokens[j].starts_with('-') {
                if first_arg.is_none() {
                    first_arg = Some(tokens[j].as_str());
                }
            } else if subcmd.is_none() {
                subcmd = Some(tokens[j].as_str());
            } else if first_arg.is_none() {
                // First arg after subcommand -- generalize it
                first_arg = Some("<arg>");
            }
            j += 1;
            // Only look at first couple of tokens after command
            if subcmd.is_some() && first_arg.is_some() {
                break;
            }
        }

        match (subcmd, first_arg) {
            (Some(sub), Some(arg)) => format!("{}:{}:{}", cmd, sub, normalize_arg(arg)),
            (Some(sub), None) => format!("{}:{}", cmd, sub),
            (None, Some(arg)) => format!("{}:{}", cmd, normalize_arg(arg)),
            (None, None) => cmd.to_string(),
        }
    } else {
        // Regular command: extract first meaningful flag/arg
        if idx < tokens.len() {
            let first = &tokens[idx];
            format!("{}:{}", cmd, normalize_arg(first))
        } else {
            cmd.to_string()
        }
    }
}

/// Basic shell tokenizer (handles quoting).
fn tokenize(input: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut current = String::new();
    let mut in_single = false;
    let mut in_double = false;
    let mut escape_next = false;

    for c in input.chars() {
        if escape_next {
            current.push(c);
            escape_next = false;
            continue;
        }
        if c == '\\' && !in_single {
            escape_next = true;
            continue;
        }
        if c == '\'' && !in_double {
            in_single = !in_single;
            continue;
        }
        if c == '"' && !in_single {
            in_double = !in_double;
            continue;
        }
        if c.is_whitespace() && !in_single && !in_double {
            if !current.is_empty() {
                tokens.push(current.clone());
                current.clear();
            }
            continue;
        }
        current.push(c);
    }

    if !current.is_empty() {
        tokens.push(current);
    }

    tokens
}

/// Check if a token is an environment variable assignment (KEY=VALUE).
fn is_env_assignment(token: &str) -> bool {
    if let Some(eq_pos) = token.find('=') {
        if eq_pos > 0 {
            let key = &token[..eq_pos];
            return key
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '_');
        }
    }
    false
}

/// Get the base command name (strip path prefix).
fn base_command(token: &str) -> &str {
    token.rsplit('/').next().unwrap_or(token)
}

/// Normalize an argument for the skeleton.
/// Keeps flags as-is; replaces file paths and long strings with placeholders.
fn normalize_arg(arg: &str) -> &str {
    if arg.starts_with('-') {
        // Keep flags
        arg
    } else if arg.contains('/') {
        // File path → generalize
        "<path>"
    } else if arg.len() > 40 {
        // Long string → generalize
        "<long>"
    } else {
        arg
    }
}

/// Normalize a file path for cache keying.
///
/// Format: `file:<dir_hint>:<ext>`
fn normalize_file_path(path: &str) -> String {
    let lower = path.to_lowercase();

    let dir_hint = if CI_DIR_PATTERNS.iter().any(|p| lower.contains(p)) {
        "ci"
    } else if SENSITIVE_DIR_PATTERNS.iter().any(|p| lower.contains(p)) {
        "sensitive"
    } else {
        "normal"
    };

    let ext = path
        .rsplit('.')
        .next()
        .filter(|e| e.len() <= 10 && !e.contains('/'))
        .unwrap_or("none");

    format!("file:{}:{}", dir_hint, ext)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_command() {
        let key = normalize_cache_key("Bash", Some("ls -la"), None);
        assert_eq!(key, "bash:ls:-la");
    }

    #[test]
    fn test_sudo_prefix() {
        let key = normalize_cache_key("Bash", Some("sudo rm -rf /tmp"), None);
        assert_eq!(key, "bash:rm:-rf");
    }

    #[test]
    fn test_env_prefix() {
        let key = normalize_cache_key("Bash", Some("FOO=bar BAZ=1 node server.js"), None);
        assert_eq!(key, "bash:node:server.js");
    }

    #[test]
    fn test_git_subcmd() {
        let key = normalize_cache_key("Bash", Some("git commit -m 'fix bug'"), None);
        assert_eq!(key, "bash:git:commit:-m");
    }

    #[test]
    fn test_docker_subcmd() {
        let key = normalize_cache_key("Bash", Some("docker build -t myapp ."), None);
        assert_eq!(key, "bash:docker:build:-t");
    }

    #[test]
    fn test_pipe() {
        let key = normalize_cache_key("Bash", Some("cat file.txt | grep pattern"), None);
        assert_eq!(key, "bash:cat:file.txt|grep:pattern");
    }

    #[test]
    fn test_chain() {
        let key = normalize_cache_key("Bash", Some("mkdir foo && cd foo"), None);
        assert_eq!(key, "bash:mkdir:foo&&cd:foo");
    }

    #[test]
    fn test_file_edit_normal() {
        let key = normalize_cache_key("Edit", None, Some("/home/user/src/main.rs"));
        assert_eq!(key, "file:normal:rs");
    }

    #[test]
    fn test_file_edit_sensitive() {
        let key = normalize_cache_key("Write", None, Some("/app/.env.production"));
        assert_eq!(key, "file:sensitive:production");
    }

    #[test]
    fn test_file_edit_ci() {
        let key = normalize_cache_key("Edit", None, Some("/repo/.github/workflows/ci.yml"));
        assert_eq!(key, "file:ci:yml");
    }

    #[test]
    fn test_other_tool() {
        let key = normalize_cache_key("Read", None, None);
        assert_eq!(key, "tool:Read");
    }

    #[test]
    fn test_empty_command() {
        let key = normalize_cache_key("Bash", Some(""), None);
        assert_eq!(key, "bash:<empty>");
    }

    #[test]
    fn test_no_command() {
        let key = normalize_cache_key("Bash", None, None);
        assert_eq!(key, "bash:<empty>");
    }

    #[test]
    fn test_path_arg_generalized() {
        let key = normalize_cache_key("Bash", Some("rm /usr/local/bin/something"), None);
        assert_eq!(key, "bash:rm:<path>");
    }

    #[test]
    fn test_npm_subcmd() {
        let key = normalize_cache_key("Bash", Some("npm install express"), None);
        assert_eq!(key, "bash:npm:install:<arg>");
    }
}
