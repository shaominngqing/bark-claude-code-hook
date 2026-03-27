use std::time::Instant;

use crate::config;
use crate::core::engine::AssessmentEngine;
use crate::core::protocol::HookInput;
use crate::core::risk::RiskLevel;
use crate::ui::gradient::{BOLD, DIM, GREEN, NC, RED, YELLOW};

/// Test a command's risk level offline.
pub fn run(cmd: Vec<String>, verbose: bool, dry_run: bool) {
    if cmd.is_empty() {
        eprintln!("Usage: bark test <command>");
        eprintln!("Example: bark test rm -rf /tmp/test");
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

    // Display results
    let (color, label) = match assessment.level {
        RiskLevel::Low => (GREEN, "LOW"),
        RiskLevel::Medium => (YELLOW, "MEDIUM"),
        RiskLevel::High => (RED, "HIGH"),
    };

    println!();
    println!(
        "  {}Command{}  {}",
        DIM, NC, command_str
    );
    println!(
        "  {}Risk{}     {}{}{}{}",
        DIM, NC, color, BOLD, label, NC
    );
    println!(
        "  {}Source{}   {}",
        DIM, NC, assessment.source
    );
    println!(
        "  {}Reason{}   {}",
        DIM, NC, assessment.reason
    );
    println!(
        "  {}Time{}     {:.1}ms",
        DIM, NC, elapsed.as_secs_f64() * 1000.0
    );

    if dry_run && assessment.level == RiskLevel::High {
        println!();
        println!(
            "  {}(dry-run: high risk would be overridden to allow){}",
            DIM, NC
        );
    }

    if verbose {
        println!();
        println!("  {}Verbose output:{}", DIM, NC);
        println!("    Tool: {}", input.tool_name);
        println!("    Input: {}", input.tool_input);
        println!("    Duration: {:?}", assessment.duration);
    }

    println!();
}
