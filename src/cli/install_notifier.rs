use std::io::Write;
use std::process::Command;

use crate::config;
use crate::i18n::Locale;
use crate::ui::style;

const REPO: &str = "shaominngqing/bark-claude-code-hook";

pub fn run() {
    let locale = Locale::detect();

    println!();
    style::print_section(locale.t("notifier.title"));
    println!();

    // Step 1: Platform check
    style::print_step(locale.t("notifier.detect"));

    #[cfg(not(target_os = "macos"))]
    {
        style::print_warn(locale.t("notifier.macos_only"));
        println!();
        return;
    }

    #[cfg(target_os = "macos")]
    {
        let arch = if cfg!(target_arch = "aarch64") {
            "aarch64"
        } else {
            "x86_64"
        };
        style::print_ok(&format!("macOS {}", arch));

        let app_path = config::notifier_app_path();

        if app_path.exists() {
            style::print_ok(locale.t("notifier.already"));
            offer_start(&app_path, &locale);
            return;
        }

        // Step 2: Download or build
        let installed = try_download(&app_path, arch, &locale)
            || try_build_from_source(&app_path, &locale);

        if !installed {
            style::print_err("Failed to install BarkNotifier.app");
            return;
        }

        // Step 3: Clear quarantine + ad-hoc sign
        style::print_step(locale.t("notifier.quarantine"));
        let xattr_ok = Command::new("xattr")
            .args(["-cr", &app_path.to_string_lossy()])
            .status()
            .map(|s| s.success())
            .unwrap_or(false);
        if xattr_ok {
            style::print_ok("xattr -cr");
        } else {
            style::print_warn("xattr -cr failed (may need manual clear)");
        }

        // Ad-hoc code sign (required for notification permission prompt)
        let sign_ok = Command::new("codesign")
            .args(["--force", "--deep", "--sign", "-", &app_path.to_string_lossy()])
            .status()
            .map(|s| s.success())
            .unwrap_or(false);
        if sign_ok {
            style::print_ok("codesign (ad-hoc)");
        } else {
            style::print_warn("codesign failed (notifications may not work)");
        }

        // Step 4: Register LaunchAgent
        style::print_step(locale.t("notifier.launchd"));
        register_launchd(&app_path, &locale);

        // Done
        println!();
        print_completion_banner(&locale);

        offer_start(&app_path, &locale);
    }
}

#[cfg(target_os = "macos")]
fn try_download(
    app_path: &std::path::Path,
    arch: &str,
    locale: &Locale,
) -> bool {
    style::print_step(locale.t("notifier.download"));

    let version = env!("CARGO_PKG_VERSION");
    let url = format!(
        "https://github.com/{}/releases/download/v{}/BarkNotifier-macos.zip",
        REPO, version
    );

    let tmp_dir = std::env::temp_dir().join("bark-notifier-install");
    std::fs::create_dir_all(&tmp_dir).ok();
    let zip_path = tmp_dir.join("BarkNotifier.zip");

    // Download
    let download_ok = Command::new("curl")
        .args(["-fsSL", &url, "-o", &zip_path.to_string_lossy()])
        .status()
        .map(|s| s.success())
        .unwrap_or(false);

    if !download_ok || !zip_path.exists() || std::fs::metadata(&zip_path).map(|m| m.len()).unwrap_or(0) == 0 {
        style::print_warn("No pre-built binary available");
        std::fs::remove_dir_all(&tmp_dir).ok();
        return false;
    }

    // Unzip
    let extract_dir = tmp_dir.join("extracted");
    std::fs::create_dir_all(&extract_dir).ok();
    let unzip_ok = Command::new("unzip")
        .args(["-q", "-o", &zip_path.to_string_lossy(), "-d", &extract_dir.to_string_lossy()])
        .status()
        .map(|s| s.success())
        .unwrap_or(false);

    if !unzip_ok {
        style::print_warn("Failed to unzip");
        std::fs::remove_dir_all(&tmp_dir).ok();
        return false;
    }

    // Move to ~/Applications/
    let extracted_app = extract_dir.join("BarkNotifier.app");
    if !extracted_app.exists() {
        style::print_warn("BarkNotifier.app not found in zip");
        std::fs::remove_dir_all(&tmp_dir).ok();
        return false;
    }

    style::print_step(locale.t("notifier.install"));
    let parent = app_path.parent().unwrap();
    std::fs::create_dir_all(parent).ok();

    // Remove existing if any
    if app_path.exists() {
        std::fs::remove_dir_all(app_path).ok();
    }

    let cp_ok = Command::new("cp")
        .args(["-r", &extracted_app.to_string_lossy(), &app_path.to_string_lossy()])
        .status()
        .map(|s| s.success())
        .unwrap_or(false);

    std::fs::remove_dir_all(&tmp_dir).ok();

    if cp_ok {
        style::print_ok(&format!("{}", app_path.display()));
        true
    } else {
        style::print_warn("Failed to copy to ~/Applications");
        false
    }
}

#[cfg(target_os = "macos")]
fn try_build_from_source(
    app_path: &std::path::Path,
    locale: &Locale,
) -> bool {
    style::print_step(locale.t("notifier.build"));
    style::print_warn(locale.t("notifier.build_hint"));

    // Check if swiftc is available
    let swiftc_ok = Command::new("swiftc")
        .arg("--version")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false);

    if !swiftc_ok {
        style::print_err("swiftc not found. Install Xcode Command Line Tools: xcode-select --install");
        return false;
    }

    // Find the build script relative to the bark binary or try known locations
    let build_script = find_build_script();

    if let Some(script) = build_script {
        let build_ok = Command::new("bash")
            .arg(&script)
            .status()
            .map(|s| s.success())
            .unwrap_or(false);

        if !build_ok {
            style::print_err("Build failed");
            return false;
        }

        // The build script puts it in notifier/build/BarkNotifier.app
        let build_dir = script.parent().unwrap().join("build").join("BarkNotifier.app");
        if build_dir.exists() {
            style::print_step(locale.t("notifier.install"));
            let parent = app_path.parent().unwrap();
            std::fs::create_dir_all(parent).ok();

            if app_path.exists() {
                std::fs::remove_dir_all(app_path).ok();
            }

            let cp_ok = Command::new("cp")
                .args(["-r", &build_dir.to_string_lossy(), &app_path.to_string_lossy()])
                .status()
                .map(|s| s.success())
                .unwrap_or(false);

            if cp_ok {
                style::print_ok(&format!("{}", app_path.display()));
                return true;
            }
        }
    }

    style::print_err("Could not find notifier build script. Try building manually from the repo.");
    false
}

#[cfg(target_os = "macos")]
fn find_build_script() -> Option<std::path::PathBuf> {
    // Try relative to the current executable
    if let Ok(exe) = std::env::current_exe() {
        // If installed from repo: binary might be in target/release/bark
        // build script would be at repo_root/notifier/build.sh
        let mut path = exe.clone();
        for _ in 0..5 {
            path = match path.parent() {
                Some(p) => p.to_path_buf(),
                None => break,
            };
            let candidate = path.join("notifier").join("build.sh");
            if candidate.exists() {
                return Some(candidate);
            }
        }
    }

    // Try current working directory
    if let Ok(cwd) = std::env::current_dir() {
        let candidate = cwd.join("notifier").join("build.sh");
        if candidate.exists() {
            return Some(candidate);
        }
    }

    None
}

#[cfg(target_os = "macos")]
fn register_launchd(app_path: &std::path::Path, _locale: &Locale) {
    let plist_path = config::notifier_launchd_plist_path();
    let executable = app_path.join("Contents").join("MacOS").join("BarkNotifier");

    let plist_content = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>com.bark.notifier</string>
    <key>ProgramArguments</key>
    <array>
        <string>{}</string>
    </array>
    <key>RunAtLoad</key>
    <true/>
    <key>KeepAlive</key>
    <true/>
</dict>
</plist>"#,
        executable.display()
    );

    if let Some(parent) = plist_path.parent() {
        std::fs::create_dir_all(parent).ok();
    }

    match std::fs::write(&plist_path, &plist_content) {
        Ok(()) => style::print_ok(&format!("{}", plist_path.display())),
        Err(e) => {
            style::print_warn(&format!("Failed to write plist: {}", e));
            return;
        }
    }

    // Load the launch agent
    Command::new("launchctl")
        .args(["unload", &plist_path.to_string_lossy()])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .ok();

    Command::new("launchctl")
        .args(["load", &plist_path.to_string_lossy()])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .ok();
}

#[cfg(target_os = "macos")]
fn print_completion_banner(locale: &Locale) {
    let banner_width = 44;
    let top = format!("  \u{256d}{}\u{256e}", "\u{2500}".repeat(banner_width));
    let bot = format!("  \u{2570}{}\u{256f}", "\u{2500}".repeat(banner_width));
    let sep = format!("  \u{251c}{}\u{2524}", "\u{2500}".repeat(banner_width));
    println!("{}", style::dim(&top));

    let title = locale.t("notifier.complete");
    let title_w: usize = title.chars().map(|c| if c > '\u{FF}' { 2 } else { 1 }).sum();
    let title_pad = banner_width.saturating_sub(title_w + 1);
    println!(
        "  {}  {}{}{}",
        style::dim("\u{2502}"),
        style::gradient(title),
        " ".repeat(title_pad),
        style::dim("\u{2502}"),
    );
    println!("{}", style::dim(&sep));

    let features = [
        locale.t("notifier.icon"),
        locale.t("notifier.buttons"),
        locale.t("notifier.fallback"),
    ];

    for feat in &features {
        let w: usize = feat.chars().map(|c| if c > '\u{FF}' { 2 } else { 1 }).sum();
        let pad = banner_width.saturating_sub(w + 5);
        println!(
            "  {}  {} {}{}{}",
            style::dim("\u{2502}"),
            style::check(),
            feat,
            " ".repeat(pad),
            style::dim("\u{2502}"),
        );
    }

    println!("{}", style::dim(&bot));
}

#[cfg(target_os = "macos")]
fn offer_start(app_path: &std::path::Path, locale: &Locale) {
    println!();
    print!("  {} [Y/n] ", locale.t("notifier.start_prompt"));
    std::io::stdout().flush().ok();

    // Read from /dev/tty directly — stdin may be a pipe when called from install.sh
    let tty = std::fs::File::open("/dev/tty").ok();
    let mut input = String::new();
    let read_ok = if let Some(mut tty) = tty {
        use std::io::BufRead;
        std::io::BufReader::new(&mut tty).read_line(&mut input).is_ok()
    } else {
        std::io::stdin().read_line(&mut input).is_ok()
    };
    if read_ok {
        let answer = input.trim().to_lowercase();
        if answer.is_empty() || answer == "y" || answer == "yes" {
            Command::new("open")
                .arg("-a")
                .arg(app_path)
                .spawn()
                .ok();
            println!("  {} {}", style::check(), locale.t("notifier.started"));
        }
    }
    println!();
}
