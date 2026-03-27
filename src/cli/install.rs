use crate::config;
use crate::i18n::Locale;
use crate::ui::style;
use crossterm::style::Color;

/// Install the bark hook into Claude Code settings.json.
pub fn run() {
    let locale = Locale::detect();

    if config::bark_dir().join("bark.sh").exists() {
        println!();
        style::print_warn(locale.t("install.old_hook"));
    }

    // Step 1: Prepare directories
    style::print_step(locale.t("install.prepare_dirs"));
    std::fs::create_dir_all(config::bark_dir()).ok();
    style::print_ok(&format!("{}", config::bark_dir().display()));

    // Step 2: Initialize SQLite cache
    style::print_step(locale.t("install.init_cache"));
    match crate::cache::SqliteCache::new(&config::bark_db_path()) {
        Ok(_) => style::print_ok(&format!("{}: {}", locale.t("install.sqlite_cache"), config::bark_db_path().display())),
        Err(e) => style::print_err(&format!("{}: {}", locale.t("install.cache_failed"), e)),
    }

    // Step 3: Register hook
    style::print_step(locale.t("install.register_hook"));
    if config::has_hook() {
        style::print_ok(locale.t("install.hook_exists"));
    } else {
        match config::enable_hook() {
            Ok(()) => style::print_ok(locale.t("install.hook_ok")),
            Err(e) => {
                style::print_err(&format!("{}: {}", locale.t("install.hook_failed"), e));
                return;
            }
        }
    }

    // ── Completion banner ──
    println!();
    let banner_width = 50;
    let top = format!("  \u{256d}{}\u{256e}", "\u{2500}".repeat(banner_width));
    let bot = format!("  \u{2570}{}\u{256f}", "\u{2500}".repeat(banner_width));
    let sep = format!("  \u{251c}{}\u{2524}", "\u{2500}".repeat(banner_width));
    println!("{}", style::dim(&top));
    // Title line
    let title = locale.t("install.complete");
    let title_pad = banner_width - title.chars().map(|c| if c > '\u{FF}' { 2 } else { 1 }).sum::<usize>() - 1;
    println!("  {}  {}{}{}",
        style::dim("\u{2502}"),
        style::gradient(title),
        " ".repeat(title_pad),
        style::dim("\u{2502}"),
    );
    println!("{}", style::dim(&sep));

    // Pipeline rows inside the box
    let pipeline: &[(&str, &str, &str, Color)] = &[
        ("install.readonly_label", "install.readonly_tools", "install.readonly_action", Color::Green),
        ("install.edits_label", "install.edits_tools", "install.edits_action", Color::Green),
        ("install.bash_label", "install.bash_tools", "install.bash_action", Color::AnsiValue(45)),
        ("install.repeat_label", "install.repeat_tools", "install.repeat_action", Color::Green),
        ("install.danger_label", "install.danger_tools", "install.danger_action", Color::Red),
    ];

    let display_width = |s: &str| -> usize {
        s.chars().map(|c| if c > '\u{FF}' { 2 } else { 1 }).sum()
    };

    for (label_key, tools_key, action_key, color) in pipeline {
        let label = locale.t(label_key);
        let tools = locale.t(tools_key);
        let action = locale.t(action_key);
        // Pad label to 10 display columns
        let label_w = display_width(label);
        let label_pad = if label_w < 10 { 10 - label_w } else { 1 };
        // Calculate total visible width: " ◆ label  tools  ──▸ action"
        let row_width = 1 + 1 + 1 + label_w + label_pad + display_width(tools) + 2 + 3 + 1 + display_width(action);
        let trail = if row_width < banner_width { banner_width - row_width } else { 0 };
        println!("  {} {} {}{}{}  {} {}{}{}",
            style::dim("\u{2502}"),
            style::colored("\u{25c6}", *color),
            style::dim(label),
            " ".repeat(label_pad),
            style::dim(tools),
            style::flow_arrow(),
            style::colored(action, *color),
            " ".repeat(trail),
            style::dim("\u{2502}"),
        );
    }

    println!("{}", style::dim(&bot));

    // Quick start commands
    println!();
    style::print_section(locale.t("install.quick_start"));
    println!();
    println!("    {} {}           {}",
        style::brand("bark"), style::dim("help"), locale.t("install.cmd_help"));
    println!("    {} {}          {}",
        style::brand("bark"), style::dim("stats"), locale.t("install.cmd_stats"));
    println!("    {} {}  {}",
        style::brand("bark"), style::dim("test rm -rf /"), locale.t("install.cmd_test"));
    println!();
    println!("  {} {}", style::accent("\u{25b8}"), locale.t("install.takes_effect"));
    println!();
}
