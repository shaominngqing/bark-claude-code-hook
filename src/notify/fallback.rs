use std::process::Command;

/// Send a desktop notification using platform-native commands.
///
/// - macOS: `osascript` (always available, zero config)
/// - Linux: `notify-send` (part of libnotify)
/// - Windows: PowerShell toast notification
///
/// Fire-and-forget: spawns a background process, returns immediately.
pub fn notify(title: &str, subtitle: &str, body: &str, sound: Option<&str>) {
    #[cfg(target_os = "macos")]
    notify_macos(title, subtitle, body, sound);

    #[cfg(target_os = "linux")]
    notify_linux(title, subtitle, body);

    #[cfg(target_os = "windows")]
    notify_windows(title, subtitle, body);

    #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
    {
        let _ = (title, subtitle, body, sound);
    }
}

#[cfg(target_os = "macos")]
fn notify_macos(title: &str, subtitle: &str, body: &str, sound: Option<&str>) {
    let sound_part = match sound {
        Some(name) => format!(" sound name \"{}\"", name),
        None => String::new(),
    };

    let script = format!(
        "display notification \"{}\" with title \"{}\" subtitle \"{}\"{}",
        escape_applescript(body),
        escape_applescript(title),
        escape_applescript(subtitle),
        sound_part,
    );

    // Fire and forget
    Command::new("osascript")
        .arg("-e")
        .arg(&script)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .ok();
}

#[cfg(target_os = "macos")]
fn escape_applescript(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}

#[cfg(target_os = "linux")]
fn notify_linux(title: &str, subtitle: &str, body: &str) {
    let summary = format!("{} — {}", title, subtitle);

    Command::new("notify-send")
        .arg("--app-name=Bark")
        .arg(&summary)
        .arg(body)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .ok();
}

#[cfg(target_os = "windows")]
fn notify_windows(title: &str, subtitle: &str, body: &str) {
    let message = format!("{} — {}", subtitle, body);
    let script = format!(
        "[Windows.UI.Notifications.ToastNotificationManager, Windows.UI.Notifications, ContentType = WindowsRuntime] > $null; \
         $xml = [Windows.UI.Notifications.ToastNotificationManager]::GetTemplateContent([Windows.UI.Notifications.ToastTemplateType]::ToastText02); \
         $text = $xml.GetElementsByTagName('text'); \
         $text[0].AppendChild($xml.CreateTextNode('{}')) > $null; \
         $text[1].AppendChild($xml.CreateTextNode('{}')) > $null; \
         $toast = [Windows.UI.Notifications.ToastNotification]::new($xml); \
         [Windows.UI.Notifications.ToastNotificationManager]::CreateToastNotifier('Bark').Show($toast)",
        title.replace('\'', "''"),
        message.replace('\'', "''"),
    );

    Command::new("powershell")
        .arg("-NoProfile")
        .arg("-Command")
        .arg(&script)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .ok();
}
