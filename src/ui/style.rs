//! Semantic terminal styling built on crossterm.
//!
//! Provides color-aware output that respects NO_COLOR / TERM=dumb.
//! Replaces all raw ANSI escape codes scattered across the codebase.

use std::fmt;
use std::sync::OnceLock;

use crossterm::style::{Attribute, Color, Stylize};

// ── Theme colors (ANSI 256) ───────────────────────────────────────

/// Brand gradient: dodger blue → deep sky blue
pub const C1: Color = Color::AnsiValue(39);
pub const C2: Color = Color::AnsiValue(45);
#[allow(dead_code)]
pub const C3: Color = Color::AnsiValue(51);

/// Accent colors
pub const ACCENT: Color = Color::AnsiValue(213);  // pink
pub const ORANGE: Color = Color::AnsiValue(208);
pub const PURPLE: Color = Color::AnsiValue(141);

// ── Color support detection ───────────────────────────────────────

/// Returns true if the terminal supports color output.
fn color_enabled() -> bool {
    static ENABLED: OnceLock<bool> = OnceLock::new();
    *ENABLED.get_or_init(|| {
        if std::env::var_os("NO_COLOR").is_some() {
            return false;
        }
        if std::env::var("TERM").is_ok_and(|t| t == "dumb") {
            return false;
        }
        true
    })
}

// ── Styled wrapper that respects NO_COLOR ─────────────────────────

/// A display wrapper that strips ANSI codes when color is disabled.
pub struct S {
    styled: String,
    plain: String,
}

impl fmt::Display for S {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if color_enabled() {
            f.write_str(&self.styled)
        } else {
            f.write_str(&self.plain)
        }
    }
}

fn s(plain: impl fmt::Display, styled: impl fmt::Display) -> S {
    S {
        styled: styled.to_string(),
        plain: plain.to_string(),
    }
}

// ── Semantic style functions ──────────────────────────────────────

/// Success / low risk: green
pub fn success(content: impl fmt::Display) -> S {
    let text = content.to_string();
    let styled = text.as_str().green();
    s(&text, styled)
}

/// Warning / medium risk: yellow bold
pub fn warning(content: impl fmt::Display) -> S {
    let text = content.to_string();
    let styled = text.as_str().yellow().attribute(Attribute::Bold);
    s(&text, styled)
}

/// Error / high risk: red bold
pub fn danger(content: impl fmt::Display) -> S {
    let text = content.to_string();
    let styled = text.as_str().red().attribute(Attribute::Bold);
    s(&text, styled)
}

/// Bold text
pub fn bold(content: impl fmt::Display) -> S {
    let text = content.to_string();
    let styled = text.as_str().attribute(Attribute::Bold);
    s(&text, styled)
}

/// Dim / muted text
pub fn dim(content: impl fmt::Display) -> S {
    let text = content.to_string();
    let styled = text.as_str().attribute(Attribute::Dim);
    s(&text, styled)
}

/// Brand color (C1 blue bold)
pub fn brand(content: impl fmt::Display) -> S {
    let text = content.to_string();
    let styled = text.as_str().with(C1).attribute(Attribute::Bold);
    s(&text, styled)
}

/// Accent color (pink)
pub fn accent(content: impl fmt::Display) -> S {
    let text = content.to_string();
    let styled = text.as_str().with(ACCENT);
    s(&text, styled)
}

/// Custom color
pub fn colored(content: impl fmt::Display, color: Color) -> S {
    let text = content.to_string();
    let styled = text.as_str().with(color);
    s(&text, styled)
}

/// Custom color with bold
#[allow(dead_code)]
pub fn colored_bold(content: impl fmt::Display, color: Color) -> S {
    let text = content.to_string();
    let styled = text.as_str().with(color).attribute(Attribute::Bold);
    s(&text, styled)
}

// ── Risk-level color helper ───────────────────────────────────────

use crate::core::risk::RiskLevel;

/// Color a string based on risk level.
pub fn risk_colored(content: impl fmt::Display, level: RiskLevel) -> S {
    match level {
        RiskLevel::Low => success(content),
        RiskLevel::Medium => warning(content),
        RiskLevel::High => danger(content),
    }
}

// ── UI symbols ────────────────────────────────────────────────────

/// Green check mark
pub fn check() -> S {
    success("\u{2714}")
}

/// Red cross mark
pub fn cross() -> S {
    danger("\u{2718}")
}

/// Warning triangle
pub fn warn_icon() -> S {
    warning("\u{26a0}")
}

/// Status dot (colored based on active/inactive)
pub fn status_dot(active: bool) -> S {
    if active {
        success("\u{25cf}")
    } else {
        danger("\u{25cf}")
    }
}

/// Step indicator (brand colored bullet)
pub fn step_marker() -> S {
    brand("\u{25b8}")
}

/// Pipeline flow arrow
pub fn flow_arrow() -> S {
    dim("\u{2500}\u{2500}\u{25b8}")
}

/// Bullet point for lists
#[allow(dead_code)]
pub fn bullet() -> S {
    colored("\u{25c6}", C2)
}

// ── Gradient text ─────────────────────────────────────────────────

const GRADIENT_START: u8 = 39;
const GRADIENT_COUNT: u8 = 6;

/// Wrap each character in a cycling bold gradient (39-44).
pub fn gradient(text: &str) -> String {
    if !color_enabled() {
        return text.to_string();
    }
    let mut out = String::with_capacity(text.len() * 16);
    for (i, ch) in text.chars().enumerate() {
        let code = GRADIENT_START.wrapping_add((i as u8) % GRADIENT_COUNT);
        out.push_str(&format!("\x1b[1;38;5;{}m{}", code, ch));
    }
    out.push_str("\x1b[0m");
    out
}

// ── Box drawing helpers ───────────────────────────────────────────

/// Print a bordered card with a label and value.
pub fn print_card(label: &str, value: &str, level: Option<RiskLevel>) {
    let width = 40;
    let border = "\u{2500}".repeat(width);
    println!("  {}", dim(format!("\u{256d}{}\u{256e}", border)));
    println!("  {}  {:<w$}{}", dim("\u{2502}"), label, dim("\u{2502}"), w = width - 1);
    let val_display = match level {
        Some(l) => format!("{}", risk_colored(value, l)),
        None => format!("{}", bold(value)),
    };
    let pad = if value.len() < width - 2 { width - 2 - value.len() } else { 0 };
    println!("  {}  {}{}{}",
        dim("\u{2502}"), val_display, " ".repeat(pad), dim("\u{2502}"));
    println!("  {}", dim(format!("\u{2570}{}\u{256f}", border)));
}

/// Render a progress bar string.
pub fn progress_bar(value: usize, max: usize, width: usize, color: Color) -> String {
    let max = if max == 0 { 1 } else { max };
    let filled = (value * width / max).min(width);
    let empty = width - filled;

    if !color_enabled() {
        return format!("{}{}", "\u{2588}".repeat(filled), "\u{2591}".repeat(empty));
    }

    let mut out = String::with_capacity(width * 16);
    for _ in 0..filled {
        out.push_str(&format!("{}", colored("\u{2588}", color)));
    }
    for _ in 0..empty {
        out.push_str(&format!("{}", dim("\u{2591}")));
    }
    out
}

// ── Section / label helpers ───────────────────────────────────────

/// Print a step header like "▸ Check environment"
pub fn print_step(label: &str) {
    println!("  {} {}", step_marker(), bold(label));
}

/// Print a success line like "✔ something"
pub fn print_ok(msg: &str) {
    println!("  {} {}", check(), msg);
}

/// Print a warning line like "⚠ something"
pub fn print_warn(msg: &str) {
    println!("  {} {}", warn_icon(), msg);
}

/// Print an error line like "✘ something"
pub fn print_err(msg: &str) {
    println!("  {} {}", cross(), msg);
}

/// Print a key-value status line with consistent width.
///
/// Uses display width accounting for CJK characters (2 columns each).
pub fn print_kv(label: &str, value: &str) {
    let target_width = 10;
    let display_width = label.chars().map(|c| if c > '\u{FF}' { 2 } else { 1 }).sum::<usize>();
    let padding = if display_width < target_width { target_width - display_width } else { 1 };
    println!("  {}{}{}", dim(label), " ".repeat(padding), value);
}

/// Print a section title in bold.
pub fn print_section(title: &str) {
    println!("  {}", bold(title));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gradient_non_empty() {
        let result = gradient("Hello");
        assert!(!result.is_empty());
    }

    #[test]
    fn test_gradient_empty() {
        let result = gradient("");
        assert!(result.is_empty() || result == "\x1b[0m");
    }

    #[test]
    fn test_progress_bar_full() {
        let bar = progress_bar(10, 10, 20, Color::Green);
        assert!(!bar.is_empty());
    }

    #[test]
    fn test_progress_bar_zero_max() {
        let bar = progress_bar(5, 0, 20, Color::Green);
        assert!(!bar.is_empty());
    }

    #[test]
    fn test_success_display() {
        let s = success("OK");
        let output = format!("{}", s);
        assert!(output.contains("OK"));
    }

    #[test]
    fn test_dim_display() {
        let s = dim("muted");
        let output = format!("{}", s);
        assert!(output.contains("muted"));
    }
}
