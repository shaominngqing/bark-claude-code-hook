//! Read/write `~/.claude/settings.json` for hook management.
//!
//! The settings file controls which hooks Claude Code invokes.
//! This module provides functions to check, enable, and disable the
//! Bark hook entry.

use anyhow::{Context, Result};
use serde_json::Value;

use crate::config::settings_path;

/// Check if the Bark hook is currently registered in settings.json.
pub fn has_hook() -> bool {
    let path = settings_path();
    let content = match std::fs::read_to_string(&path) {
        Ok(c) => c,
        Err(_) => return false,
    };

    let json: Value = match serde_json::from_str(&content) {
        Ok(v) => v,
        Err(_) => return false,
    };

    find_bark_hook(&json)
}

/// Enable the Bark hook by adding it to settings.json.
///
/// If the file does not exist, it will be created. If the hook is
/// already present, this is a no-op.
pub fn enable_hook() -> Result<()> {
    let path = settings_path();

    // Ensure parent directory exists
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).ok();
    }

    // Read existing settings or start with empty object
    let mut json: Value = if path.exists() {
        let content = std::fs::read_to_string(&path)
            .with_context(|| format!("failed to read {}", path.display()))?;
        serde_json::from_str(&content)
            .with_context(|| format!("failed to parse {}", path.display()))?
    } else {
        Value::Object(serde_json::Map::new())
    };

    // Check if hook already exists
    if find_bark_hook(&json) {
        return Ok(());
    }

    // Build the hook command with absolute path for reliability
    let bark_bin = std::env::current_exe()
        .ok()
        .map(|p| format!("{} hook", p.display()))
        .unwrap_or_else(|| "bark hook".to_string());

    // Claude Code expects this exact format:
    // {"matcher":"*","hooks":[{"type":"command","command":"bark hook","timeout":30}]}
    let hook_entry = serde_json::json!({
        "matcher": "*",
        "hooks": [{
            "type": "command",
            "command": bark_bin,
            "timeout": 30
        }]
    });

    let hooks = json
        .as_object_mut()
        .context("settings.json root is not an object")?
        .entry("hooks")
        .or_insert_with(|| Value::Object(serde_json::Map::new()));

    let pre_tool_use = hooks
        .as_object_mut()
        .context("hooks is not an object")?
        .entry("PreToolUse")
        .or_insert_with(|| Value::Array(Vec::new()));

    let arr = pre_tool_use
        .as_array_mut()
        .context("PreToolUse is not an array")?;

    arr.push(hook_entry);

    // Write back
    let formatted = serde_json::to_string_pretty(&json)
        .context("failed to serialize settings.json")?;
    std::fs::write(&path, formatted)
        .with_context(|| format!("failed to write {}", path.display()))?;

    Ok(())
}

/// Disable the Bark hook by removing it from settings.json.
///
/// Removes all hook entries whose `command` field contains "bark".
/// If no hooks remain, the file is left with an empty PreToolUse array.
pub fn disable_hook() -> Result<()> {
    let path = settings_path();

    if !path.exists() {
        return Ok(());
    }

    let content = std::fs::read_to_string(&path)
        .with_context(|| format!("failed to read {}", path.display()))?;

    let mut json: Value = serde_json::from_str(&content)
        .with_context(|| format!("failed to parse {}", path.display()))?;

    // Navigate to hooks.PreToolUse and remove bark entries
    // Handles both nested format (matcher+hooks[]) and flat format
    if let Some(hooks) = json.get_mut("hooks") {
        if let Some(pre_tool_use) = hooks.get_mut("PreToolUse") {
            if let Some(arr) = pre_tool_use.as_array_mut() {
                arr.retain(|entry| {
                    // Flat format: {"command":"bark hook"}
                    if entry_has_bark(entry) {
                        return false;
                    }
                    // Nested format: {"matcher":"*","hooks":[{"command":"bark hook"}]}
                    if let Some(inner) = entry.get("hooks").and_then(|h| h.as_array()) {
                        if inner.iter().any(|h| entry_has_bark(h)) {
                            return false;
                        }
                    }
                    true
                });
            }
        }
    }

    // Write back
    let formatted = serde_json::to_string_pretty(&json)
        .context("failed to serialize settings.json")?;
    std::fs::write(&path, formatted)
        .with_context(|| format!("failed to write {}", path.display()))?;

    Ok(())
}

/// Search the settings JSON for a bark hook entry.
/// Handles both the nested format:
///   {"matcher":"*","hooks":[{"type":"command","command":"bark hook"}]}
/// and the flat format:
///   {"type":"command","command":"bark hook"}
fn find_bark_hook(json: &Value) -> bool {
    let arr = match json
        .get("hooks")
        .and_then(|h| h.get("PreToolUse"))
        .and_then(|p| p.as_array())
    {
        Some(a) => a,
        None => return false,
    };

    arr.iter().any(|entry| {
        // Check flat format
        if entry_has_bark(entry) {
            return true;
        }
        // Check nested format: entry.hooks[]
        if let Some(inner_hooks) = entry.get("hooks").and_then(|h| h.as_array()) {
            return inner_hooks.iter().any(|h| entry_has_bark(h));
        }
        false
    })
}

fn entry_has_bark(entry: &Value) -> bool {
    entry
        .get("command")
        .and_then(|c| c.as_str())
        .map(|c| c.contains("bark"))
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_bark_hook_nested_format() {
        let json: Value = serde_json::json!({
            "hooks": {
                "PreToolUse": [
                    {
                        "matcher": "*",
                        "hooks": [
                            {"type": "command", "command": "bark hook", "timeout": 30}
                        ]
                    }
                ]
            }
        });
        assert!(find_bark_hook(&json));
    }

    #[test]
    fn test_find_bark_hook_flat_format() {
        let json: Value = serde_json::json!({
            "hooks": {
                "PreToolUse": [
                    {"type": "command", "command": "bark hook"}
                ]
            }
        });
        assert!(find_bark_hook(&json));
    }

    #[test]
    fn test_find_bark_hook_absent() {
        let json: Value = serde_json::json!({
            "hooks": {
                "PreToolUse": [
                    {
                        "matcher": "*",
                        "hooks": [
                            {"type": "command", "command": "other-tool"}
                        ]
                    }
                ]
            }
        });
        assert!(!find_bark_hook(&json));
    }

    #[test]
    fn test_find_bark_hook_no_hooks() {
        let json: Value = serde_json::json!({});
        assert!(!find_bark_hook(&json));
    }

    #[test]
    fn test_find_bark_hook_empty_array() {
        let json: Value = serde_json::json!({
            "hooks": {
                "PreToolUse": []
            }
        });
        assert!(!find_bark_hook(&json));
    }
}
