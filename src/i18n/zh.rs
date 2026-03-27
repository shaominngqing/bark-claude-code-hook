
/// Chinese translation strings.
///
/// Ported from the bash `_t()` function in install.sh (Chinese section).

#[cfg(test)]
pub fn get(key: &str) -> &'static str {
    match key {
        // --- Status ---
        "enabled" => "Bark: 启用中",
        "disabled" => "Bark: 已禁用",

        // --- Toggle ---
        "already_on" => "Bark 已经是启用状态",
        "turned_on" => "Bark 已启用（新会话生效）",
        "already_off" => "Bark 已经是禁用状态",
        "turned_off" => "Bark 已禁用（新会话生效）",

        // --- Cache ---
        "cache_cleared" => "缓存已清空",
        "cache_empty" => "缓存为空",
        "cache_stats" => "缓存统计:",
        "cache_count" => "条目数",
        "cache_size" => "占用",
        "cache_dir" => "目录",
        "cache_entries" => "缓存条目",
        "cache_recent" => "最近缓存的判断:",

        // --- Log ---
        "log_cleared" => "日志已清空",
        "log_empty" => "日志为空",
        "log_lines" => "日志行数",
        "log_recent" => "最近",
        "log_suffix" => "条日志:",

        // --- Uninstall ---
        "uninstalling" => "正在卸载 Bark...",
        "uninstalled" => "Bark 已完全卸载",

        // --- Test ---
        "test_usage" => "用法: bark test [--verbose] [--dry-run] <bash command>",
        "test_example" => "示例: bark test --verbose rm -rf node_modules",

        // --- Help ---
        "help_title" => "Bark \u{2014} Claude Code AI 风险守卫",
        "cmd_status" => "  status            查看状态",
        "cmd_onoff" => "  on / off          启用 / 禁用",
        "cmd_toggle" => "  toggle            切换开关",
        "cmd_test" => "  test <cmd>        测试命令风险等级",
        "cmd_cache" => "  cache [clear]     查看/清空缓存",
        "cmd_log" => "  log [N|clear]     查看最近 N 条日志 / 清空日志",
        "cmd_stats" => "  stats             查看统计数据",
        "cmd_rules" => "  rules [edit]      查看/编辑自定义规则",
        "cmd_update" => "  update            更新到最新版本",
        "cmd_uninstall" => "  uninstall         完全卸载",
        "cmd_version" => "  version           显示版本号",
        "cmd_help" => "  help              显示此帮助",

        // --- Rules ---
        "rules_empty" => "未定义自定义规则",
        "rules_header" => "自定义规则:",
        "rules_path" => "文件",

        // --- Stats ---
        "stats_empty" => "暂无统计数据（日志为空）",

        // --- Update ---
        "updating" => "正在更新 Bark...",
        "update_ok" => "Bark 已更新到最新版本",
        "update_fail" => "更新失败，请检查网络连接",

        // --- Risk assessment labels ---
        "read_only" => "只读操作",
        "task_mgmt" => "任务管理操作",
        "normal_edit" => "普通文件编辑",
        "ai_fallback" => "AI 评估失败，保守回退",

        // --- Status display ---
        "status_active" => "运行中 \u{2014} hook 已启用",
        "status_inactive" => "已停用 \u{2014} hook 已禁用",

        // --- Misc ---
        "version_label" => "版本",
        "confirm_uninstall" => "确定要卸载 Bark 吗？",
        "cancelled" => "已取消",

        // --- Fallback: delegate to English ---
        _ => super::en::get(key),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_known_keys() {
        assert_eq!(get("enabled"), "Bark: 启用中");
        assert_eq!(get("disabled"), "Bark: 已禁用");
        assert_eq!(get("cache_cleared"), "缓存已清空");
    }

    #[test]
    fn test_unknown_key_falls_back_to_english() {
        // Unknown keys in Chinese fall back to the English translation.
        assert_eq!(get("does_not_exist"), "???");
    }

    #[test]
    fn test_all_cmd_keys_match_english() {
        let cmd_keys = [
            "cmd_status", "cmd_onoff", "cmd_toggle", "cmd_test",
            "cmd_cache", "cmd_log", "cmd_stats", "cmd_rules",
            "cmd_update", "cmd_uninstall", "cmd_version", "cmd_help",
        ];
        for key in cmd_keys {
            let val = get(key);
            assert_ne!(val, "???", "Missing Chinese translation for key: {}", key);
        }
    }
}
