/// ASCII art logo and banner display.
use super::style;

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
        println!("  {}", style::gradient(line));
    }
}

/// Print the logo followed by a version tagline.
pub fn print_banner() {
    print_logo();
    let version = env!("CARGO_PKG_VERSION");
    println!("  {} {}", style::dim("v"), style::dim(version));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_logo_lines_count() {
        assert_eq!(LOGO_LINES.len(), 4);
    }
}
