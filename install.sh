#!/bin/bash
# =============================================================================
# Bark for Claude Code — 一键安装脚本 (AI 驱动版)
# =============================================================================
# 安装方式:
#   curl -fsSL https://raw.githubusercontent.com/YOUR_REPO/install-bark.sh | bash
#   或本地: bash install-bark.sh
#
# 卸载: bark-uninstall
# =============================================================================

set -euo pipefail

BARK_VERSION="1.0.0"

HOOKS_DIR="$HOME/.claude/hooks"
SETTINGS="$HOME/.claude/settings.json"
HOOK_SCRIPT="$HOOKS_DIR/bark.sh"
CTL_SCRIPT="$HOOKS_DIR/bark-ctl.sh"

NC='\033[0m'; BOLD='\033[1m'; DIM='\033[2m'; ITALIC='\033[3m'
RED='\033[0;31m'; GREEN='\033[0;32m'; YELLOW='\033[1;33m'; CYAN='\033[0;36m'
C1='\033[38;5;39m'; C2='\033[38;5;45m'; C3='\033[38;5;51m'; C4='\033[38;5;87m'  # gradient blues
ACCENT='\033[38;5;213m'  # pink accent

# Animated typewriter
_type() { local s="$1" d="${2:-0.02}"; for ((i=0;i<${#s};i++)); do printf '%s' "${s:$i:1}"; sleep "$d"; done; }

# Spinner for a command: _spin "message" command args...
_spin() {
    local msg="$1"; shift
    local frames=('▸▹▹▹▹' '▹▸▹▹▹' '▹▹▸▹▹' '▹▹▹▸▹' '▹▹▹▹▸')
    "$@" &>/dev/null &
    local pid=$!
    local i=0
    while kill -0 "$pid" 2>/dev/null; do
        printf "\r  ${C2}${frames[$((i % ${#frames[@]}))]}${NC} ${DIM}%s${NC}" "$msg"
        sleep 0.08
        ((i++))
    done
    wait "$pid" 2>/dev/null
    local rc=$?
    printf "\r\033[K"
    return $rc
}

ok()    { echo -e "  ${GREEN}✓${NC} $1"; }
warn()  { echo -e "  ${YELLOW}⚠${NC} $1"; }
fail()  { echo -e "  ${RED}✗${NC} $1"; exit 1; }
step()  { echo -e "\n  ${C1}${BOLD}▸${NC} ${BOLD}$1${NC}"; }

# i18n: detect locale
is_zh() { [[ "${LANG:-}${LC_ALL:-}" =~ zh ]]; }

# --- Animated ASCII Art banner with gradient ---
_gradient_line() {
    local line="$1" start="${2:-39}" count="${3:-6}"
    printf "  "
    for ((i=0; i<${#line}; i++)); do
        local c=$(( start + (i % count) ))
        printf "\033[1;38;5;${c}m%s" "${line:$i:1}"
    done
    printf "${NC}\n"
}

echo ""
{
cat << 'BANNER'
    ____             __
   / __ )____ ______/ /__
  / __  / __ `/ ___/ //_/
 / /_/ / /_/ / /  / ,<
/_____/\__,_/_/  /_/|_|
BANNER
} | while IFS= read -r line; do
    _gradient_line "$line" 39 6
    sleep 0.04
done
echo ""
printf "  ${DIM}${ITALIC}  🐕 AI-Powered Risk Assessment for Claude Code  v${BARK_VERSION}${NC}\n"
echo ""

if is_zh; then
    step "检查环境"
    command -v jq      >/dev/null 2>&1 && ok "jq" || fail "需要 jq (brew install jq)"
    command -v claude  >/dev/null 2>&1 && ok "claude CLI" || warn "未检测到 claude 命令（AI 评估将不可用）"
    step "准备目录"
else
    step "Check environment"
    command -v jq      >/dev/null 2>&1 && ok "jq" || fail "jq is required (brew install jq)"
    command -v claude  >/dev/null 2>&1 && ok "claude CLI" || warn "claude CLI not found (AI assessment will be unavailable)"
    step "Prepare directories"
fi
mkdir -p "$HOOKS_DIR"
([ ! -f "$SETTINGS" ] || [ ! -s "$SETTINGS" ]) && echo '{}' > "$SETTINGS"
ok "$HOOKS_DIR"

# ═══ 写入 bark.sh ═══
cat > "$HOOK_SCRIPT" << 'HOOK__EOF'
#!/bin/bash
# =============================================================================
# Claude Code Bark - AI 驱动的风险评估 PreToolUse Hook
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
CACHE_TTL=86400  # 24h
LOG_FILE="$HOME/.claude/hooks/bark.log"
LOG_MAX_LINES=500
mkdir -p "$CACHE_DIR"

# i18n
_is_zh() { [[ "${LANG:-}${LC_ALL:-}" =~ zh ]]; }

# verbose / dry-run (controlled via env vars)
_VERBOSE="${BARK_VERBOSE:-0}"
_DRY_RUN="${BARK_DRY_RUN:-0}"
_dbg() { [ "$_VERBOSE" = "1" ] && echo -e "\033[2m  [debug] $*\033[0m" >&2 || true; }

INPUT=$(cat)

TOOL_NAME=$(echo "$INPUT" | jq -r '.tool_name // empty' 2>/dev/null || echo "")
TOOL_INPUT=$(echo "$INPUT" | jq -c '.tool_input // {}' 2>/dev/null || echo "{}")

COMMAND=""
if [ "$TOOL_NAME" = "Bash" ]; then
    COMMAND=$(echo "$TOOL_INPUT" | jq -r '.command // empty' 2>/dev/null || echo "")
fi

FILE_PATH=""
if [ "$TOOL_NAME" = "Edit" ] || [ "$TOOL_NAME" = "Write" ] || [ "$TOOL_NAME" = "NotebookEdit" ]; then
    FILE_PATH=$(echo "$TOOL_INPUT" | jq -r '.file_path // .path // empty' 2>/dev/null || echo "")
fi

# =============================================================================
# 输出函数
# =============================================================================
output_decision() {
    jq -n --arg decision "$1" --arg reason "$2" \
      '{hookSpecificOutput:{hookEventName:"PreToolUse",permissionDecision:$decision,permissionDecisionReason:$reason}}'
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

    # dry-run: override high risk to allow
    if [ "$_DRY_RUN" = "1" ] && [ "$level" = "2" ]; then
        _dbg "dry-run: overriding level 2 → allow"
        reason="[dry-run] $reason"
    fi

    if _is_zh; then
        case "$level" in
            0) level_tag="LOW " ; output_decision "allow" "[低风险] $reason" ;;
            1) level_tag="MED " ; notify "⚠️ Claude Code" "已自动放行" "$reason"
               output_decision "allow" "[中风险] $reason" ;;
            2) level_tag="HIGH" ;
               if [ "$_DRY_RUN" = "1" ]; then
                   output_decision "allow" "[高风险-dry-run] $reason"
               else
                   notify "🚨 Claude Code" "需要确认" "$reason" "Funk"
                   output_decision "ask" "[高风险] $reason"
               fi ;;
        esac
    else
        case "$level" in
            0) level_tag="LOW " ; output_decision "allow" "[Low] $reason" ;;
            1) level_tag="MED " ; notify "⚠️ Claude Code" "Auto-allowed" "$reason"
               output_decision "allow" "[Medium] $reason" ;;
            2) level_tag="HIGH" ;
               if [ "$_DRY_RUN" = "1" ]; then
                   output_decision "allow" "[High-dry-run] $reason"
               else
                   notify "🚨 Claude Code" "Confirmation needed" "$reason" "Funk"
                   output_decision "ask" "[High] $reason"
               fi ;;
        esac
    fi

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
            local pattern=""
            # Split by pipe/chain operators and extract command skeleton
            local remaining="$cmd"
            local SUBCMD_TOOLS=" git npm npx yarn pnpm bun cargo go docker kubectl pip pip3 brew apt dnf yum systemctl launchctl "
            while [ -n "$remaining" ]; do
                local segment="" op=""
                # Extract next operator (||, &&, |, ;)
                if [[ "$remaining" =~ ^([^|;\&]*)(\ *(\|\||&&|\||;)\ *)(.*) ]]; then
                    segment="${BASH_REMATCH[1]}"
                    op="${BASH_REMATCH[3]}"
                    remaining="${BASH_REMATCH[4]}"
                else
                    segment="$remaining"
                    remaining=""
                fi
                segment=$(echo "$segment" | xargs) # trim
                [ -z "$segment" ] && { [ -n "$op" ] && pattern+="$op"; continue; }
                # Split into words
                read -ra words <<< "$segment"
                # Skip env/sudo/nohup/time/nice prefixes
                local i=0
                while [ $i -lt ${#words[@]} ] && [[ " env sudo nohup time nice " == *" ${words[$i]} "* ]]; do
                    ((i++))
                done
                if [ $i -lt ${#words[@]} ]; then
                    local frag="${words[$i]}"
                    local next_i=$((i+1))
                    local next2_i=$((i+2))
                    if [[ "$SUBCMD_TOOLS" == *" $frag "* ]] && [ $next_i -lt ${#words[@]} ] && [[ "${words[$next_i]}" != -* ]]; then
                        frag+=" ${words[$next_i]}"
                        if [ $next2_i -lt ${#words[@]} ] && [[ "${words[$next2_i]}" == -* ]]; then
                            frag+=" ${words[$next2_i]}"
                        fi
                    elif [ $next_i -lt ${#words[@]} ] && [[ "${words[$next_i]}" == -* ]]; then
                        frag+=" ${words[$next_i]}"
                    fi
                    pattern+="$frag"
                fi
                [ -n "$op" ] && pattern+="$op"
            done
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
    local _ro _task _edit
    if _is_zh; then _ro="只读操作"; _task="任务管理操作"; _edit="普通文件编辑"
    else _ro="Read-only operation"; _task="Task management"; _edit="Normal file edit"; fi

    case "$TOOL_NAME" in
        Read|Glob|Grep|Agent|AskUserQuestion|EnterPlanMode|ExitPlanMode|WebFetch|Skill)
            echo "0|$_ro"; return 0 ;;
        TaskCreate|TaskGet|TaskList|TaskOutput|TaskUpdate|TaskStop)
            echo "0|$_task"; return 0 ;;
    esac

    if [ "$TOOL_NAME" = "Edit" ] || [ "$TOOL_NAME" = "Write" ] || [ "$TOOL_NAME" = "NotebookEdit" ]; then
        if [ -n "$FILE_PATH" ] && ! echo "$FILE_PATH" | grep -qiE '(\.env|credentials|secret|password|token|\.pem|\.key|id_rsa|authorized_keys|sudoers|shadow|passwd|\.github/workflows|\.gitlab-ci|Jenkinsfile)'; then
            echo "0|$_edit: $FILE_PATH"; return 0
        fi
    fi

    # --- Bash 命令：全部交给 AI + 缓存 ---
    # 不在这里做任何 Bash 命令的规则判断
    # AI 理解语义，缓存保证速度，比人为写规则更准确

    return 1
}

# =============================================================================
# 第 1.5 层: 用户自定义规则
# =============================================================================
# 配置文件: ~/.claude/hooks/bark.conf
# 格式: allow: <pattern>   → level 0 (silent allow)
#       notify: <pattern>  → level 1 (notify + allow)
#       block: <pattern>   → level 2 (notify + confirm)
# 支持 * 通配符，仅匹配 Bash 命令
# =============================================================================
CONF_FILE="$HOME/.claude/hooks/bark.conf"

check_custom_rules() {
    [ "$TOOL_NAME" != "Bash" ] && return 1
    [ ! -f "$CONF_FILE" ] && return 1

    local line action pattern
    while IFS= read -r line || [ -n "$line" ]; do
        # skip comments and empty lines
        line=$(echo "$line" | sed 's/#.*//' | xargs)
        [ -z "$line" ] && continue

        # parse "action: pattern"
        action=$(echo "$line" | cut -d: -f1 | xargs | tr '[:upper:]' '[:lower:]')
        pattern=$(echo "$line" | cut -d: -f2- | xargs)
        [ -z "$pattern" ] && continue

        # glob match: convert pattern * to regex .*
        local regex="^$(echo "$pattern" | sed 's/[.[\^$()+?{|\\]/\\&/g; s/\*/.\*/g')$"
        if echo "$COMMAND" | grep -qE "$regex"; then
            local level reason_prefix
            case "$action" in
                allow)  level=0; if _is_zh; then reason_prefix="自定义放行"; else reason_prefix="Custom allow"; fi ;;
                notify) level=1; if _is_zh; then reason_prefix="自定义通知"; else reason_prefix="Custom notify"; fi ;;
                block)  level=2; if _is_zh; then reason_prefix="自定义拦截"; else reason_prefix="Custom block"; fi ;;
                *) continue ;;
            esac
            echo "${level}|${reason_prefix}: ${pattern}"
            return 0
        fi
    done < "$CONF_FILE"
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

    local lang_hint="in English"
    _is_zh && lang_hint="in Chinese"

    local ai_result
    ai_result=$(env -u CLAUDECODE claude -p --no-session-persistence \
      --system-prompt "You are a JSON-only risk assessment API. You MUST output exactly one JSON object per request. No markdown, no explanation, no conversation. Output format: {\"level\":<0|1|2>,\"reason\":\"<10 words max $lang_hint>\"}" \
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
        level=$(echo "$json_line" | jq -r '.level // 1' 2>/dev/null || echo "1")
        reason=$(echo "$json_line" | jq -r '.reason // "AI assessment"' 2>/dev/null || echo "AI assessment")

        # 写入缓存
        local key
        key=$(cache_key "$TOOL_NAME" "$COMMAND" "$FILE_PATH")
        cache_set "$key" "${level}|${reason}"

        echo "${level}|${reason}"
    else
        # AI call failed → fallback to medium risk (don't cache failures)
        if _is_zh; then echo "1|AI评估失败，保守处理: $description"
        else echo "1|AI assessment failed, conservative fallback: $description"; fi
    fi
}

# =============================================================================
# 主逻辑
# =============================================================================

_dbg "tool=$TOOL_NAME command='${COMMAND:-}' file='${FILE_PATH:-}'"
[ "$_DRY_RUN" = "1" ] && _dbg "dry-run mode: all decisions will be allow"

# 第一层: 快速规则
FAST_RESULT=$(fast_check) && {
    LEVEL=$(echo "$FAST_RESULT" | cut -d'|' -f1)
    REASON=$(echo "$FAST_RESULT" | cut -d'|' -f2-)
    _dbg "layer=FAST level=$LEVEL reason='$REASON'"
    emit "$LEVEL" "$REASON" "FAST"
    exit 0
}
_dbg "layer=FAST miss"

# 第 1.5 层: 用户自定义规则
CUSTOM_RESULT=$(check_custom_rules) && {
    LEVEL=$(echo "$CUSTOM_RESULT" | cut -d'|' -f1)
    REASON=$(echo "$CUSTOM_RESULT" | cut -d'|' -f2-)
    _dbg "layer=RULE level=$LEVEL reason='$REASON'"
    emit "$LEVEL" "$REASON" "RULE"
    exit 0
}
_dbg "layer=RULE miss"

# 第二层: 缓存查询
CACHE_KEY=$(cache_key "$TOOL_NAME" "$COMMAND" "$FILE_PATH")
_dbg "cache_key='$CACHE_KEY'"
CACHE_RESULT=$(cache_get "$CACHE_KEY") && {
    LEVEL=$(echo "$CACHE_RESULT" | cut -d'|' -f1)
    REASON=$(echo "$CACHE_RESULT" | cut -d'|' -f2-)
    _dbg "layer=CACHE level=$LEVEL reason='$REASON'"
    emit "$LEVEL" "$REASON (cached)" "CACHE"
    exit 0
}
_dbg "layer=CACHE miss"

# 第三层: AI 评估
_dbg "layer=AI calling claude..."
AI_RESULT=$(ai_assess)
LEVEL=$(echo "$AI_RESULT" | cut -d'|' -f1)
REASON=$(echo "$AI_RESULT" | cut -d'|' -f2-)
_dbg "layer=AI level=$LEVEL reason='$REASON'"
emit "$LEVEL" "$REASON" "AI"
HOOK__EOF
chmod +x "$HOOK_SCRIPT"

if is_zh; then step "安装组件"; ok "风险评估引擎  bark.sh"
else step "Install components"; ok "Risk assessment engine  bark.sh"; fi

# ═══ 写入默认 bark.conf（仅首次安装） ═══
CONF_FILE="$HOOKS_DIR/bark.conf"
if [ ! -f "$CONF_FILE" ]; then
    cat > "$CONF_FILE" << 'CONF__EOF'
# Bark Custom Rules
# Format: action: pattern (* wildcard supported)
#   allow:  <pattern>   → Level 0, silent allow
#   notify: <pattern>   → Level 1, notify + allow
#   block:  <pattern>   → Level 2, notify + confirm
#
# Examples:
# allow: npm test
# allow: npm run *
# allow: make *
# notify: git push
# block: rm -rf /
CONF__EOF
    if is_zh; then ok "自定义规则    bark.conf"
    else ok "Custom rules   bark.conf"; fi
fi

# ═══ 写入 bark-ctl.sh ═══
cat > "$CTL_SCRIPT" << 'CTL__EOF'
#!/bin/bash
# =============================================================================
# bark — Bark 控制命令
# =============================================================================
# 用法:
#   bark status        查看状态
#   bark on            启用
#   bark off           禁用
#   bark toggle        切换
#   bark test <cmd>    测试命令风险等级
#   bark cache         查看缓存统计
#   bark cache clear   清空缓存
#   bark log [N]       查看最近 N 条日志（默认 20）
#   bark log clear     清空日志
# =============================================================================

SETTINGS="$HOME/.claude/settings.json"
HOOK_SCRIPT="$HOME/.claude/hooks/bark.sh"
CACHE_DIR="$HOME/.claude/hooks/cache"
LOG_FILE="$HOME/.claude/hooks/bark.log"
VERSION="__BARK_VERSION__"

NC='\033[0m'; BOLD='\033[1m'; DIM='\033[2m'; ITALIC='\033[3m'
RED='\033[0;31m'; GREEN='\033[0;32m'; YELLOW='\033[1;33m'; CYAN='\033[0;36m'
BLUE='\033[0;34m'; MAGENTA='\033[0;35m'
C1='\033[38;5;39m'; C2='\033[38;5;45m'; C3='\033[38;5;51m'; C4='\033[38;5;87m'
ACCENT='\033[38;5;213m'; ORANGE='\033[38;5;208m'; PURPLE='\033[38;5;141m'

# Gradient text: _gradient "text" start_color_code count
_gradient() {
    local text="$1" start="${2:-39}" count="${3:-6}"
    for ((i=0; i<${#text}; i++)); do
        local c=$(( start + (i % count) ))
        printf "\033[38;5;${c}m%s" "${text:$i:1}"
    done
    printf "${NC}"
}

# Small ASCII art logo (figlet -f small "Bark")
_logo() {
    local lines=(
        ' ___           _   '
        '| _ ) __ _ _ _| |__'
        '| _ \/ _` | '\''_| / /'
        '|___/\__,_|_| |_\_\'
    )
    for line in "${lines[@]}"; do
        printf "  "
        for ((i=0; i<${#line}; i++)); do
            local c=$(( 39 + (i % 6) ))
            printf "\033[1;38;5;${c}m%s" "${line:$i:1}"
        done
        printf "${NC}\n"
    done
}

# Animated progress bar: _abar <value> <max> <width> <color>
_abar() {
    local val=$1 max=$2 width=${3:-20} color="${4:-$C2}"
    [ "$max" -eq 0 ] && max=1
    local filled=$(( val * width / max ))
    [ "$filled" -gt "$width" ] && filled=$width
    local empty=$(( width - filled ))
    # Fill with gradient blocks
    for ((i=0; i<filled; i++)); do
        local shade=$(( 236 + (i * 18 / width) ))
        [ "$shade" -gt 255 ] && shade=255
        printf "${color}▓${NC}"
    done
    for ((i=0; i<empty; i++)); do
        printf "${DIM}░${NC}"
    done
}

# Metric card
_card() {
    local label="$1" value="$2" color="${3:-$C2}"
    echo -e "  ${DIM}┌──────────────────────┐${NC}"
    printf "  ${DIM}│${NC} %-20s ${DIM}│${NC}\n" "$label"
    echo -e "  ${DIM}│${NC}  ${color}${BOLD}${value}${NC}$(printf '%*s' $((19 - ${#value})) '')${DIM}│${NC}"
    echo -e "  ${DIM}└──────────────────────┘${NC}"
}

([ ! -f "$SETTINGS" ] || [ ! -s "$SETTINGS" ]) && echo '{}' > "$SETTINGS"

# i18n
_is_zh() { [[ "${LANG:-}${LC_ALL:-}" =~ zh ]]; }
_t() {
    local key="$1"
    if _is_zh; then
        case "$key" in
            enabled)        echo "🟢 Bark: 启用中" ;;
            disabled)       echo "🔴 Bark: 已禁用" ;;
            cache_entries)  echo "   缓存条目" ;;
            log_lines)     echo "   日志行数" ;;
            already_on)    echo -e "\n  ${GREEN}●${NC} Bark 已经是启用状态\n" ;;
            turned_on)     echo -e "\n  ${GREEN}●${NC} Bark 已启用 ${DIM}(新会话生效)${NC}\n" ;;
            already_off)   echo -e "\n  ${DIM}●${NC} Bark 已经是禁用状态\n" ;;
            turned_off)    echo -e "\n  ${DIM}●${NC} Bark 已禁用 ${DIM}(新会话生效)${NC}\n" ;;
            cache_cleared) echo "✅ 缓存已清空" ;;
            cache_empty)   echo "📦 缓存为空" ;;
            cache_stats)   echo "📦 缓存统计:" ;;
            cache_count)   echo "   条目数" ;;
            cache_size)    echo "   占用" ;;
            cache_dir)     echo "   目录" ;;
            cache_recent)  echo "   最近缓存的判断:" ;;
            log_cleared)   echo "✅ 日志已清空" ;;
            log_empty)     echo "📋 日志为空" ;;
            log_recent)    echo "📋 最近" ;;
            log_suffix)    echo "条日志:" ;;
            uninstalling)  echo "正在卸载 Bark..." ;;
            uninstalled)   echo "✅ Bark 已完全卸载" ;;
            test_usage)    echo "用法: bark test [--verbose] [--dry-run] <bash command>" ;;
            test_example)  echo "示例: bark test --verbose rm -rf node_modules" ;;
            help_title)    echo "Bark — Claude Code AI 风险守卫" ;;
            cmd_status)    echo "  status            查看状态" ;;
            cmd_onoff)     echo "  on / off          启用 / 禁用" ;;
            cmd_toggle)    echo "  toggle            切换开关" ;;
            cmd_test)      echo "  test <cmd>        测试命令风险等级" ;;
            cmd_cache)     echo "  cache [clear]     查看/清空缓存" ;;
            cmd_log)       echo "  log [N|clear]     查看最近 N 条日志 / 清空日志" ;;
            cmd_stats)     echo "  stats             查看统计数据" ;;
            cmd_rules)     echo "  rules [edit]      查看/编辑自定义规则" ;;
            rules_empty)   echo "📋 未定义自定义规则" ;;
            rules_header)  echo "📋 自定义规则:" ;;
            rules_path)    echo "   文件" ;;
            stats_empty)   echo "📊 暂无统计数据（日志为空）" ;;
            cmd_update)    echo "  update            更新到最新版本" ;;
            cmd_uninstall) echo "  uninstall         完全卸载" ;;
            cmd_version)   echo "  version           显示版本号" ;;
            cmd_help)      echo "  help              显示此帮助" ;;
            updating)      echo "正在更新 Bark..." ;;
            update_ok)     echo "✅ Bark 已更新到最新版本" ;;
            update_fail)   echo "❌ 更新失败，请检查网络连接" ;;
            *)             echo "$key" ;;
        esac
    else
        case "$key" in
            enabled)        echo "🟢 Bark: Enabled" ;;
            disabled)       echo "🔴 Bark: Disabled" ;;
            cache_entries)  echo "   Cache entries" ;;
            log_lines)     echo "   Log lines" ;;
            already_on)    echo -e "\n  ${GREEN}●${NC} Bark is already enabled\n" ;;
            turned_on)     echo -e "\n  ${GREEN}●${NC} Bark enabled ${DIM}(takes effect in new sessions)${NC}\n" ;;
            already_off)   echo -e "\n  ${DIM}●${NC} Bark is already disabled\n" ;;
            turned_off)    echo -e "\n  ${DIM}●${NC} Bark disabled ${DIM}(takes effect in new sessions)${NC}\n" ;;
            cache_cleared) echo "✅ Cache cleared" ;;
            cache_empty)   echo "📦 Cache is empty" ;;
            cache_stats)   echo "📦 Cache statistics:" ;;
            cache_count)   echo "   Entries" ;;
            cache_size)    echo "   Size" ;;
            cache_dir)     echo "   Directory" ;;
            cache_recent)  echo "   Recent cached decisions:" ;;
            log_cleared)   echo "✅ Log cleared" ;;
            log_empty)     echo "📋 Log is empty" ;;
            log_recent)    echo "📋 Last" ;;
            log_suffix)    echo "log entries:" ;;
            uninstalling)  echo "Uninstalling Bark..." ;;
            uninstalled)   echo "✅ Bark fully uninstalled" ;;
            test_usage)    echo "Usage: bark test [--verbose] [--dry-run] <bash command>" ;;
            test_example)  echo "Example: bark test --verbose rm -rf node_modules" ;;
            help_title)    echo "Bark — AI Risk Assessment for Claude Code" ;;
            cmd_status)    echo "  status            Show status" ;;
            cmd_onoff)     echo "  on / off          Enable / disable" ;;
            cmd_toggle)    echo "  toggle            Toggle on/off" ;;
            cmd_test)      echo "  test <cmd>        Test a command's risk level" ;;
            cmd_cache)     echo "  cache [clear]     View/clear cache" ;;
            cmd_log)       echo "  log [N|clear]     View last N log entries / clear log" ;;
            cmd_stats)     echo "  stats             Show statistics" ;;
            cmd_rules)     echo "  rules [edit]      View/edit custom rules" ;;
            rules_empty)   echo "📋 No custom rules defined" ;;
            rules_header)  echo "📋 Custom rules:" ;;
            rules_path)    echo "   File" ;;
            stats_empty)   echo "📊 No statistics yet (log is empty)" ;;
            cmd_update)    echo "  update            Update to latest version" ;;
            cmd_uninstall) echo "  uninstall         Completely uninstall" ;;
            cmd_version)   echo "  version           Show version" ;;
            cmd_help)      echo "  help              Show this help" ;;
            updating)      echo "Updating Bark..." ;;
            update_ok)     echo "✅ Bark updated to latest version" ;;
            update_fail)   echo "❌ Update failed, please check your network" ;;
            *)             echo "$key" ;;
        esac
    fi
}

has_hook() {
    jq -e '.hooks.PreToolUse[]?.hooks[]? | select(.command | contains("bark.sh"))' "$SETTINGS" >/dev/null 2>&1
}

enable_hook() {
    if has_hook; then
        _t already_on
    else
        local tmp
        tmp=$(jq --arg cmd "$HOOK_SCRIPT" \
          '.hooks.PreToolUse = (.hooks.PreToolUse // []) + [{matcher:"*",hooks:[{type:"command",command:$cmd,timeout:30}]}]' \
          "$SETTINGS") && echo "$tmp" > "$SETTINGS"
        _t turned_on
    fi
}

disable_hook() {
    if ! has_hook; then
        _t already_off
    else
        local tmp
        tmp=$(jq '[.hooks.PreToolUse[]? | select(.hooks | all(.command | contains("bark.sh") | not))] as $new |
          if ($new | length) > 0 then .hooks.PreToolUse = $new
          else del(.hooks.PreToolUse) | if (.hooks | length) == 0 then del(.hooks) else . end
          end' "$SETTINGS") && echo "$tmp" > "$SETTINGS"
        _t turned_off
    fi
}

show_status() {
    echo ""
    _logo
    echo ""
    if has_hook; then
        echo -e "  ${GREEN}●${NC} ${BOLD}Active${NC}  ${DIM}─── hook is running${NC}"
    else
        echo -e "  ${RED}●${NC} ${BOLD}Inactive${NC}  ${DIM}─── hook is disabled${NC}"
    fi
    echo ""
    local count=0
    [ -d "$CACHE_DIR" ] && count=$(find "$CACHE_DIR" -type f 2>/dev/null | wc -l | tr -d ' ')
    local log_lines=0
    [ -f "$LOG_FILE" ] && log_lines=$(wc -l < "$LOG_FILE" | tr -d ' ')
    echo -e "  ${DIM}Cache entries${NC}  ${C2}${count}${NC}    ${DIM}Log entries${NC}  ${C2}${log_lines}${NC}"
    echo ""
}

test_command() {
    # Parse flags
    local verbose=0 dry_run=0
    while [[ "${1:-}" == --* ]]; do
        case "$1" in
            --verbose|-v) verbose=1; shift ;;
            --dry-run|-n) dry_run=1; verbose=1; shift ;;
            *) shift ;;
        esac
    done

    local cmd="$*"
    if [ -z "$cmd" ]; then
        _t test_usage
        _t test_example
        exit 1
    fi

    local payload
    payload=$(jq -n --arg cmd "$cmd" '{tool_name:"Bash",tool_input:{command:$cmd}}')

    local start end
    start=$(date +%s)
    local debug_file
    debug_file=$(mktemp)
    local json_result
    json_result=$(echo "$payload" | BARK_VERBOSE="$verbose" BARK_DRY_RUN="$dry_run" env -u CLAUDECODE "$HOOK_SCRIPT" 2>"$debug_file")
    end=$(date +%s)

    local elapsed="$((end - start))s"

    # Show debug output if any
    if [ -s "$debug_file" ]; then
        cat "$debug_file"
    fi
    rm -f "$debug_file"

    local decision reason icon
    decision=$(echo "$json_result" | jq -r '.hookSpecificOutput.permissionDecision // empty' 2>/dev/null)
    reason=$(echo "$json_result" | jq -r '.hookSpecificOutput.permissionDecisionReason // empty' 2>/dev/null)
    if [ -n "$decision" ]; then
        case "$decision" in
            allow) icon="${GREEN}◆${NC}" ;; ask) icon="${RED}◆${NC}" ;; deny) icon="${RED}✗${NC}" ;; *) icon="?" ;;
        esac
        echo -e "$icon $decision  ${DIM}($reason)${NC}  ${DIM}[$elapsed]${NC}"
    else
        echo -e "${RED}◆${NC} Parse error: ${json_result:0:100}"
    fi
}

cache_cmd() {
    case "${1:-}" in
        clear|clean)
            rm -rf "$CACHE_DIR"/*
            _t cache_cleared
            ;;
        *)
            echo ""
            if [ ! -d "$CACHE_DIR" ] || [ -z "$(ls -A "$CACHE_DIR" 2>/dev/null)" ]; then
                _t cache_empty
                echo ""
                return
            fi
            local count size
            count=$(find "$CACHE_DIR" -type f | wc -l | tr -d ' ')
            size=$(du -sh "$CACHE_DIR" 2>/dev/null | cut -f1)
            printf "  "; _gradient "Cache" 39 6; echo ""
            echo ""
            echo -e "  ${DIM}Entries${NC}  ${C2}${BOLD}${count}${NC}    ${DIM}Size${NC}  ${C2}${size}${NC}"
            echo -e "  ${DIM}${CACHE_DIR}${NC}"
            echo ""
            if _is_zh; then echo -e "  ${BOLD}最近缓存${NC}"
            else echo -e "  ${BOLD}Recent${NC}"; fi
            echo ""
            for f in $(ls -t "$CACHE_DIR" | head -10); do
                local content age_s age_h
                content=$(cat "$CACHE_DIR/$f")
                age_s=$(( $(date +%s) - $(stat -f %m "$CACHE_DIR/$f" 2>/dev/null || stat -c %Y "$CACHE_DIR/$f" 2>/dev/null || echo 0) ))
                if [ "$age_s" -lt 3600 ]; then
                    age_h="${age_s}s"
                elif [ "$age_s" -lt 86400 ]; then
                    age_h="$(( age_s / 3600 ))h"
                else
                    age_h="$(( age_s / 86400 ))d"
                fi
                local level reason color
                level=$(echo "$content" | cut -d'|' -f1)
                reason=$(echo "$content" | cut -d'|' -f2-)
                case "$level" in
                    0) color="$GREEN"  ;; 1) color="$YELLOW" ;; 2) color="$RED" ;; *) color="$DIM" ;;
                esac
                echo -e "    ${color}◆${NC} ${reason}  ${DIM}${age_h} ago${NC}"
            done
            echo ""
            ;;
    esac
}

log_cmd() {
    case "${1:-}" in
        clear|clean)
            > "$LOG_FILE"
            _t log_cleared
            ;;
        *)
            if [ ! -f "$LOG_FILE" ] || [ ! -s "$LOG_FILE" ]; then
                echo ""
                _t log_empty
                echo ""
                return
            fi
            local n="${1:-20}"
            echo ""
            if _is_zh; then printf "  "; _gradient "最近 ${n} 条日志" 39 6; echo ""
            else printf "  "; _gradient "Last ${n} entries" 39 6; echo ""; fi
            echo ""
            tail -n "$n" "$LOG_FILE" | while IFS= read -r line; do
                # Colorize log lines based on level
                if echo "$line" | grep -q 'HIGH'; then
                    echo -e "    ${RED}▎${NC}${DIM}${line}${NC}"
                elif echo "$line" | grep -q 'MED '; then
                    echo -e "    ${YELLOW}▎${NC}${DIM}${line}${NC}"
                else
                    echo -e "    ${GREEN}▎${NC}${DIM}${line}${NC}"
                fi
            done
            echo ""
            ;;
    esac
}

uninstall_cmd() {
    _t uninstalling
    disable_hook 2>/dev/null
    for dir in /usr/local/bin /opt/homebrew/bin; do
        [ -L "$dir/bark" ] && rm -f "$dir/bark"
    done
    rm -rf "$CACHE_DIR"
    rm -f "$HOOK_SCRIPT" "$LOG_FILE"
    local self="$HOME/.claude/hooks/bark-ctl.sh"
    _t uninstalled
    rm -f "$self"
}

stats_cmd() {
    if [ ! -f "$LOG_FILE" ] || [ ! -s "$LOG_FILE" ]; then
        _t stats_empty
        return
    fi

    local total fast cache rule ai low med high
    total=$(wc -l < "$LOG_FILE" | tr -d ' ')
    fast=$(grep -c '\[FAST\]' "$LOG_FILE" 2>/dev/null || echo 0)
    cache=$(grep -c '\[CACHE\]' "$LOG_FILE" 2>/dev/null || echo 0)
    rule=$(grep -c '\[RULE\]' "$LOG_FILE" 2>/dev/null || echo 0)
    ai=$(grep -c '\[AI\]' "$LOG_FILE" 2>/dev/null || echo 0)
    low=$(grep -c 'LOW ' "$LOG_FILE" 2>/dev/null || echo 0)
    med=$(grep -c 'MED ' "$LOG_FILE" 2>/dev/null || echo 0)
    high=$(grep -c 'HIGH' "$LOG_FILE" 2>/dev/null || echo 0)

    local cache_plus_ai=$((cache + ai))
    local hit_rate=0
    [ "$cache_plus_ai" -gt 0 ] && hit_rate=$((cache * 100 / cache_plus_ai))

    # Animated header
    echo ""
    echo -e "  ${DIM}╭───────────────────────────────────────────────────╮${NC}"
    printf "  ${DIM}│${NC}  "; _gradient "◆ Bark Statistics" 39 6; printf "                         ${DIM}│${NC}\n"
    echo -e "  ${DIM}╰───────────────────────────────────────────────────╯${NC}"
    echo ""

    # Big number - total
    if _is_zh; then printf "  ${DIM}总评估${NC}  "; else printf "  ${DIM}Total${NC}  "; fi
    echo -e "${C1}${BOLD}${total}${NC}"
    echo ""

    # Source breakdown with animated bars
    if _is_zh; then echo -e "  ${BOLD}评估来源${NC}"
    else echo -e "  ${BOLD}Assessment Source${NC}"; fi
    echo ""
    local pct
    for src_name in FAST CACHE RULE AI; do
        local src_val src_color src_label
        case "$src_name" in
            FAST)  src_val=$fast;  src_color="$GREEN";   src_label="${DIM}fast rules${NC}" ;;
            CACHE) src_val=$cache; src_color="$C2";      src_label="${DIM}cache hit${NC}" ;;
            RULE)  src_val=$rule;  src_color="$PURPLE";  src_label="${DIM}custom rules${NC}" ;;
            AI)    src_val=$ai;    src_color="$ORANGE";  src_label="${DIM}AI assessed${NC}" ;;
        esac
        [ "$total" -gt 0 ] && pct=$((src_val * 100 / total)) || pct=0
        printf "    ${src_color}%-6s${NC} " "$src_name"
        _abar "$src_val" "$total" 24 "$src_color"
        printf "  ${BOLD}%3d${NC} ${DIM}(%2d%%)${NC}  %b\n" "$src_val" "$pct" "$src_label"
    done
    echo ""

    # Risk level distribution
    if _is_zh; then echo -e "  ${BOLD}风险分布${NC}"
    else echo -e "  ${BOLD}Risk Distribution${NC}"; fi
    echo ""
    for lvl_name in Low Medium High; do
        local lvl_val lvl_color lvl_icon
        case "$lvl_name" in
            Low)    lvl_val=$low;  lvl_color="$GREEN";  lvl_icon="◉" ;;
            Medium) lvl_val=$med;  lvl_color="$YELLOW"; lvl_icon="◉" ;;
            High)   lvl_val=$high; lvl_color="$RED";    lvl_icon="◉" ;;
        esac
        [ "$total" -gt 0 ] && pct=$((lvl_val * 100 / total)) || pct=0
        printf "    ${lvl_color}${lvl_icon}${NC} %-8s " "$lvl_name"
        _abar "$lvl_val" "$total" 24 "$lvl_color"
        printf "  ${BOLD}%3d${NC} ${DIM}(%2d%%)${NC}\n" "$lvl_val" "$pct"
    done
    echo ""

    # Metric cards side by side
    local hit_str="${hit_rate}%"
    [ "$cache_plus_ai" -eq 0 ] && hit_str="—"
    echo -e "  ${DIM}┌──────────────────────┐  ┌──────────────────────┐${NC}"
    echo -e "  ${DIM}│${NC}  ${ITALIC}Cache Hit Rate${NC}       ${DIM}│${NC}  ${DIM}│${NC}  ${ITALIC}High Risk Blocked${NC}    ${DIM}│${NC}"
    printf "  ${DIM}│${NC}  ${C1}${BOLD}%-20s${NC} ${DIM}│${NC}" "$hit_str"
    echo -e "  ${DIM}│${NC}  ${RED}${BOLD}${high}${NC}$(printf '%*s' $((19 - ${#high})) '')${DIM}│${NC}"
    echo -e "  ${DIM}└──────────────────────┘  └──────────────────────┘${NC}"
    echo ""
}

rules_cmd() {
    local conf="$HOME/.claude/hooks/bark.conf"
    case "${1:-}" in
        edit)
            ${EDITOR:-vi} "$conf"
            ;;
        *)
            echo ""
            if [ ! -f "$conf" ] || [ ! -s "$conf" ]; then
                _t rules_empty
                echo -e "  ${DIM}$conf${NC}"
                echo ""
                return
            fi
            printf "  "; _gradient "Custom Rules" 39 6; echo ""
            echo ""
            local line count=0
            while IFS= read -r line || [ -n "$line" ]; do
                line=$(echo "$line" | sed 's/#.*//' | xargs)
                [ -z "$line" ] && continue
                local action pattern color
                action=$(echo "$line" | cut -d: -f1 | xargs | tr '[:upper:]' '[:lower:]')
                pattern=$(echo "$line" | cut -d: -f2- | xargs)
                case "$action" in
                    allow)  color="$GREEN"  ;; notify) color="$YELLOW" ;; block) color="$RED" ;; *) color="$DIM" ;;
                esac
                echo -e "    ${color}◆${NC} ${BOLD}${action}${NC}  ${DIM}${pattern}${NC}"
                ((count++))
            done < "$conf"
            [ "$count" -eq 0 ] && echo -e "    ${DIM}(no rules defined)${NC}"
            echo ""
            echo -e "  ${DIM}$conf${NC}"
            echo ""
            ;;
    esac
}

update_cmd() {
    local old_ver="$VERSION"
    _t updating
    local url="https://raw.githubusercontent.com/shaominngqing/Risk-Guard/main/install.sh"
    local tmp
    tmp=$(mktemp)
    if curl -fsSL "$url" -o "$tmp" 2>/dev/null; then
        # Extract new version before running
        local new_ver
        new_ver=$(grep '^BARK_VERSION=' "$tmp" | head -1 | cut -d'"' -f2)
        bash "$tmp"
        rm -f "$tmp"
        if [ "$old_ver" = "$new_ver" ]; then
            if _is_zh; then echo -e "\n  ${GREEN}●${NC} 已是最新版本 ${DIM}v${old_ver}${NC}\n"
            else echo -e "\n  ${GREEN}●${NC} Already up to date ${DIM}v${old_ver}${NC}\n"; fi
        else
            if _is_zh; then echo -e "\n  ${GREEN}●${NC} 已更新 ${DIM}v${old_ver}${NC} → ${C1}${BOLD}v${new_ver}${NC}\n"
            else echo -e "\n  ${GREEN}●${NC} Updated ${DIM}v${old_ver}${NC} → ${C1}${BOLD}v${new_ver}${NC}\n"; fi
        fi
    else
        rm -f "$tmp"
        _t update_fail
        exit 1
    fi
}

usage() {
    echo ""
    _logo
    echo ""
    if _is_zh; then echo -e "  ${DIM}用法:${NC} ${BOLD}bark${NC} ${DIM}<command>${NC}"
    else echo -e "  ${DIM}Usage:${NC} ${BOLD}bark${NC} ${DIM}<command>${NC}"; fi
    echo ""
    if _is_zh; then echo -e "  ${BOLD}命令${NC}"; else echo -e "  ${BOLD}Commands${NC}"; fi
    echo ""
    _t cmd_status
    _t cmd_onoff
    _t cmd_toggle
    _t cmd_test
    _t cmd_cache
    _t cmd_log
    _t cmd_stats
    _t cmd_rules
    _t cmd_update
    _t cmd_uninstall
    _t cmd_version
    _t cmd_help
    echo ""
}

case "${1:-status}" in
    on|enable)      enable_hook ;;
    off|disable)    disable_hook ;;
    toggle)         if has_hook; then disable_hook; else enable_hook; fi ;;
    status)         show_status ;;
    test)           shift; test_command "$@" ;;
    cache)          shift; cache_cmd "$@" ;;
    log|logs)       shift; log_cmd "$@" ;;
    stats|stat)     stats_cmd ;;
    rules|rule)     shift; rules_cmd "$@" ;;
    update|upgrade) update_cmd ;;
    uninstall)      uninstall_cmd ;;
    version|-V|--version) echo ""; _logo; echo -e "  ${DIM}v${VERSION}${NC}"; echo "" ;;
    help|-h|--help) usage ;;
    *)              if _is_zh; then echo "未知命令: $1"; else echo "Unknown command: $1"; fi; usage; exit 1 ;;
esac
CTL__EOF
# Inject version into ctl script (heredoc is single-quoted so no expansion)
sed -i.bak "s/__BARK_VERSION__/$BARK_VERSION/" "$CTL_SCRIPT" && rm -f "$CTL_SCRIPT.bak"
chmod +x "$CTL_SCRIPT"
if is_zh; then ok "控制面板      bark-ctl.sh"
else ok "Control panel  bark-ctl.sh"; fi

# ═══ 全局命令 ═══
if is_zh; then step "注册全局命令"; else step "Register global command"; fi
BIN_DIR=""
if [ -d /opt/homebrew/bin ] && echo "$PATH" | grep -q "/opt/homebrew/bin"; then
    BIN_DIR="/opt/homebrew/bin"
elif [ -d /usr/local/bin ]; then
    BIN_DIR="/usr/local/bin"
fi

if [ -n "$BIN_DIR" ]; then
    if is_zh; then
        ln -sf "$CTL_SCRIPT" "$BIN_DIR/bark" 2>/dev/null && ok "bark → $BIN_DIR/" || warn "请手动: ln -s $CTL_SCRIPT /usr/local/bin/bark"
    else
        ln -sf "$CTL_SCRIPT" "$BIN_DIR/bark" 2>/dev/null && ok "bark → $BIN_DIR/" || warn "Please run: ln -s $CTL_SCRIPT /usr/local/bin/bark"
    fi
else
    if is_zh; then warn "请手动: alias bark='$CTL_SCRIPT'"
    else warn "Please run: alias bark='$CTL_SCRIPT'"; fi
fi

# ═══ 注入 hooks 配置 ═══
if is_zh; then step "配置 Claude Code Hooks"; else step "Configure Claude Code Hooks"; fi
# Inject hook into settings if not already present
if ! jq -e '.hooks.PreToolUse[]?.hooks[]? | select(.command | contains("bark.sh"))' "$SETTINGS" >/dev/null 2>&1; then
    tmp=$(jq --arg cmd "$HOOK_SCRIPT" \
      '.hooks.PreToolUse = (.hooks.PreToolUse // []) + [{matcher:"*",hooks:[{type:"command",command:$cmd,timeout:30}]}]' \
      "$SETTINGS") && echo "$tmp" > "$SETTINGS"
fi
ok "PreToolUse hook → settings.json"

echo ""
sleep 0.1
echo -e "  ${DIM}╭───────────────────────────────────────────────╮${NC}"
printf "  ${DIM}│${NC}  "; _gradient "✦ Install complete" 34 8; printf "                          ${DIM}│${NC}\n"
echo -e "  ${DIM}╰───────────────────────────────────────────────╯${NC}"
echo ""
if is_zh; then
echo -e "  ${BOLD}工作原理${NC}"
echo ""
echo -e "    ${GREEN}◆${NC} ${DIM}只读工具${NC}  Read / Grep / Glob     ${DIM}──▸${NC} ${GREEN}直接放行${NC}"
sleep 0.05
echo -e "    ${GREEN}◆${NC} ${DIM}文件编辑${NC}  普通源代码文件         ${DIM}──▸${NC} ${GREEN}直接放行${NC}"
sleep 0.05
echo -e "    ${C2}◆${NC} ${DIM}Bash命令${NC}  所有命令               ${DIM}──▸${NC} ${C2}AI 评估 (~7s)${NC}"
sleep 0.05
echo -e "    ${GREEN}◆${NC} ${DIM}重复模式${NC}  同类命令第二次         ${DIM}──▸${NC} ${GREEN}缓存命中 (0s)${NC}"
sleep 0.05
echo -e "    ${RED}◆${NC} ${DIM}高风险  ${NC}  rm -rf / force push   ${DIM}──▸${NC} ${RED}通知 + 确认${NC}"
echo ""
echo -e "  ${BOLD}快速开始${NC}"
echo ""
echo -e "    ${C1}bark${NC} ${DIM}help${NC}           查看所有命令"
echo -e "    ${C1}bark${NC} ${DIM}stats${NC}          查看统计数据"
echo ""
echo -e "  ${ACCENT}▸ 新开的 Claude Code 会话自动生效${NC}"
else
echo -e "  ${BOLD}How it works${NC}"
echo ""
echo -e "    ${GREEN}◆${NC} ${DIM}Read-only${NC}  Read / Grep / Glob     ${DIM}──▸${NC} ${GREEN}Allow${NC}"
sleep 0.05
echo -e "    ${GREEN}◆${NC} ${DIM}Edits   ${NC}  Normal source files     ${DIM}──▸${NC} ${GREEN}Allow${NC}"
sleep 0.05
echo -e "    ${C2}◆${NC} ${DIM}Bash    ${NC}  All commands             ${DIM}──▸${NC} ${C2}AI assess (~7s)${NC}"
sleep 0.05
echo -e "    ${GREEN}◆${NC} ${DIM}Repeat  ${NC}  Same pattern again      ${DIM}──▸${NC} ${GREEN}Cache hit (0s)${NC}"
sleep 0.05
echo -e "    ${RED}◆${NC} ${DIM}Danger  ${NC}  rm -rf / force push     ${DIM}──▸${NC} ${RED}Notify + confirm${NC}"
echo ""
echo -e "  ${BOLD}Quick start${NC}"
echo ""
echo -e "    ${C1}bark${NC} ${DIM}help${NC}           Show all commands"
echo -e "    ${C1}bark${NC} ${DIM}stats${NC}          View statistics"
echo ""
echo -e "  ${ACCENT}▸ Takes effect in new Claude Code sessions${NC}"
fi
echo ""
