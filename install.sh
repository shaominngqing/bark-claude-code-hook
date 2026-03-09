#!/bin/bash
# =============================================================================
# Risk Guard for Claude Code — 一键安装脚本 (AI 驱动版)
# =============================================================================
# 安装方式:
#   curl -fsSL https://raw.githubusercontent.com/YOUR_REPO/install-risk-guard.sh | bash
#   或本地: bash install-risk-guard.sh
#
# 卸载: risk-guard-uninstall
# =============================================================================

set -euo pipefail

HOOKS_DIR="$HOME/.claude/hooks"
SETTINGS="$HOME/.claude/settings.json"
HOOK_SCRIPT="$HOOKS_DIR/risk-guard.sh"
CTL_SCRIPT="$HOOKS_DIR/risk-guard-ctl.sh"

RED='\033[0;31m'; GREEN='\033[0;32m'; YELLOW='\033[1;33m'; CYAN='\033[0;36m'
BOLD='\033[1m'; DIM='\033[2m'; NC='\033[0m'
ok()    { echo -e "  ${GREEN}✓${NC} $1"; }
warn()  { echo -e "  ${YELLOW}⚠${NC} $1"; }
fail()  { echo -e "  ${RED}✗${NC} $1"; exit 1; }
step()  { echo -e "\n${BOLD}${CYAN}▸ $1${NC}"; }

echo ""
echo -e "${BOLD}  ╭───────────────────────────────────────────╮${NC}"
echo -e "${BOLD}  │                                           │${NC}"
echo -e "${BOLD}  │   ${CYAN}⚡ Risk Guard${NC}${BOLD}  for Claude Code          │${NC}"
echo -e "${BOLD}  │   ${DIM}AI-Powered Risk Assessment Hook${NC}${BOLD}         │${NC}"
echo -e "${BOLD}  │                                           │${NC}"
echo -e "${BOLD}  ╰───────────────────────────────────────────╯${NC}"

step "检查环境"
command -v python3 >/dev/null 2>&1 && ok "python3" || fail "需要 python3"
command -v claude  >/dev/null 2>&1 && ok "claude CLI" || warn "未检测到 claude 命令（AI 评估将不可用）"

step "准备目录"
mkdir -p "$HOOKS_DIR"
[ ! -f "$SETTINGS" ] && echo '{}' > "$SETTINGS"
ok "$HOOKS_DIR"

# ═══ 写入 risk-guard.sh ═══
cat > "$HOOK_SCRIPT" << 'HOOK__EOF'
#!/bin/bash
# =============================================================================
# Claude Code Risk Guard - AI 驱动的风险评估 PreToolUse Hook
# =============================================================================
# 三层评估:
#   第一层: 快速规则 — 明确安全的工具/命令直接放行（零延迟）
#   第二层: 缓存查询 — 相似命令模式复用之前 AI 的判断（零延迟）
#   第三层: AI 评估 — 不确定的交给 Claude 判断（几秒延迟）
#
# 风险等级:
#   0 低风险 → 自动放行
#   1 中风险 → macOS 通知 + 自动放行
#   2 高风险 → macOS 通知 + 声音 + 终端确认
# =============================================================================

set -euo pipefail

CACHE_DIR="$HOME/.claude/hooks/cache"
CACHE_TTL=86400  # 缓存有效期：24小时
LOG_FILE="$HOME/.claude/hooks/risk-guard.log"
LOG_MAX_LINES=500
mkdir -p "$CACHE_DIR"

INPUT=$(cat)

TOOL_NAME=$(echo "$INPUT" | python3 -c "import sys,json; d=json.load(sys.stdin); print(d.get('tool_name',''))" 2>/dev/null || echo "")
TOOL_INPUT=$(echo "$INPUT" | python3 -c "import sys,json; d=json.load(sys.stdin); print(json.dumps(d.get('tool_input',{})))" 2>/dev/null || echo "{}")

COMMAND=""
if [ "$TOOL_NAME" = "Bash" ]; then
    COMMAND=$(echo "$TOOL_INPUT" | python3 -c "import sys,json; d=json.load(sys.stdin); print(d.get('command',''))" 2>/dev/null || echo "")
fi

FILE_PATH=""
if [ "$TOOL_NAME" = "Edit" ] || [ "$TOOL_NAME" = "Write" ] || [ "$TOOL_NAME" = "NotebookEdit" ]; then
    FILE_PATH=$(echo "$TOOL_INPUT" | python3 -c "import sys,json; d=json.load(sys.stdin); print(d.get('file_path','') or d.get('path',''))" 2>/dev/null || echo "")
fi

# =============================================================================
# 输出函数
# =============================================================================
output_decision() {
    python3 -c "
import json, sys
print(json.dumps({
    'hookSpecificOutput': {
        'hookEventName': 'PreToolUse',
        'permissionDecision': sys.argv[1],
        'permissionDecisionReason': sys.argv[2]
    }
}))" "$1" "$2"
}

notify() {
    local title="$1" subtitle="$2" body="$3" sound="${4:-}"
    if command -v osascript >/dev/null 2>&1; then
        local snd=""
        [ -n "$sound" ] && snd=" sound name \"$sound\""
        osascript -e "display notification \"${body:0:200}\" with title \"$title\" subtitle \"$subtitle\"$snd" 2>/dev/null &
    elif command -v notify-send >/dev/null 2>&1; then
        notify-send "$title" "$subtitle: $body" 2>/dev/null &
    fi
}

log_entry() {
    local level_tag="$1" source="$2" tool="$3" detail="$4" reason="$5"
    local ts
    ts=$(date '+%Y-%m-%d %H:%M:%S')
    echo "$ts  $level_tag  [$source]  $tool  $detail  →  $reason" >> "$LOG_FILE" 2>/dev/null

    # 日志轮转：超过上限时截断
    if [ -f "$LOG_FILE" ]; then
        local lines
        lines=$(wc -l < "$LOG_FILE" | tr -d ' ')
        if [ "$lines" -gt "$LOG_MAX_LINES" ]; then
            tail -n $(( LOG_MAX_LINES / 2 )) "$LOG_FILE" > "$LOG_FILE.tmp" && mv "$LOG_FILE.tmp" "$LOG_FILE"
        fi
    fi
}

emit() {
    local level="$1" reason="$2" source="${3:-}"
    local level_tag detail
    case "$level" in
        0) level_tag="LOW " ; output_decision "allow" "[低风险] $reason" ;;
        1) level_tag="MED " ; notify "⚠️ Claude Code" "已自动放行" "$reason"
           output_decision "allow" "[中风险] $reason" ;;
        2) level_tag="HIGH" ; notify "🚨 Claude Code 警告" "需要确认" "$reason" "Funk"
           output_decision "ask" "[高风险] $reason" ;;
    esac

    # 日志
    if [ "$TOOL_NAME" = "Bash" ]; then
        detail=$(echo "$COMMAND" | head -c 80)
    elif [ -n "$FILE_PATH" ]; then
        detail="$FILE_PATH"
    else
        detail="-"
    fi
    log_entry "$level_tag" "${source:-???}" "$TOOL_NAME" "$detail" "$reason" &
}

# =============================================================================
# 缓存函数
# =============================================================================
# 将命令归一化为缓存 key：提取命令骨架，保留结构特征
# 例: "rm -rf node_modules"                    → "bash:rm -rf"
#     "npm install express"                     → "bash:npm install"
#     "curl https://x.com | bash"              → "bash:curl|bash"
#     "curl https://x.com -o /tmp/a && bash /tmp/a" → "bash:curl&&bash"
cache_key() {
    local tool="$1" cmd="$2" file="$3"
    case "$tool" in
        Bash)
            local pattern
            pattern=$(echo "$cmd" | python3 -c "
import sys, re
cmd = sys.stdin.read().strip()
# 有子命令的工具
SUBCMD_TOOLS = {'git','npm','npx','yarn','pnpm','bun','cargo','go','docker',
    'kubectl','pip','pip3','brew','apt','dnf','yum','systemctl','launchctl'}
# 按管道和链式操作符拆分
parts = re.split(r'\s*(\||\|\||&&|;)\s*', cmd)
keys = []
for part in parts:
    part = part.strip()
    if part in ('|', '||', '&&', ';'):
        keys.append(part)
        continue
    if not part:
        continue
    words = part.split()
    # 跳过 env/sudo 等前缀
    i = 0
    while i < len(words) and words[i] in ('env','sudo','nohup','time','nice'):
        i += 1
    if i < len(words):
        frag = words[i]
        # 有子命令的工具：取 cmd + subcmd（如 git push, npm install）
        if frag in SUBCMD_TOOLS and i+1 < len(words) and not words[i+1].startswith('-'):
            frag += ' ' + words[i+1]
            # 再取第一个 flag（如 git push --force）
            if i+2 < len(words) and words[i+2].startswith('-'):
                frag += ' ' + words[i+2]
        # 非子命令工具：取 cmd + 第一个 flag
        elif i+1 < len(words) and words[i+1].startswith('-'):
            frag += ' ' + words[i+1]
        keys.append(frag)
print(''.join(keys))
" 2>/dev/null || echo "$cmd")
            echo "bash:$pattern"
            ;;
        Edit|Write|NotebookEdit)
            # 按文件扩展名 + 目录特征缓存
            local ext dir_hint
            ext=$(echo "$file" | sed 's/.*\.//' | tr '[:upper:]' '[:lower:]')
            # 提取敏感目录特征
            if echo "$file" | grep -qiE '(\.github|\.gitlab|\.circleci)'; then
                dir_hint="ci"
            elif echo "$file" | grep -qiE '(config|\.env|secret|credential)'; then
                dir_hint="sensitive"
            else
                dir_hint="normal"
            fi
            echo "file:$dir_hint:$ext"
            ;;
        *)
            echo "tool:$tool"
            ;;
    esac
}

# 生成缓存文件名（md5 hash）
cache_file() {
    local key="$1"
    local hash
    hash=$(echo -n "$key" | md5 2>/dev/null || echo -n "$key" | md5sum 2>/dev/null | cut -d' ' -f1)
    echo "$CACHE_DIR/$hash"
}

# 查询缓存，返回 "level|reason" 或空
cache_get() {
    local key="$1"
    local file
    file=$(cache_file "$key")
    if [ -f "$file" ]; then
        # 检查 TTL
        local file_age now file_mtime
        now=$(date +%s)
        # macOS 和 Linux 兼容的获取文件修改时间
        file_mtime=$(stat -f %m "$file" 2>/dev/null || stat -c %Y "$file" 2>/dev/null || echo "0")
        file_age=$((now - file_mtime))
        if [ "$file_age" -lt "$CACHE_TTL" ]; then
            cat "$file"
            return 0
        else
            rm -f "$file"
        fi
    fi
    return 1
}

# 写入缓存
cache_set() {
    local key="$1" value="$2"
    local file
    file=$(cache_file "$key")
    echo "$value" > "$file"
}

# 清理过期缓存（概率性执行，避免每次都扫描）
if [ $((RANDOM % 50)) -eq 0 ]; then
    find "$CACHE_DIR" -type f -mtime +1 -delete 2>/dev/null &
fi

# =============================================================================
# 第一层: 快速规则
# =============================================================================
# 设计原则：只处理「工具层面确定无风险」的情况
# 所有 Bash 命令一律交给 AI + 缓存，不做人为判断
# =============================================================================
fast_check() {
    # --- 工具本身是只读的，不管参数是什么都安全 ---
    case "$TOOL_NAME" in
        Read|Glob|Grep|Agent|AskUserQuestion|EnterPlanMode|ExitPlanMode|WebFetch|Skill)
            echo "0|只读操作"; return 0 ;;
        TaskCreate|TaskGet|TaskList|TaskOutput|TaskUpdate|TaskStop)
            echo "0|任务管理操作"; return 0 ;;
    esac

    # --- 文件编辑：普通源代码文件直接放行，敏感文件交 AI ---
    if [ "$TOOL_NAME" = "Edit" ] || [ "$TOOL_NAME" = "Write" ] || [ "$TOOL_NAME" = "NotebookEdit" ]; then
        if [ -n "$FILE_PATH" ] && ! echo "$FILE_PATH" | grep -qiE '(\.env|credentials|secret|password|token|\.pem|\.key|id_rsa|authorized_keys|sudoers|shadow|passwd|\.github/workflows|\.gitlab-ci|Jenkinsfile)'; then
            echo "0|普通文件编辑: $FILE_PATH"; return 0
        fi
    fi

    # --- Bash 命令：全部交给 AI + 缓存 ---
    # 不在这里做任何 Bash 命令的规则判断
    # AI 理解语义，缓存保证速度，比人为写规则更准确

    return 1
}

# =============================================================================
# 第二层: 缓存查询
# =============================================================================
check_cache() {
    local key
    key=$(cache_key "$TOOL_NAME" "$COMMAND" "$FILE_PATH")
    cache_get "$key"
}

# =============================================================================
# 第三层: AI 评估
# =============================================================================
ai_assess() {
    local description=""
    case "$TOOL_NAME" in
        Bash)    description="Bash command: $COMMAND" ;;
        Edit)    description="Edit file: $FILE_PATH" ;;
        Write)   description="Write file: $FILE_PATH" ;;
        *)       description="Tool: $TOOL_NAME, Input: $(echo "$TOOL_INPUT" | head -c 500)" ;;
    esac

    local ai_result
    ai_result=$(env -u CLAUDECODE claude -p --no-session-persistence \
      --system-prompt 'You are a JSON-only risk assessment API. You MUST output exactly one JSON object per request. No markdown, no explanation, no conversation. Output format: {"level":<0|1|2>,"reason":"<10 words max in Chinese>"}' \
"Assess risk level of this dev operation:
0=safe(read-only, builds, tests, normal edits)
1=medium(side effects but recoverable: pkg install, git push, mv, config edits)
2=high(destructive/irreversible: force push, rm -rf critical dirs, secrets, remote exec, DB drops, sudo)

Be practical: rm -rf node_modules=1, npm install=1, git push=1. Only truly dangerous=2.

Operation: $description" 2>/dev/null)

    # 从结果中提取 JSON（处理可能的 markdown 包裹）
    local json_line
    json_line=$(echo "$ai_result" | grep -o '{[^}]*}' | head -1)

    if [ -n "$json_line" ]; then
        local level reason
        level=$(echo "$json_line" | python3 -c "import sys,json; print(json.load(sys.stdin).get('level',1))" 2>/dev/null || echo "1")
        reason=$(echo "$json_line" | python3 -c "import sys,json; print(json.load(sys.stdin).get('reason','AI评估'))" 2>/dev/null || echo "AI评估")

        # 写入缓存
        local key
        key=$(cache_key "$TOOL_NAME" "$COMMAND" "$FILE_PATH")
        cache_set "$key" "${level}|${reason}"

        echo "${level}|${reason}"
    else
        # AI 调用失败 → 回退到中风险（不缓存失败）
        echo "1|AI评估失败，保守处理: $description"
    fi
}

# =============================================================================
# 主逻辑
# =============================================================================

# 第一层: 快速规则
FAST_RESULT=$(fast_check) && {
    LEVEL=$(echo "$FAST_RESULT" | cut -d'|' -f1)
    REASON=$(echo "$FAST_RESULT" | cut -d'|' -f2-)
    emit "$LEVEL" "$REASON" "FAST"
    exit 0
}

# 第二层: 缓存查询
CACHE_RESULT=$(check_cache) && {
    LEVEL=$(echo "$CACHE_RESULT" | cut -d'|' -f1)
    REASON=$(echo "$CACHE_RESULT" | cut -d'|' -f2-)
    emit "$LEVEL" "$REASON (cached)" "CACHE"
    exit 0
}

# 第三层: AI 评估
AI_RESULT=$(ai_assess)
LEVEL=$(echo "$AI_RESULT" | cut -d'|' -f1)
REASON=$(echo "$AI_RESULT" | cut -d'|' -f2-)
emit "$LEVEL" "$REASON" "AI"
HOOK__EOF
chmod +x "$HOOK_SCRIPT"

step "安装组件"
ok "风险评估引擎  risk-guard.sh"

# ═══ 写入 risk-guard-ctl.sh ═══
cat > "$CTL_SCRIPT" << 'CTL__EOF'
#!/bin/bash
# =============================================================================
# risk-guard — Risk Guard 控制命令
# =============================================================================
# 用法:
#   risk-guard status        查看状态
#   risk-guard on            启用
#   risk-guard off           禁用
#   risk-guard toggle        切换
#   risk-guard test <cmd>    测试命令风险等级
#   risk-guard cache         查看缓存统计
#   risk-guard cache clear   清空缓存
#   risk-guard log [N]       查看最近 N 条日志（默认 20）
#   risk-guard log clear     清空日志
# =============================================================================

SETTINGS="$HOME/.claude/settings.json"
HOOK_SCRIPT="$HOME/.claude/hooks/risk-guard.sh"
CACHE_DIR="$HOME/.claude/hooks/cache"
LOG_FILE="$HOME/.claude/hooks/risk-guard.log"

[ ! -f "$SETTINGS" ] && echo '{}' > "$SETTINGS"

has_hook() {
    python3 -c "
import json
with open('$SETTINGS') as f:
    d = json.load(f)
hooks = d.get('hooks', {}).get('PreToolUse', [])
for h in hooks:
    for cmd in h.get('hooks', []):
        if 'risk-guard.sh' in cmd.get('command', ''):
            exit(0)
exit(1)" 2>/dev/null
}

enable_hook() {
    python3 -c "
import json
with open('$SETTINGS') as f:
    d = json.load(f)
hook_entry = {'matcher': '*', 'hooks': [{'type': 'command', 'command': '$HOOK_SCRIPT', 'timeout': 30}]}
if 'hooks' not in d:
    d['hooks'] = {}
pre = d['hooks'].get('PreToolUse', [])
for h in pre:
    for cmd in h.get('hooks', []):
        if 'risk-guard.sh' in cmd.get('command', ''):
            print('⚡ Risk Guard 已经是启用状态'); exit(0)
pre.append(hook_entry)
d['hooks']['PreToolUse'] = pre
with open('$SETTINGS', 'w') as f:
    json.dump(d, f, indent=2, ensure_ascii=False)
print('✅ Risk Guard 已启用 (新会话生效)')"
}

disable_hook() {
    python3 -c "
import json
with open('$SETTINGS') as f:
    d = json.load(f)
pre = d.get('hooks', {}).get('PreToolUse', [])
new_pre, found = [], False
for h in pre:
    keep = True
    for cmd in h.get('hooks', []):
        if 'risk-guard.sh' in cmd.get('command', ''):
            keep, found = False, True
    if keep:
        new_pre.append(h)
if not found:
    print('⚡ Risk Guard 已经是禁用状态'); exit(0)
if new_pre:
    d['hooks']['PreToolUse'] = new_pre
else:
    d.get('hooks', {}).pop('PreToolUse', None)
    if not d.get('hooks'):
        d.pop('hooks', None)
with open('$SETTINGS', 'w') as f:
    json.dump(d, f, indent=2, ensure_ascii=False)
print('⛔ Risk Guard 已禁用 (新会话生效)')"
}

show_status() {
    if has_hook; then
        echo "🟢 Risk Guard: 启用中"
    else
        echo "🔴 Risk Guard: 已禁用"
    fi
    # 缓存统计
    local count=0
    [ -d "$CACHE_DIR" ] && count=$(find "$CACHE_DIR" -type f 2>/dev/null | wc -l | tr -d ' ')
    echo "   缓存条目: $count"
    # 日志行数
    local log_lines=0
    [ -f "$LOG_FILE" ] && log_lines=$(wc -l < "$LOG_FILE" | tr -d ' ')
    echo "   日志行数: $log_lines"
}

test_command() {
    local cmd="$*"
    if [ -z "$cmd" ]; then
        echo "用法: risk-guard test <bash command>"
        echo "示例: risk-guard test rm -rf node_modules"
        exit 1
    fi

    local payload
    payload=$(python3 -c "import json; print(json.dumps({'tool_name': 'Bash', 'tool_input': {'command': '$cmd'}}))")

    local start end
    start=$(python3 -c "import time; print(time.time())")
    local result
    result=$(echo "$payload" | env -u CLAUDECODE "$HOOK_SCRIPT" 2>/dev/null)
    end=$(python3 -c "import time; print(time.time())")

    local elapsed
    elapsed=$(python3 -c "print(f'{$end - $start:.1f}s')")

    python3 -c "
import json, sys
try:
    d = json.loads('''$result''')
    o = d['hookSpecificOutput']
    decision = o['permissionDecision']
    reason = o['permissionDecisionReason']
    icons = {'allow': '✅', 'ask': '🚨', 'deny': '❌'}
    print(f\"{icons.get(decision, '?')} {decision}  ({reason})  [{sys.argv[1]}]\")
except:
    print('⚠️  解析失败:', '''$result'''[:100])
" "$elapsed"
}

cache_cmd() {
    case "${1:-}" in
        clear|clean)
            rm -rf "$CACHE_DIR"/*
            echo "✅ 缓存已清空"
            ;;
        *)
            if [ ! -d "$CACHE_DIR" ] || [ -z "$(ls -A "$CACHE_DIR" 2>/dev/null)" ]; then
                echo "📦 缓存为空"
                return
            fi
            local count
            count=$(find "$CACHE_DIR" -type f | wc -l | tr -d ' ')
            local size
            size=$(du -sh "$CACHE_DIR" 2>/dev/null | cut -f1)
            echo "📦 缓存统计:"
            echo "   条目数: $count"
            echo "   占用:   $size"
            echo "   目录:   $CACHE_DIR"
            echo ""
            echo "   最近缓存的判断:"
            for f in $(ls -t "$CACHE_DIR" | head -10); do
                local content age_s age_h
                content=$(cat "$CACHE_DIR/$f")
                age_s=$(( $(date +%s) - $(stat -f %m "$CACHE_DIR/$f" 2>/dev/null || stat -c %Y "$CACHE_DIR/$f" 2>/dev/null || echo 0) ))
                if [ "$age_s" -lt 3600 ]; then
                    age_h="${age_s}s ago"
                elif [ "$age_s" -lt 86400 ]; then
                    age_h="$(( age_s / 3600 ))h ago"
                else
                    age_h="$(( age_s / 86400 ))d ago"
                fi
                local level reason
                level=$(echo "$content" | cut -d'|' -f1)
                reason=$(echo "$content" | cut -d'|' -f2-)
                local icon
                case "$level" in
                    0) icon="✅" ;; 1) icon="⚠️ " ;; 2) icon="🚨" ;; *) icon="?" ;;
                esac
                echo "   $icon $reason  ($age_h)"
            done
            ;;
    esac
}

log_cmd() {
    case "${1:-}" in
        clear|clean)
            > "$LOG_FILE"
            echo "✅ 日志已清空"
            ;;
        *)
            if [ ! -f "$LOG_FILE" ] || [ ! -s "$LOG_FILE" ]; then
                echo "📋 日志为空"
                return
            fi
            local n="${1:-20}"
            echo "📋 最近 $n 条日志:"
            echo ""
            tail -n "$n" "$LOG_FILE"
            ;;
    esac
}

usage() {
    echo "Risk Guard — Claude Code AI 风险守卫"
    echo ""
    echo "用法: risk-guard <command>"
    echo ""
    echo "命令:"
    echo "  status            查看状态"
    echo "  on / off          启用 / 禁用"
    echo "  toggle            切换开关"
    echo "  test <cmd>        测试命令风险等级"
    echo "  cache [clear]     查看/清空缓存"
    echo "  log [N|clear]     查看最近 N 条日志 / 清空日志"
    echo "  help              显示此帮助"
}

case "${1:-status}" in
    on|enable)      enable_hook ;;
    off|disable)    disable_hook ;;
    toggle)         if has_hook; then disable_hook; else enable_hook; fi ;;
    status)         show_status ;;
    test)           shift; test_command "$@" ;;
    cache)          shift; cache_cmd "$@" ;;
    log|logs)       shift; log_cmd "$@" ;;
    help|-h|--help) usage ;;
    *)              echo "未知命令: $1"; usage; exit 1 ;;
esac
CTL__EOF
chmod +x "$CTL_SCRIPT"
ok "控制面板      risk-guard-ctl.sh"

# ═══ 卸载脚本 ═══
cat > "$HOOKS_DIR/risk-guard-uninstall.sh" << 'UNINSTALL_EOF'
#!/bin/bash
echo "正在卸载 Risk Guard..."
bash "$HOME/.claude/hooks/risk-guard-ctl.sh" off 2>/dev/null
for cmd in risk-guard risk-guard-uninstall; do
    for dir in /usr/local/bin /opt/homebrew/bin; do
        [ -L "$dir/$cmd" ] && rm -f "$dir/$cmd"
    done
done
rm -rf "$HOME/.claude/hooks/cache"
rm -f "$HOME/.claude/hooks/risk-guard.sh" "$HOME/.claude/hooks/risk-guard-ctl.sh"
rm -f "$HOME/.claude/hooks/install-risk-guard.sh"
rm -f "$HOME/.claude/hooks/risk-guard.log"
rm -f "$HOME/.claude/hooks/risk-guard-uninstall.sh"
echo "✅ Risk Guard 已完全卸载"
UNINSTALL_EOF
chmod +x "$HOOKS_DIR/risk-guard-uninstall.sh"

ok "卸载工具      risk-guard-uninstall.sh"

# ═══ 全局命令 ═══
step "注册全局命令"
BIN_DIR=""
if [ -d /opt/homebrew/bin ] && echo "$PATH" | grep -q "/opt/homebrew/bin"; then
    BIN_DIR="/opt/homebrew/bin"
elif [ -d /usr/local/bin ]; then
    BIN_DIR="/usr/local/bin"
fi

if [ -n "$BIN_DIR" ]; then
    ln -sf "$CTL_SCRIPT" "$BIN_DIR/risk-guard" 2>/dev/null && ok "risk-guard → $BIN_DIR/" || warn "请手动: ln -s $CTL_SCRIPT /usr/local/bin/risk-guard"
    ln -sf "$HOOKS_DIR/risk-guard-uninstall.sh" "$BIN_DIR/risk-guard-uninstall" 2>/dev/null && ok "risk-guard-uninstall → $BIN_DIR/"
else
    warn "请手动: alias risk-guard='$CTL_SCRIPT'"
fi

# ═══ 注入 hooks 配置 ═══
step "配置 Claude Code Hooks"
python3 -c "
import json, os
p = os.path.expanduser('$SETTINGS')
with open(p) as f:
    d = json.load(f)
hook_entry = {'matcher': '*', 'hooks': [{'type': 'command', 'command': '$HOOK_SCRIPT', 'timeout': 30}]}
if 'hooks' not in d:
    d['hooks'] = {}
pre = d['hooks'].get('PreToolUse', [])
for h in pre:
    for cmd in h.get('hooks', []):
        if 'risk-guard.sh' in cmd.get('command', ''):
            exit(0)
pre.append(hook_entry)
d['hooks']['PreToolUse'] = pre
with open(p, 'w') as f:
    json.dump(d, f, indent=2, ensure_ascii=False)
"
ok "PreToolUse hook 已注入 settings.json"

echo ""
echo -e "${BOLD}  ╭───────────────────────────────────────────╮${NC}"
echo -e "${BOLD}  │  ${GREEN}安装完成！${NC}${BOLD}                                │${NC}"
echo -e "${BOLD}  ╰───────────────────────────────────────────╯${NC}"
echo ""
echo -e "  ${BOLD}工作原理${NC}"
echo ""
echo -e "    ${DIM}只读工具${NC}  Read / Grep / Glob ...  ${GREEN}━━▸${NC} 直接放行"
echo -e "    ${DIM}文件编辑${NC}  普通源代码文件          ${GREEN}━━▸${NC} 直接放行"
echo -e "    ${DIM}Bash命令${NC}  所有命令                ${CYAN}━━▸${NC} AI 评估 (~7s)"
echo -e "    ${DIM}重复模式${NC}  同类命令第二次          ${GREEN}━━▸${NC} 缓存命中 (0s)"
echo -e "    ${DIM}高风险  ${NC}  rm -rf / force push ... ${RED}━━▸${NC} 通知 + 确认"
echo ""
echo -e "  ${BOLD}常用命令${NC}"
echo ""
echo -e "    ${CYAN}risk-guard${NC} ${DIM}status${NC}         查看状态"
echo -e "    ${CYAN}risk-guard${NC} ${DIM}on / off${NC}       启用 / 禁用"
echo -e "    ${CYAN}risk-guard${NC} ${DIM}test <cmd>${NC}     测试命令风险"
echo -e "    ${CYAN}risk-guard${NC} ${DIM}cache [clear]${NC}  缓存管理"
echo -e "    ${CYAN}risk-guard${NC} ${DIM}log [N|clear]${NC}  日志查看"
echo -e "    ${CYAN}risk-guard-uninstall${NC}       完全卸载"
echo ""
echo -e "  ${YELLOW}▸ 新开的 Claude Code 会话自动生效${NC}"
echo ""
