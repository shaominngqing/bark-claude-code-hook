use crate::cli::RulesAction;
use crate::config;
use crate::ui::gradient::{BOLD, DIM, NC, YELLOW};

/// Default content for a new bark.toml file.
const DEFAULT_TOML: &str = r#"# Bark Custom Rules
# See: https://github.com/shaominngqing/bark-claude-code-hook#custom-rules
#
# Each rule has:
#   name    - Human-readable rule name
#   risk    - "low", "medium", or "high"
#   reason  - Why this rule exists
#   [rules.match]
#     tool      - Tool name ("Bash", "Edit|Write", etc.)
#     command   - Glob pattern for command (Bash only)
#     file_path - Glob pattern for file path
#   [rules.conditions]  (optional)
#     cwd_contains - CWD must contain this string
#     git_branch   - Glob pattern for current branch
#     file_exists  - Path that must exist
#     not          - Negate the condition (default: false)

# Example: Allow cargo commands
# [[rules]]
# name = "allow-cargo"
# risk = "low"
# reason = "Cargo commands are generally safe"
# [rules.match]
# tool = "Bash"
# command = "cargo *"

# Example: Block force push
# [[rules]]
# name = "block-force-push"
# risk = "high"
# reason = "Force push can destroy remote history"
# [rules.match]
# tool = "Bash"
# command = "git push*--force*"
"#;

/// View or edit custom rules.
pub fn run(action: Option<RulesAction>) {
    let toml_path = config::bark_toml_path();

    match action {
        Some(RulesAction::Edit) => {
            // Create the file with defaults if it doesn't exist
            if !toml_path.exists() {
                if let Err(e) = std::fs::write(&toml_path, DEFAULT_TOML) {
                    eprintln!("  Error creating {}: {}", toml_path.display(), e);
                    return;
                }
                println!("  Created {}", toml_path.display());
            }

            // Open in $EDITOR
            let editor = std::env::var("EDITOR")
                .or_else(|_| std::env::var("VISUAL"))
                .unwrap_or_else(|_| "vi".to_string());

            println!("  Opening {} in {}...", toml_path.display(), editor);

            let status = std::process::Command::new(&editor)
                .arg(&toml_path)
                .status();

            match status {
                Ok(s) if s.success() => {
                    println!("  Rules file saved.");
                }
                Ok(s) => {
                    eprintln!("  Editor exited with status: {}", s);
                }
                Err(e) => {
                    eprintln!("  Failed to open editor '{}': {}", editor, e);
                    eprintln!("  Set $EDITOR to your preferred editor.");
                }
            }
        }
        None => {
            // Display current rules
            if !toml_path.exists() {
                println!();
                println!(
                    "  {}No custom rules file found.{}",
                    DIM, NC
                );
                println!(
                    "  Run {}bark rules edit{} to create one.",
                    BOLD, NC
                );
                println!("  Path: {}", toml_path.display());
                println!();
                return;
            }

            match std::fs::read_to_string(&toml_path) {
                Ok(content) => {
                    println!();
                    println!(
                        "  {}{}Custom Rules{} ({})",
                        YELLOW, BOLD, NC, toml_path.display()
                    );
                    println!();

                    if content.trim().is_empty() || !content.contains("[[rules]]") {
                        println!("  {}No rules defined.{}", DIM, NC);
                        println!(
                            "  Run {}bark rules edit{} to add rules.",
                            BOLD, NC
                        );
                    } else {
                        // Parse and display rules
                        match crate::core::custom_rules::RuleConfig::from_toml(&content) {
                            Ok(config) => {
                                if config.rules.is_empty() {
                                    println!("  {}No rules defined.{}", DIM, NC);
                                } else {
                                    for rule in &config.rules {
                                        let risk_color = match rule.risk.to_lowercase().as_str() {
                                            "low" => "\x1b[0;32m",
                                            "high" => "\x1b[0;31m",
                                            _ => "\x1b[1;33m",
                                        };
                                        println!(
                                            "  {}{}  {}{}{} {}{}{}",
                                            risk_color,
                                            rule.risk.to_uppercase(),
                                            NC,
                                            BOLD,
                                            rule.name,
                                            NC,
                                            DIM,
                                            NC,
                                        );
                                        println!(
                                            "       {}{}{}",
                                            DIM, rule.reason, NC
                                        );
                                        if let Some(ref tool) = rule.match_criteria.tool {
                                            println!(
                                                "       {}tool: {}{}",
                                                DIM, tool, NC
                                            );
                                        }
                                        if let Some(ref cmd) = rule.match_criteria.command {
                                            println!(
                                                "       {}command: {}{}",
                                                DIM, cmd, NC
                                            );
                                        }
                                        if let Some(ref fp) = rule.match_criteria.file_path {
                                            println!(
                                                "       {}file_path: {}{}",
                                                DIM, fp, NC
                                            );
                                        }
                                        println!();
                                    }
                                }
                            }
                            Err(e) => {
                                eprintln!(
                                    "  Error parsing {}: {}",
                                    toml_path.display(),
                                    e
                                );
                            }
                        }
                    }

                    println!();
                }
                Err(e) => {
                    eprintln!("  Error reading {}: {}", toml_path.display(), e);
                }
            }
        }
    }
}
