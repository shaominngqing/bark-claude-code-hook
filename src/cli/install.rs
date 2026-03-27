use crate::config;
use crate::i18n::Locale;
use crate::ui::gradient::{self, BOLD, DIM, GREEN, NC, RED, YELLOW, C1, ACCENT};
use crate::ui::logo;

/// Install the bark hook into Claude Code settings.json.
pub fn run() {
    let locale = Locale::detect();

    println!();
    logo::print_logo();
    println!();

    // Step 1: Check environment
    let step_label = if locale == Locale::Zh { "检查环境" } else { "Check environment" };
    println!("  {}{}▸{} {}{}{}", C1, BOLD, NC, BOLD, step_label, NC);

    // Check if bark binary is accessible
    let bark_path = std::env::current_exe().unwrap_or_default();
    println!("  {}✓{} bark binary: {}", GREEN, NC, bark_path.display());

    // Check jq (optional now, Rust handles JSON)
    println!("  {}✓{} JSON parsing: built-in (no jq needed)", GREEN, NC);

    // Check for existing old bash hook
    if config::bark_dir().join("bark.sh").exists() {
        println!("  {}⚠{} Old Bash hook detected — will be replaced", YELLOW, NC);
    }
    println!();

    // Step 2: Prepare directories
    let step_label = if locale == Locale::Zh { "准备目录" } else { "Prepare directories" };
    println!("  {}{}▸{} {}{}{}", C1, BOLD, NC, BOLD, step_label, NC);

    std::fs::create_dir_all(config::bark_dir()).ok();
    println!("  {}✓{} {}", GREEN, NC, config::bark_dir().display());

    println!();

    // Step 3: Initialize SQLite cache
    let step_label = if locale == Locale::Zh { "初始化缓存" } else { "Initialize cache" };
    println!("  {}{}▸{} {}{}{}", C1, BOLD, NC, BOLD, step_label, NC);

    match crate::cache::SqliteCache::new(&config::bark_db_path()) {
        Ok(_) => println!("  {}✓{} SQLite cache: {}", GREEN, NC, config::bark_db_path().display()),
        Err(e) => println!("  {}✗{} Cache init failed: {}", RED, NC, e),
    }
    println!();

    // Step 4: Register hook
    let step_label = if locale == Locale::Zh { "注册 Hook" } else { "Register hook" };
    println!("  {}{}▸{} {}{}{}", C1, BOLD, NC, BOLD, step_label, NC);

    if config::has_hook() {
        println!("  {}✓{} Hook already registered in settings.json", GREEN, NC);
    } else {
        match config::enable_hook() {
            Ok(()) => {
                println!("  {}✓{} PreToolUse hook → settings.json", GREEN, NC);
            }
            Err(e) => {
                println!("  {}✗{} Failed to register hook: {}", RED, NC, e);
                return;
            }
        }
    }
    println!();

    // Step 5: Verify PATH
    let step_label = if locale == Locale::Zh { "验证命令" } else { "Verify command" };
    println!("  {}{}▸{} {}{}{}", C1, BOLD, NC, BOLD, step_label, NC);

    // Check if bark is in PATH
    let in_path = std::process::Command::new("which")
        .arg("bark")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);

    if in_path {
        println!("  {}✓{} `bark` is in PATH", GREEN, NC);
    } else {
        let bin_path = bark_path.display();
        println!("  {}⚠{} `bark` not in PATH. Add it:", YELLOW, NC);
        println!("     {}sudo ln -sf {} /usr/local/bin/bark{}", DIM, bin_path, NC);
        println!("     {}# or add to your shell profile:{}", DIM, NC);
        println!("     {}export PATH=\"{}:$PATH\"{}", DIM, bark_path.parent().unwrap_or(&bark_path).display(), NC);
    }

    // Summary
    println!();
    println!("  {}╭───────────────────────────────────────────────╮{}", DIM, NC);
    print!("  {}│{}  ", DIM, NC);
    print!("{}", gradient::gradient_text("✦ Install complete"));
    println!("                          {}│{}", DIM, NC);
    println!("  {}╰───────────────────────────────────────────────╯{}", DIM, NC);
    println!();

    if locale == Locale::Zh {
        println!("  {}工作原理{}", BOLD, NC);
        println!();
        println!("    {}◆{} {}只读工具{}  Read / Grep / Glob     {}──▸{} {}直接放行{}", GREEN, NC, DIM, NC, DIM, NC, GREEN, NC);
        println!("    {}◆{} {}文件编辑{}  普通源代码文件         {}──▸{} {}直接放行{}", GREEN, NC, DIM, NC, DIM, NC, GREEN, NC);
        println!("    {}◆{} {}Bash命令{}  所有命令               {}──▸{} {}AST + AI 评估{}", gradient::C2, NC, DIM, NC, DIM, NC, gradient::C2, NC);
        println!("    {}◆{} {}重复模式{}  同类命令第二次         {}──▸{} {}缓存命中 (0ms){}", GREEN, NC, DIM, NC, DIM, NC, GREEN, NC);
        println!("    {}◆{} {}高风险  {}  rm -rf / force push   {}──▸{} {}通知 + 确认{}", RED, NC, DIM, NC, DIM, NC, RED, NC);
        println!();
        println!("  {}快速开始{}", BOLD, NC);
        println!();
        println!("    {}bark{} {}help{}           查看所有命令", C1, NC, DIM, NC);
        println!("    {}bark{} {}stats{}          查看统计数据", C1, NC, DIM, NC);
        println!("    {}bark{} {}test rm -rf /{}  测试风险评估", C1, NC, DIM, NC);
        println!();
        println!("  {}▸ 新开的 Claude Code 会话自动生效{}", ACCENT, NC);
    } else {
        println!("  {}How it works{}", BOLD, NC);
        println!();
        println!("    {}◆{} {}Read-only{}  Read / Grep / Glob     {}──▸{} {}Allow{}", GREEN, NC, DIM, NC, DIM, NC, GREEN, NC);
        println!("    {}◆{} {}Edits   {}  Normal source files     {}──▸{} {}Allow{}", GREEN, NC, DIM, NC, DIM, NC, GREEN, NC);
        println!("    {}◆{} {}Bash    {}  All commands             {}──▸{} {}AST + AI assess{}", gradient::C2, NC, DIM, NC, DIM, NC, gradient::C2, NC);
        println!("    {}◆{} {}Repeat  {}  Same pattern again      {}──▸{} {}Cache hit (0ms){}", GREEN, NC, DIM, NC, DIM, NC, GREEN, NC);
        println!("    {}◆{} {}Danger  {}  rm -rf / force push     {}──▸{} {}Notify + confirm{}", RED, NC, DIM, NC, DIM, NC, RED, NC);
        println!();
        println!("  {}Quick start{}", BOLD, NC);
        println!();
        println!("    {}bark{} {}help{}           Show all commands", C1, NC, DIM, NC);
        println!("    {}bark{} {}stats{}          View statistics", C1, NC, DIM, NC);
        println!("    {}bark{} {}test rm -rf /{}  Test risk assessment", C1, NC, DIM, NC);
        println!();
        println!("  {}▸ Takes effect in new Claude Code sessions{}", ACCENT, NC);
    }
    println!();
}
