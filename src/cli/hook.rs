use std::io::Read;
use std::process;

use crate::cache::sqlite::{LogEntry, SqliteCache};
use crate::config;
use crate::core::engine::AssessmentEngine;
use crate::core::protocol::{HookInput, HookOutput};

/// Derive a stable session ID from environment.
///
/// Claude Code sets CLAUDE_SESSION_ID or similar env vars per session.
/// If not available, fall back to parent PID (each Claude Code window
/// is a separate process tree, so PPID is a reasonable proxy).
#[cfg(unix)]
fn get_session_id() -> String {
    // Try Claude Code's session env vars
    if let Ok(sid) = std::env::var("CLAUDE_SESSION_ID") {
        return sid;
    }
    // Fallback: use parent PID as session proxy
    #[cfg(unix)]
    {
        format!("ppid-{}", std::os::unix::process::parent_id())
    }
    #[cfg(not(unix))]
    {
        format!("pid-{}", std::process::id())
    }
}

/// Run the PreToolUse hook handler.
///
/// Flow:
/// 1. Try daemon (auto-spawn if not running, Unix only)
/// 2. Fallback to standalone mode
/// 3. Always output valid JSON, never block Claude Code
pub fn run() {
    // Read JSON from stdin
    let mut input_json = String::new();
    if let Err(e) = std::io::stdin().read_to_string(&mut input_json) {
        eprintln!("bark: failed to read stdin: {}", e);
        let output = HookOutput::allow_with_reason("Failed to read hook input");
        println!("{}", output.to_json());
        process::exit(0);
    }

    let input = match HookInput::from_json(&input_json) {
        Some(input) => input,
        None => {
            let output = HookOutput::allow_with_reason("Invalid hook input JSON");
            println!("{}", output.to_json());
            process::exit(0);
        }
    };

    let rt = match tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
    {
        Ok(rt) => rt,
        Err(_) => {
            let output = HookOutput::allow_with_reason("Runtime creation failed");
            println!("{}", output.to_json());
            process::exit(0);
        }
    };

    let output = rt.block_on(async {
        // Try daemon first (Unix only)
        #[cfg(unix)]
        {
            let session_id = get_session_id();

            // Auto-spawn daemon if not running
            crate::daemon::client::spawn_daemon().ok();

            let sock = crate::daemon::client::socket_path();
            if sock.exists() {
                if let Ok(output) = crate::daemon::client::assess(&sock, &input, &session_id).await {
                    return output;
                }
            }
        }

        // Standalone mode fallback
        run_standalone(&input).await
    });

    println!("{}", output.to_json());
}

/// Run assessment in standalone mode (no daemon).
async fn run_standalone(input: &HookInput) -> HookOutput {
    let cache_path = config::bark_db_path();
    let toml_path = config::bark_toml_path();

    let engine = AssessmentEngine::new_standalone(
        Some(toml_path.as_path()),
        Some(cache_path.as_path()),
    );

    let assessment = engine.assess(input).await;
    let locale = engine.locale().clone();

    // Try helper notification (may override permission decision for High risk)
    let decision_override = crate::notify::notify_and_decide(&assessment, &locale).await;

    // Log to SQLite
    if let Ok(cache) = SqliteCache::open(&config::bark_db_path()) {
        let log_entry = LogEntry {
            timestamp: String::new(),
            tool_name: input.tool_name.clone(),
            command: input.command().map(String::from),
            file_path: input.file_path().map(String::from),
            risk_level: assessment.level,
            reason: assessment.reason.clone(),
            source: assessment.source.to_string(),
            duration_ms: assessment.duration.as_millis() as u64,
            session_id: Some(engine.session_id().to_string()),
        };
        cache.log_assessment(&log_entry).ok();
    }

    let mut output = HookOutput::from_assessment(&assessment, &locale);
    if let Some(decision) = decision_override {
        output.hook_specific_output.permission_decision = decision;
    }
    output
}
