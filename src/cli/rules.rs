use crate::cli::RulesAction;
use crate::config;
use crate::i18n::Locale;
use crate::ui::style;

/// Default content for a new bark.toml file.
const DEFAULT_TOML: &str = r#"# Bark Custom Rules
# See: https://github.com/shaominngqing/bark-claude-code-hook#custom-rules
#
# Each rule has:
#   name    - Human-readable rule name
#   risk    - "low", "medium", or "high"
#   reason  - Why this rule exists
#   [rules.match]
#   tool      - Tool name ("Bash", "Edit|Write", etc.)
#   command   - Glob pattern for command (Bash only)
#   file_path - Glob pattern for file path
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
    let locale = Locale::detect();
    let toml_path = config::bark_toml_path();

    match action {
        Some(RulesAction::Edit) => {
            // Create the file with defaults if it doesn't exist
            if !toml_path.exists() {
                if let Err(e) = std::fs::write(&toml_path, DEFAULT_TOML) {
                    eprintln!("  {} Error creating {}: {}", style::cross(), toml_path.display(), e);
                    return;
                }
                println!("  {} {} {}", style::check(), locale.t("rules.created"), toml_path.display());
            }

            // Open in $EDITOR
            let editor = std::env::var("EDITOR")
                .or_else(|_| std::env::var("VISUAL"))
                .unwrap_or_else(|_| "vi".to_string());

            println!("  {} {} {} {} {}...",
                style::dim("\u{270e}"),
                locale.t("rules.opening").split("{editor}").next().unwrap_or("Opening"),
                toml_path.display(),
                style::dim("in"),
                editor,
            );

            let status = std::process::Command::new(&editor)
                .arg(&toml_path)
                .status();

            match status {
                Ok(s) if s.success() => {
                    println!("  {} {}", style::check(), locale.t("rules.saved"));
                }
                Ok(s) => {
                    eprintln!("  {} {}: {}", style::cross(), locale.t("rules.editor_exit"), s);
                }
                Err(e) => {
                    eprintln!("  {} {} '{}': {}", style::cross(), locale.t("rules.editor_fail"), editor, e);
                    eprintln!("  {}", style::dim(locale.t("rules.editor_hint")));
                }
            }
        }
        None => {
            // Display current rules
            if !toml_path.exists() {
                println!();
                println!("  {}", style::dim(locale.t("rules.no_file")));
                println!("  {} {} {}",
                    style::dim(locale.t("rules.create_hint").split("{cmd}").next().unwrap_or("Run")),
                    style::bold("bark rules edit"),
                    style::dim(locale.t("rules.create_hint").split("{cmd}").nth(1).unwrap_or("")),
                );
                println!("  {} {}", style::dim("Path:"), toml_path.display());
                println!();
                return;
            }

            match std::fs::read_to_string(&toml_path) {
                Ok(content) => {
                    println!();
                    println!("  {} {}",
                        style::bold(locale.t("rules.title")),
                        style::dim(format!("({})", toml_path.display())),
                    );
                    println!();

                    if content.trim().is_empty() || !content.contains("[[rules]]") {
                        println!("  {}", style::dim(locale.t("rules.no_rules")));
                        println!("  {} {} {}",
                            style::dim(locale.t("rules.edit_hint").split("{cmd}").next().unwrap_or("Run")),
                            style::bold("bark rules edit"),
                            style::dim(locale.t("rules.edit_hint").split("{cmd}").nth(1).unwrap_or("")),
                        );
                    } else {
                        // Parse and display rules
                        match crate::core::custom_rules::RuleConfig::from_toml(&content) {
                            Ok(config) => {
                                if config.rules.is_empty() {
                                    println!("  {}", style::dim(locale.t("rules.no_rules")));
                                } else {
                                    for rule in &config.rules {
                                        let level = match rule.risk.to_lowercase().as_str() {
                                            "low" => crate::core::risk::RiskLevel::Low,
                                            "high" => crate::core::risk::RiskLevel::High,
                                            _ => crate::core::risk::RiskLevel::Medium,
                                        };
                                        println!(
                                            "    {} {}  {}",
                                            style::risk_colored(rule.risk.to_uppercase(), level),
                                            style::bold(&rule.name),
                                            style::dim(&rule.reason),
                                        );
                                        if let Some(ref tool) = rule.match_criteria.tool {
                                            println!("       {} tool: {}", style::dim("\u{2514}"), tool);
                                        }
                                        if let Some(ref cmd) = rule.match_criteria.command {
                                            println!("       {} command: {}", style::dim("\u{2514}"), cmd);
                                        }
                                        if let Some(ref fp) = rule.match_criteria.file_path {
                                            println!("       {} file_path: {}", style::dim("\u{2514}"), fp);
                                        }
                                        println!();
                                    }
                                }
                            }
                            Err(e) => {
                                eprintln!("  {} Error parsing {}: {}", style::cross(), toml_path.display(), e);
                            }
                        }
                    }

                    println!();
                }
                Err(e) => {
                    eprintln!("  {} Error reading {}: {}", style::cross(), toml_path.display(), e);
                }
            }
        }
    }
}
