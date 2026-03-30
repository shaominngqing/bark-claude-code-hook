import Foundation

// MARK: - Data Models

struct LogEntry {
    let timestamp: String
    let toolName: String
    let command: String?
    let filePath: String?
    let riskLevel: Int  // 0=Low, 1=Medium, 2=High
    let reason: String
    let source: String
    let durationMs: Int
    let sessionId: String?

    var riskString: String {
        switch riskLevel {
        case 0: return "LOW"
        case 1: return "MEDIUM"
        case 2: return "HIGH"
        default: return "LOW"
        }
    }

    var displayDescription: String {
        if let cmd = command, !cmd.isEmpty {
            return cmd
        }
        if let fp = filePath, !fp.isEmpty {
            return "\(toolName): \(fp)"
        }
        return toolName
    }
}

struct AggregateStats {
    var total: Int = 0
    var byLevel: [String: Int] = ["LOW": 0, "MEDIUM": 0, "HIGH": 0]
    var bySource: [String: Int] = [:]
    var cacheHitRate: Double = 0
}

struct CacheStats {
    var count: Int = 0
    var sizeBytes: Int = 0
}

// MARK: - SQLiteReader

class SQLiteReader {

    static let shared = SQLiteReader()

    private let dbPath: String

    init() {
        let home = FileManager.default.homeDirectoryForCurrentUser.path
        dbPath = "\(home)/.claude/hooks/bark.db"
    }

    // MARK: - Log Queries

    func getLog(count: Int = 100, riskFilter: Int? = nil) -> [LogEntry] {
        guard let db = openDB() else { return [] }
        defer { sqlite3_close(db) }

        var sql = "SELECT timestamp, tool_name, command, file_path, risk_level, reason, source, duration_ms, session_id FROM assessments_log"
        if let filter = riskFilter {
            sql += " WHERE risk_level = \(filter)"
        }
        sql += " ORDER BY id DESC LIMIT \(count)"

        var stmt: OpaquePointer?
        guard sqlite3_prepare_v2(db, sql, -1, &stmt, nil) == SQLITE_OK else { return [] }
        defer { sqlite3_finalize(stmt) }

        var entries: [LogEntry] = []
        while sqlite3_step(stmt) == SQLITE_ROW {
            entries.append(LogEntry(
                timestamp: colText(stmt, 0),
                toolName: colText(stmt, 1),
                command: colOptText(stmt, 2),
                filePath: colOptText(stmt, 3),
                riskLevel: Int(sqlite3_column_int(stmt, 4)),
                reason: colText(stmt, 5),
                source: colText(stmt, 6),
                durationMs: Int(sqlite3_column_int(stmt, 7)),
                sessionId: colOptText(stmt, 8)
            ))
        }
        return entries
    }

    func clearLog() {
        guard let db = openDB() else { return }
        defer { sqlite3_close(db) }
        sqlite3_exec(db, "DELETE FROM assessments_log", nil, nil, nil)
    }

    // MARK: - Stats Queries

    func getStats() -> AggregateStats {
        guard let db = openDB() else { return AggregateStats() }
        defer { sqlite3_close(db) }

        var stats = AggregateStats()

        // Total
        var stmt: OpaquePointer?
        if sqlite3_prepare_v2(db, "SELECT COUNT(*) FROM assessments_log", -1, &stmt, nil) == SQLITE_OK {
            if sqlite3_step(stmt) == SQLITE_ROW {
                stats.total = Int(sqlite3_column_int(stmt, 0))
            }
            sqlite3_finalize(stmt)
        }

        // By level
        let levelNames = ["LOW", "MEDIUM", "HIGH"]
        for (i, name) in levelNames.enumerated() {
            if sqlite3_prepare_v2(db, "SELECT COUNT(*) FROM assessments_log WHERE risk_level = \(i)", -1, &stmt, nil) == SQLITE_OK {
                if sqlite3_step(stmt) == SQLITE_ROW {
                    stats.byLevel[name] = Int(sqlite3_column_int(stmt, 0))
                }
                sqlite3_finalize(stmt)
            }
        }

        // By source
        if sqlite3_prepare_v2(db, "SELECT source, COUNT(*) FROM assessments_log GROUP BY source", -1, &stmt, nil) == SQLITE_OK {
            while sqlite3_step(stmt) == SQLITE_ROW {
                let source = colText(stmt, 0)
                let count = Int(sqlite3_column_int(stmt, 1))
                stats.bySource[source] = count
            }
            sqlite3_finalize(stmt)
        }

        // Cache hit rate
        if stats.total > 0 {
            let cacheHits = stats.bySource["CACHE"] ?? 0
            stats.cacheHitRate = Double(cacheHits) / Double(stats.total)
        }

        return stats
    }

    // MARK: - Cache Queries

    func getCacheStats() -> CacheStats {
        guard let db = openDB() else { return CacheStats() }
        defer { sqlite3_close(db) }

        var stats = CacheStats()

        var stmt: OpaquePointer?
        if sqlite3_prepare_v2(db, "SELECT COUNT(*) FROM cache", -1, &stmt, nil) == SQLITE_OK {
            if sqlite3_step(stmt) == SQLITE_ROW {
                stats.count = Int(sqlite3_column_int(stmt, 0))
            }
            sqlite3_finalize(stmt)
        }

        // DB file size
        if let attrs = try? FileManager.default.attributesOfItem(atPath: dbPath),
           let size = attrs[.size] as? Int {
            stats.sizeBytes = size
        }

        return stats
    }

    func clearCache() {
        guard let db = openDB() else { return }
        defer { sqlite3_close(db) }
        sqlite3_exec(db, "DELETE FROM cache", nil, nil, nil)
    }

    // MARK: - Private

    private func openDB() -> OpaquePointer? {
        var db: OpaquePointer?
        guard FileManager.default.fileExists(atPath: dbPath) else { return nil }
        guard sqlite3_open_v2(dbPath, &db, SQLITE_OPEN_READONLY | SQLITE_OPEN_NOMUTEX, nil) == SQLITE_OK else {
            return nil
        }
        return db
    }

    private func colText(_ stmt: OpaquePointer?, _ col: Int32) -> String {
        if let cStr = sqlite3_column_text(stmt, col) {
            return String(cString: cStr)
        }
        return ""
    }

    private func colOptText(_ stmt: OpaquePointer?, _ col: Int32) -> String? {
        if sqlite3_column_type(stmt, col) == SQLITE_NULL { return nil }
        if let cStr = sqlite3_column_text(stmt, col) {
            return String(cString: cStr)
        }
        return nil
    }
}
