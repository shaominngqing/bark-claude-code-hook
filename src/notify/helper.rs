use std::path::PathBuf;
use std::time::Duration;

use anyhow::{Context, Result};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixStream;

use crate::core::risk::Assessment;
use crate::i18n::Locale;

use super::helper_protocol::{DecisionAction, NotifyRequest, NotifyResponse};

/// How long to wait for the helper to respond to a Confirm request.
///
/// Must be well under the hook timeout (30s). Assessment takes ~2-5s,
/// so 15s leaves a comfortable buffer.
const CONFIRM_TIMEOUT: Duration = Duration::from_secs(5);

/// Socket path for the notification helper app.
pub fn notifier_socket_path() -> PathBuf {
    let home = dirs::home_dir().expect("Could not determine home directory");
    home.join(".claude").join("bark-notifier.sock")
}

/// Check if the notification helper is available.
///
/// Verifies the socket is actually connectable (not just a stale file).
/// Cleans up stale socket files automatically.
pub async fn is_available() -> bool {
    let sock = notifier_socket_path();
    if !sock.exists() {
        return false;
    }

    // Try to actually connect — catches stale sockets from dead processes
    match tokio::time::timeout(
        Duration::from_millis(500),
        UnixStream::connect(&sock),
    )
    .await
    {
        Ok(Ok(stream)) => {
            // Connection succeeded, helper is alive
            drop(stream);
            true
        }
        _ => {
            // Stale socket — clean it up
            std::fs::remove_file(&sock).ok();
            false
        }
    }
}

/// Send an informational notification (Medium risk). Fire-and-forget.
pub async fn send_info(assessment: &Assessment, locale: &Locale) {
    let subtitle = locale.t("notify.auto_allowed");
    let req = NotifyRequest::Info {
        title: format!("Bark — {}", subtitle),
        body: assessment.reason.clone(),
    };

    let sock = notifier_socket_path();
    if let Ok(stream) = UnixStream::connect(&sock).await {
        // Don't care about the response
        send_only(stream, &req).await.ok();
    }
}

/// Send a confirmation notification (High risk) and wait for user decision.
///
/// Returns:
/// - `Ok(DecisionAction::Allow)` — user clicked Allow
/// - `Ok(DecisionAction::Deny)` — user clicked Deny
/// - `Ok(DecisionAction::Skip)` — user dismissed, notification timed out,
///   or helper auto-skipped after 5s
/// - `Err(_)` — connection failed, should fallback
pub async fn send_confirm(assessment: &Assessment, locale: &Locale) -> Result<DecisionAction> {
    let id = uuid::Uuid::new_v4().to_string();
    let subtitle = locale.t("notify.needs_confirm");

    let req = NotifyRequest::Confirm {
        id: id.clone(),
        title: format!("Bark — {}", subtitle),
        body: assessment.reason.clone(),
        reason: format!("[{}] {}ms", assessment.source, assessment.duration.as_millis()),
    };

    let sock = notifier_socket_path();
    let stream = UnixStream::connect(&sock)
        .await
        .context("failed to connect to notifier")?;

    let response = tokio::time::timeout(CONFIRM_TIMEOUT, send_and_read(stream, &req))
        .await
        .context("notifier confirm timed out")?
        .context("notifier communication failed")?;

    match response {
        NotifyResponse::Decision { action, .. } => Ok(action),
        NotifyResponse::Error { message } => {
            tracing::warn!("Notifier error: {}", message);
            Ok(DecisionAction::Skip)
        }
        _ => Ok(DecisionAction::Skip),
    }
}

/// Send a request and read one response line.
async fn send_and_read(stream: UnixStream, req: &NotifyRequest) -> Result<NotifyResponse> {
    let (reader, mut writer) = stream.into_split();

    let mut json = serde_json::to_string(req).context("serialize request")?;
    json.push('\n');

    writer
        .write_all(json.as_bytes())
        .await
        .context("write to notifier")?;
    writer.flush().await.ok();

    let mut reader = BufReader::new(reader);
    let mut line = String::new();
    reader
        .read_line(&mut line)
        .await
        .context("read from notifier")?;

    serde_json::from_str(line.trim()).context("parse notifier response")
}

/// Send a request without waiting for response.
async fn send_only(stream: UnixStream, req: &NotifyRequest) -> Result<()> {
    let (_, mut writer) = stream.into_split();

    let mut json = serde_json::to_string(req).context("serialize request")?;
    json.push('\n');

    writer
        .write_all(json.as_bytes())
        .await
        .context("write to notifier")?;
    writer.flush().await.ok();

    Ok(())
}

