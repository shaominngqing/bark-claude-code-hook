
/// ANSI 256-color gradient and color constants.
///
/// Ported from the bash `_gradient()` function and color declarations in install.sh.

// --- Gradient parameters ---
/// Starting ANSI 256-color code for the gradient cycle.
pub const GRADIENT_START: u8 = 39;
/// Number of colors in the gradient cycle (39, 40, 41, 42, 43, 44).
pub const GRADIENT_COUNT: u8 = 6;

// --- Named colors (ANSI 256-color codes) ---
pub const C1: u8 = 39;   // #00afff — dodger blue
pub const C2: u8 = 45;   // #00d7ff — deep sky blue
pub const ACCENT: u8 = 213; // #ff87ff — pink

// --- ANSI escape sequences ---
/// Reset all attributes.
pub const NC: &str = "\x1b[0m";
/// Bold text.
pub const BOLD: &str = "\x1b[1m";
/// Dim (faint) text.
pub const DIM: &str = "\x1b[2m";
// --- Standard colors (ANSI SGR) ---
pub const RED: &str = "\x1b[0;31m";
pub const GREEN: &str = "\x1b[0;32m";
pub const YELLOW: &str = "\x1b[1;33m";

/// Format an ANSI 256-color foreground escape sequence.
#[inline]
pub fn fg256(code: u8) -> String {
    format!("\x1b[38;5;{}m", code)
}

/// Wrap each character of `text` in a cycling ANSI 256-color gradient.
///
/// Colors cycle through `GRADIENT_START` to `GRADIENT_START + GRADIENT_COUNT - 1`
/// (codes 39, 40, 41, 42, 43, 44).
pub fn gradient_text(text: &str) -> String {
    gradient_text_custom(text, GRADIENT_START, GRADIENT_COUNT)
}

/// Wrap each character in a cycling gradient with custom start and count.
pub fn gradient_text_custom(text: &str, start: u8, count: u8) -> String {
    let mut out = String::with_capacity(text.len() * 14); // ~14 bytes per char with escapes
    for (i, ch) in text.chars().enumerate() {
        let color_code = start.wrapping_add((i as u8) % count);
        out.push_str(&format!("\x1b[38;5;{}m{}", color_code, ch));
    }
    out.push_str(NC);
    out
}

/// Wrap each character in a bold cycling gradient.
pub fn gradient_text_bold(text: &str) -> String {
    let mut out = String::with_capacity(text.len() * 16);
    for (i, ch) in text.chars().enumerate() {
        let color_code = GRADIENT_START.wrapping_add((i as u8) % GRADIENT_COUNT);
        out.push_str(&format!("\x1b[1;38;5;{}m{}", color_code, ch));
    }
    out.push_str(NC);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gradient_text_non_empty() {
        let result = gradient_text("Hello");
        assert!(!result.is_empty());
        // Should contain ANSI escape codes
        assert!(result.contains("\x1b[38;5;39m"));
        // Should end with reset
        assert!(result.ends_with(NC));
    }

    #[test]
    fn test_gradient_text_empty() {
        let result = gradient_text("");
        // Only the reset code
        assert_eq!(result, NC);
    }

    #[test]
    fn test_fg256() {
        assert_eq!(fg256(39), "\x1b[38;5;39m");
        assert_eq!(fg256(208), "\x1b[38;5;208m");
    }

    #[test]
    fn test_constants() {
        assert_eq!(C1, 39);
        assert_eq!(C2, 45);
        assert_eq!(ACCENT, 213);
        assert_eq!(GRADIENT_START, 39);
        assert_eq!(GRADIENT_COUNT, 6);
    }
}
