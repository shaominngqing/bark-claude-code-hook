
/// Card-style UI components for terminal display.
///
/// Ported from `_abar()`, `_card()`, and status display in install.sh / bark-ctl.sh.

use super::gradient::{DIM, GREEN, NC, RED, BOLD};

/// Render a progress bar as a string.
///
/// Returns a string of `width` characters: filled portion uses `color` (an ANSI
/// escape sequence), unfilled portion is dimmed.
///
/// # Arguments
/// * `value` - Current value.
/// * `max` - Maximum value (if 0, treated as 1 to avoid division by zero).
/// * `width` - Total width in characters.
/// * `color` - ANSI color escape sequence for the filled portion (e.g. `"\x1b[38;5;45m"`).
///
/// # Example
/// ```
/// use bark::ui::cards::progress_bar;
/// let bar = progress_bar(7, 10, 20, "\x1b[38;5;45m");
/// // Returns a string with 14 filled chars and 6 empty chars (plus ANSI codes).
/// assert!(bar.contains('\u{2593}')); // filled block
/// ```
pub fn progress_bar(value: usize, max: usize, width: usize, color: &str) -> String {
    let max = if max == 0 { 1 } else { max };
    let filled = (value * width / max).min(width);
    let empty = width - filled;

    let mut out = String::with_capacity(width * 16);

    // Filled portion with gradient shading
    for i in 0..filled {
        let shade = (236 + (i * 18 / width.max(1))).min(255) as u8;
        // Use the provided color for the block character
        let _ = shade; // shade available for future gradient enhancement
        out.push_str(color);
        out.push('\u{2593}'); // ▓
        out.push_str(NC);
    }

    // Empty portion
    for _ in 0..empty {
        out.push_str(DIM);
        out.push('\u{2591}'); // ░
        out.push_str(NC);
    }

    out
}

/// Print a metric card with a boxed label and value.
///
/// ```text
///   ┌──────────────────────┐
///   │ Some Label           │
///   │  42                  │
///   └──────────────────────┘
/// ```
///
/// # Arguments
/// * `label` - The card heading text.
/// * `value` - The value to display (bold, colored).
/// * `color` - ANSI color escape sequence for the value text.
pub fn print_card(label: &str, value: &str, color: &str) {
    let box_width = 22;
    println!("  {}┌{}┐{}", DIM, "─".repeat(box_width), NC);
    println!(
        "  {}│{} {:<width$} {}│{}",
        DIM,
        NC,
        label,
        DIM,
        NC,
        width = box_width - 2
    );
    // Value line: color + bold + value, padded to box width
    let value_display = format!("{}{}{}{}", color, BOLD, value, NC);
    let padding = if value.len() < box_width - 3 {
        box_width - 3 - value.len()
    } else {
        0
    };
    println!(
        "  {}│{}  {}{}{}│{}",
        DIM,
        NC,
        value_display,
        " ".repeat(padding),
        DIM,
        NC
    );
    println!("  {}└{}┘{}", DIM, "─".repeat(box_width), NC);
}

/// Print a status line indicating whether Bark's hook is active.
///
/// Displays:
/// - Active: green dot + "Active --- hook is running"
/// - Inactive: red dot + "Inactive --- hook is disabled"
pub fn print_status_line(active: bool) {
    if active {
        println!(
            "  {}●{} {}Active{}  {}── hook is running{}",
            GREEN, NC, BOLD, NC, DIM, NC
        );
    } else {
        println!(
            "  {}●{} {}Inactive{}  {}── hook is disabled{}",
            RED, NC, BOLD, NC, DIM, NC
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_progress_bar_full() {
        let bar = progress_bar(10, 10, 20, GREEN);
        // Should contain only filled blocks (no empty blocks)
        assert!(bar.contains('\u{2593}'));
        assert!(!bar.contains('\u{2591}'));
    }

    #[test]
    fn test_progress_bar_empty() {
        let bar = progress_bar(0, 10, 20, GREEN);
        // Should contain only empty blocks
        assert!(!bar.contains('\u{2593}'));
        assert!(bar.contains('\u{2591}'));
    }

    #[test]
    fn test_progress_bar_half() {
        let bar = progress_bar(5, 10, 20, GREEN);
        assert!(bar.contains('\u{2593}'));
        assert!(bar.contains('\u{2591}'));
    }

    #[test]
    fn test_progress_bar_zero_max() {
        // Should not panic with max=0
        let bar = progress_bar(5, 0, 20, GREEN);
        assert!(!bar.is_empty());
    }

}
