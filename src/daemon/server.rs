
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Instant;

use anyhow::{Context, Result};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixListener;
use tokio::signal;

use crate::cache::sqlite::{LogEntry, SqliteCache};
use crate::config;
use crate::core::engine::AssessmentEngine;
use crate::core::protocol::HookOutput;
use crate::daemon::protocol::{DaemonRequest, DaemonResponse};

/// Shared state for the daemon, accessible from connection handlers.
struct DaemonState {
    engine: AssessmentEngine,
    start_time: Instant,
    assessment_count: AtomicU64,
    shutdown: AtomicBool,
}

/// The Bark daemon server.
///
/// Listens on a Unix domain socket for assessment requests from clients.
/// Keeps the `AssessmentEngine` alive in memory with warm caches,
/// eliminating per-invocation startup cost.
pub struct DaemonServer {
    state: Arc<DaemonState>,
    socket_path: PathBuf,
    pid_path: PathBuf,
}

impl DaemonServer {
    /// Create a new daemon server.
    ///
    /// # Arguments
    /// * `engine` - The assessment engine to use for processing requests.
    /// * `socket_path` - Path to the Unix domain socket to bind.
    /// * `pid_path` - Path to the PID file to write.
    pub fn new(engine: AssessmentEngine, socket_path: PathBuf, pid_path: PathBuf) -> Self {
        Self {
            state: Arc::new(DaemonState {
                engine,
                start_time: Instant::now(),
                assessment_count: AtomicU64::new(0),
                shutdown: AtomicBool::new(false),
            }),
            socket_path,
            pid_path,
        }
    }

    /// Run the daemon server.
    ///
    /// Binds a Unix domain socket, writes a PID file, and enters the
    /// accept loop. Handles `SIGTERM` and `SIGINT` for graceful shutdown.
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
            "Bark daemon started"
        );

        // Accept loop with graceful shutdown on SIGTERM/SIGINT
        loop {
            tokio::select! {
                accept_result = listener.accept() => {
                    match accept_result {
                        Ok((stream, _addr)) => {
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
                _ = signal::ctrl_c() => {
                    tracing::info!("Received shutdown signal");
                    break;
                }
            }

            // Check if a Shutdown request was received
            if self.state.shutdown.load(Ordering::Relaxed) {
                tracing::info!("Shutdown requested via protocol");
                break;
            }
        }

        // Cleanup: remove socket and PID file
        self.cleanup();

        tracing::info!("Bark daemon stopped");
        Ok(())
    }

    /// Remove the socket and PID files.
    fn cleanup(&self) {
        std::fs::remove_file(&self.socket_path).ok();
        std::fs::remove_file(&self.pid_path).ok();
    }
}

/// Handle a single client connection.
///
/// Protocol: read one line of JSON, parse as `DaemonRequest`, process,
/// write `DaemonResponse` as JSON + newline, then close the connection.
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

/// Process a single daemon request and produce a response.
async fn process_request(
    request: DaemonRequest,
    state: &Arc<DaemonState>,
) -> DaemonResponse {
    match request {
        DaemonRequest::Assess { payload, .. } => {
            let start = Instant::now();

            let assessment = state.engine.assess(&payload).await;
            state.assessment_count.fetch_add(1, Ordering::Relaxed);

            let locale = state.engine.locale().clone();
            let output = HookOutput::from_assessment(&assessment, &locale);
            let duration_ms = start.elapsed().as_millis() as u64;

            // Send desktop notification for Medium/High risk
            crate::notify::notify_assessment(&assessment, &locale);

            // Log assessment to SQLite
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
                    session_id: Some(state.engine.session_id().to_string()),
                };
                if let Err(e) = cache.log_assessment(&log_entry) {
                    tracing::warn!("Failed to log assessment: {}", e);
                }
            }

            DaemonResponse::Result {
                payload: output,
                duration_ms,
            }
        }
        DaemonRequest::Status => {
            let uptime_seconds = state.start_time.elapsed().as_secs();
            let assessments = state.assessment_count.load(Ordering::Relaxed);

            DaemonResponse::Status {
                uptime_seconds,
                assessments,
                cache_entries: 0, // TODO: query cache when available
            }
        }
        DaemonRequest::Shutdown => {
            state.shutdown.store(true, Ordering::Relaxed);
            DaemonResponse::Ok
        }
    }
}

