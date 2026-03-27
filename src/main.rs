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

fn main() {
    let cli = cli::Cli::parse();

    // Default to Status if no subcommand given
    match cli.command.unwrap_or(cli::Commands::Status) {
        cli::Commands::Hook => cli::hook::run(),
        #[cfg(unix)]
        cli::Commands::Daemon => cli::daemon_cmd::run(),
        cli::Commands::Status => cli::status::run(),
        cli::Commands::On => {
            match config::enable_hook() {
                Ok(()) => {
                    println!("  \x1b[0;32m\x1b[1mBark enabled.\x1b[0m");
                }
                Err(e) => {
                    eprintln!("  Error enabling Bark: {}", e);
                }
            }
        }
        cli::Commands::Off => {
            match config::disable_hook() {
                Ok(()) => {
                    println!("  \x1b[1;33m\x1b[1mBark disabled.\x1b[0m");
                }
                Err(e) => {
                    eprintln!("  Error disabling Bark: {}", e);
                }
            }
        }
        cli::Commands::Toggle => {
            if config::has_hook() {
                match config::disable_hook() {
                    Ok(()) => println!("  \x1b[1;33mBark disabled.\x1b[0m"),
                    Err(e) => eprintln!("  Error disabling Bark: {}", e),
                }
            } else {
                match config::enable_hook() {
                    Ok(()) => println!("  \x1b[0;32mBark enabled.\x1b[0m"),
                    Err(e) => eprintln!("  Error enabling Bark: {}", e),
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
        cli::Commands::Update => cli::update::run(),
        cli::Commands::Uninstall => cli::uninstall::run(),
        #[cfg(feature = "tui")]
        cli::Commands::Tui => tui::run(),
    }
}
