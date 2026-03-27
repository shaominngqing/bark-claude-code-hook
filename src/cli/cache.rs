use crate::cache::SqliteCache;
use crate::cli::CacheAction;
use crate::config;
use crate::core::risk::RiskLevel;
use crate::ui::gradient::{BOLD, DIM, GREEN, NC, RED, YELLOW};

/// View or clear the assessment cache.
pub fn run(action: Option<CacheAction>) {
    let db_path = config::bark_db_path();

    if !db_path.exists() {
        println!("  No cache database found at {}", db_path.display());
        println!("  Run some assessments first to populate the cache.");
        return;
    }

    let cache = match SqliteCache::open(&db_path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("  Error opening cache: {}", e);
            return;
        }
    };

    match action {
        Some(CacheAction::Clear) => {
            match cache.clear() {
                Ok(()) => {
                    println!("  {}{}Cache cleared.{}", GREEN, BOLD, NC);
                }
                Err(e) => {
                    eprintln!("  Error clearing cache: {}", e);
                }
            }
        }
        None => {
            // Show cache stats and recent entries
            show_cache_info(&cache);
        }
    }
}

fn show_cache_info(cache: &SqliteCache) {
    // Stats
    match cache.stats() {
        Ok(stats) => {
            println!();
            println!(
                "  {}Cache Statistics{}",
                BOLD, NC
            );
            println!(
                "  {}Entries{}  {}",
                DIM, NC, stats.count
            );
            println!(
                "  {}Size{}    {} KB",
                DIM, NC, stats.size_bytes / 1024
            );
            println!();
        }
        Err(e) => {
            eprintln!("  Error reading cache stats: {}", e);
            return;
        }
    }

    // Recent entries
    match cache.recent(10) {
        Ok(entries) => {
            if entries.is_empty() {
                println!("  No cached entries.");
                return;
            }

            println!("  {}Recent entries:{}", BOLD, NC);
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

                // Truncate the cache key for display
                let key_display = if entry.cache_key.len() > 50 {
                    format!("{}...", &entry.cache_key[..50])
                } else {
                    entry.cache_key.clone()
                };

                println!(
                    "  {}{}{}{}  {}  {}hits:{}{}{}  {}",
                    color, BOLD, level_str, NC,
                    key_display,
                    DIM, NC,
                    entry.hit_count,
                    DIM,
                    NC
                );
            }
            println!();
        }
        Err(e) => {
            eprintln!("  Error reading recent entries: {}", e);
        }
    }
}
