mod ai;
mod analysis;
mod cache;
mod cli;
mod config;
mod core;
#[cfg(unix)]
mod daemon;
mod i18n;
mod notify;
#[cfg(feature = "tui")]
mod tui;
mod ui;

use clap::Parser;
use i18n::Locale;
use ui::style;

fn main() {
    let cli = cli::Cli::parse();
    let locale = Locale::detect();

    // Default to Status if no subcommand given
    match cli.command.unwrap_or(cli::Commands::Status) {
        cli::Commands::Hook => cli::hook::run(),
        #[cfg(unix)]
        cli::Commands::Daemon => cli::daemon_cmd::run(),
        cli::Commands::Status => cli::status::run(),
        cli::Commands::On => {
            match config::enable_hook() {
                Ok(()) => println!("  {} {}", style::check(), style::bold(locale.t("on.enabled"))),
                Err(e) => eprintln!("  {} {}: {}", style::cross(), locale.t("on.error"), e),
            }
        }
        cli::Commands::Off => {
            match config::disable_hook() {
                Ok(()) => println!("  {} {}", style::warn_icon(), style::bold(locale.t("off.disabled"))),
                Err(e) => eprintln!("  {} {}: {}", style::cross(), locale.t("off.error"), e),
            }
        }
        cli::Commands::Toggle => {
            if config::has_hook() {
                match config::disable_hook() {
                    Ok(()) => println!("  {} {}", style::warn_icon(), style::bold(locale.t("off.disabled"))),
                    Err(e) => eprintln!("  {} {}: {}", style::cross(), locale.t("off.error"), e),
                }
            } else {
                match config::enable_hook() {
                    Ok(()) => println!("  {} {}", style::check(), style::bold(locale.t("on.enabled"))),
                    Err(e) => eprintln!("  {} {}: {}", style::cross(), locale.t("on.error"), e),
                }
            }
        }
        cli::Commands::Test { verbose, dry_run, cmd } => {
            cli::test::run(cmd, verbose, dry_run);
        }
        cli::Commands::Cache { action } => {
            cli::cache::run(action);
        }
        cli::Commands::Log { action, count } => {
            cli::log_cmd::run(action, count.unwrap_or(20));
        }
        cli::Commands::Stats => cli::stats::run(),
        cli::Commands::Rules { action } => {
            cli::rules::run(action);
        }
        cli::Commands::Install => cli::install::run(),
        cli::Commands::InstallNotifier => cli::install_notifier::run(),
        cli::Commands::Update => cli::update::run(),
        cli::Commands::Uninstall => cli::uninstall::run(),
        #[cfg(feature = "tui")]
        cli::Commands::Tui => tui::run(),
    }
}
