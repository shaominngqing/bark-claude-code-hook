import Foundation

// MARK: - Parsed Rule (from bark.toml)

struct ParsedRule {
    let name: String
    let risk: String
    let reason: String
    let tool: String?
    let command: String?
    let filePath: String?
}

// MARK: - DataStore

class BarkDataStore {

    static let shared = BarkDataStore()

    // Live session stats (from socket events)
    var sessionAssessments: Int = 0
    var sessionHighRisk: Int = 0
    var sessionAllowed: Int = 0
    var sessionDenied: Int = 0

    // Historical data (from SQLite)
    var aggregateStats = AggregateStats()
    var recentLog: [LogEntry] = []
    var cacheStats = CacheStats()

    // Rules (from bark.toml)
    var rules: [ParsedRule] = []

    // State
    var hookEnabled: Bool = false
    var isRunning: Bool = false

    // Paths
    let tomlPath: String
    let settingsPath: String

    private init() {
        let home = FileManager.default.homeDirectoryForCurrentUser.path
        tomlPath = "\(home)/.claude/bark.toml"
        settingsPath = "\(home)/.claude/settings.json"
    }

    /// Refresh all data from disk (SQLite + files).
    func refresh() {
        aggregateStats = SQLiteReader.shared.getStats()
        recentLog = SQLiteReader.shared.getLog(count: 100)
        cacheStats = SQLiteReader.shared.getCacheStats()
        rules = parseTomlRules()
        hookEnabled = checkHookEnabled()
    }

    // MARK: - Hook Status

    private func checkHookEnabled() -> Bool {
        guard let data = try? Data(contentsOf: URL(fileURLWithPath: settingsPath)),
              let json = try? JSONSerialization.jsonObject(with: data) as? [String: Any] else {
            return false
        }

        // Check hooks.PreToolUse array for bark entry
        if let hooks = json["hooks"] as? [String: Any],
           let preToolUse = hooks["PreToolUse"] as? [[String: Any]] {
            return preToolUse.contains { entry in
                if let hookList = entry["hooks"] as? [[String: Any]] {
                    return hookList.contains { h in
                        (h["command"] as? String)?.contains("bark") ?? false
                    }
                }
                if let cmd = entry["command"] as? String {
                    return cmd.contains("bark")
                }
                return false
            }
        }

        // Flat format
        if let preToolUse = json["PreToolUse"] as? [[String: Any]] {
            return preToolUse.contains { entry in
                (entry["command"] as? String)?.contains("bark") ?? false
            }
        }

        return false
    }

    // MARK: - TOML Rules Parser

    private func parseTomlRules() -> [ParsedRule] {
        guard FileManager.default.fileExists(atPath: tomlPath),
              let content = try? String(contentsOfFile: tomlPath, encoding: .utf8) else {
            return []
        }

        var rules: [ParsedRule] = []
        var currentRule: [String: String] = [:]
        var currentSection = ""  // "", "rules", "rules.match", "rules.conditions"

        for rawLine in content.components(separatedBy: "\n") {
            let line = rawLine.trimmingCharacters(in: .whitespaces)

            // Skip comments and empty lines
            if line.isEmpty || line.hasPrefix("#") { continue }

            // Section headers
            if line == "[[rules]]" {
                // Save previous rule
                if let name = currentRule["name"] {
                    rules.append(ParsedRule(
                        name: name,
                        risk: currentRule["risk"] ?? "medium",
                        reason: currentRule["reason"] ?? "",
                        tool: currentRule["tool"],
                        command: currentRule["command"],
                        filePath: currentRule["file_path"]
                    ))
                }
                currentRule = [:]
                currentSection = "rules"
                continue
            }

            if line == "[rules.match]" {
                currentSection = "rules.match"
                continue
            }

            if line == "[rules.conditions]" {
                currentSection = "rules.conditions"
                continue
            }

            // Key = "value" pairs
            if let eqIdx = line.firstIndex(of: "=") {
                let key = line[line.startIndex..<eqIdx].trimmingCharacters(in: .whitespaces)
                var val = line[line.index(after: eqIdx)...].trimmingCharacters(in: .whitespaces)
                // Remove quotes
                if val.hasPrefix("\"") && val.hasSuffix("\"") {
                    val = String(val.dropFirst().dropLast())
                }

                switch currentSection {
                case "rules":
                    currentRule[key] = val
                case "rules.match":
                    currentRule[key] = val
                case "rules.conditions":
                    currentRule["cond_\(key)"] = val
                default:
                    break
                }
            }
        }

        // Save last rule
        if let name = currentRule["name"] {
            rules.append(ParsedRule(
                name: name,
                risk: currentRule["risk"] ?? "medium",
                reason: currentRule["reason"] ?? "",
                tool: currentRule["tool"],
                command: currentRule["command"],
                filePath: currentRule["file_path"]
            ))
        }

        return rules
    }
}
