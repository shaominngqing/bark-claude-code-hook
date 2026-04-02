use std::process::Command;

use crate::config;
use crate::i18n::Locale;
use crate::ui::style;

const REPO: &str = "shaominngqing/bark-claude-code-hook";

/// Self-update bark binary and BarkNotifier.app to the latest release.
pub fn run() {
    let locale = Locale::detect();
    let current_version = env!("CARGO_PKG_VERSION");

    println!();
    style::print_section(locale.t("update.title"));
    println!();

    // Step 1: Fetch latest version from GitHub
    style::print_step(locale.t("update.check"));
    let latest = match fetch_latest_version() {
        Some(v) => v,
        None => {
            style::print_err(locale.t("update.fetch_failed"));
            return;
        }
    };

    println!("  {} {} → {}", style::dim("current:"), current_version, latest);

    if latest == current_version {
        style::print_ok(locale.t("update.already_latest"));
        println!();
        return;
    }

    // Step 2: Detect platform
    let target = detect_target();
    let Some(target) = target else {
        style::print_err("Unsupported platform");
        return;
    };

    // Step 3: Download new binary
    style::print_step(locale.t("update.download"));
    let exe_suffix = if cfg!(target_os = "windows") { ".exe" } else { "" };
    let url = format!(
        "https://github.com/{}/releases/download/v{}/bark-{}{}",
        REPO, latest, target, exe_suffix
    );

    let tmp_dir = std::env::temp_dir().join("bark-update");
    std::fs::create_dir_all(&tmp_dir).ok();
    let tmp_bin = tmp_dir.join(format!("bark{}", exe_suffix));

    let download_ok = Command::new("curl")
        .args(["-fsSL", &url, "-o", &tmp_bin.to_string_lossy()])
        .status()
        .map(|s| s.success())
        .unwrap_or(false);

    if !download_ok || !tmp_bin.exists() || std::fs::metadata(&tmp_bin).map(|m| m.len()).unwrap_or(0) == 0 {
        style::print_err(locale.t("update.download_failed"));
        std::fs::remove_dir_all(&tmp_dir).ok();
        return;
    }

    // Step 4: Replace binary
    style::print_step(locale.t("update.install"));

    // Find current binary path
    let current_exe = std::env::current_exe().ok();
    let install_path = current_exe
        .as_ref()
        .filter(|p| !p.to_string_lossy().contains("target/"))
        .cloned()
        .unwrap_or_else(|| {
            // Fallback: find bark in common paths
            for dir in &["/opt/homebrew/bin", "/usr/local/bin"] {
                let p = std::path::PathBuf::from(dir).join("bark");
                if p.exists() { return p; }
            }
            std::path::PathBuf::from("/usr/local/bin/bark")
        });

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&tmp_bin, std::fs::Permissions::from_mode(0o755)).ok();
    }

    let replace_ok = if std::fs::copy(&tmp_bin, &install_path).is_ok() {
        true
    } else {
        // Try with sudo
        Command::new("sudo")
            .args(["cp", &tmp_bin.to_string_lossy(), &install_path.to_string_lossy()])
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
    };

    if !replace_ok {
        style::print_err(&format!("Failed to replace {}", install_path.display()));
        std::fs::remove_dir_all(&tmp_dir).ok();
        return;
    }

    // Clear quarantine and re-sign (macOS)
    #[cfg(target_os = "macos")]
    {
        Command::new("xattr")
            .args(["-cr", &install_path.to_string_lossy()])
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status()
            .ok();
        Command::new("codesign")
            .args(["--force", "--sign", "-", &install_path.to_string_lossy()])
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status()
            .ok();
    }

    style::print_ok(&format!("bark v{} → {}", current_version, latest));

    // Step 5: Update BarkNotifier if installed (macOS)
    #[cfg(target_os = "macos")]
    {
        let app_path = config::notifier_app_path();
        if app_path.exists() {
            style::print_step(locale.t("update.notifier"));
            update_notifier(&app_path, &latest, &locale);
        }
    }

    std::fs::remove_dir_all(&tmp_dir).ok();

    println!();
    style::print_ok(&format!("{} v{}", style::bold(locale.t("update.complete")), latest));
    println!();
}

fn fetch_latest_version() -> Option<String> {
    // Try gh CLI first (authenticated, no rate limit)
    if let Ok(output) = Command::new("gh")
        .args(["release", "view", "--repo", REPO, "--json", "tagName", "--jq", ".tagName"])
        .output()
    {
        if output.status.success() {
            let tag = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !tag.is_empty() {
                return Some(tag.strip_prefix('v').unwrap_or(&tag).to_string());
            }
        }
    }

    // Fallback: curl GitHub API
    let output = Command::new("curl")
        .args(["-fsSL", "--connect-timeout", "10",
               &format!("https://api.github.com/repos/{}/releases/latest", REPO)])
        .output()
        .ok()?;

    if !output.status.success() { return None; }

    let body = String::from_utf8_lossy(&output.stdout);
    let tag_start = body.find("\"tag_name\"")?;
    let rest = &body[tag_start..];
    let v_start = rest.find('"')? + 1;
    let rest = &rest[v_start..];
    let v_start = rest.find('"')? + 1;
    let rest = &rest[v_start..];
    let v_end = rest.find('"')?;
    let tag = &rest[..v_end];

    Some(tag.strip_prefix('v').unwrap_or(tag).to_string())
}

fn detect_target() -> Option<String> {
    let os = if cfg!(target_os = "macos") {
        "darwin"
    } else if cfg!(target_os = "linux") {
        "linux"
    } else if cfg!(target_os = "windows") {
        "windows"
    } else {
        return None;
    };

    let arch = if cfg!(target_arch = "aarch64") {
        "aarch64"
    } else if cfg!(target_arch = "x86_64") {
        "x86_64"
    } else {
        return None;
    };

    Some(format!("{}-{}", os, arch))
}

#[cfg(target_os = "macos")]
fn update_notifier(app_path: &std::path::Path, version: &str, locale: &Locale) {
    let url = format!(
        "https://github.com/{}/releases/download/v{}/BarkNotifier-macos.zip",
        REPO, version
    );

    let tmp_dir = std::env::temp_dir().join("bark-notifier-update");
    std::fs::create_dir_all(&tmp_dir).ok();
    let zip_path = tmp_dir.join("BarkNotifier.zip");

    // Download
    let ok = Command::new("curl")
        .args(["-fsSL", &url, "-o", &zip_path.to_string_lossy()])
        .status()
        .map(|s| s.success())
        .unwrap_or(false);

    if !ok || std::fs::metadata(&zip_path).map(|m| m.len()).unwrap_or(0) == 0 {
        style::print_warn("BarkNotifier download failed, skipping");
        std::fs::remove_dir_all(&tmp_dir).ok();
        return;
    }

    // Kill running notifier
    Command::new("pkill")
        .args(["-f", "BarkNotifier"])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .ok();
    std::thread::sleep(std::time::Duration::from_millis(500));

    // Unzip
    let extract_dir = tmp_dir.join("extracted");
    std::fs::create_dir_all(&extract_dir).ok();
    let unzip_ok = Command::new("unzip")
        .args(["-q", "-o", &zip_path.to_string_lossy(), "-d", &extract_dir.to_string_lossy()])
        .status()
        .map(|s| s.success())
        .unwrap_or(false);

    if !unzip_ok {
        style::print_warn("Failed to unzip BarkNotifier");
        std::fs::remove_dir_all(&tmp_dir).ok();
        return;
    }

    // Replace
    let extracted_app = extract_dir.join("BarkNotifier.app");
    if extracted_app.exists() {
        std::fs::remove_dir_all(app_path).ok();
        Command::new("cp")
            .args(["-r", &extracted_app.to_string_lossy(), &app_path.to_string_lossy()])
            .status()
            .ok();

        Command::new("xattr")
            .args(["-cr", &app_path.to_string_lossy()])
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status()
            .ok();

        Command::new("codesign")
            .args(["--force", "--deep", "--sign", "-", &app_path.to_string_lossy()])
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status()
            .ok();

        // Restart
        Command::new("open")
            .arg("-a")
            .arg(app_path)
            .spawn()
            .ok();

        style::print_ok(&format!("BarkNotifier v{}", version));
    }

    std::fs::remove_dir_all(&tmp_dir).ok();
}
