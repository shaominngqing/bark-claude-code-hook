use crate::cache::SqliteCache;
use crate::config;
use crate::ui::gradient::{BOLD, DIM, NC};
use crate::ui::logo::print_logo;
use crate::ui::cards::print_status_line;

/// Show bark status: hook registration, cache count, log count.
pub fn run() {
    println!();
    print_logo();
    println!();

    // Check if hook is registered
    let hook_active = config::has_hook();
    print_status_line(hook_active);
    println!();

    // Show version
    let version = env!("CARGO_PKG_VERSION");
    println!("  {}Version{}  {}{}{}", DIM, NC, BOLD, version, NC);

    // Show cache stats if DB exists
    let db_path = config::bark_db_path();
    if db_path.exists() {
        if let Ok(cache) = SqliteCache::open(&db_path) {
            if let Ok(stats) = cache.stats() {
                println!(
                    "  {}Cache{}    {} entries ({} KB)",
                    DIM,
                    NC,
                    stats.count,
                    stats.size_bytes / 1024
                );
            }
            if let Ok(log_stats) = cache.get_stats() {
                println!(
                    "  {}Log{}      {} assessments logged",
                    DIM, NC, log_stats.total
                );
            }
        }
    }

    // Show settings path
    let settings = config::settings_path();
    println!(
        "  {}Settings{} {}",
        DIM, NC, settings.display()
    );

    // Show rules path
    let rules = config::bark_toml_path();
    let exists = rules.exists();
    println!(
        "  {}Rules{}    {} {}",
        DIM,
        NC,
        rules.display(),
        if exists { "" } else { "(not created)" }
    );

    println!();
}
