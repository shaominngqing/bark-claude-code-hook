use crate::config;
use crate::core::engine::AssessmentEngine;
use crate::daemon::server::DaemonServer;

/// Start the bark daemon.
pub fn run() {
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("Failed to create runtime");

    rt.block_on(async {
        let cache_path = config::bark_db_path();
        let toml_path = config::bark_toml_path();
        let engine = AssessmentEngine::new_standalone(
            Some(toml_path.as_path()),
            Some(cache_path.as_path()),
        );

        let server = DaemonServer::new(
            engine,
            config::socket_path(),
            config::pid_path(),
        );

        // Print startup message
        println!("Bark daemon starting...");
        println!("  Socket: {}", config::socket_path().display());
        println!("  PID:    {}", std::process::id());

        if let Err(e) = server.run().await {
            eprintln!("Daemon error: {}", e);
        }
    });
}
