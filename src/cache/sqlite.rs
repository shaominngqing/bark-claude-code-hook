
use std::collections::HashMap;
use std::path::Path;

use anyhow::{Context, Result};
use md5::{Digest, Md5};
use parking_lot::Mutex;
use rusqlite::{params, Connection, OptionalExtension};

use crate::cache::schema;
use crate::core::risk::{Assessment, AssessmentSource, RiskLevel};

/// Statistics about the cache table.
#[derive(Debug, Clone)]
pub struct CacheStats {
    pub count: usize,
    pub size_bytes: u64,
}

/// A single entry in the cache table.
#[derive(Debug, Clone)]
pub struct CacheEntry {
    pub cache_key: String,
    pub risk_level: RiskLevel,
    pub hit_count: i64,
}

/// A single entry in the assessments_log table.
#[derive(Debug, Clone)]
pub struct LogEntry {
    pub timestamp: String,
    pub tool_name: String,
    pub command: Option<String>,
    pub file_path: Option<String>,
    pub risk_level: RiskLevel,
    pub reason: String,
    pub source: String,
    pub duration_ms: u64,
    pub session_id: Option<String>,
}

/// Aggregate statistics derived from the assessments_log.
#[derive(Debug, Clone)]
pub struct Stats {
    pub total: usize,
    pub by_source: HashMap<String, usize>,
    pub by_level: HashMap<String, usize>,
    pub cache_hit_rate: f64,
}

/// SQLite-backed cache for risk assessments.
///
/// Thread-safe via `parking_lot::Mutex` on the connection. All public
/// methods lock internally, so callers never need to worry about
/// synchronization.
pub struct SqliteCache {
    conn: Mutex<Connection>,
}

/// Compute MD5 hex digest of a string.
fn md5_hex(input: &str) -> String {
    let mut hasher = Md5::new();
    hasher.update(input.as_bytes());
    format!("{:x}", hasher.finalize())
}

/// Convert a source string back to an `AssessmentSource`.
fn assessment_source_from_str(s: &str) -> AssessmentSource {
    match s {
        "FAST" => AssessmentSource::FastRule,
        "RULE" => AssessmentSource::CustomRule,
        "AST" => AssessmentSource::AstAnalysis,
        "CACHE" => AssessmentSource::Cache,
        "AI" => AssessmentSource::AI,
        "PLUGIN" => AssessmentSource::Plugin,
        "CHAIN" => AssessmentSource::ChainTracker,
        "FALLBACK" => AssessmentSource::Fallback,
        _ => AssessmentSource::FastRule,
    }
}

impl SqliteCache {
    /// Open (or create) the SQLite database at `db_path`, run schema migrations,
    /// and enable WAL mode for better concurrent read performance.
    pub fn new(db_path: &Path) -> Result<Self> {
        let conn = Connection::open(db_path)
            .with_context(|| format!("failed to open cache database at {}", db_path.display()))?;

        // Enable WAL mode for better concurrency
        conn.execute_batch("PRAGMA journal_mode=WAL;")?;
        // Run schema creation (idempotent due to IF NOT EXISTS)
        conn.execute_batch(schema::SCHEMA)?;

        Ok(Self {
            conn: Mutex::new(conn),
        })
    }

    /// Alias for `new` -- used by the engine to open a cache database.
    pub fn open(db_path: &Path) -> Result<Self> {
        Self::new(db_path)
    }

    /// Look up a cached assessment by key. Returns `None` if not found or expired.
    /// On hit, updates `hit_count` and `last_hit`.
    pub fn get(&self, key: &str) -> Result<Option<Assessment>> {
        let hash = md5_hex(key);
        let conn = self.conn.lock();

        let result: Option<(i64, String, String)> = conn
            .query_row(
                "SELECT risk_level, reason, source FROM cache
                 WHERE command_hash = ?1
                   AND datetime(created_at, '+' || ttl_seconds || ' seconds') >= datetime('now')",
                params![hash],
                |row| {
                    Ok((
                        row.get::<_, i64>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, String>(2)?,
                    ))
                },
            )
            .optional()?;

        match result {
            Some((level, reason, source)) => {
                // Update hit count and last_hit timestamp
                conn.execute(
                    "UPDATE cache SET hit_count = hit_count + 1, last_hit = datetime('now')
                     WHERE command_hash = ?1",
                    params![hash],
                )?;

                let risk_level = RiskLevel::from_u8(level as u8);
                let source = assessment_source_from_str(&source);
                let mut assessment = Assessment::new(risk_level, reason, source);
                assessment.source = AssessmentSource::Cache;

                Ok(Some(assessment))
            }
            None => Ok(None),
        }
    }

    /// Insert or update a cache entry for the given key.
    pub fn set(&self, key: &str, assessment: &Assessment) -> Result<()> {
        let hash = md5_hex(key);
        let conn = self.conn.lock();

        conn.execute(
            "INSERT INTO cache (command_hash, cache_key, risk_level, reason, source)
             VALUES (?1, ?2, ?3, ?4, ?5)
             ON CONFLICT(command_hash) DO UPDATE SET
               risk_level = excluded.risk_level,
               reason = excluded.reason,
               source = excluded.source,
               created_at = datetime('now'),
               last_hit = NULL,
               hit_count = 0",
            params![
                hash,
                key,
                assessment.level as u8,
                assessment.reason,
                assessment.source.to_string(),
            ],
        )?;

        Ok(())
    }

    /// Delete all entries from the cache table.
    pub fn clear(&self) -> Result<()> {
        let conn = self.conn.lock();
        conn.execute("DELETE FROM cache", [])?;
        Ok(())
    }

    /// Return basic cache statistics: entry count and database file size.
    pub fn stats(&self) -> Result<CacheStats> {
        let conn = self.conn.lock();

        let count: usize = conn.query_row("SELECT COUNT(*) FROM cache", [], |row| {
            row.get::<_, usize>(0)
        })?;

        let size_bytes: u64 = conn
            .query_row(
                "SELECT page_count * page_size FROM pragma_page_count(), pragma_page_size()",
                [],
                |row| row.get::<_, u64>(0),
            )
            .unwrap_or(0);

        Ok(CacheStats { count, size_bytes })
    }

    /// Return the most recent `n` cache entries, ordered newest first.
    pub fn recent(&self, n: usize) -> Result<Vec<CacheEntry>> {
        let conn = self.conn.lock();

        let mut stmt = conn.prepare(
            "SELECT cache_key, risk_level, hit_count
             FROM cache
             ORDER BY created_at DESC
             LIMIT ?1",
        )?;

        let entries = stmt
            .query_map(params![n as i64], |row| {
                Ok(CacheEntry {
                    cache_key: row.get(0)?,
                    risk_level: RiskLevel::from_u8(row.get::<_, u8>(1)?),
                    hit_count: row.get(2)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(entries)
    }

    /// Insert a log entry into the assessments_log table.
    pub fn log_assessment(&self, entry: &LogEntry) -> Result<()> {
        let conn = self.conn.lock();

        conn.execute(
            "INSERT INTO assessments_log (tool_name, command, file_path, risk_level, reason, source, duration_ms, session_id)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                entry.tool_name,
                entry.command,
                entry.file_path,
                entry.risk_level as u8,
                entry.reason,
                entry.source,
                entry.duration_ms as i64,
                entry.session_id,
            ],
        )?;

        Ok(())
    }

    /// Return the most recent `n` assessment log entries, ordered newest first.
    pub fn get_log(&self, n: usize) -> Result<Vec<LogEntry>> {
        let conn = self.conn.lock();

        let mut stmt = conn.prepare(
            "SELECT timestamp, tool_name, command, file_path, risk_level, reason, source, duration_ms, session_id
             FROM assessments_log
             ORDER BY timestamp DESC
             LIMIT ?1",
        )?;

        let entries = stmt
            .query_map(params![n as i64], |row| {
                Ok(LogEntry {
                    timestamp: row.get(0)?,
                    tool_name: row.get(1)?,
                    command: row.get(2)?,
                    file_path: row.get(3)?,
                    risk_level: RiskLevel::from_u8(row.get::<_, u8>(4)?),
                    reason: row.get(5)?,
                    source: row.get(6)?,
                    duration_ms: row.get::<_, i64>(7)? as u64,
                    session_id: row.get(8)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(entries)
    }

    /// Delete all entries from the assessments_log table.
    pub fn clear_log(&self) -> Result<()> {
        let conn = self.conn.lock();
        conn.execute("DELETE FROM assessments_log", [])?;
        Ok(())
    }

    /// Compute aggregate statistics from the assessments_log.
    pub fn get_stats(&self) -> Result<Stats> {
        let conn = self.conn.lock();

        // Total count
        let total: usize =
            conn.query_row("SELECT COUNT(*) FROM assessments_log", [], |row| {
                row.get::<_, usize>(0)
            })?;

        // Breakdown by source
        let mut by_source = HashMap::new();
        {
            let mut stmt =
                conn.prepare("SELECT source, COUNT(*) FROM assessments_log GROUP BY source")?;
            let rows = stmt.query_map([], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, usize>(1)?))
            })?;
            for row in rows {
                let (source, count) = row?;
                by_source.insert(source, count);
            }
        }

        // Breakdown by risk level
        let mut by_level = HashMap::new();
        {
            let mut stmt = conn
                .prepare("SELECT risk_level, COUNT(*) FROM assessments_log GROUP BY risk_level")?;
            let rows = stmt.query_map([], |row| {
                Ok((row.get::<_, u8>(0)?, row.get::<_, usize>(1)?))
            })?;
            for row in rows {
                let (level, count) = row?;
                let label = RiskLevel::from_u8(level).to_string();
                by_level.insert(label, count);
            }
        }

        // Cache hit rate: count of CACHE source / total
        let cache_count = by_source.get("CACHE").copied().unwrap_or(0);
        let cache_hit_rate = if total > 0 {
            cache_count as f64 / total as f64
        } else {
            0.0
        };

        Ok(Stats {
            total,
            by_source,
            by_level,
            cache_hit_rate,
        })
    }

}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    fn test_cache() -> SqliteCache {
        let tmp = NamedTempFile::new().unwrap();
        SqliteCache::new(tmp.path()).unwrap()
    }

    #[test]
    fn test_set_and_get() {
        let cache = test_cache();
        let assessment =
            Assessment::new(RiskLevel::Low, "safe operation", AssessmentSource::FastRule);

        cache.set("ls -la", &assessment).unwrap();
        let result = cache.get("ls -la").unwrap();
        assert!(result.is_some());

        let cached = result.unwrap();
        assert_eq!(cached.level, RiskLevel::Low);
        assert_eq!(cached.reason, "safe operation");
        assert_eq!(cached.source, AssessmentSource::Cache);
    }

    #[test]
    fn test_cache_miss() {
        let cache = test_cache();
        let result = cache.get("nonexistent").unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_clear() {
        let cache = test_cache();
        let assessment = Assessment::new(RiskLevel::Medium, "test", AssessmentSource::AI);

        cache.set("key1", &assessment).unwrap();
        cache.set("key2", &assessment).unwrap();
        assert_eq!(cache.stats().unwrap().count, 2);

        cache.clear().unwrap();
        assert_eq!(cache.stats().unwrap().count, 0);
    }

    #[test]
    fn test_recent() {
        let cache = test_cache();
        for i in 0..5 {
            let assessment = Assessment::new(
                RiskLevel::Low,
                format!("reason {}", i),
                AssessmentSource::FastRule,
            );
            cache.set(&format!("cmd{}", i), &assessment).unwrap();
        }

        let recent = cache.recent(3).unwrap();
        assert_eq!(recent.len(), 3);
    }

    #[test]
    fn test_log_assessment() {
        let cache = test_cache();
        let entry = LogEntry {
            timestamp: String::new(),
            tool_name: "Bash".to_string(),
            command: Some("ls -la".to_string()),
            file_path: None,
            risk_level: RiskLevel::Low,
            reason: "read-only".to_string(),
            source: "FAST".to_string(),
            duration_ms: 5,
            session_id: Some("sess-001".to_string()),
        };

        cache.log_assessment(&entry).unwrap();

        let log = cache.get_log(10).unwrap();
        assert_eq!(log.len(), 1);
        assert_eq!(log[0].tool_name, "Bash");
        assert_eq!(log[0].duration_ms, 5);
    }

    #[test]
    fn test_get_stats_empty() {
        let cache = test_cache();
        let stats = cache.get_stats().unwrap();
        assert_eq!(stats.total, 0);
        assert_eq!(stats.cache_hit_rate, 0.0);
    }

    #[test]
    fn test_md5_deterministic() {
        let h1 = md5_hex("hello world");
        let h2 = md5_hex("hello world");
        assert_eq!(h1, h2);
        assert_ne!(md5_hex("a"), md5_hex("b"));
    }

    #[test]
    fn test_open_alias() {
        let tmp = NamedTempFile::new().unwrap();
        let _cache = SqliteCache::open(tmp.path()).unwrap();
    }
}
