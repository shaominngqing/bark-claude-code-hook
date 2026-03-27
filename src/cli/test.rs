use std::time::Instant;

use crate::config;
use crate::core::engine::AssessmentEngine;
use crate::core::protocol::HookInput;
use crate::core::risk::RiskLevel;
use crate::i18n::Locale;
use crate::ui::style;

/// Test a command's risk level offline.
pub fn run(cmd: Vec<String>, verbose: bool, dry_run: bool) {
    let locale = Locale::detect();

    if cmd.is_empty() {
        eprintln!("{}", locale.t("test.usage"));
        eprintln!("{}", locale.t("test.example"));
        return;
    }

    let command_str = cmd.join(" ");

    // Build a HookInput simulating a Bash tool call
    let input = HookInput {
        tool_name: "Bash".to_string(),
        tool_input: serde_json::json!({ "command": command_str }),
    };

    // Create a standalone engine
    let cache_path = config::bark_db_path();
    let toml_path = config::bark_toml_path();

    let engine = AssessmentEngine::new_standalone(
        Some(toml_path.as_path()),
        Some(cache_path.as_path()),
    );

    // Run assessment
    let start = Instant::now();
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("Failed to create tokio runtime");

    let assessment = rt.block_on(engine.assess(&input));
    let elapsed = start.elapsed();

    let level_str = match assessment.level {
        RiskLevel::Low => "LOW",
        RiskLevel::Medium => "MEDIUM",
        RiskLevel::High => "HIGH",
    };

    // Display results
    println!();
    style::print_kv(locale.t("test.command"), &command_str);
    println!("  {:<10}{}", style::dim(locale.t("test.risk")), style::risk_colored(level_str, assessment.level));
    style::print_kv(locale.t("test.source"), &assessment.source.to_string());
    style::print_kv(locale.t("test.reason"), &assessment.reason);
    style::print_kv(locale.t("test.time"), &format!("{:.1}ms", elapsed.as_secs_f64() * 1000.0));

    if dry_run && assessment.level == RiskLevel::High {
        println!();
        println!("  {}", style::dim(locale.t("test.dry_run_note")));
    }

    if verbose {
        println!();
        println!("  {}", style::dim(locale.t("test.verbose")));
        println!("    Tool: {}", input.tool_name);
        println!("    Input: {}", input.tool_input);
        println!("    Duration: {:?}", assessment.duration);
    }

    println!();
}
