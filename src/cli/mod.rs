pub mod cache;
pub mod daemon_cmd;
pub mod hook;
pub mod install;
pub mod log_cmd;
pub mod rules;
pub mod stats;
pub mod status;
pub mod test;
pub mod uninstall;
pub mod update;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "bark", version, about = "AI-Powered Risk Assessment for Claude Code")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Run as PreToolUse hook (reads stdin, writes stdout)
    Hook,
    /// Start the daemon process
    Daemon,
    /// Show status
    Status,
    /// Enable Bark
    On,
    /// Disable Bark
    Off,
    /// Toggle Bark on/off
    Toggle,
    /// Test a command's risk level
    Test {
        /// Enable verbose output
        #[arg(short, long)]
        verbose: bool,
        /// Dry-run mode (override high risk to allow)
        #[arg(short = 'n', long)]
        dry_run: bool,
        /// Command to test
        #[arg(trailing_var_arg = true, num_args = 1..)]
        cmd: Vec<String>,
    },
    /// View or clear cache
    Cache {
        #[command(subcommand)]
        action: Option<CacheAction>,
    },
    /// View or clear logs
    Log {
        #[command(subcommand)]
        action: Option<LogAction>,
        /// Number of entries to show
        #[arg(short, long, default_value = "20")]
        count: Option<usize>,
    },
    /// Show statistics
    Stats,
    /// View or edit custom rules
    Rules {
        #[command(subcommand)]
        action: Option<RulesAction>,
    },
    /// Install hook into Claude Code settings
    Install,
    /// Update to latest version
    Update,
    /// Completely uninstall Bark
    Uninstall,
    /// Open TUI dashboard
    #[cfg(feature = "tui")]
    Tui,
}

#[derive(Subcommand)]
pub enum CacheAction {
    /// Clear all cached assessments
    Clear,
}

#[derive(Subcommand)]
pub enum LogAction {
    /// Clear all log entries
    Clear,
}

#[derive(Subcommand)]
pub enum RulesAction {
    /// Open custom rules file in $EDITOR
    Edit,
}
