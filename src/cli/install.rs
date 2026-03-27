use crate::config;
use crate::i18n::Locale;
use crate::ui::logo;
use crate::ui::style;
use crossterm::style::Color;

/// Install the bark hook into Claude Code settings.json.
pub fn run() {
    let locale = Locale::detect();

    println!();
    logo::print_logo();
    println!();

    // Step 1: Check environment
    style::print_step(locale.t("install.check_env"));

    let bark_path = std::env::current_exe().unwrap_or_default();
    style::print_ok(&format!("{}: {}", locale.t("install.bark_binary"), bark_path.display()));
    style::print_ok(locale.t("install.json_builtin"));

    if config::bark_dir().join("bark.sh").exists() {
        style::print_warn(locale.t("install.old_hook"));
    }
    println!();

    // Step 2: Prepare directories
    style::print_step(locale.t("install.prepare_dirs"));
    std::fs::create_dir_all(config::bark_dir()).ok();
    style::print_ok(&format!("{}", config::bark_dir().display()));
    println!();

    // Step 3: Initialize SQLite cache
    style::print_step(locale.t("install.init_cache"));
    match crate::cache::SqliteCache::new(&config::bark_db_path()) {
        Ok(_) => style::print_ok(&format!("{}: {}", locale.t("install.sqlite_cache"), config::bark_db_path().display())),
        Err(e) => style::print_err(&format!("{}: {}", locale.t("install.cache_failed"), e)),
    }
    println!();

    // Step 4: Register hook
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
    println!();

    // Step 5: Verify PATH
    style::print_step(locale.t("install.verify_cmd"));
    let in_path = std::process::Command::new("which")
        .arg("bark")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    if in_path {
        style::print_ok(locale.t("install.in_path"));
    } else {
        style::print_warn(locale.t("install.not_in_path"));
        let bin_path = bark_path.display();
        println!("     {}",
            style::dim(format!("sudo ln -sf {} /usr/local/bin/bark", bin_path)));
        println!("     {}",
            style::dim(format!("export PATH=\"{}:$PATH\"", bark_path.parent().unwrap_or(&bark_path).display())));
    }

    // Summary banner
    println!();
    let banner_width = 42;
    let border = "\u{2500}".repeat(banner_width);
    println!("  {}", style::dim(format!("\u{256d}{}\u{256e}", border)));
    let pad = banner_width - locale.t("install.complete").chars().count() - 1;
    println!("  {}  {}{}{}",
        style::dim("\u{2502}"),
        style::gradient(locale.t("install.complete")),
        " ".repeat(pad),
        style::dim("\u{2502}"),
    );
    println!("  {}", style::dim(format!("\u{2570}{}\u{256f}", border)));
    println!();

    // How it works table
    style::print_section(locale.t("install.how_it_works"));
    println!();

    let pipeline: &[(&str, &str, &str, Color)] = &[
        ("install.readonly_label", "install.readonly_tools", "install.readonly_action", Color::Green),
        ("install.edits_label", "install.edits_tools", "install.edits_action", Color::Green),
        ("install.bash_label", "install.bash_tools", "install.bash_action", Color::AnsiValue(45)),
        ("install.repeat_label", "install.repeat_tools", "install.repeat_action", Color::Green),
        ("install.danger_label", "install.danger_tools", "install.danger_action", Color::Red),
    ];

    for (label_key, tools_key, action_key, color) in pipeline {
        println!("    {} {:<10} {}  {} {}",
            style::colored("\u{25c6}", *color),
            style::dim(locale.t(label_key)),
            style::dim(locale.t(tools_key)),
            style::flow_arrow(),
            style::colored(locale.t(action_key), *color),
        );
    }

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
