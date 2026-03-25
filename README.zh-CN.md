<p align="center">
  <img src="assets/banner.svg" alt="Bark" width="800">
</p>

<p align="center">
  <strong>为 Claude Code 打造的 AI 风险评估 Hook</strong><br>
  <sub>自动判断每个操作的风险等级，决定放行、通知还是拦截。</sub>
</p>

<p align="center">
  <a href="./README.md">English</a> | 中文
</p>

<p align="center">
  <video src="https://github.com/user-attachments/assets/dc618b98-f39a-4aee-b72f-e60b842c2910" width="800" controls autoplay muted loop></video>
</p>

## 解决什么问题

同时开多个 Claude Code 窗口跑任务时，安全操作也要反复确认，打断工作流、拖慢进度。

## 怎么解决的

Bark 拦截每一次工具调用，智能决策：

| 场景 | 处理方式 | 延迟 |
|---|---|---|
| 只读工具 (Read, Grep, Glob...) | 静默放行 | 0s |
| 普通文件编辑 | 静默放行 | 0s |
| Bash 命令（有缓存） | 自动放行 / 通知 | 0s |
| Bash 命令（首次） | AI 评估风险 | ~7s |
| 高风险操作 | 系统通知 + 终端确认 | — |

### 风险等级

- **Level 0** (低风险) — 静默放行。只读命令、构建、测试。
- **Level 1** (中风险) — macOS 通知 + 自动放行。安装依赖、git push、文件移动。
- **Level 2** (高风险) — 通知 + 声音 + 终端确认。force push、`rm -rf /`、数据库删除、远程代码执行。

## 安装

```bash
curl -fsSL https://raw.githubusercontent.com/shaominngqing/Bark/main/install.sh | bash
```

或者克隆后本地安装：

```bash
git clone https://github.com/shaominngqing/Bark.git
bash Bark/install.sh
```

> 新开的 Claude Code 会话自动生效。

## 使用

```bash
bark status         # 查看状态
bark on / off       # 启用 / 禁用
bark toggle         # 切换开关
bark test <cmd>     # 测试命令风险等级
bark test -v <cmd>  # 详细模式，显示评估过程
bark test -n <cmd>  # 模拟运行，仅展示不拦截
bark cache [clear]  # 查看 / 清空缓存
bark log [N|clear]  # 查看 / 清空日志
bark stats          # 查看统计数据
bark rules [edit]   # 查看 / 编辑自定义规则
bark update         # 更新到最新版本
bark uninstall      # 完全卸载
```

### 测试示例

```bash
bark test ls -la
# ✅ allow  ([低风险] 只读目录查看)  [0.0s]  — 缓存命中

bark test rm -rf /
# 🚨 ask  ([高风险] 极度危险的递归删除根目录)  [6.8s]

bark test npm install express
# ✅ allow  ([中风险] 安装npm依赖包)  [7.2s]
```

## 工作原理

```
Claude Code 调用工具
        │
        ▼
┌──────────────────────────┐
│  第一层: 快速规则          │  只做工具级别的确定性判断
│  Read/Grep/Glob → 放行   │  不写任何 Bash 命令规则
│  普通文件编辑 → 放行      │
└──────────┬───────────────┘
           │ 未命中
           ▼
┌──────────────────────────┐
│  第 1.5 层: 自定义规则     │  用户定义的命令模式
│  bark.conf          │  allow / notify / block
└──────────┬───────────────┘
           │ 未命中
           ▼
┌──────────────────────────┐
│  第二层: 缓存查询          │  命令归一化为模式
│  "rm -rf" → 复用上次      │  md5 哈希, 24小时有效
│  AI 的判断结果             │
└──────────┬───────────────┘
           │ 缓存未命中
           ▼
┌──────────────────────────┐
│  第三层: AI 评估           │  claude -p 理解命令语义
│  返回风险等级和原因        │  结果写入缓存供下次复用
└──────────┬───────────────┘
           │
           ▼
  Level 0 → 静默放行
  Level 1 → 系统通知 + 放行
  Level 2 → 通知 + 声音 + 终端确认
```

### 自定义规则

创建 `~/.claude/hooks/bark.conf` 定义自己的规则（在 AI 评估之前检查）：

```conf
# 格式: 动作: 模式（支持 * 通配符）
allow: npm test
allow: npm run *
allow: make *
notify: git push
block: rm -rf /
```

**设计哲学**：不写硬编码的 Bash 规则。AI 理解命令语义，比正则匹配更准确。缓存保证同类命令第二次零延迟。

## Bark 与其他权限模式的对比

Claude Code 内置了多种权限模式，但各有局限：

| 模式 | 命令 | 行为 | 局限 |
|---|---|---|---|
| **默认模式** | `claude` | 每个操作都要确认 | 不断被打断 |
| **接受编辑** | `claude -y` | 自动放行文件编辑，Bash 仍需确认 | Bash 命令仍被阻断 |
| **Auto Mode** | `claude --permission-mode auto` | 基于规则的分类器（仅 Team 计划） | 无 AI 理解、无缓存、不可自定义 |
| **跳过权限** | `claude --dangerously-skip-permissions` | 放行一切 | 零安全性 — 危险 |
| **Bark** 🐕 | `claude`（安装 Bark 后） | AI 驱动的语义评估 | — |

### 为什么选择 Bark？

| 特性 | Auto Mode | Bark |
|---|---|---|
| **可用性** | 仅 Team 计划 | 免费开源 |
| **评估方式** | 静态规则匹配 | AI 语义理解 |
| **自定义规则** | 不支持 | `bark.conf` — allow/notify/block |
| **缓存** | 无 | 24h TTL，同类命令 0ms |
| **统计面板** | 无 | `bark stats` — 完整仪表板 |
| **日志** | 无 | `bark log` — 彩色、可筛选 |
| **系统通知** | 无 | macOS/Linux 系统通知 |
| **离线测试** | 不支持 | `bark test <cmd>` |

Bark 填补了"每次手动确认"和"跳过所有安全检查"之间的空白。它理解命令**在做什么**，而不只是**长什么样**。

## 环境要求

- 已安装 [Claude Code](https://docs.anthropic.com/en/docs/claude-code)
- PATH 中有 `jq`（`brew install jq` / `apt install jq`）
- 有 `claude` CLI（AI 评估层需要）
- macOS（系统通知）或 Linux（需要 `notify-send`）

## 卸载

```bash
bark uninstall
```

## License

MIT
