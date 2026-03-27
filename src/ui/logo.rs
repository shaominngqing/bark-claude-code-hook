
/// ASCII art logo and banner display.
///
/// Ported from `_logo()` and the install.sh banner in the bash version.

use super::gradient::gradient_text_bold;

/// Small ASCII art logo (figlet -f small "Bark").
const LOGO_LINES: [&str; 4] = [
    r" ___           _   ",
    r"| _ ) __ _ _ _| |__",
    r"| _ \/ _` | '_| / /",
    r"|___/\__,_|_| |_\_\",
];

/// Print the small logo with gradient coloring.
pub fn print_logo() {
    for line in &LOGO_LINES {
        print!("  ");
        println!("{}", gradient_text_bold(line));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_logo_lines_count() {
        assert_eq!(LOGO_LINES.len(), 4);
    }
}
