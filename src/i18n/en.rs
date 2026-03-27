
/// English translation strings.
///
/// Ported from the bash `_t()` function in install.sh (English section).

#[cfg(test)]
pub fn get(key: &str) -> &'static str {
    match key {
        // --- Status ---
        "enabled" => "Bark: Enabled",
        "disabled" => "Bark: Disabled",

        // --- Toggle ---
        "already_on" => "Bark is already enabled",
        "turned_on" => "Bark enabled (takes effect in new sessions)",
        "already_off" => "Bark is already disabled",
        "turned_off" => "Bark disabled (takes effect in new sessions)",

        // --- Cache ---
        "cache_cleared" => "Cache cleared",
        "cache_empty" => "Cache is empty",
        "cache_stats" => "Cache statistics:",
        "cache_count" => "Entries",
        "cache_size" => "Size",
        "cache_dir" => "Directory",
        "cache_entries" => "Cache entries",
        "cache_recent" => "Recent cached decisions:",

        // --- Log ---
        "log_cleared" => "Log cleared",
        "log_empty" => "Log is empty",
        "log_lines" => "Log lines",
        "log_recent" => "Last",
        "log_suffix" => "log entries:",

        // --- Uninstall ---
        "uninstalling" => "Uninstalling Bark...",
        "uninstalled" => "Bark fully uninstalled",

        // --- Test ---
        "test_usage" => "Usage: bark test [--verbose] [--dry-run] <bash command>",
        "test_example" => "Example: bark test --verbose rm -rf node_modules",

        // --- Help ---
        "help_title" => "Bark \u{2014} AI Risk Assessment for Claude Code",
        "cmd_status" => "  status            Show status",
        "cmd_onoff" => "  on / off          Enable / disable",
        "cmd_toggle" => "  toggle            Toggle on/off",
        "cmd_test" => "  test <cmd>        Test a command's risk level",
        "cmd_cache" => "  cache [clear]     View/clear cache",
        "cmd_log" => "  log [N|clear]     View last N log entries / clear log",
        "cmd_stats" => "  stats             Show statistics",
        "cmd_rules" => "  rules [edit]      View/edit custom rules",
        "cmd_update" => "  update            Update to latest version",
        "cmd_uninstall" => "  uninstall         Completely uninstall",
        "cmd_version" => "  version           Show version",
        "cmd_help" => "  help              Show this help",

        // --- Rules ---
        "rules_empty" => "No custom rules defined",
        "rules_header" => "Custom rules:",
        "rules_path" => "File",

        // --- Stats ---
        "stats_empty" => "No statistics yet (log is empty)",

        // --- Update ---
        "updating" => "Updating Bark...",
        "update_ok" => "Bark updated to latest version",
        "update_fail" => "Update failed, please check your network",

        // --- Risk assessment labels ---
        "read_only" => "Read-only operation",
        "task_mgmt" => "Task management",
        "normal_edit" => "Normal file edit",
        "ai_fallback" => "AI assessment failed, conservative fallback",

        // --- Status display ---
        "status_active" => "Active \u{2014} hook is running",
        "status_inactive" => "Inactive \u{2014} hook is disabled",

        // --- Misc ---
        "version_label" => "Version",
        "confirm_uninstall" => "Are you sure you want to uninstall Bark?",
        "cancelled" => "Cancelled",

        // --- Fallback ---
        _ => "???",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_known_keys() {
        assert_eq!(get("enabled"), "Bark: Enabled");
        assert_eq!(get("disabled"), "Bark: Disabled");
        assert_eq!(get("cache_cleared"), "Cache cleared");
        assert_eq!(get("help_title"), "Bark \u{2014} AI Risk Assessment for Claude Code");
    }

    #[test]
    fn test_unknown_key() {
        assert_eq!(get("does_not_exist"), "???");
    }

    #[test]
    fn test_all_cmd_keys() {
        let cmd_keys = [
            "cmd_status", "cmd_onoff", "cmd_toggle", "cmd_test",
            "cmd_cache", "cmd_log", "cmd_stats", "cmd_rules",
            "cmd_update", "cmd_uninstall", "cmd_version", "cmd_help",
        ];
        for key in cmd_keys {
            let val = get(key);
            assert_ne!(val, "???", "Missing translation for key: {}", key);
        }
    }
}
