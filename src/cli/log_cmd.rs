use crate::cache::SqliteCache;
use crate::cli::LogAction;
use crate::config;
use crate::core::risk::RiskLevel;
use crate::ui::gradient::{BOLD, DIM, GREEN, NC, RED, YELLOW};

/// View or clear the assessment log.
pub fn run(action: Option<LogAction>, count: usize) {
    let db_path = config::bark_db_path();

    if !db_path.exists() {
        println!("  No database found at {}", db_path.display());
        println!("  Run some assessments first to populate the log.");
        return;
    }

    let cache = match SqliteCache::open(&db_path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("  Error opening database: {}", e);
            return;
        }
    };

    match action {
        Some(LogAction::Clear) => {
            match cache.clear_log() {
                Ok(()) => {
                    println!("  {}{}Log cleared.{}", GREEN, BOLD, NC);
                }
                Err(e) => {
                    eprintln!("  Error clearing log: {}", e);
                }
            }
        }
        None => {
            show_log(&cache, count);
        }
    }
}

fn show_log(cache: &SqliteCache, count: usize) {
    match cache.get_log(count) {
        Ok(entries) => {
            if entries.is_empty() {
                println!("  No log entries found.");
                return;
            }

            println!();
            println!(
                "  {}Assessment Log{} (last {} entries)",
                BOLD, NC, entries.len()
            );
            println!();

            for entry in &entries {
                let color = match entry.risk_level {
                    RiskLevel::Low => GREEN,
                    RiskLevel::Medium => YELLOW,
                    RiskLevel::High => RED,
                };
                let level_str = match entry.risk_level {
                    RiskLevel::Low => "LOW",
                    RiskLevel::Medium => "MED",
                    RiskLevel::High => "HI ",
                };

                // Build the operation description
                let op_desc = if let Some(ref cmd) = entry.command {
                    let truncated = if cmd.len() > 60 {
                        format!("{}...", &cmd[..60])
                    } else {
                        cmd.clone()
                    };
                    format!("{}({})", entry.tool_name, truncated)
                } else if let Some(ref fp) = entry.file_path {
                    format!("{}({})", entry.tool_name, fp)
                } else {
                    entry.tool_name.clone()
                };

                println!(
                    "  {}{}  {}{}{}{}  {}{}{} {}ms  {}",
                    DIM, entry.timestamp, color, BOLD, level_str, NC,
                    DIM, entry.source, NC, entry.duration_ms, op_desc
                );
            }

            println!();
        }
        Err(e) => {
            eprintln!("  Error reading log: {}", e);
        }
    }
}
