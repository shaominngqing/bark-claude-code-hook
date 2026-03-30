pub mod fallback;
#[cfg(unix)]
pub mod helper;
pub mod helper_protocol;

use crate::core::protocol::PermissionDecision;
use crate::core::risk::{Assessment, RiskLevel};
use crate::i18n::Locale;

/// Send a desktop notification for the given assessment (fire-and-forget).
///
/// This is the synchronous, non-blocking version that never changes the
/// permission decision. Used as the fallback path.
///
/// - Low: silent, no notification
/// - Medium: informational notification (auto-allowed)
/// - High: urgent notification with sound
pub fn notify_assessment(assessment: &Assessment, locale: &Locale) {
    let subtitle = match assessment.level {
        RiskLevel::Low => return,
        RiskLevel::Medium => locale.t("notify.auto_allowed"),
        RiskLevel::High => locale.t("notify.needs_confirm"),
    };

    let sound = if assessment.level == RiskLevel::High {
        Some("Funk")
    } else {
        None
    };

    fallback::notify("Bark", subtitle, &assessment.reason, sound);
}

/// Try to use the notification helper for rich notifications, falling back
/// to platform-native commands if unavailable.
///
/// For High risk:
/// - If helper is available: sends a Confirm notification with Allow/Deny/Skip
///   buttons and waits for user response. Returns `Some(decision)` to override
///   the default "ask" permission.
/// - If helper unavailable or errors: sends fallback notification and returns
///   `None` (keeps default "ask").
///
/// For Medium risk:
/// - Sends info notification via helper or fallback. Returns `None`.
///
/// For Low risk:
/// - Returns `None` immediately (silent).
#[cfg(unix)]
pub async fn notify_and_decide(
    assessment: &Assessment,
    locale: &Locale,
) -> Option<PermissionDecision> {
    if assessment.level == RiskLevel::Low {
        return None;
    }

    // Try helper first
    if helper::is_available().await {
        match assessment.level {
            RiskLevel::Low => None,
            RiskLevel::Medium => {
                helper::send_info(assessment, locale).await;
                None
            }
            RiskLevel::High => {
                match helper::send_confirm(assessment, locale).await {
                    Ok(action) => {
                        use helper_protocol::DecisionAction;
                        match action {
                            DecisionAction::Allow => Some(PermissionDecision::Allow),
                            DecisionAction::Deny => Some(PermissionDecision::Deny),
                            DecisionAction::Skip => None,
                        }
                    }
                    Err(e) => {
                        tracing::debug!("Helper confirm failed, falling back: {e}");
                        notify_assessment(assessment, locale);
                        None
                    }
                }
            }
        }
    } else {
        // No helper available — use fallback notification
        notify_assessment(assessment, locale);
        None
    }
}

/// Non-unix fallback: always uses platform-native notifications.
#[cfg(not(unix))]
pub async fn notify_and_decide(
    assessment: &Assessment,
    locale: &Locale,
) -> Option<PermissionDecision> {
    notify_assessment(assessment, locale);
    None
}
