
pub mod en;
pub mod zh;

use serde::{Deserialize, Serialize};

/// Supported locales.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Locale {
    #[default]
    En,
    Zh,
}

impl Locale {
    /// Auto-detect locale from environment variables.
    ///
    /// Checks BARK_LANG, LC_ALL, LANG in order.
    /// Defaults to En if detection fails.
    pub fn detect() -> Self {
        // Check BARK_LANG first (explicit override)
        if let Ok(lang) = std::env::var("BARK_LANG") {
            return Self::from_lang_str(&lang);
        }
        // Check LC_ALL
        if let Ok(lang) = std::env::var("LC_ALL") {
            return Self::from_lang_str(&lang);
        }
        // Check LANG
        if let Ok(lang) = std::env::var("LANG") {
            return Self::from_lang_str(&lang);
        }
        Locale::En
    }

    fn from_lang_str(s: &str) -> Self {
        let lower = s.to_lowercase();
        if lower.starts_with("zh") || lower.contains("zh") {
            Locale::Zh
        } else {
            Locale::En
        }
    }

    /// Look up a translation key for this locale.
    pub fn t(&self, key: &str) -> &'static str {
        match self {
            Locale::En => en::get(key),
            Locale::Zh => zh::get(key),
        }
    }

    /// Returns the language hint string for AI prompts.
    pub fn prompt_hint(&self) -> &'static str {
        match self {
            Locale::En => "in English",
            Locale::Zh => "in Chinese",
        }
    }
}

impl std::fmt::Display for Locale {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Locale::En => write!(f, "en"),
            Locale::Zh => write!(f, "zh"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_is_en() {
        assert_eq!(Locale::default(), Locale::En);
    }

    #[test]
    fn test_t_en() {
        assert_eq!(Locale::En.t("on.enabled"), "Bark enabled.");
    }

    #[test]
    fn test_t_zh() {
        let val = Locale::Zh.t("on.enabled");
        assert!(val.contains("Bark"));
    }

    #[test]
    fn test_t_fallback() {
        assert_eq!(Locale::En.t("nonexistent"), "???");
        // Zh falls back to En, which returns "???"
        assert_eq!(Locale::Zh.t("nonexistent"), "???");
    }

    #[test]
    fn test_prompt_hint() {
        assert_eq!(Locale::En.prompt_hint(), "in English");
        assert_eq!(Locale::Zh.prompt_hint(), "in Chinese");
    }

    #[test]
    fn test_display() {
        assert_eq!(format!("{}", Locale::En), "en");
        assert_eq!(format!("{}", Locale::Zh), "zh");
    }
}
