/// Chinese translation strings.
pub fn get(key: &str) -> &'static str {
    match key {
        // --- Risk assessment (fast_rules / engine) ---
        "risk.readonly" => "\u{53ea}\u{8bfb}\u{5de5}\u{5177}\u{ff0c}\u{65e0}\u{98ce}\u{9669}",
        "risk.task_mgmt" => "\u{4efb}\u{52a1}\u{7ba1}\u{7406}\u{5de5}\u{5177}\u{ff0c}\u{65e0}\u{98ce}\u{9669}",
        "risk.safe_cmd" => "\u{5b89}\u{5168}\u{547d}\u{4ee4}",
        "risk.file_edit" => "\u{6587}\u{4ef6}\u{7f16}\u{8f91}",
        "risk.unknown_op" => "\u{672a}\u{77e5}\u{64cd}\u{4f5c}",
        "risk.needs_confirm" => "\u{9700}\u{8981}\u{4eba}\u{5de5}\u{786e}\u{8ba4}",
        "risk.suspicious_chain" => "\u{68c0}\u{6d4b}\u{5230}\u{53ef}\u{7591}\u{64cd}\u{4f5c}\u{94fe}",

        // --- Status ---
        "status.active" => "\u{8fd0}\u{884c}\u{4e2d}",
        "status.active_hint" => "hook \u{5df2}\u{542f}\u{7528}",
        "status.inactive" => "\u{5df2}\u{505c}\u{7528}",
        "status.inactive_hint" => "hook \u{5df2}\u{7981}\u{7528}",
        "status.version" => "\u{7248}\u{672c}",
        "status.cache" => "\u{7f13}\u{5b58}",
        "status.log" => "\u{65e5}\u{5fd7}",
        "status.settings" => "\u{914d}\u{7f6e}",
        "status.rules" => "\u{89c4}\u{5219}",
        "status.not_created" => "(\u{672a}\u{521b}\u{5efa})",
        "status.entries" => "\u{6761}",
        "status.assessments_logged" => "\u{6761}\u{8bc4}\u{4f30}\u{8bb0}\u{5f55}",

        // --- On / Off / Toggle ---
        "on.enabled" => "Bark \u{5df2}\u{542f}\u{7528}\u{3002}",
        "off.disabled" => "Bark \u{5df2}\u{7981}\u{7528}\u{3002}",
        "on.error" => "\u{542f}\u{7528} Bark \u{5931}\u{8d25}",
        "off.error" => "\u{7981}\u{7528} Bark \u{5931}\u{8d25}",

        // --- Install ---
        "install.old_hook" => "\u{68c0}\u{6d4b}\u{5230}\u{65e7}\u{7248} Bash hook \u{2014} \u{5c06}\u{88ab}\u{66ff}\u{6362}",
        "install.prepare_dirs" => "\u{51c6}\u{5907}\u{76ee}\u{5f55}",
        "install.init_cache" => "\u{521d}\u{59cb}\u{5316}\u{7f13}\u{5b58}",
        "install.sqlite_cache" => "SQLite \u{7f13}\u{5b58}",
        "install.cache_failed" => "\u{7f13}\u{5b58}\u{521d}\u{59cb}\u{5316}\u{5931}\u{8d25}",
        "install.register_hook" => "\u{6ce8}\u{518c} Hook",
        "install.hook_exists" => "Hook \u{5df2}\u{6ce8}\u{518c}\u{5230} settings.json",
        "install.hook_ok" => "PreToolUse hook \u{2192} settings.json",
        "install.hook_failed" => "\u{6ce8}\u{518c} hook \u{5931}\u{8d25}",
        "install.complete" => "\u{2728} \u{5b89}\u{88c5}\u{5b8c}\u{6210}",
        "install.how_it_works" => "\u{5de5}\u{4f5c}\u{539f}\u{7406}",
        "install.readonly_label" => "\u{53ea}\u{8bfb}\u{5de5}\u{5177}",
        "install.readonly_tools" => "Read / Grep / Glob",
        "install.readonly_action" => "\u{76f4}\u{63a5}\u{653e}\u{884c}",
        "install.edits_label" => "\u{6587}\u{4ef6}\u{7f16}\u{8f91}",
        "install.edits_tools" => "\u{666e}\u{901a}\u{6e90}\u{4ee3}\u{7801}\u{6587}\u{4ef6}",
        "install.edits_action" => "\u{76f4}\u{63a5}\u{653e}\u{884c}",
        "install.bash_label" => "Bash\u{547d}\u{4ee4}",
        "install.bash_tools" => "\u{6240}\u{6709}\u{547d}\u{4ee4}",
        "install.bash_action" => "AST + AI \u{8bc4}\u{4f30}",
        "install.repeat_label" => "\u{91cd}\u{590d}\u{6a21}\u{5f0f}",
        "install.repeat_tools" => "\u{540c}\u{7c7b}\u{547d}\u{4ee4}\u{7b2c}\u{4e8c}\u{6b21}",
        "install.repeat_action" => "\u{7f13}\u{5b58}\u{547d}\u{4e2d} (0ms)",
        "install.danger_label" => "\u{9ad8}\u{98ce}\u{9669}",
        "install.danger_tools" => "rm -rf / force push",
        "install.danger_action" => "\u{901a}\u{77e5} + \u{786e}\u{8ba4}",
        "install.quick_start" => "\u{5feb}\u{901f}\u{5f00}\u{59cb}",
        "install.cmd_help" => "\u{67e5}\u{770b}\u{6240}\u{6709}\u{547d}\u{4ee4}",
        "install.cmd_stats" => "\u{67e5}\u{770b}\u{7edf}\u{8ba1}\u{6570}\u{636e}",
        "install.cmd_test" => "\u{6d4b}\u{8bd5}\u{98ce}\u{9669}\u{8bc4}\u{4f30}",
        "install.takes_effect" => "\u{65b0}\u{5f00}\u{7684} Claude Code \u{4f1a}\u{8bdd}\u{81ea}\u{52a8}\u{751f}\u{6548}",

        // --- Test ---
        "test.usage" => "\u{7528}\u{6cd5}: bark test <\u{547d}\u{4ee4}>",
        "test.example" => "\u{793a}\u{4f8b}: bark test rm -rf /tmp/test",
        "test.command" => "\u{547d}\u{4ee4}",
        "test.risk" => "\u{98ce}\u{9669}",
        "test.source" => "\u{6765}\u{6e90}",
        "test.reason" => "\u{539f}\u{56e0}",
        "test.time" => "\u{8017}\u{65f6}",
        "test.dry_run_note" => "(\u{6a21}\u{62df}\u{8fd0}\u{884c}: \u{9ad8}\u{98ce}\u{9669}\u{5c06}\u{88ab}\u{8986}\u{76d6}\u{4e3a}\u{5141}\u{8bb8})",
        "test.verbose" => "\u{8be6}\u{7ec6}\u{8f93}\u{51fa}:",

        // --- Cache ---
        "cache.no_db" => "\u{672a}\u{627e}\u{5230}\u{7f13}\u{5b58}\u{6570}\u{636e}\u{5e93}",
        "cache.run_first" => "\u{5148}\u{8fd0}\u{884c}\u{4e00}\u{4e9b}\u{8bc4}\u{4f30}\u{6765}\u{586b}\u{5145}\u{7f13}\u{5b58}\u{3002}",
        "cache.cleared" => "\u{7f13}\u{5b58}\u{5df2}\u{6e05}\u{7a7a}\u{3002}",
        "cache.stats_title" => "\u{7f13}\u{5b58}\u{7edf}\u{8ba1}",
        "cache.entries" => "\u{6761}\u{76ee}\u{6570}",
        "cache.size" => "\u{5360}\u{7528}",
        "cache.no_entries" => "\u{6ca1}\u{6709}\u{7f13}\u{5b58}\u{6761}\u{76ee}\u{3002}",
        "cache.recent" => "\u{6700}\u{8fd1}\u{6761}\u{76ee}:",

        // --- Log ---
        "log.no_db" => "\u{672a}\u{627e}\u{5230}\u{6570}\u{636e}\u{5e93}",
        "log.run_first" => "\u{5148}\u{8fd0}\u{884c}\u{4e00}\u{4e9b}\u{8bc4}\u{4f30}\u{6765}\u{586b}\u{5145}\u{65e5}\u{5fd7}\u{3002}",
        "log.cleared" => "\u{65e5}\u{5fd7}\u{5df2}\u{6e05}\u{7a7a}\u{3002}",
        "log.title" => "\u{8bc4}\u{4f30}\u{65e5}\u{5fd7}",
        "log.no_entries" => "\u{6ca1}\u{6709}\u{65e5}\u{5fd7}\u{6761}\u{76ee}\u{3002}",

        // --- Stats ---
        "stats.no_data" => "\u{6682}\u{65e0}\u{8bc4}\u{4f30}\u{6570}\u{636e}\u{3002}",
        "stats.run_first" => "\u{8fd0}\u{884c}\u{4e00}\u{4e9b}\u{8bc4}\u{4f30}\u{6765}\u{67e5}\u{770b}\u{7edf}\u{8ba1}\u{3002}",
        "stats.total" => "\u{603b}\u{8bc4}\u{4f30}\u{6570}",
        "stats.no_assessments" => "\u{6682}\u{65e0}\u{8bc4}\u{4f30}\u{8bb0}\u{5f55}\u{3002}",
        "stats.source_breakdown" => "\u{6765}\u{6e90}\u{5206}\u{5e03}",
        "stats.risk_distribution" => "\u{98ce}\u{9669}\u{5206}\u{5e03}",
        "stats.cache_hit_rate" => "\u{7f13}\u{5b58}\u{547d}\u{4e2d}\u{7387}",
        "stats.cache_label" => "\u{7f13}\u{5b58}",

        // --- Rules ---
        "rules.no_file" => "\u{672a}\u{627e}\u{5230}\u{81ea}\u{5b9a}\u{4e49}\u{89c4}\u{5219}\u{6587}\u{4ef6}\u{3002}",
        "rules.create_hint" => "\u{8fd0}\u{884c} {cmd} \u{6765}\u{521b}\u{5efa}\u{3002}",
        "rules.title" => "\u{81ea}\u{5b9a}\u{4e49}\u{89c4}\u{5219}",
        "rules.no_rules" => "\u{672a}\u{5b9a}\u{4e49}\u{89c4}\u{5219}\u{3002}",
        "rules.edit_hint" => "\u{8fd0}\u{884c} {cmd} \u{6765}\u{6dfb}\u{52a0}\u{89c4}\u{5219}\u{3002}",
        "rules.created" => "\u{5df2}\u{521b}\u{5efa}",
        "rules.opening" => "\u{6b63}\u{5728}\u{7528} {editor} \u{6253}\u{5f00} {path}...",
        "rules.saved" => "\u{89c4}\u{5219}\u{6587}\u{4ef6}\u{5df2}\u{4fdd}\u{5b58}\u{3002}",
        "rules.editor_exit" => "\u{7f16}\u{8f91}\u{5668}\u{9000}\u{51fa}\u{72b6}\u{6001}",
        "rules.editor_fail" => "\u{6253}\u{5f00}\u{7f16}\u{8f91}\u{5668}\u{5931}\u{8d25}",
        "rules.editor_hint" => "\u{8bf7}\u{8bbe}\u{7f6e} $EDITOR \u{4e3a}\u{4f60}\u{7684}\u{9996}\u{9009}\u{7f16}\u{8f91}\u{5668}\u{3002}",

        // --- Uninstall ---
        "uninstall.title" => "\u{6b63}\u{5728}\u{5378}\u{8f7d} Bark...",
        "uninstall.removed" => "\u{5df2}\u{5220}\u{9664}",
        "uninstall.failed" => "\u{5220}\u{9664}\u{5931}\u{8d25}",
        "uninstall.hook" => "settings.json \u{4e2d}\u{7684} hook",
        "uninstall.cache_db" => "\u{7f13}\u{5b58}\u{6570}\u{636e}\u{5e93}",
        "uninstall.cache_wal" => "\u{7f13}\u{5b58} WAL",
        "uninstall.cache_shm" => "\u{7f13}\u{5b58} SHM",
        "uninstall.custom_rules" => "\u{81ea}\u{5b9a}\u{4e49}\u{89c4}\u{5219}",
        "uninstall.log_file" => "\u{65e5}\u{5fd7}\u{6587}\u{4ef6}",
        "uninstall.daemon_socket" => "\u{5b88}\u{62a4}\u{8fdb}\u{7a0b}\u{5957}\u{63a5}\u{5b57}",
        "uninstall.daemon_pid" => "\u{5b88}\u{62a4}\u{8fdb}\u{7a0b} PID \u{6587}\u{4ef6}",
        "uninstall.binary" => "\u{5df2}\u{5220}\u{9664}\u{53ef}\u{6267}\u{884c}\u{6587}\u{4ef6}",
        "uninstall.binary_fail" => "\u{5220}\u{9664}\u{53ef}\u{6267}\u{884c}\u{6587}\u{4ef6}\u{5931}\u{8d25}",
        "uninstall.skip_dev" => "\u{8df3}\u{8fc7}\u{53ef}\u{6267}\u{884c}\u{6587}\u{4ef6}\u{5220}\u{9664}(\u{5f00}\u{53d1}\u{6784}\u{5efa})",
        "uninstall.done" => "Bark \u{5df2}\u{5b8c}\u{5168}\u{5378}\u{8f7d}\u{3002}",

        // --- Update ---
        "update.current" => "Bark v{version}",
        "update.not_impl" => "Rust \u{7248}\u{672c}\u{5c1a}\u{672a}\u{5b9e}\u{73b0}\u{66f4}\u{65b0}\u{68c0}\u{67e5}\u{3002}",
        "update.manual" => "\u{8bf7}\u{4ece}\u{6e90}\u{7801}\u{91cd}\u{65b0}\u{6784}\u{5efa}\u{6216}\u{4e0b}\u{8f7d}\u{6700}\u{65b0}\u{53d1}\u{5e03}\u{3002}",

        // --- Notify ---
        "notify.auto_allowed" => "\u{5df2}\u{81ea}\u{52a8}\u{653e}\u{884c}",
        "notify.needs_confirm" => "\u{9700}\u{8981}\u{786e}\u{8ba4}",

        // --- Fallback: delegate to English ---
        _ => super::en::get(key),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_known_keys() {
        assert_eq!(get("on.enabled"), "Bark \u{5df2}\u{542f}\u{7528}\u{3002}");
        assert_eq!(get("off.disabled"), "Bark \u{5df2}\u{7981}\u{7528}\u{3002}");
    }

    #[test]
    fn test_unknown_key_falls_back_to_english() {
        assert_eq!(get("does_not_exist"), "???");
    }
}
