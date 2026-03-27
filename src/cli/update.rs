use crate::ui::gradient::{DIM, NC};

/// Self-update (placeholder).
pub fn run() {
    let version = env!("CARGO_PKG_VERSION");
    println!();
    println!("  Bark v{}", version);
    println!(
        "  {}Update check is not yet implemented in the Rust version.{}",
        DIM, NC
    );
    println!(
        "  {}To update manually, rebuild from source or download the latest release.{}",
        DIM, NC
    );
    println!();
}
