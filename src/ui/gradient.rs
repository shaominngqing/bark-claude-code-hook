/// Legacy ANSI color constants — kept for backward compatibility with tests.
/// New code should use `ui::style` instead.

#[cfg(test)]
pub const NC: &str = "\x1b[0m";

#[cfg(test)]
pub const GREEN: &str = "\x1b[0;32m";
