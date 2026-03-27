use crate::config;
use crate::ui::gradient::{BOLD, GREEN, NC, RED};

/// Completely uninstall Bark.
pub fn run() {
    println!();
    println!("  {}{}Uninstalling Bark...{}", RED, BOLD, NC);
    println!();

    // 1. Remove hook from settings.json
    match config::disable_hook() {
        Ok(()) => println!("  {} Removed hook from settings.json", check_mark()),
        Err(e) => eprintln!("  {} Failed to remove hook: {}", cross_mark(), e),
    }

    // 2. Remove cache database
    remove_if_exists(&config::bark_db_path(), "cache database");
    // Also remove WAL/SHM files left by SQLite
    remove_if_exists(&config::bark_db_path().with_extension("db-wal"), "cache WAL");
    remove_if_exists(&config::bark_db_path().with_extension("db-shm"), "cache SHM");

    // 3. Remove bark.toml
    remove_if_exists(&config::bark_toml_path(), "custom rules");

    // 4. Remove log file
    remove_if_exists(&config::bark_log_path(), "log file");

    // 5. Remove old bash hook remnants
    let hooks_dir = config::bark_dir();
    for name in &["bark.sh", "bark-ctl.sh", "bark.conf", "bark-logo.png", "bark-logo.svg"] {
        let p = hooks_dir.join(name);
        if p.exists() {
            std::fs::remove_file(&p).ok();
        }
    }
    // Remove old file-based cache directory
    let old_cache = hooks_dir.join("cache");
    if old_cache.is_dir() {
        std::fs::remove_dir_all(&old_cache).ok();
    }

    // 6. Remove daemon socket and pid
    remove_if_exists(&config::socket_path(), "daemon socket");
    remove_if_exists(&config::pid_path(), "daemon PID file");

    // 7. Remove the bark binary itself
    let self_path = std::env::current_exe().ok();
    if let Some(ref exe) = self_path {
        // Only delete if it's outside the build directory (i.e., installed copy)
        let is_installed = !exe.to_string_lossy().contains("target/");
        if is_installed {
            match std::fs::remove_file(exe) {
                Ok(()) => println!("  {} Removed binary: {}", check_mark(), exe.display()),
                Err(e) => eprintln!("  {} Failed to remove binary: {}", cross_mark(), e),
            }
            // Also remove symlinks in common bin dirs
            for dir in &["/usr/local/bin", "/opt/homebrew/bin"] {
                let link = std::path::Path::new(dir).join("bark");
                if link.exists() && link != *exe {
                    std::fs::remove_file(&link).ok();
                }
            }
        } else {
            println!("  {} Skipped binary removal (development build)", check_mark());
        }
    }

    println!();
    println!("  {}{}Bark has been fully uninstalled.{}", GREEN, BOLD, NC);
    println!();
}

fn remove_if_exists(path: &std::path::Path, label: &str) {
    if path.exists() {
        match std::fs::remove_file(path) {
            Ok(()) => println!("  {} Removed {}", check_mark(), label),
            Err(e) => eprintln!("  {} Failed to remove {}: {}", cross_mark(), label, e),
        }
    }
}

fn check_mark() -> &'static str {
    "\x1b[0;32m\u{2714}\x1b[0m"
}

fn cross_mark() -> &'static str {
    "\x1b[0;31m\u{2718}\x1b[0m"
}
