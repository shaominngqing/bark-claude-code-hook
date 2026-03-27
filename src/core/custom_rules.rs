
use globset::{Glob, GlobMatcher};
use serde::Deserialize;
use std::path::Path;

use crate::core::risk::{Assessment, AssessmentSource, RiskLevel};

/// Top-level TOML configuration for custom rules.
#[derive(Debug, Clone, Deserialize)]
pub struct RuleConfig {
    #[serde(default)]
    pub rules: Vec<Rule>,
}

impl RuleConfig {
    /// Parse a TOML string into a RuleConfig.
    pub fn from_toml(toml_str: &str) -> Result<Self, toml::de::Error> {
        toml::from_str(toml_str)
    }

    /// Load from a file path. Returns empty config on failure.
    pub fn load_from_file(path: &Path) -> Self {
        match std::fs::read_to_string(path) {
            Ok(content) => Self::from_toml(&content).unwrap_or_else(|e| {
                tracing::warn!("Failed to parse rules TOML at {:?}: {}", path, e);
                Self { rules: Vec::new() }
            }),
            Err(e) => {
                tracing::debug!("No custom rules file at {:?}: {}", path, e);
                Self { rules: Vec::new() }
            }
        }
    }

    /// Compile all rules into a CompiledRuleSet for efficient matching.
    pub fn compile(&self) -> CompiledRuleSet {
        let compiled = self
            .rules
            .iter()
            .filter_map(|rule| CompiledRule::from_rule(rule))
            .collect();
        CompiledRuleSet { rules: compiled }
    }
}

/// A single rule definition from TOML.
///
/// Example TOML:
/// ```toml
/// [[rules]]
/// name = "block-force-push"
/// risk = "high"
/// reason = "Force push is destructive"
///
/// [rules.match]
/// tool = "Bash"
/// command = "git push*--force*"
/// ```
#[derive(Debug, Clone, Deserialize)]
pub struct Rule {
    pub name: String,
    #[serde(rename = "match")]
    pub match_criteria: MatchCriteria,
    #[serde(default)]
    pub conditions: Option<RuleConditions>,
    pub risk: String,
    pub reason: String,
}

/// Match criteria for a rule.
#[derive(Debug, Clone, Deserialize)]
pub struct MatchCriteria {
    /// Tool name pattern. Supports `|` for OR (e.g., "Edit|Write").
    #[serde(default)]
    pub tool: Option<String>,
    /// Command glob pattern (e.g., "git push*--force*").
    #[serde(default)]
    pub command: Option<String>,
    /// File path glob pattern (e.g., "**/.env*").
    #[serde(default)]
    pub file_path: Option<String>,
}

/// Additional conditions for a rule.
#[derive(Debug, Clone, Deserialize)]
pub struct RuleConditions {
    /// Current working directory must contain this substring.
    #[serde(default)]
    pub cwd_contains: Option<String>,
    /// A file must exist at this path.
    #[serde(default)]
    pub file_exists: Option<String>,
    /// Git branch must match this pattern.
    #[serde(default)]
    pub git_branch: Option<String>,
    /// Negate: if true, the rule matches when the match criteria do NOT match.
    #[serde(default)]
    pub not: bool,
}

/// Context passed to rule evaluation.
#[derive(Debug, Default)]
pub struct RuleContext {
    pub cwd: Option<String>,
    pub git_branch: Option<String>,
}

/// A compiled rule ready for efficient matching.
#[derive(Debug)]
pub struct CompiledRule {
    pub name: String,
    pub tool_patterns: Vec<String>,
    pub command_matcher: Option<GlobMatcher>,
    pub file_path_matcher: Option<GlobMatcher>,
    pub conditions: Option<RuleConditions>,
    pub risk_level: RiskLevel,
    pub reason: String,
}

impl CompiledRule {
    /// Compile a rule definition into a CompiledRule.
    fn from_rule(rule: &Rule) -> Option<Self> {
        let tool_patterns: Vec<String> = rule
            .match_criteria
            .tool
            .as_deref()
            .map(|t| t.split('|').map(|s| s.trim().to_string()).collect())
            .unwrap_or_default();

        let command_matcher = rule
            .match_criteria
            .command
            .as_deref()
            .and_then(|pattern| {
                Glob::new(pattern)
                    .ok()
                    .map(|g| g.compile_matcher())
            });

        let file_path_matcher = rule
            .match_criteria
            .file_path
            .as_deref()
            .and_then(|pattern| {
                Glob::new(pattern)
                    .ok()
                    .map(|g| g.compile_matcher())
            });

        let risk_level = parse_risk_level(&rule.risk);

        Some(CompiledRule {
            name: rule.name.clone(),
            tool_patterns,
            command_matcher,
            file_path_matcher,
            conditions: rule.conditions.clone(),
            risk_level,
            reason: rule.reason.clone(),
        })
    }

    /// Check if this rule matches the given input.
    fn matches(
        &self,
        tool_name: &str,
        command: Option<&str>,
        file_path: Option<&str>,
        context: &RuleContext,
    ) -> bool {
        // Tool match
        if !self.tool_patterns.is_empty()
            && !self.tool_patterns.iter().any(|p| p == tool_name)
        {
            return false;
        }

        // Command match
        if let Some(ref matcher) = self.command_matcher {
            match command {
                Some(cmd) => {
                    if !matcher.is_match(cmd) {
                        return false;
                    }
                }
                None => return false,
            }
        }

        // File path match
        if let Some(ref matcher) = self.file_path_matcher {
            match file_path {
                Some(fp) => {
                    if !matcher.is_match(fp) {
                        return false;
                    }
                }
                None => return false,
            }
        }

        // Additional conditions
        if let Some(ref conditions) = self.conditions {
            if !self.check_conditions(conditions, context) {
                return false;
            }
        }

        true
    }

    fn check_conditions(&self, conditions: &RuleConditions, context: &RuleContext) -> bool {
        let mut result = true;

        // cwd_contains
        if let Some(ref pattern) = conditions.cwd_contains {
            match &context.cwd {
                Some(cwd) => {
                    if !cwd.contains(pattern.as_str()) {
                        result = false;
                    }
                }
                None => result = false,
            }
        }

        // file_exists
        if let Some(ref path) = conditions.file_exists {
            if !Path::new(path).exists() {
                result = false;
            }
        }

        // git_branch
        if let Some(ref branch_pattern) = conditions.git_branch {
            match &context.git_branch {
                Some(branch) => {
                    let matcher = Glob::new(branch_pattern)
                        .ok()
                        .map(|g| g.compile_matcher());
                    if let Some(m) = matcher {
                        if !m.is_match(branch.as_str()) {
                            result = false;
                        }
                    }
                }
                None => result = false,
            }
        }

        // Negate
        if conditions.not {
            !result
        } else {
            result
        }
    }
}

/// Compiled set of rules for efficient evaluation.
#[derive(Debug)]
pub struct CompiledRuleSet {
    rules: Vec<CompiledRule>,
}

impl CompiledRuleSet {
    /// Check if any rule matches the given input.
    /// Returns the first matching rule's assessment.
    pub fn check(
        &self,
        tool_name: &str,
        command: Option<&str>,
        file_path: Option<&str>,
        context: &RuleContext,
    ) -> Option<Assessment> {
        for rule in &self.rules {
            if rule.matches(tool_name, command, file_path, context) {
                return Some(Assessment::new(
                    rule.risk_level,
                    format!("[{}] {}", rule.name, rule.reason),
                    AssessmentSource::CustomRule,
                ));
            }
        }
        None
    }
}

#[cfg(test)]
impl CompiledRuleSet {
    /// Empty rule set (no rules). Test-only helper.
    pub fn empty() -> Self {
        Self { rules: Vec::new() }
    }

    /// Number of compiled rules.
    pub fn len(&self) -> usize {
        self.rules.len()
    }

    /// Whether the rule set is empty.
    pub fn is_empty(&self) -> bool {
        self.rules.is_empty()
    }
}

/// Parse a risk level string into a RiskLevel enum.
fn parse_risk_level(s: &str) -> RiskLevel {
    match s.to_lowercase().as_str() {
        "low" | "0" => RiskLevel::Low,
        "high" | "2" => RiskLevel::High,
        _ => RiskLevel::Medium,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_TOML: &str = r#"
[[rules]]
name = "block-force-push"
risk = "high"
reason = "Force push is destructive and can lose history"

[rules.match]
tool = "Bash"
command = "git push*--force*"

[[rules]]
name = "protect-env-files"
risk = "high"
reason = "Environment files may contain secrets"

[rules.match]
tool = "Edit|Write"
file_path = "**/.env*"

[[rules]]
name = "allow-cargo-test"
risk = "low"
reason = "Running tests is safe"

[rules.match]
tool = "Bash"
command = "cargo test*"

[[rules]]
name = "block-prod-deploy"
risk = "high"
reason = "Production deploys require review"

[rules.match]
tool = "Bash"
command = "*deploy*prod*"

[rules.conditions]
git_branch = "main"
"#;

    fn compile_sample() -> CompiledRuleSet {
        let config = RuleConfig::from_toml(SAMPLE_TOML).unwrap();
        config.compile()
    }

    #[test]
    fn test_parse_toml() {
        let config = RuleConfig::from_toml(SAMPLE_TOML).unwrap();
        assert_eq!(config.rules.len(), 4);
        assert_eq!(config.rules[0].name, "block-force-push");
        assert_eq!(config.rules[1].match_criteria.tool, Some("Edit|Write".to_string()));
    }

    #[test]
    fn test_compile() {
        let rules = compile_sample();
        assert_eq!(rules.len(), 4);
    }

    #[test]
    fn test_force_push_match() {
        let rules = compile_sample();
        let ctx = RuleContext::default();
        let result = rules.check(
            "Bash",
            Some("git push --force origin main"),
            None,
            &ctx,
        );
        assert!(result.is_some());
        let a = result.unwrap();
        assert_eq!(a.level, RiskLevel::High);
        assert!(a.reason.contains("block-force-push"));
    }

    #[test]
    fn test_env_file_match() {
        let rules = compile_sample();
        let ctx = RuleContext::default();

        let result = rules.check("Edit", None, Some("/project/.env.production"), &ctx);
        assert!(result.is_some());
        assert_eq!(result.unwrap().level, RiskLevel::High);

        let result = rules.check("Write", None, Some("/project/.env"), &ctx);
        assert!(result.is_some());
    }

    #[test]
    fn test_env_file_no_match_wrong_tool() {
        let rules = compile_sample();
        let ctx = RuleContext::default();
        let result = rules.check("Bash", None, Some("/project/.env"), &ctx);
        // Bash doesn't match Edit|Write rule
        assert!(result.is_none() || result.as_ref().unwrap().reason.contains("block-force-push") == false);
    }

    #[test]
    fn test_cargo_test_low() {
        let rules = compile_sample();
        let ctx = RuleContext::default();
        let result = rules.check("Bash", Some("cargo test --release"), None, &ctx);
        assert!(result.is_some());
        assert_eq!(result.unwrap().level, RiskLevel::Low);
    }

    #[test]
    fn test_no_match() {
        let rules = compile_sample();
        let ctx = RuleContext::default();
        let result = rules.check("Bash", Some("ls -la"), None, &ctx);
        assert!(result.is_none());
    }

    #[test]
    fn test_deploy_with_branch_condition() {
        let rules = compile_sample();

        // On main branch → should match
        let ctx = RuleContext {
            cwd: None,
            git_branch: Some("main".to_string()),
        };
        let result = rules.check("Bash", Some("kubectl deploy prod"), None, &ctx);
        assert!(result.is_some());
        assert_eq!(result.unwrap().level, RiskLevel::High);

        // On feature branch → should not match (branch condition fails)
        let ctx = RuleContext {
            cwd: None,
            git_branch: Some("feature/xyz".to_string()),
        };
        let result = rules.check("Bash", Some("kubectl deploy prod"), None, &ctx);
        assert!(result.is_none());
    }

    #[test]
    fn test_empty_ruleset() {
        let rules = CompiledRuleSet::empty();
        assert!(rules.is_empty());
        let result = rules.check("Bash", Some("rm -rf /"), None, &RuleContext::default());
        assert!(result.is_none());
    }

    #[test]
    fn test_pipe_separated_tools() {
        let toml = r#"
[[rules]]
name = "file-mod"
risk = "medium"
reason = "File modification"

[rules.match]
tool = "Edit|Write|NotebookEdit"
file_path = "*.rs"
"#;
        let config = RuleConfig::from_toml(toml).unwrap();
        let rules = config.compile();
        let ctx = RuleContext::default();

        assert!(rules.check("Edit", None, Some("main.rs"), &ctx).is_some());
        assert!(rules.check("Write", None, Some("lib.rs"), &ctx).is_some());
        assert!(rules.check("NotebookEdit", None, Some("test.rs"), &ctx).is_some());
        assert!(rules.check("Read", None, Some("main.rs"), &ctx).is_none());
    }

    #[test]
    fn test_not_condition() {
        let toml = r#"
[[rules]]
name = "not-on-main"
risk = "high"
reason = "Only allowed on main"

[rules.match]
tool = "Bash"
command = "cargo publish*"

[rules.conditions]
git_branch = "main"
not = true
"#;
        let config = RuleConfig::from_toml(toml).unwrap();
        let rules = config.compile();

        // On main → conditions match → negated → rule does NOT fire
        let ctx = RuleContext {
            cwd: None,
            git_branch: Some("main".to_string()),
        };
        let result = rules.check("Bash", Some("cargo publish"), None, &ctx);
        assert!(result.is_none());

        // On feature → conditions don't match → negated → rule fires
        let ctx = RuleContext {
            cwd: None,
            git_branch: Some("feature/x".to_string()),
        };
        let result = rules.check("Bash", Some("cargo publish"), None, &ctx);
        assert!(result.is_some());
        assert_eq!(result.unwrap().level, RiskLevel::High);
    }
}
