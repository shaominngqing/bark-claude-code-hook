
pub const SCHEMA: &str = "
CREATE TABLE IF NOT EXISTS cache (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    command_hash TEXT NOT NULL UNIQUE,
    cache_key TEXT NOT NULL,
    risk_level INTEGER NOT NULL,
    reason TEXT NOT NULL,
    source TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    last_hit TEXT,
    hit_count INTEGER NOT NULL DEFAULT 0,
    ttl_seconds INTEGER NOT NULL DEFAULT 86400
);
CREATE INDEX IF NOT EXISTS idx_cache_hash ON cache(command_hash);
CREATE INDEX IF NOT EXISTS idx_cache_created ON cache(created_at);

CREATE TABLE IF NOT EXISTS assessments_log (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    timestamp TEXT NOT NULL DEFAULT (datetime('now')),
    tool_name TEXT NOT NULL,
    command TEXT,
    file_path TEXT,
    risk_level INTEGER NOT NULL,
    reason TEXT NOT NULL,
    source TEXT NOT NULL,
    duration_ms INTEGER NOT NULL,
    session_id TEXT
);
CREATE INDEX IF NOT EXISTS idx_log_timestamp ON assessments_log(timestamp);

CREATE TABLE IF NOT EXISTS metrics (
    key TEXT PRIMARY KEY,
    value INTEGER NOT NULL DEFAULT 0
);
";

