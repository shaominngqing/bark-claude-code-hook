use crate::i18n::Locale;
use crate::ui::style;

/// Self-update (placeholder).
pub fn run() {
    let locale = Locale::detect();
    let version = env!("CARGO_PKG_VERSION");
    println!();
    println!("  Bark v{}", version);
    println!("  {}", style::dim(locale.t("update.not_impl")));
    println!("  {}", style::dim(locale.t("update.manual")));
    println!();
}
