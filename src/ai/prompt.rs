use crate::core::chain_tracker::ChainContext;
use crate::core::protocol::HookInput;
use crate::i18n::Locale;

/// Build the system prompt that constrains the AI to JSON-only output.
pub fn build_system_prompt(locale: &Locale) -> String {
    let lang_hint = locale.prompt_hint();
    format!(
        "You are a JSON-only risk assessment API. \
         You MUST output exactly one JSON object per request. \
         No markdown, no explanation, no conversation. \
         Output format: {{\"level\":<0|1|2>,\"reason\":\"<10 words max [{lang_hint}]>\"}}"
    )
}

/// Build the user prompt describing the operation to assess,
/// using the chain tracker's `ChainContext` for session context.
pub fn build_user_prompt_from_chain(input: &HookInput, chain: &ChainContext) -> String {
    let operation = describe_input(input);

    let mut prompt = format!(
        "Assess risk level of this dev operation:\n\
         0=safe(read-only, builds, tests, normal edits)\n\
         1=medium(side effects but recoverable: pkg install, git push, mv, config edits)\n\
         2=high(destructive/irreversible: force push, rm -rf critical dirs, secrets, remote exec, DB drops, sudo)\n\
         \n\
         Be practical: rm -rf node_modules=1, npm install=1, git push=1. Only truly dangerous=2.\n\
         \n\
         Operation: {operation}"
    );

    // Add chain context if there are recent commands
    if !chain.recent_commands.is_empty() {
        prompt.push_str(&format!(
            "\n\nRecent session context ({} operations): {}",
            chain.operation_count,
            chain.recent_commands.join("; ")
        ));

        // Add suspicious patterns if detected
        if !chain.suspicious_patterns.is_empty() {
            let patterns: Vec<String> = chain
                .suspicious_patterns
                .iter()
                .map(|p| p.to_string())
                .collect();
            prompt.push_str(&format!(
                "\nWARNING: Suspicious patterns detected: {}",
                patterns.join(", ")
            ));
        }
    }

    prompt
}

/// Convert a `HookInput` into a human-readable operation description for the prompt.
fn describe_input(input: &HookInput) -> String {
    match input.tool_name.as_str() {
        "Bash" => {
            if let Some(cmd) = input.command() {
                format!("Bash command: {}", cmd)
            } else {
                "Bash command: (unknown)".to_string()
            }
        }
        "Edit" => {
            let path = input.file_path().unwrap_or("(unknown)");
            let old = input.old_string().unwrap_or("");
            let new = input.new_string().unwrap_or("");
            format!(
                "Edit file: {} (replacing '{}' with '{}')",
                path,
                truncate(old, 80),
                truncate(new, 80)
            )
        }
        "Write" => {
            let path = input.file_path().unwrap_or("(unknown)");
            let content = input.content().unwrap_or("");
            format!("Write file: {} ({} bytes)", path, content.len())
        }
        other => {
            format!(
                "Tool '{}' with input: {}",
                other,
                truncate(&input.tool_input.to_string(), 200)
            )
        }
    }
}

/// Truncate a string to at most `max_len` characters, appending "..." if truncated.
fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_system_prompt_english() {
        let prompt = build_system_prompt(&Locale::En);
        assert!(prompt.contains("JSON-only"));
        assert!(prompt.contains("in English"));
        assert!(prompt.contains("\"level\""));
    }

    #[test]
    fn test_system_prompt_chinese() {
        let prompt = build_system_prompt(&Locale::Zh);
        assert!(prompt.contains("in Chinese"));
    }

    #[test]
    fn test_user_prompt_bash() {
        let input = HookInput {
            tool_name: "Bash".to_string(),
            tool_input: serde_json::json!({"command": "rm -rf /tmp/test"}),
        };
        let chain = ChainContext::default();
        let prompt = build_user_prompt_from_chain(&input, &chain);
        assert!(prompt.contains("rm -rf /tmp/test"));
        assert!(prompt.contains("0=safe"));
    }

    #[test]
    fn test_user_prompt_with_chain_context() {
        let input = HookInput {
            tool_name: "Bash".to_string(),
            tool_input: serde_json::json!({"command": "ls"}),
        };
        let chain = ChainContext {
            recent_commands: vec![
                "Bash(git status)".to_string(),
                "Edit(src/main.rs)".to_string(),
            ],
            suspicious_patterns: Vec::new(),
            session_risk_trend: crate::core::chain_tracker::RiskTrend::Stable,
            operation_count: 5,
        };
        let prompt = build_user_prompt_from_chain(&input, &chain);
        assert!(prompt.contains("Recent session context"));
        assert!(prompt.contains("5 operations"));
    }

    #[test]
    fn test_truncate() {
        assert_eq!(truncate("hello", 10), "hello");
        assert_eq!(truncate("hello world", 5), "hello...");
    }
}
