use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use anyhow::{Context, Result};
use parking_lot::Mutex;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixListener;
use tokio::signal;

use crate::cache::sqlite::{LogEntry, SqliteCache};
use crate::config;
use crate::core::engine::AssessmentEngine;
use crate::core::protocol::HookOutput;
use crate::daemon::protocol::{DaemonRequest, DaemonResponse};

/// How long the daemon waits without any request before auto-exiting.
const IDLE_TIMEOUT: Duration = Duration::from_secs(30 * 60); // 30 minutes

/// Shared state for the daemon, accessible from connection handlers.
struct DaemonState {
    engine: AssessmentEngine,
    start_time: Instant,
    last_activity: Mutex<Instant>,
    assessment_count: AtomicU64,
    shutdown: AtomicBool,
}

impl DaemonState {
    fn touch(&self) {
        *self.last_activity.lock() = Instant::now();
    }

    fn idle_duration(&self) -> Duration {
        self.last_activity.lock().elapsed()
    }
}

/// The Bark daemon server.
pub struct DaemonServer {
    state: Arc<DaemonState>,
    socket_path: PathBuf,
    pid_path: PathBuf,
}

impl DaemonServer {
    pub fn new(engine: AssessmentEngine, socket_path: PathBuf, pid_path: PathBuf) -> Self {
        let now = Instant::now();
        Self {
            state: Arc::new(DaemonState {
                engine,
                start_time: now,
                last_activity: Mutex::new(now),
                assessment_count: AtomicU64::new(0),
                shutdown: AtomicBool::new(false),
            }),
            socket_path,
            pid_path,
        }
    }

    /// Run the daemon server with idle timeout.
    ///
    /// Auto-exits after 30 minutes of no requests.
    pub async fn run(&self) -> Result<()> {
        // Remove stale socket file if it exists
        if self.socket_path.exists() {
            std::fs::remove_file(&self.socket_path).ok();
        }

        // Write PID file
        let pid = std::process::id();
        std::fs::write(&self.pid_path, pid.to_string())
            .with_context(|| format!("failed to write PID file at {:?}", self.pid_path))?;

        // Bind the Unix domain socket
        let listener = UnixListener::bind(&self.socket_path)
            .with_context(|| format!("failed to bind Unix socket at {:?}", self.socket_path))?;

        tracing::info!(
            socket = %self.socket_path.display(),
            pid = pid,
            "Bark daemon started (idle timeout: 30m)"
        );

        // Accept loop with idle timeout and graceful shutdown
        loop {
            // Check idle timeout
            if self.state.idle_duration() > IDLE_TIMEOUT {
                tracing::info!("Idle timeout reached, shutting down");
                break;
            }

            // Check shutdown flag
            if self.state.shutdown.load(Ordering::Relaxed) {
                tracing::info!("Shutdown requested");
                break;
            }

            tokio::select! {
                // Accept new connection
                accept_result = listener.accept() => {
                    match accept_result {
                        Ok((stream, _addr)) => {
                            self.state.touch();
                            let state = Arc::clone(&self.state);
                            tokio::spawn(async move {
                                if let Err(e) = handle_connection(stream, state).await {
                                    tracing::warn!("Connection handler error: {}", e);
                                }
                            });
                        }
                        Err(e) => {
                            tracing::warn!("Failed to accept connection: {}", e);
                        }
                    }
                }
                // Ctrl+C
                _ = signal::ctrl_c() => {
                    tracing::info!("Received shutdown signal");
                    break;
                }
                // Periodic idle check (every 60s)
                _ = tokio::time::sleep(Duration::from_secs(60)) => {
                    // Loop will check idle_duration at top
                }
            }
        }

        self.cleanup();
        tracing::info!("Bark daemon stopped");
        Ok(())
    }

    fn cleanup(&self) {
        std::fs::remove_file(&self.socket_path).ok();
        std::fs::remove_file(&self.pid_path).ok();
    }
}

/// Handle a single client connection.
async fn handle_connection(
    stream: tokio::net::UnixStream,
    state: Arc<DaemonState>,
) -> Result<()> {
    let (reader, mut writer) = stream.into_split();
    let mut reader = BufReader::new(reader);
    let mut line = String::new();

    reader
        .read_line(&mut line)
        .await
        .context("failed to read request line")?;

    let line = line.trim();
    if line.is_empty() {
        return Ok(());
    }

    let request: DaemonRequest = serde_json::from_str(line)
        .with_context(|| format!("failed to parse request JSON: {}", line))?;

    let response = process_request(request, &state).await;

    let mut response_json = serde_json::to_string(&response)
        .context("failed to serialize response")?;
    response_json.push('\n');

    writer
        .write_all(response_json.as_bytes())
        .await
        .context("failed to write response")?;

    writer.flush().await.ok();
    Ok(())
}

/// Process a single daemon request.
async fn process_request(
    request: DaemonRequest,
    state: &Arc<DaemonState>,
) -> DaemonResponse {
    match request {
        DaemonRequest::Assess { payload, session_id } => {
            let start = Instant::now();

            // Use the client's session_id for chain tracking isolation
            let assessment = state.engine.assess_with_session(&payload, session_id.as_deref()).await;
            state.assessment_count.fetch_add(1, Ordering::Relaxed);

            let locale = state.engine.locale().clone();
            let mut output = HookOutput::from_assessment(&assessment, &locale);
            let duration_ms = start.elapsed().as_millis() as u64;

            // Try helper notification (may override permission decision for High risk)
            if let Some(decision) = crate::notify::notify_and_decide(&assessment, &locale).await {
                output.hook_specific_output.permission_decision = decision;
            }

            if let Ok(cache) = SqliteCache::open(&config::bark_db_path()) {
                let log_entry = LogEntry {
                    timestamp: String::new(),
                    tool_name: payload.tool_name.clone(),
                    command: payload.command().map(String::from),
                    file_path: payload.file_path().map(String::from),
                    risk_level: assessment.level,
                    reason: assessment.reason.clone(),
                    source: assessment.source.to_string(),
                    duration_ms,
                    session_id,
                };
                cache.log_assessment(&log_entry).ok();
            }

            DaemonResponse::Result {
                payload: output,
                duration_ms,
            }
        }
        DaemonRequest::Status => {
            let uptime_seconds = state.start_time.elapsed().as_secs();
            let assessments = state.assessment_count.load(Ordering::Relaxed);
            let idle_secs = state.idle_duration().as_secs();

            DaemonResponse::Status {
                uptime_seconds,
                assessments,
                cache_entries: 0,
                idle_seconds: idle_secs,
            }
        }
        DaemonRequest::Shutdown => {
            state.shutdown.store(true, Ordering::Relaxed);
            DaemonResponse::Ok
        }
    }
}
