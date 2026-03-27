
use std::sync::Arc;
use std::time::Instant;

use parking_lot::{Mutex, RwLock};

use crate::ai::ClaudeCliClient;
use crate::analysis::BashAnalyzer;
use crate::cache::SqliteCache;
use crate::core::chain_tracker::{ChainContext, ChainTracker};
use crate::core::custom_rules::{CompiledRuleSet, RuleConfig, RuleContext};
use crate::core::fast_rules;
use crate::core::normalizer;
use crate::core::protocol::HookInput;
use crate::core::risk::{Assessment, AssessmentSource};
use crate::i18n::Locale;

/// The central assessment engine. Coordinates all layers of the risk
/// assessment pipeline:
///
/// 1. Fast rules (read-only tools, safe file edits)
/// 2. Custom TOML rules
/// 3. Cache lookup
/// 4. AST analysis (for Bash commands)
/// 5. Chain context (multi-step attack detection)
/// 6. AI assessment (Anthropic API)
///
/// On any error, returns a Medium/Fallback assessment (never panics).
pub struct AssessmentEngine {
    /// Custom TOML rules, hot-reloadable.
    custom_rules: Arc<RwLock<CompiledRuleSet>>,
    /// SQLite cache for assessment results.
    cache: Option<SqliteCache>,
    /// Claude CLI client for AI assessment (uses `claude -p`).
    ai_client: Option<ClaudeCliClient>,
    /// Bash AST analyzer (needs interior mutability for tree-sitter parser).
    bash_analyzer: Mutex<BashAnalyzer>,
    /// Chain tracker for multi-step attack detection.
    chain_tracker: Arc<Mutex<ChainTracker>>,
    /// Locale for user-facing messages.
    locale: Locale,
    /// Session ID for chain tracking.
    session_id: String,
    /// Rule evaluation context (cwd, git branch, etc.)
    rule_context: RuleContext,
}

impl AssessmentEngine {
    /// Create a standalone engine for non-daemon mode.
    ///
    /// Auto-detects `claude` CLI for AI assessment (same as the old Bash version).
    pub fn new_standalone(rules_path: Option<&std::path::Path>, cache_path: Option<&std::path::Path>) -> Self {
        let locale = Locale::detect();

        // Load custom rules
        let rule_config = rules_path
            .map(RuleConfig::load_from_file)
            .unwrap_or(RuleConfig { rules: Vec::new() });
        let compiled = rule_config.compile();
        let custom_rules = Arc::new(RwLock::new(compiled));

        // Initialize cache
        let cache = cache_path.and_then(|p| {
            SqliteCache::open(p).map_err(|e| {
                tracing::warn!("Failed to open cache at {:?}: {}", p, e);
                e
            }).ok()
        });

        // Auto-detect claude CLI for AI assessment
        let ai_client = if ClaudeCliClient::is_available() {
            Some(ClaudeCliClient::new(30))
        } else {
            tracing::info!("claude CLI not found — AI assessment layer disabled");
            None
        };

        let session_id = uuid::Uuid::new_v4().to_string();

        Self {
            custom_rules,
            cache,
            ai_client,
            bash_analyzer: Mutex::new(BashAnalyzer::new()),
            chain_tracker: Arc::new(Mutex::new(ChainTracker::new())),
            locale,
            session_id,
            rule_context: RuleContext::default(),
        }
    }

    /// Main assessment pipeline. Processes a HookInput through all layers.
    ///
    /// Pipeline order:
    /// 1. Fast rules
    /// 2. Custom TOML rules
    /// 3. Cache lookup
    /// 4. Bash AST analysis
    /// 5. Chain context check
    /// 6. AI assessment
    /// 7. Fallback (if all else fails or errors)
    ///
    /// Always records the result to the chain tracker.
    pub async fn assess(&self, input: &HookInput) -> Assessment {
        let start = Instant::now();

        let result = self.assess_inner(input).await;

        let assessment = result.with_duration(start.elapsed());

        // Record to chain tracker
        self.record_to_chain(input, &assessment);

        // Cache the result (if we have a cache and it came from a non-cache source)
        if assessment.source != AssessmentSource::Cache {
            self.cache_result(input, &assessment);
        }

        assessment
    }

    /// Inner assessment logic, separated for cleaner error handling.
    async fn assess_inner(&self, input: &HookInput) -> Assessment {
        let tool_name = input.tool_name.as_str();
        let command = input.command();
        let file_path = input.file_path();

        // Layer 1: Fast rules (including safe Bash command whitelist)
        if let Some(assessment) = fast_rules::fast_check_with_command(tool_name, command, file_path, &self.locale) {
            return assessment;
        }

        // Layer 2: Custom TOML rules
        {
            let rules = self.custom_rules.read();
            if let Some(assessment) = rules.check(tool_name, command, file_path, &self.rule_context) {
                return assessment;
            }
        }

        // Layer 3: Cache lookup
        if let Some(assessment) = self.cache_lookup(input) {
            return assessment;
        }

        // Layer 4: Bash AST analysis
        if tool_name == "Bash" {
            if let Some(cmd) = command {
                let analyzer = self.bash_analyzer.lock(); // tree-sitter parser needs &mut self
                if let Some(assessment) = analyzer.analyze(cmd) {
                    return assessment;
                }
            }
        }

        // Layer 5: Chain context check
        let chain_context = {
            let tracker = self.chain_tracker.lock();
            tracker.get_context(&self.session_id)
        };

        if let Some(assessment) = self.assess_chain_context(&chain_context) {
            return assessment;
        }

        // Layer 6: AI assessment via claude CLI
        if let Some(ref ai_client) = self.ai_client {
            // Run claude CLI in a blocking task to avoid blocking the async runtime
            let result = ai_client.assess(input, &chain_context);
            match result {
                Ok(assessment) => return assessment,
                Err(e) => {
                    tracing::warn!("AI assessment failed: {}", e);
                    // Fall through to fallback
                }
            }
        }

        // Layer 7: Fallback - medium risk for anything that reached here
        let reason = format!(
            "{}: {} - {}",
            self.locale.t("risk.unknown_op"),
            tool_name,
            self.locale.t("risk.needs_confirm"),
        );
        Assessment::medium(reason, AssessmentSource::Fallback)
    }

    /// Check chain context for suspicious patterns that warrant higher risk.
    fn assess_chain_context(&self, context: &ChainContext) -> Option<Assessment> {
        if context.suspicious_patterns.is_empty() {
            return None;
        }

        let patterns: Vec<String> = context
            .suspicious_patterns
            .iter()
            .map(|p| p.to_string())
            .collect();

        let reason = format!(
            "{}: {}",
            self.locale.t("risk.suspicious_chain"),
            patterns.join(", "),
        );

        Some(Assessment::high(reason, AssessmentSource::ChainTracker))
    }

    /// Look up a cached assessment.
    fn cache_lookup(&self, input: &HookInput) -> Option<Assessment> {
        let cache = self.cache.as_ref()?;
        let key = normalizer::normalize_cache_key(
            &input.tool_name,
            input.command(),
            input.file_path(),
        );

        match cache.get(&key) {
            Ok(Some(mut assessment)) => {
                assessment.source = AssessmentSource::Cache;
                Some(assessment)
            }
            Ok(None) => None,
            Err(e) => {
                tracing::debug!("Cache lookup error: {}", e);
                None
            }
        }
    }

    /// Cache an assessment result.
    fn cache_result(&self, input: &HookInput, assessment: &Assessment) {
        if let Some(ref cache) = self.cache {
            let key = normalizer::normalize_cache_key(
                &input.tool_name,
                input.command(),
                input.file_path(),
            );
            if let Err(e) = cache.set(&key, assessment) {
                tracing::debug!("Cache write error: {}", e);
            }
        }
    }

    /// Record an operation to the chain tracker.
    fn record_to_chain(&self, input: &HookInput, assessment: &Assessment) {
        let mut tracker = self.chain_tracker.lock();
        tracker.record(
            &self.session_id,
            &input.tool_name,
            input.command(),
            input.file_path(),
            assessment.level,
        );
    }

    /// Assess with an explicit session ID (used by daemon for per-client isolation).
    ///
    /// Each Claude Code window has its own session. The daemon uses this to keep
    /// chain tracking separate per window, so operations from window A don't
    /// pollute the chain context of window B.
    #[cfg(unix)]
    pub async fn assess_with_session(&self, input: &HookInput, session_id: Option<&str>) -> Assessment {
        let sid = session_id.unwrap_or(&self.session_id);
        let start = Instant::now();

        let result = self.assess_inner_with_session(input, sid).await;
        let assessment = result.with_duration(start.elapsed());

        // Record to chain tracker under the caller's session
        {
            let mut tracker = self.chain_tracker.lock();
            tracker.record(
                sid,
                &input.tool_name,
                input.command(),
                input.file_path(),
                assessment.level,
            );
        }

        if assessment.source != AssessmentSource::Cache {
            self.cache_result(input, &assessment);
        }

        assessment
    }

    /// Inner assess using a specific session ID for chain context.
    #[cfg(unix)]
    async fn assess_inner_with_session(&self, input: &HookInput, session_id: &str) -> Assessment {
        let tool_name = input.tool_name.as_str();
        let command = input.command();
        let file_path = input.file_path();

        // Layer 1-4 same as assess_inner
        if let Some(assessment) = fast_rules::fast_check_with_command(tool_name, command, file_path, &self.locale) {
            return assessment;
        }
        {
            let rules = self.custom_rules.read();
            if let Some(assessment) = rules.check(tool_name, command, file_path, &self.rule_context) {
                return assessment;
            }
        }
        if let Some(assessment) = self.cache_lookup(input) {
            return assessment;
        }
        if tool_name == "Bash" {
            if let Some(cmd) = command {
                let analyzer = self.bash_analyzer.lock();
                if let Some(assessment) = analyzer.analyze(cmd) {
                    return assessment;
                }
            }
        }

        // Layer 5: Chain context with caller's session ID
        let chain_context = {
            let tracker = self.chain_tracker.lock();
            tracker.get_context(session_id)
        };
        if let Some(assessment) = self.assess_chain_context(&chain_context) {
            return assessment;
        }

        // Layer 6: AI
        if let Some(ref ai_client) = self.ai_client {
            match ai_client.assess(input, &chain_context) {
                Ok(assessment) => return assessment,
                Err(e) => {
                    tracing::warn!("AI assessment failed: {}", e);
                }
            }
        }

        let reason = format!(
            "{}: {} - {}",
            self.locale.t("risk.unknown_op"),
            tool_name,
            self.locale.t("risk.needs_confirm"),
        );
        Assessment::medium(reason, AssessmentSource::Fallback)
    }

    /// Get the locale.
    pub fn locale(&self) -> &Locale {
        &self.locale
    }

    /// Get the session ID.
    pub fn session_id(&self) -> &str {
        &self.session_id
    }
}

#[cfg(test)]
impl AssessmentEngine {
    /// Create an engine with all components. Test-only constructor.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        custom_rules: Arc<RwLock<CompiledRuleSet>>,
        cache: Option<SqliteCache>,
        ai_client: Option<ClaudeCliClient>,
        bash_analyzer: BashAnalyzer,
        chain_tracker: Arc<Mutex<ChainTracker>>,
        locale: Locale,
        session_id: String,
        rule_context: RuleContext,
    ) -> Self {
        Self {
            custom_rules,
            cache,
            ai_client,
            bash_analyzer: Mutex::new(bash_analyzer),
            chain_tracker,
            locale,
            session_id,
            rule_context,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::risk::RiskLevel;
    use serde_json::json;

    fn make_engine() -> AssessmentEngine {
        AssessmentEngine::new(
            Arc::new(RwLock::new(CompiledRuleSet::empty())),
            None,
            None,
            BashAnalyzer::new(),
            Arc::new(Mutex::new(ChainTracker::new())),
            Locale::En,
            "test-session".to_string(),
            RuleContext::default(),
        )
    }

    fn make_input(tool_name: &str, tool_input: serde_json::Value) -> HookInput {
        HookInput {
            tool_name: tool_name.to_string(),
            tool_input,
        }
    }

    #[tokio::test]
    async fn test_readonly_tool_fast_path() {
        let engine = make_engine();
        let input = make_input("Read", json!({"file_path": "/src/main.rs"}));
        let result = engine.assess(&input).await;
        assert_eq!(result.level, RiskLevel::Low);
        assert_eq!(result.source, AssessmentSource::FastRule);
    }

    #[tokio::test]
    async fn test_safe_file_edit() {
        let engine = make_engine();
        let input = make_input("Edit", json!({"file_path": "/src/main.rs", "old_string": "foo", "new_string": "bar"}));
        let result = engine.assess(&input).await;
        assert_eq!(result.level, RiskLevel::Low);
        assert_eq!(result.source, AssessmentSource::FastRule);
    }

    #[tokio::test]
    async fn test_bash_falls_through_to_fallback() {
        let engine = make_engine();
        let input = make_input("Bash", json!({"command": "some-unknown-cmd"}));
        let result = engine.assess(&input).await;
        // Without AI or AST hits, should fall through to Fallback
        assert_eq!(result.source, AssessmentSource::Fallback);
        assert_eq!(result.level, RiskLevel::Medium);
    }

    #[tokio::test]
    async fn test_chain_tracking() {
        let engine = make_engine();

        let input1 = make_input("Read", json!({"file_path": "/src/main.rs"}));
        let result = engine.assess(&input1).await;

        // Chain tracking is internal; verify the assessment succeeded
        assert_eq!(result.level, RiskLevel::Low);
    }

    #[tokio::test]
    async fn test_custom_rule_override() {
        let toml = r#"
[[rules]]
name = "allow-cargo"
risk = "low"
reason = "Cargo commands are safe"

[rules.match]
tool = "Bash"
command = "cargo *"
"#;
        let config = RuleConfig::from_toml(toml).unwrap();
        let compiled = config.compile();

        let engine = AssessmentEngine::new(
            Arc::new(RwLock::new(compiled)),
            None,
            None,
            BashAnalyzer::new(),
            Arc::new(Mutex::new(ChainTracker::new())),
            Locale::En,
            "test".to_string(),
            RuleContext::default(),
        );

        // Use "cargo publish" — not in the safe whitelist, so it reaches custom rules
        let input = make_input("Bash", json!({"command": "cargo publish"}));
        let result = engine.assess(&input).await;
        assert_eq!(result.level, RiskLevel::Low);
        assert_eq!(result.source, AssessmentSource::CustomRule);
    }

    #[tokio::test]
    async fn test_sensitive_file_not_fast_ruled() {
        let engine = make_engine();
        let input = make_input("Edit", json!({"file_path": "/app/.env", "old_string": "a", "new_string": "b"}));
        let result = engine.assess(&input).await;
        // .env is sensitive, so fast_rules returns None → falls through to Fallback
        assert_ne!(result.source, AssessmentSource::FastRule);
    }
}
