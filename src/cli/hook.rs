use std::io::Read;
use std::process;

use crate::cache::sqlite::{LogEntry, SqliteCache};
use crate::config;
use crate::core::engine::AssessmentEngine;
use crate::core::protocol::{HookInput, HookOutput};

/// Run the PreToolUse hook handler.
///
/// Reads JSON from stdin, runs assessment, outputs JSON to stdout.
/// On ANY error, outputs an allow JSON (never blocks Claude Code).
pub fn run() {
    // Read JSON from stdin
    let mut input_json = String::new();
    if let Err(e) = std::io::stdin().read_to_string(&mut input_json) {
        eprintln!("bark: failed to read stdin: {}", e);
        let output = HookOutput::allow_with_reason("Failed to read hook input");
        println!("{}", output.to_json());
        process::exit(0);
    }

    // Parse the hook input
    let input = match HookInput::from_json(&input_json) {
        Some(input) => input,
        None => {
            eprintln!("bark: invalid JSON input");
            let output = HookOutput::allow_with_reason("Invalid hook input JSON");
            println!("{}", output.to_json());
            process::exit(0);
        }
    };

    // Build a tokio runtime for async operations
    let rt = match tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
    {
        Ok(rt) => rt,
        Err(e) => {
            eprintln!("bark: failed to create runtime: {}", e);
            let output = HookOutput::allow_with_reason("Runtime creation failed");
            println!("{}", output.to_json());
            process::exit(0);
        }
    };

    let output = rt.block_on(async {
        // Try daemon first if socket exists (Unix only)
        #[cfg(unix)]
        {
            let sock = crate::daemon::client::socket_path();
            if sock.exists() {
                if let Ok(output) = crate::daemon::client::assess(&sock, &input).await {
                    return output;
                }
            }
        }

        // Standalone mode
        let cache_path = config::bark_db_path();
        let toml_path = config::bark_toml_path();

        let engine = AssessmentEngine::new_standalone(
            Some(toml_path.as_path()),
            Some(cache_path.as_path()),
        );

        let assessment = engine.assess(&input).await;
        let locale = engine.locale().clone();

        // Send desktop notification for Medium/High risk
        crate::notify::notify_assessment(&assessment, &locale);

        // Log assessment to SQLite
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
            if let Err(e) = cache.log_assessment(&log_entry) {
                eprintln!("bark: failed to log assessment: {}", e);
            }
        }

        HookOutput::from_assessment(&assessment, &locale)
    });

    println!("{}", output.to_json());
}
