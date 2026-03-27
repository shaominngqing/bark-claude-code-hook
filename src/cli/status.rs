use crate::cache::SqliteCache;
use crate::config;
use crate::i18n::Locale;
use crate::ui::logo;
use crate::ui::style;

/// Show bark status: hook registration, cache count, log count.
pub fn run() {
    let locale = Locale::detect();

    println!();
    logo::print_banner();
    println!();

    // Check if hook is registered
    let hook_active = config::has_hook();
    if hook_active {
        println!("  {} {}  {} {}",
            style::status_dot(true),
            style::bold(locale.t("status.active")),
            style::dim("\u{2500}\u{2500}"),
            style::dim(locale.t("status.active_hint")),
        );
    } else {
        println!("  {} {}  {} {}",
            style::status_dot(false),
            style::bold(locale.t("status.inactive")),
            style::dim("\u{2500}\u{2500}"),
            style::dim(locale.t("status.inactive_hint")),
        );
    }
    println!();

    // Show version
    let version = env!("CARGO_PKG_VERSION");
    style::print_kv(locale.t("status.version"), version);

    // Show cache stats if DB exists
    let db_path = config::bark_db_path();
    if db_path.exists() {
        if let Ok(cache) = SqliteCache::open(&db_path) {
            if let Ok(stats) = cache.stats() {
                style::print_kv(
                    locale.t("status.cache"),
                    &format!("{} {} ({} KB)", stats.count, locale.t("status.entries"), stats.size_bytes / 1024),
                );
            }
            if let Ok(log_stats) = cache.get_stats() {
                style::print_kv(
                    locale.t("status.log"),
                    &format!("{} {}", log_stats.total, locale.t("status.assessments_logged")),
                );
            }
        }
    }

    // Show settings path
    let settings = config::settings_path();
    style::print_kv(locale.t("status.settings"), &settings.display().to_string());

    // Show rules path
    let rules = config::bark_toml_path();
    let exists = rules.exists();
    let rules_val = if exists {
        rules.display().to_string()
    } else {
        format!("{} {}", rules.display(), locale.t("status.not_created"))
    };
    style::print_kv(locale.t("status.rules"), &rules_val);

    println!();
}
