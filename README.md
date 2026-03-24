# ⚡ Risk Guard for Claude Code

English | [中文](./README.zh-CN.md)

AI-powered risk assessment hook for [Claude Code](https://docs.anthropic.com/en/docs/claude-code). Automatically evaluates the risk level of every tool call — allow, notify, or block.

## The Problem

When running multiple Claude Code sessions in parallel, even safe operations require repeated manual confirmation, breaking your flow and slowing you down.

## How It Works

Risk Guard intercepts every tool call and makes smart decisions:

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
curl -fsSL https://raw.githubusercontent.com/shaominngqing/Risk-Guard/main/install.sh | bash
```

Or clone and install locally:

```bash
git clone https://github.com/shaominngqing/Risk-Guard.git
bash Risk-Guard/install.sh
```

> Takes effect in new Claude Code sessions automatically.

## Usage

```bash
risk-guard status         # Show status
risk-guard on / off       # Enable / disable
risk-guard toggle         # Toggle on/off
risk-guard test <cmd>     # Test a command's risk level
risk-guard cache [clear]  # View / clear cache
risk-guard log [N|clear]  # View / clear logs
risk-guard uninstall      # Completely uninstall
```

### Examples

```bash
risk-guard test ls -la
# ✅ allow  ([Low] Read-only directory listing)  [0.0s]  — cache hit

risk-guard test rm -rf /
# 🚨 ask  ([High] Extremely dangerous recursive root deletion)  [6.8s]

risk-guard test npm install express
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

**Design philosophy**: No hardcoded Bash rules. AI understands command semantics better than regex matching. Cache ensures zero latency for repeated command patterns.

## Requirements

- [Claude Code](https://docs.anthropic.com/en/docs/claude-code) installed
- `jq` in PATH (`brew install jq` / `apt install jq`)
- `claude` CLI (required for AI assessment layer)
- macOS (system notifications) or Linux (requires `notify-send`)

## Uninstall

```bash
risk-guard uninstall
```

## License

MIT
