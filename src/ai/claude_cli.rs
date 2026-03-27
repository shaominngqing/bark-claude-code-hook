//! AI assessment via `claude` CLI (Claude Code's built-in command).
//!
//! This is the fallback when no `ANTHROPIC_API_KEY` is set. It works
//! exactly like the original Bash version: fork `claude -p` with a
//! system prompt and parse the JSON response.

use std::process::Command;
use std::time::Instant;

use anyhow::{bail, Context, Result};

use crate::ai::prompt;
use crate::core::chain_tracker::ChainContext;
use crate::core::protocol::HookInput;
use crate::core::risk::{Assessment, AssessmentSource};
use crate::i18n::Locale;

/// AI assessor that shells out to the `claude` CLI.
pub struct ClaudeCliClient;

impl ClaudeCliClient {
    pub fn new(_timeout_seconds: u64) -> Self {
        Self
    }

    /// Check if the `claude` CLI is available in PATH.
    pub fn is_available() -> bool {
        Command::new("claude")
            .arg("--version")
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
    }

    /// Assess risk by calling `claude -p --no-session-persistence`.
    ///
    /// This mirrors the original Bash implementation exactly:
    /// ```bash
    /// env -u CLAUDECODE claude -p --no-session-persistence \
    ///   --system-prompt "..." "Assess risk level..."
    /// ```
    pub fn assess(
        &self,
        input: &HookInput,
        chain_context: &ChainContext,
    ) -> Result<Assessment> {
        let start = Instant::now();

        let locale = Locale::detect();
        let system_prompt = prompt::build_system_prompt(&locale);
        let user_prompt = prompt::build_user_prompt_from_chain(input, chain_context);

        // Unset CLAUDECODE to avoid recursion (same as Bash: env -u CLAUDECODE)
        let output = Command::new("claude")
            .arg("-p")
            .arg("--no-session-persistence")
            .arg("--system-prompt")
            .arg(&system_prompt)
            .arg(&user_prompt)
            .env_remove("CLAUDECODE")
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .output()
            .context("failed to execute `claude` CLI")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            bail!("claude CLI exited with {}: {}", output.status, stderr.chars().take(200).collect::<String>());
        }

        let stdout = String::from_utf8_lossy(&output.stdout);

        // Extract JSON from response (handle markdown wrapping)
        let json_str = extract_json(&stdout)
            .context("no JSON found in claude CLI output")?;

        let json: serde_json::Value = serde_json::from_str(&json_str)
            .context("failed to parse JSON from claude CLI")?;

        let level = json
            .get("level")
            .and_then(|v| v.as_u64())
            .map(|v| crate::core::risk::RiskLevel::from_u8(v as u8))
            .unwrap_or(crate::core::risk::RiskLevel::Medium);

        let reason = json
            .get("reason")
            .and_then(|v| v.as_str())
            .unwrap_or("AI assessment")
            .to_string();

        Ok(Assessment {
            level,
            reason,
            source: AssessmentSource::AI,
            duration: start.elapsed(),
        })
    }
}

/// Extract the first JSON object `{...}` from a string.
/// Handles markdown code blocks wrapping.
fn extract_json(text: &str) -> Option<String> {
    // Find first { and matching }
    let start = text.find('{')?;
    let sub = &text[start..];
    let mut depth = 0;
    for (i, ch) in sub.char_indices() {
        match ch {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    return Some(sub[..=i].to_string());
                }
            }
            _ => {}
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_json_clean() {
        let input = r#"{"level":0,"reason":"safe"}"#;
        assert_eq!(extract_json(input), Some(r#"{"level":0,"reason":"safe"}"#.to_string()));
    }

    #[test]
    fn test_extract_json_with_markdown() {
        let input = "```json\n{\"level\":2,\"reason\":\"danger\"}\n```";
        assert_eq!(extract_json(input), Some(r#"{"level":2,"reason":"danger"}"#.to_string()));
    }

    #[test]
    fn test_extract_json_with_text() {
        let input = "Here is the result: {\"level\":1,\"reason\":\"test\"} done.";
        assert_eq!(extract_json(input), Some(r#"{"level":1,"reason":"test"}"#.to_string()));
    }

    #[test]
    fn test_extract_json_none() {
        assert_eq!(extract_json("no json here"), None);
    }

    #[test]
    fn test_is_available() {
        // Just ensure it doesn't panic — whether claude is installed varies
        let _ = ClaudeCliClient::is_available();
    }
}
