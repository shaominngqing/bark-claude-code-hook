use crate::config;
use crate::i18n::Locale;
use crate::ui::style;

/// Completely uninstall Bark.
pub fn run() {
    let locale = Locale::detect();

    println!();
    println!("  {} {}", style::danger("\u{25cf}"), style::bold(locale.t("uninstall.title")));
    println!();

    // 1. Remove hook from settings.json
    match config::disable_hook() {
        Ok(()) => print_removed(locale.t("uninstall.hook"), &locale),
        Err(e) => print_failed(locale.t("uninstall.hook"), &e.to_string(), &locale),
    }

    // 2. Remove cache database
    remove_if_exists(&config::bark_db_path(), locale.t("uninstall.cache_db"), &locale);
    remove_if_exists(&config::bark_db_path().with_extension("db-wal"), locale.t("uninstall.cache_wal"), &locale);
    remove_if_exists(&config::bark_db_path().with_extension("db-shm"), locale.t("uninstall.cache_shm"), &locale);

    // 3. Remove bark.toml
    remove_if_exists(&config::bark_toml_path(), locale.t("uninstall.custom_rules"), &locale);

    // 4. Remove log file
    remove_if_exists(&config::bark_log_path(), locale.t("uninstall.log_file"), &locale);

    // 5. Remove old bash hook remnants
    let hooks_dir = config::bark_dir();
    for name in &["bark.sh", "bark-ctl.sh", "bark.conf", "bark-logo.png", "bark-logo.svg"] {
        let p = hooks_dir.join(name);
        if p.exists() {
            std::fs::remove_file(&p).ok();
        }
    }
    let old_cache = hooks_dir.join("cache");
    if old_cache.is_dir() {
        std::fs::remove_dir_all(&old_cache).ok();
    }

    // 6. Remove daemon socket and pid
    remove_if_exists(&config::socket_path(), locale.t("uninstall.daemon_socket"), &locale);
    remove_if_exists(&config::pid_path(), locale.t("uninstall.daemon_pid"), &locale);

    // 7. Remove the bark binary itself
    let self_path = std::env::current_exe().ok();
    if let Some(ref exe) = self_path {
        let is_installed = !exe.to_string_lossy().contains("target/");
        if is_installed {
            match std::fs::remove_file(exe) {
                Ok(()) => println!("  {} {} {}: {}",
                    style::check(), locale.t("uninstall.removed"), locale.t("uninstall.binary").split(':').next().unwrap_or("binary"), exe.display()),
                Err(e) => println!("  {} {}: {}",
                    style::cross(), locale.t("uninstall.binary_fail"), e),
            }
            for dir in &["/usr/local/bin", "/opt/homebrew/bin"] {
                let link = std::path::Path::new(dir).join("bark");
                if link.exists() && link != *exe {
                    std::fs::remove_file(&link).ok();
                }
            }
        } else {
            println!("  {} {}", style::check(), locale.t("uninstall.skip_dev"));
        }
    }

    println!();
    println!("  {} {}", style::check(), style::bold(locale.t("uninstall.done")));
    println!();
}

fn print_removed(label: &str, _locale: &Locale) {
    println!("  {} Removed {}", style::check(), label);
}

fn print_failed(label: &str, error: &str, _locale: &Locale) {
    eprintln!("  {} Failed to remove {}: {}", style::cross(), label, error);
}

fn remove_if_exists(path: &std::path::Path, label: &str, locale: &Locale) {
    if path.exists() {
        match std::fs::remove_file(path) {
            Ok(()) => print_removed(label, locale),
            Err(e) => print_failed(label, &e.to_string(), locale),
        }
    }
}
