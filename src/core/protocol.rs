
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::core::risk::{Assessment, RiskLevel};
use crate::i18n::Locale;

/// Raw hook input from Claude Code, matching the PreToolUse hook protocol.
///
/// Example JSON:
/// ```json
/// {"tool_name": "Bash", "tool_input": {"command": "rm -rf /"}}
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HookInput {
    pub tool_name: String,
    pub tool_input: Value,
}

impl HookInput {
    /// Extract the `command` field from `tool_input` (for Bash tools).
    pub fn command(&self) -> Option<&str> {
        self.tool_input.get("command").and_then(|v| v.as_str())
    }

    /// Extract the `file_path` field from `tool_input` (for Edit/Write/NotebookEdit).
    pub fn file_path(&self) -> Option<&str> {
        self.tool_input
            .get("file_path")
            .and_then(|v| v.as_str())
    }

    /// Extract the `content` field from `tool_input` (for Write).
    pub fn content(&self) -> Option<&str> {
        self.tool_input.get("content").and_then(|v| v.as_str())
    }

    /// Extract the `old_string` field from `tool_input` (for Edit).
    pub fn old_string(&self) -> Option<&str> {
        self.tool_input.get("old_string").and_then(|v| v.as_str())
    }

    /// Extract the `new_string` field from `tool_input` (for Edit).
    pub fn new_string(&self) -> Option<&str> {
        self.tool_input.get("new_string").and_then(|v| v.as_str())
    }

    /// Parse from stdin JSON. Returns `None` on invalid JSON.
    pub fn from_json(json: &str) -> Option<Self> {
        serde_json::from_str(json).ok()
    }
}

/// Permission decision for the hook output.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum PermissionDecision {
    Allow,
    Ask,
    Deny,
}

impl From<RiskLevel> for PermissionDecision {
    fn from(level: RiskLevel) -> Self {
        // Matches the original Bash bark.sh behavior:
        // Low (0)    → allow  (silent)
        // Medium (1) → allow  (with notification)
        // High (2)   → ask    (notification + confirmation)
        match level {
            RiskLevel::Low => PermissionDecision::Allow,
            RiskLevel::Medium => PermissionDecision::Allow,
            RiskLevel::High => PermissionDecision::Ask,
        }
    }
}

/// Hook-specific output payload.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HookSpecificOutput {
    pub hook_event_name: String,
    pub permission_decision: PermissionDecision,
    pub permission_decision_reason: String,
}

/// The full hook output JSON structure.
///
/// Example output:
/// ```json
/// {
///   "hookSpecificOutput": {
///     "hookEventName": "PreToolUse",
///     "permissionDecision": "allow",
///     "permissionDecisionReason": "Read-only tool"
///   }
/// }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HookOutput {
    pub hook_specific_output: HookSpecificOutput,
}

impl HookOutput {
    /// Build a hook output from an assessment.
    ///
    /// Maps risk levels to permission decisions:
    /// - Low → allow
    /// - Medium → ask
    /// - High → deny
    pub fn from_assessment(assessment: &Assessment, locale: &Locale) -> Self {
        let decision = PermissionDecision::from(assessment.level);

        let reason = format_reason(assessment, locale);

        Self {
            hook_specific_output: HookSpecificOutput {
                hook_event_name: "PreToolUse".to_string(),
                permission_decision: decision,
                permission_decision_reason: reason,
            },
        }
    }

    /// Create an "allow" output with a custom reason.
    pub fn allow_with_reason(reason: impl Into<String>) -> Self {
        Self {
            hook_specific_output: HookSpecificOutput {
                hook_event_name: "PreToolUse".to_string(),
                permission_decision: PermissionDecision::Allow,
                permission_decision_reason: reason.into(),
            },
        }
    }

    /// Serialize to JSON string for stdout output.
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap_or_else(|_| {
            // Fallback: always produce valid JSON even if serialization somehow fails
            r#"{"hookSpecificOutput":{"hookEventName":"PreToolUse","permissionDecision":"ask","permissionDecisionReason":"serialization error"}}"#.to_string()
        })
    }
}

/// Format the reason string with source/duration metadata.
fn format_reason(assessment: &Assessment, locale: &Locale) -> String {
    let source_tag = format!("[{}]", assessment.source);
    let duration_ms = assessment.duration.as_millis();

    let prefix = match locale {
        Locale::Zh => "Bark",
        Locale::En => "Bark",
    };

    if duration_ms > 0 {
        format!(
            "{} {} {}ms | {}",
            prefix, source_tag, duration_ms, assessment.reason
        )
    } else {
        format!("{} {} | {}", prefix, source_tag, assessment.reason)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::risk::{AssessmentSource, RiskLevel};
    use std::time::Duration;

    #[test]
    fn test_hook_input_parse() {
        let json = r#"{"tool_name":"Bash","tool_input":{"command":"ls -la"}}"#;
        let input = HookInput::from_json(json).unwrap();
        assert_eq!(input.tool_name, "Bash");
        assert_eq!(input.command(), Some("ls -la"));
        assert_eq!(input.file_path(), None);
    }

    #[test]
    fn test_hook_input_file_path() {
        let json = r#"{"tool_name":"Edit","tool_input":{"file_path":"/tmp/test.rs","old_string":"foo","new_string":"bar"}}"#;
        let input = HookInput::from_json(json).unwrap();
        assert_eq!(input.tool_name, "Edit");
        assert_eq!(input.file_path(), Some("/tmp/test.rs"));
        assert_eq!(input.command(), None);
    }

    #[test]
    fn test_hook_output_from_assessment() {
        let assessment = Assessment::new(
            RiskLevel::Low,
            "Read-only tool",
            AssessmentSource::FastRule,
        );
        let output = HookOutput::from_assessment(&assessment, &Locale::En);
        assert_eq!(
            output.hook_specific_output.permission_decision,
            PermissionDecision::Allow
        );

        let assessment = Assessment::new(
            RiskLevel::Medium,
            "Unknown command",
            AssessmentSource::AI,
        )
        .with_duration(Duration::from_millis(150));
        // Medium → Allow (with notification, matches original Bash behavior)
        let output = HookOutput::from_assessment(&assessment, &Locale::En);
        assert_eq!(
            output.hook_specific_output.permission_decision,
            PermissionDecision::Allow
        );
        assert!(
            output
                .hook_specific_output
                .permission_decision_reason
                .contains("150ms")
        );

        // High → Ask (notification + confirmation)
        let assessment = Assessment::new(
            RiskLevel::High,
            "Dangerous command",
            AssessmentSource::FastRule,
        );
        let output = HookOutput::from_assessment(&assessment, &Locale::En);
        assert_eq!(
            output.hook_specific_output.permission_decision,
            PermissionDecision::Ask
        );
    }

    #[test]
    fn test_hook_output_json_roundtrip() {
        let output = HookOutput::allow_with_reason("safe");
        let json = output.to_json();
        let parsed: HookOutput = serde_json::from_str(&json).unwrap();
        assert_eq!(
            parsed.hook_specific_output.permission_decision,
            PermissionDecision::Allow
        );
    }
}
