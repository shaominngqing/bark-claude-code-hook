use crate::cache::SqliteCache;
use crate::config;
use crate::ui::cards::{print_card, progress_bar};
use crate::ui::gradient::{fg256, BOLD, DIM, GREEN, NC, RED, YELLOW};
use crate::ui::logo::print_logo;

/// Color constants matching the bark theme for sources.
const C2_COLOR: &str = "\x1b[38;5;45m";  // Deep sky blue for CACHE
const ORANGE_COLOR: &str = "\x1b[38;5;208m"; // Orange for AI

/// Show the statistics dashboard.
pub fn run() {
    let db_path = config::bark_db_path();

    if !db_path.exists() {
        println!();
        print_logo();
        println!();
        println!("  No assessment data yet.");
        println!("  Run some assessments to see statistics.");
        println!();
        return;
    }

    let cache = match SqliteCache::open(&db_path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("  Error opening database: {}", e);
            return;
        }
    };

    let stats = match cache.get_stats() {
        Ok(s) => s,
        Err(e) => {
            eprintln!("  Error reading stats: {}", e);
            return;
        }
    };

    let cache_stats = cache.stats().ok();

    // Header
    println!();
    print_logo();
    println!();

    // Total assessments card
    print_card("Total Assessments", &stats.total.to_string(), GREEN);
    println!();

    if stats.total == 0 {
        println!("  No assessments recorded yet.");
        println!();
        return;
    }

    // Source breakdown
    println!("  {}Source Breakdown{}", BOLD, NC);
    println!();

    let source_colors: &[(&str, &str)] = &[
        ("FAST", GREEN),
        ("CACHE", C2_COLOR),
        ("RULE", &fg256(141)),
        ("AST", GREEN),
        ("AI", ORANGE_COLOR),
        ("CHAIN", YELLOW),
        ("FALLBACK", DIM),
        ("PLUGIN", &fg256(213)),
    ];

    for (source, color) in source_colors {
        let count = stats.by_source.get(*source).copied().unwrap_or(0);
        if count > 0 {
            let bar = progress_bar(count, stats.total, 20, color);
            let pct = (count as f64 / stats.total as f64) * 100.0;
            println!(
                "  {:<8} {} {:>4} ({:.0}%)",
                source, bar, count, pct
            );
        }
    }
    println!();

    // Risk distribution
    println!("  {}Risk Distribution{}", BOLD, NC);
    println!();

    let level_colors: &[(&str, &str)] = &[
        ("LOW", GREEN),
        ("MEDIUM", YELLOW),
        ("HIGH", RED),
    ];

    for (level, color) in level_colors {
        let count = stats.by_level.get(*level).copied().unwrap_or(0);
        let bar = progress_bar(count, stats.total, 20, color);
        let pct = if stats.total > 0 {
            (count as f64 / stats.total as f64) * 100.0
        } else {
            0.0
        };
        println!(
            "  {:<8} {} {:>4} ({:.0}%)",
            level, bar, count, pct
        );
    }
    println!();

    // Cache hit rate
    let hit_rate_pct = stats.cache_hit_rate * 100.0;
    let hit_rate_color = if hit_rate_pct > 50.0 {
        GREEN
    } else if hit_rate_pct > 20.0 {
        YELLOW
    } else {
        RED
    };
    print_card("Cache Hit Rate", &format!("{:.1}%", hit_rate_pct), hit_rate_color);

    // Cache size
    if let Some(cs) = cache_stats {
        println!();
        println!(
            "  {}Cache{}  {} entries, {} KB",
            DIM, NC, cs.count, cs.size_bytes / 1024
        );
    }

    println!();
}
