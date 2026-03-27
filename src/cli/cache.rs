use crate::cache::SqliteCache;
use crate::cli::CacheAction;
use crate::config;
use crate::core::risk::RiskLevel;
use crate::i18n::Locale;
use crate::ui::style;

/// View or clear the assessment cache.
pub fn run(action: Option<CacheAction>) {
    let locale = Locale::detect();
    let db_path = config::bark_db_path();

    if !db_path.exists() {
        println!("  {} {} {}", style::dim("\u{25cb}"), locale.t("cache.no_db"), db_path.display());
        println!("  {}", style::dim(locale.t("cache.run_first")));
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
        Some(CacheAction::Clear) => {
            match cache.clear() {
                Ok(()) => println!("  {} {}", style::check(), style::bold(locale.t("cache.cleared"))),
                Err(e) => eprintln!("  {} Error: {}", style::cross(), e),
            }
        }
        None => {
            show_cache_info(&cache, &locale);
        }
    }
}

fn show_cache_info(cache: &SqliteCache, locale: &Locale) {
    // Stats
    match cache.stats() {
        Ok(stats) => {
            println!();
            style::print_section(locale.t("cache.stats_title"));
            println!();
            style::print_kv(locale.t("cache.entries"), &stats.count.to_string());
            style::print_kv(locale.t("cache.size"), &format!("{} KB", stats.size_bytes / 1024));
            println!();
        }
        Err(e) => {
            eprintln!("  {} Error: {}", style::cross(), e);
            return;
        }
    }

    // Recent entries
    match cache.recent(10) {
        Ok(entries) => {
            if entries.is_empty() {
                println!("  {}", style::dim(locale.t("cache.no_entries")));
                return;
            }

            style::print_section(locale.t("cache.recent"));
            println!();
            for entry in &entries {
                let level_str = match entry.risk_level {
                    RiskLevel::Low => "LOW",
                    RiskLevel::Medium => "MED",
                    RiskLevel::High => "HI ",
                };

                let key_display = if entry.cache_key.len() > 50 {
                    format!("{}...", &entry.cache_key[..50])
                } else {
                    entry.cache_key.clone()
                };

                println!(
                    "    {} {}  {}",
                    style::risk_colored(level_str, entry.risk_level),
                    key_display,
                    style::dim(format!("hits:{}", entry.hit_count)),
                );
            }
            println!();
        }
        Err(e) => {
            eprintln!("  {} Error: {}", style::cross(), e);
        }
    }
}
