<p align="center">
  <img src="assets/banner.svg" alt="Bark" width="800">
</p>

<p align="center">
  <strong>AI-Powered Risk Assessment for Claude Code</strong><br>
  <sub>Automatically evaluates every tool call — allow, notify, or block.</sub>
</p>

<p align="center">
  English | <a href="./README.zh-CN.md">中文</a>
</p>

<p align="center">
  <video src="https://github.com/shaominngqing/Bark/raw/main/assets/bark-demo.mp4" width="800" controls autoplay muted loop></video>
</p>

## The Problem

When running multiple Claude Code sessions in parallel, even safe operations require repeated manual confirmation, breaking your flow and slowing you down.

## How It Works

Bark intercepts every tool call and makes smart decisions:

| Scenario | Action | Latency |
|---|---|---|
| Read-only tools (Read, Grep, Glob...) | Silent allow | 0s |
| Normal file edits | Silent allow | 0s |
| Bash commands (cached) | Auto allow / notify | 0s |
| Bash commands (first time) | AI risk assessment | ~7s |
| High-risk operations | System notification + terminal confirmation | — |

### Risk Levels

- **Level 0** (Low) — Silent allow. Read-only commands, builds, tests.
- **Level 1** (Medium) — System notification + auto allow. Package installs, git push, file moves.
- **Level 2** (High) — Notification + sound + terminal confirmation. Force push, `rm -rf /`, database drops, remote code execution.

## Install

```bash
curl -fsSL https://raw.githubusercontent.com/shaominngqing/Bark/main/install.sh | bash
```

Or clone and install locally:

```bash
git clone https://github.com/shaominngqing/Bark.git
bash Bark/install.sh
```

> Takes effect in new Claude Code sessions automatically.

## Usage

```bash
bark status         # Show status
bark on / off       # Enable / disable
bark toggle         # Toggle on/off
bark test <cmd>     # Test a command's risk level
bark test -v <cmd>  # Test with verbose debug output
bark test -n <cmd>  # Dry-run: show assessment, always allow
bark cache [clear]  # View / clear cache
bark log [N|clear]  # View / clear logs
bark stats          # Show statistics dashboard
bark rules [edit]   # View / edit custom rules
bark update         # Update to latest version
bark uninstall      # Completely uninstall
```

### Examples

```bash
bark test ls -la
# ✅ allow  ([Low] Read-only directory listing)  [0.0s]  — cache hit

bark test rm -rf /
# 🚨 ask  ([High] Extremely dangerous recursive root deletion)  [6.8s]

bark test npm install express
# ✅ allow  ([Medium] Install npm dependency)  [7.2s]
```

## Architecture

```
Claude Code tool call
        │
        ▼
┌──────────────────────────────┐
│  Layer 1: Fast Rules         │  Deterministic tool-level checks
│  Read/Grep/Glob → allow     │  No Bash command rules here
│  Normal file edits → allow  │
└──────────┬───────────────────┘
           │ miss
           ▼
┌──────────────────────────────┐
│  Layer 1.5: Custom Rules     │  User-defined patterns
│  bark.conf             │  allow / notify / block
└──────────┬───────────────────┘
           │ miss
           ▼
┌──────────────────────────────┐
│  Layer 2: Cache Lookup       │  Commands normalized to patterns
│  "rm -rf" → reuse last      │  md5 hash, 24h TTL
│  AI judgment                 │
└──────────┬───────────────────┘
           │ cache miss
           ▼
┌──────────────────────────────┐
│  Layer 3: AI Assessment      │  claude -p understands semantics
│  Returns risk level + reason │  Result cached for next time
└──────────┬───────────────────┘
           │
           ▼
  Level 0 → Silent allow
  Level 1 → System notification + allow
  Level 2 → Notification + sound + terminal confirmation
```

### Custom Rules

Create `~/.claude/hooks/bark.conf` to define your own rules (checked before AI assessment):

```conf
# Format: action: pattern (* wildcard supported)
allow: npm test
allow: npm run *
allow: make *
notify: git push
block: rm -rf /
```

**Design philosophy**: No hardcoded Bash rules. AI understands command semantics better than regex matching. Cache ensures zero latency for repeated command patterns.

## Requirements

- [Claude Code](https://docs.anthropic.com/en/docs/claude-code) installed
- `jq` in PATH (`brew install jq` / `apt install jq`)
- `claude` CLI (required for AI assessment layer)
- macOS (system notifications) or Linux (requires `notify-send`)

## Uninstall

```bash
bark uninstall
```

## License

MIT
