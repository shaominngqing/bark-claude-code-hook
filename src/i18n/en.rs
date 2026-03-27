/// English translation strings.
pub fn get(key: &str) -> &'static str {
    match key {
        // --- Risk assessment (fast_rules / engine) ---
        "risk.readonly" => "Read-only tool, no risk",
        "risk.task_mgmt" => "Task management tool, no risk",
        "risk.safe_cmd" => "Safe command",
        "risk.file_edit" => "File edit",
        "risk.unknown_op" => "Unknown operation",
        "risk.needs_confirm" => "requires human confirmation",
        "risk.suspicious_chain" => "Suspicious operation chain detected",

        // --- Status ---
        "status.active" => "Active",
        "status.active_hint" => "hook is running",
        "status.inactive" => "Inactive",
        "status.inactive_hint" => "hook is disabled",
        "status.version" => "Version",
        "status.cache" => "Cache",
        "status.log" => "Log",
        "status.settings" => "Settings",
        "status.rules" => "Rules",
        "status.not_created" => "(not created)",
        "status.entries" => "entries",
        "status.assessments_logged" => "assessments logged",

        // --- On / Off / Toggle ---
        "on.enabled" => "Bark enabled.",
        "off.disabled" => "Bark disabled.",
        "on.error" => "Error enabling Bark",
        "off.error" => "Error disabling Bark",

        // --- Install ---
        "install.check_env" => "Check environment",
        "install.bark_binary" => "bark binary",
        "install.json_builtin" => "JSON parsing: built-in (no jq needed)",
        "install.old_hook" => "Old Bash hook detected \u{2014} will be replaced",
        "install.prepare_dirs" => "Prepare directories",
        "install.init_cache" => "Initialize cache",
        "install.sqlite_cache" => "SQLite cache",
        "install.cache_failed" => "Cache init failed",
        "install.register_hook" => "Register hook",
        "install.hook_exists" => "Hook already registered in settings.json",
        "install.hook_ok" => "PreToolUse hook \u{2192} settings.json",
        "install.hook_failed" => "Failed to register hook",
        "install.verify_cmd" => "Verify command",
        "install.in_path" => "`bark` is in PATH",
        "install.not_in_path" => "`bark` not in PATH. Add it:",
        "install.complete" => "\u{2728} Install complete",
        "install.how_it_works" => "How it works",
        "install.readonly_label" => "Read-only",
        "install.readonly_tools" => "Read / Grep / Glob",
        "install.readonly_action" => "Allow",
        "install.edits_label" => "Edits",
        "install.edits_tools" => "Normal source files",
        "install.edits_action" => "Allow",
        "install.bash_label" => "Bash",
        "install.bash_tools" => "All commands",
        "install.bash_action" => "AST + AI assess",
        "install.repeat_label" => "Repeat",
        "install.repeat_tools" => "Same pattern again",
        "install.repeat_action" => "Cache hit (0ms)",
        "install.danger_label" => "Danger",
        "install.danger_tools" => "rm -rf / force push",
        "install.danger_action" => "Notify + confirm",
        "install.quick_start" => "Quick start",
        "install.cmd_help" => "Show all commands",
        "install.cmd_stats" => "View statistics",
        "install.cmd_test" => "Test risk assessment",
        "install.takes_effect" => "Takes effect in new Claude Code sessions",

        // --- Test ---
        "test.usage" => "Usage: bark test <command>",
        "test.example" => "Example: bark test rm -rf /tmp/test",
        "test.command" => "Command",
        "test.risk" => "Risk",
        "test.source" => "Source",
        "test.reason" => "Reason",
        "test.time" => "Time",
        "test.dry_run_note" => "(dry-run: high risk would be overridden to allow)",
        "test.verbose" => "Verbose output:",

        // --- Cache ---
        "cache.no_db" => "No cache database found",
        "cache.run_first" => "Run some assessments first to populate the cache.",
        "cache.cleared" => "Cache cleared.",
        "cache.stats_title" => "Cache Statistics",
        "cache.entries" => "Entries",
        "cache.size" => "Size",
        "cache.no_entries" => "No cached entries.",
        "cache.recent" => "Recent entries:",

        // --- Log ---
        "log.no_db" => "No database found",
        "log.run_first" => "Run some assessments first to populate the log.",
        "log.cleared" => "Log cleared.",
        "log.title" => "Assessment Log",
        "log.no_entries" => "No log entries found.",

        // --- Stats ---
        "stats.no_data" => "No assessment data yet.",
        "stats.run_first" => "Run some assessments to see statistics.",
        "stats.total" => "Total Assessments",
        "stats.no_assessments" => "No assessments recorded yet.",
        "stats.source_breakdown" => "Source Breakdown",
        "stats.risk_distribution" => "Risk Distribution",
        "stats.cache_hit_rate" => "Cache Hit Rate",
        "stats.cache_label" => "Cache",

        // --- Rules ---
        "rules.no_file" => "No custom rules file found.",
        "rules.create_hint" => "Run {cmd} to create one.",
        "rules.title" => "Custom Rules",
        "rules.no_rules" => "No rules defined.",
        "rules.edit_hint" => "Run {cmd} to add rules.",
        "rules.created" => "Created",
        "rules.opening" => "Opening {path} in {editor}...",
        "rules.saved" => "Rules file saved.",
        "rules.editor_exit" => "Editor exited with status",
        "rules.editor_fail" => "Failed to open editor",
        "rules.editor_hint" => "Set $EDITOR to your preferred editor.",

        // --- Uninstall ---
        "uninstall.title" => "Uninstalling Bark...",
        "uninstall.removed" => "Removed",
        "uninstall.failed" => "Failed to remove",
        "uninstall.hook" => "hook from settings.json",
        "uninstall.cache_db" => "cache database",
        "uninstall.cache_wal" => "cache WAL",
        "uninstall.cache_shm" => "cache SHM",
        "uninstall.custom_rules" => "custom rules",
        "uninstall.log_file" => "log file",
        "uninstall.daemon_socket" => "daemon socket",
        "uninstall.daemon_pid" => "daemon PID file",
        "uninstall.binary" => "Removed binary",
        "uninstall.binary_fail" => "Failed to remove binary",
        "uninstall.skip_dev" => "Skipped binary removal (development build)",
        "uninstall.done" => "Bark has been fully uninstalled.",

        // --- Update ---
        "update.current" => "Bark v{version}",
        "update.not_impl" => "Update check is not yet implemented in the Rust version.",
        "update.manual" => "To update manually, rebuild from source or download the latest release.",

        // --- Notify ---
        "notify.auto_allowed" => "Auto-allowed",
        "notify.needs_confirm" => "Confirmation needed",

        // --- Fallback ---
        _ => "???",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_known_keys() {
        assert_eq!(get("on.enabled"), "Bark enabled.");
        assert_eq!(get("off.disabled"), "Bark disabled.");
        assert_eq!(get("cache.cleared"), "Cache cleared.");
    }

    #[test]
    fn test_unknown_key() {
        assert_eq!(get("does_not_exist"), "???");
    }
}
