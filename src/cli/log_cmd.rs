use crate::cache::SqliteCache;
use crate::cli::LogAction;
use crate::config;
use crate::core::risk::RiskLevel;
use crate::i18n::Locale;
use crate::ui::style;

/// View or clear the assessment log.
pub fn run(action: Option<LogAction>, count: usize) {
    let locale = Locale::detect();
    let db_path = config::bark_db_path();

    if !db_path.exists() {
        println!("  {} {} {}", style::dim("\u{25cb}"), locale.t("log.no_db"), db_path.display());
        println!("  {}", style::dim(locale.t("log.run_first")));
        return;
    }

    let cache = match SqliteCache::open(&db_path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("  {} Error: {}", style::cross(), e);
            return;
        }
    };

    match action {
        Some(LogAction::Clear) => {
            match cache.clear_log() {
                Ok(()) => println!("  {} {}", style::check(), style::bold(locale.t("log.cleared"))),
                Err(e) => eprintln!("  {} Error: {}", style::cross(), e),
            }
        }
        None => {
            show_log(&cache, count, &locale);
        }
    }
}

fn show_log(cache: &SqliteCache, count: usize, locale: &Locale) {
    match cache.get_log(count) {
        Ok(entries) => {
            if entries.is_empty() {
                println!("  {}", style::dim(locale.t("log.no_entries")));
                return;
            }

            println!();
            println!("  {} (last {} entries)",
                style::bold(locale.t("log.title")),
                entries.len(),
            );
            println!();

            for entry in &entries {
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
                    "    {}  {}  {} {}  {}",
                    style::dim(&entry.timestamp),
                    style::risk_colored(level_str, entry.risk_level),
                    style::dim(format!("{:<8}", entry.source)),
                    style::dim(format!("{}ms", entry.duration_ms)),
                    op_desc,
                );
            }

            println!();
        }
        Err(e) => {
            eprintln!("  {} Error: {}", style::cross(), e);
        }
    }
}
