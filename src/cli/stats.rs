use crossterm::style::Color;

use crate::cache::SqliteCache;
use crate::config;
use crate::i18n::Locale;
use crate::ui::logo;
use crate::ui::style::{self, C2, ORANGE, PURPLE, ACCENT};

/// Show the statistics dashboard.
pub fn run() {
    let locale = Locale::detect();
    let db_path = config::bark_db_path();

    if !db_path.exists() {
        println!();
        logo::print_banner();
        println!();
        println!("  {}", locale.t("stats.no_data"));
        println!("  {}", style::dim(locale.t("stats.run_first")));
        println!();
        return;
    }

    let cache = match SqliteCache::open(&db_path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("  {} Error: {}", style::cross(), e);
            return;
        }
    };

    let stats = match cache.get_stats() {
        Ok(s) => s,
        Err(e) => {
            eprintln!("  {} Error: {}", style::cross(), e);
            return;
        }
    };

    let cache_stats = cache.stats().ok();

    // Header
    println!();
    logo::print_banner();
    println!();

    // Total assessments card
    style::print_card(locale.t("stats.total"), &stats.total.to_string(), None);
    println!();

    if stats.total == 0 {
        println!("  {}", locale.t("stats.no_assessments"));
        println!();
        return;
    }

    // Source breakdown
    style::print_section(locale.t("stats.source_breakdown"));
    println!();

    let source_colors: &[(&str, Color)] = &[
        ("FAST", Color::Green),
        ("CACHE", C2),
        ("RULE", PURPLE),
        ("AST", Color::Green),
        ("AI", ORANGE),
        ("CHAIN", Color::Yellow),
        ("FALLBACK", Color::DarkGrey),
        ("PLUGIN", ACCENT),
    ];

    for (source, color) in source_colors {
        let count = stats.by_source.get(*source).copied().unwrap_or(0);
        if count > 0 {
            let bar = style::progress_bar(count, stats.total, 20, *color);
            let pct = (count as f64 / stats.total as f64) * 100.0;
            println!(
                "    {:<8} {} {:>4} {}",
                source, bar, count,
                style::dim(format!("({:.0}%)", pct)),
            );
        }
    }
    println!();

    // Risk distribution
    style::print_section(locale.t("stats.risk_distribution"));
    println!();

    let level_colors: &[(&str, Color)] = &[
        ("LOW", Color::Green),
        ("MEDIUM", Color::Yellow),
        ("HIGH", Color::Red),
    ];

    for (level, color) in level_colors {
        let count = stats.by_level.get(*level).copied().unwrap_or(0);
        let bar = style::progress_bar(count, stats.total, 20, *color);
        let pct = if stats.total > 0 {
            (count as f64 / stats.total as f64) * 100.0
        } else {
            0.0
        };
        println!(
            "    {:<8} {} {:>4} {}",
            level, bar, count,
            style::dim(format!("({:.0}%)", pct)),
        );
    }
    println!();

    // Cache hit rate
    let hit_rate_pct = stats.cache_hit_rate * 100.0;
    let hit_level = if hit_rate_pct > 50.0 {
        crate::core::risk::RiskLevel::Low
    } else if hit_rate_pct > 20.0 {
        crate::core::risk::RiskLevel::Medium
    } else {
        crate::core::risk::RiskLevel::High
    };
    style::print_card(locale.t("stats.cache_hit_rate"), &format!("{:.1}%", hit_rate_pct), Some(hit_level));

    // Cache size
    if let Some(cs) = cache_stats {
        println!();
        println!("  {} {} {}, {} KB",
            style::dim(locale.t("stats.cache_label")),
            cs.count,
            locale.t("status.entries"),
            cs.size_bytes / 1024,
        );
    }

    println!();
}
