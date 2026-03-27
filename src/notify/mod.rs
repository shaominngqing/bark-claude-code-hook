pub mod fallback;

use crate::core::risk::{Assessment, RiskLevel};
use crate::i18n::Locale;

/// Send a desktop notification for the given assessment.
///
/// - Low: silent, no notification
/// - Medium: informational notification (auto-allowed)
/// - High: urgent notification with sound
///
/// Uses lightweight platform-native commands — no extra dependencies,
/// no authorization prompts, works out of the box.
pub fn notify_assessment(assessment: &Assessment, locale: &Locale) {
    let subtitle = match assessment.level {
        RiskLevel::Low => return,
        RiskLevel::Medium => {
            if matches!(locale, Locale::Zh) {
                "已自动放行"
            } else {
                "Auto-allowed"
            }
        }
        RiskLevel::High => {
            if matches!(locale, Locale::Zh) {
                "需要确认"
            } else {
                "Confirmation needed"
            }
        }
    };

    let sound = if assessment.level == RiskLevel::High {
        Some("Funk")
    } else {
        None
    };

    fallback::notify("Bark", subtitle, &assessment.reason, sound);
}
