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

# i18n: detect locale
is_zh() { [[ "${LANG:-}${LC_ALL:-}" =~ zh ]]; }

echo ""
echo -e "${BOLD}  ╭───────────────────────────────────────────╮${NC}"
echo -e "${BOLD}  │                                           │${NC}"
echo -e "${BOLD}  │   ${CYAN}⚡ Risk Guard${NC}${BOLD}  for Claude Code          │${NC}"
echo -e "${BOLD}  │   ${DIM}AI-Powered Risk Assessment Hook${NC}${BOLD}         │${NC}"
echo -e "${BOLD}  │                                           │${NC}"
echo -e "${BOLD}  ╰───────────────────────────────────────────╯${NC}"

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
CACHE_TTL=86400  # 24h
LOG_FILE="$HOME/.claude/hooks/risk-guard.log"
LOG_MAX_LINES=500
mkdir -p "$CACHE_DIR"

# i18n
_is_zh() { [[ "${LANG:-}${LC_ALL:-}" =~ zh ]]; }

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
    if _is_zh; then
        case "$level" in
            0) level_tag="LOW " ; output_decision "allow" "[低风险] $reason" ;;
            1) level_tag="MED " ; notify "⚠️ Claude Code" "已自动放行" "$reason"
               output_decision "allow" "[中风险] $reason" ;;
            2) level_tag="HIGH" ; notify "🚨 Claude Code" "需要确认" "$reason" "Funk"
               output_decision "ask" "[高风险] $reason" ;;
        esac
    else
        case "$level" in
            0) level_tag="LOW " ; output_decision "allow" "[Low] $reason" ;;
            1) level_tag="MED " ; notify "⚠️ Claude Code" "Auto-allowed" "$reason"
               output_decision "allow" "[Medium] $reason" ;;
            2) level_tag="HIGH" ; notify "🚨 Claude Code" "Confirmation needed" "$reason" "Funk"
               output_decision "ask" "[High] $reason" ;;
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

if is_zh; then step "安装组件"; ok "风险评估引擎  risk-guard.sh"
else step "Install components"; ok "Risk assessment engine  risk-guard.sh"; fi

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

([ ! -f "$SETTINGS" ] || [ ! -s "$SETTINGS" ]) && echo '{}' > "$SETTINGS"

# i18n
_is_zh() { [[ "${LANG:-}${LC_ALL:-}" =~ zh ]]; }
_t() {
    local key="$1"
    if _is_zh; then
        case "$key" in
            enabled)        echo "🟢 Risk Guard: 启用中" ;;
            disabled)       echo "🔴 Risk Guard: 已禁用" ;;
            cache_entries)  echo "   缓存条目" ;;
            log_lines)     echo "   日志行数" ;;
            already_on)    echo "⚡ Risk Guard 已经是启用状态" ;;
            turned_on)     echo "✅ Risk Guard 已启用 (新会话生效)" ;;
            already_off)   echo "⚡ Risk Guard 已经是禁用状态" ;;
            turned_off)    echo "⛔ Risk Guard 已禁用 (新会话生效)" ;;
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
            uninstalling)  echo "正在卸载 Risk Guard..." ;;
            uninstalled)   echo "✅ Risk Guard 已完全卸载" ;;
            test_usage)    echo "用法: risk-guard test <bash command>" ;;
            test_example)  echo "示例: risk-guard test rm -rf node_modules" ;;
            help_title)    echo "Risk Guard — Claude Code AI 风险守卫" ;;
            cmd_status)    echo "  status            查看状态" ;;
            cmd_onoff)     echo "  on / off          启用 / 禁用" ;;
            cmd_toggle)    echo "  toggle            切换开关" ;;
            cmd_test)      echo "  test <cmd>        测试命令风险等级" ;;
            cmd_cache)     echo "  cache [clear]     查看/清空缓存" ;;
            cmd_log)       echo "  log [N|clear]     查看最近 N 条日志 / 清空日志" ;;
            cmd_uninstall) echo "  uninstall         完全卸载" ;;
            cmd_help)      echo "  help              显示此帮助" ;;
            *)             echo "$key" ;;
        esac
    else
        case "$key" in
            enabled)        echo "🟢 Risk Guard: Enabled" ;;
            disabled)       echo "🔴 Risk Guard: Disabled" ;;
            cache_entries)  echo "   Cache entries" ;;
            log_lines)     echo "   Log lines" ;;
            already_on)    echo "⚡ Risk Guard is already enabled" ;;
            turned_on)     echo "✅ Risk Guard enabled (takes effect in new sessions)" ;;
            already_off)   echo "⚡ Risk Guard is already disabled" ;;
            turned_off)    echo "⛔ Risk Guard disabled (takes effect in new sessions)" ;;
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
            uninstalling)  echo "Uninstalling Risk Guard..." ;;
            uninstalled)   echo "✅ Risk Guard fully uninstalled" ;;
            test_usage)    echo "Usage: risk-guard test <bash command>" ;;
            test_example)  echo "Example: risk-guard test rm -rf node_modules" ;;
            help_title)    echo "Risk Guard — AI Risk Assessment for Claude Code" ;;
            cmd_status)    echo "  status            Show status" ;;
            cmd_onoff)     echo "  on / off          Enable / disable" ;;
            cmd_toggle)    echo "  toggle            Toggle on/off" ;;
            cmd_test)      echo "  test <cmd>        Test a command's risk level" ;;
            cmd_cache)     echo "  cache [clear]     View/clear cache" ;;
            cmd_log)       echo "  log [N|clear]     View last N log entries / clear log" ;;
            cmd_uninstall) echo "  uninstall         Completely uninstall" ;;
            cmd_help)      echo "  help              Show this help" ;;
            *)             echo "$key" ;;
        esac
    fi
}

has_hook() {
    jq -e '.hooks.PreToolUse[]?.hooks[]? | select(.command | contains("risk-guard.sh"))' "$SETTINGS" >/dev/null 2>&1
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
        tmp=$(jq '[.hooks.PreToolUse[]? | select(.hooks | all(.command | contains("risk-guard.sh") | not))] as $new |
          if ($new | length) > 0 then .hooks.PreToolUse = $new
          else del(.hooks.PreToolUse) | if (.hooks | length) == 0 then del(.hooks) else . end
          end' "$SETTINGS") && echo "$tmp" > "$SETTINGS"
        _t turned_off
    fi
}

show_status() {
    if has_hook; then _t enabled; else _t disabled; fi
    local count=0
    [ -d "$CACHE_DIR" ] && count=$(find "$CACHE_DIR" -type f 2>/dev/null | wc -l | tr -d ' ')
    echo "$(_t cache_entries): $count"
    local log_lines=0
    [ -f "$LOG_FILE" ] && log_lines=$(wc -l < "$LOG_FILE" | tr -d ' ')
    echo "$(_t log_lines): $log_lines"
}

test_command() {
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
    local result
    result=$(echo "$payload" | env -u CLAUDECODE "$HOOK_SCRIPT" 2>/dev/null)
    end=$(date +%s)

    local elapsed="$((end - start))s"

    local decision reason icon
    decision=$(echo "$result" | jq -r '.hookSpecificOutput.permissionDecision // empty' 2>/dev/null)
    reason=$(echo "$result" | jq -r '.hookSpecificOutput.permissionDecisionReason // empty' 2>/dev/null)
    if [ -n "$decision" ]; then
        case "$decision" in
            allow) icon="✅" ;; ask) icon="🚨" ;; deny) icon="❌" ;; *) icon="?" ;;
        esac
        echo "$icon $decision  ($reason)  [$elapsed]"
    else
        echo "⚠️  Parse error: ${result:0:100}"
    fi
}

cache_cmd() {
    case "${1:-}" in
        clear|clean)
            rm -rf "$CACHE_DIR"/*
            _t cache_cleared
            ;;
        *)
            if [ ! -d "$CACHE_DIR" ] || [ -z "$(ls -A "$CACHE_DIR" 2>/dev/null)" ]; then
                _t cache_empty
                return
            fi
            local count
            count=$(find "$CACHE_DIR" -type f | wc -l | tr -d ' ')
            local size
            size=$(du -sh "$CACHE_DIR" 2>/dev/null | cut -f1)
            _t cache_stats
            echo "$(_t cache_count): $count"
            echo "$(_t cache_size):   $size"
            echo "$(_t cache_dir):   $CACHE_DIR"
            echo ""
            _t cache_recent
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
            _t log_cleared
            ;;
        *)
            if [ ! -f "$LOG_FILE" ] || [ ! -s "$LOG_FILE" ]; then
                _t log_empty
                return
            fi
            local n="${1:-20}"
            echo "$(_t log_recent) $n $(_t log_suffix)"
            echo ""
            tail -n "$n" "$LOG_FILE"
            ;;
    esac
}

uninstall_cmd() {
    _t uninstalling
    disable_hook 2>/dev/null
    for dir in /usr/local/bin /opt/homebrew/bin; do
        [ -L "$dir/risk-guard" ] && rm -f "$dir/risk-guard"
    done
    rm -rf "$CACHE_DIR"
    rm -f "$HOOK_SCRIPT" "$LOG_FILE"
    local self="$HOME/.claude/hooks/risk-guard-ctl.sh"
    _t uninstalled
    rm -f "$self"
}

usage() {
    _t help_title
    echo ""
    if _is_zh; then echo "用法: risk-guard <command>"; else echo "Usage: risk-guard <command>"; fi
    echo ""
    if _is_zh; then echo "命令:"; else echo "Commands:"; fi
    _t cmd_status
    _t cmd_onoff
    _t cmd_toggle
    _t cmd_test
    _t cmd_cache
    _t cmd_log
    _t cmd_uninstall
    _t cmd_help
}

case "${1:-status}" in
    on|enable)      enable_hook ;;
    off|disable)    disable_hook ;;
    toggle)         if has_hook; then disable_hook; else enable_hook; fi ;;
    status)         show_status ;;
    test)           shift; test_command "$@" ;;
    cache)          shift; cache_cmd "$@" ;;
    log|logs)       shift; log_cmd "$@" ;;
    uninstall)      uninstall_cmd ;;
    help|-h|--help) usage ;;
    *)              if _is_zh; then echo "未知命令: $1"; else echo "Unknown command: $1"; fi; usage; exit 1 ;;
esac
CTL__EOF
chmod +x "$CTL_SCRIPT"
if is_zh; then ok "控制面板      risk-guard-ctl.sh"
else ok "Control panel  risk-guard-ctl.sh"; fi

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
        ln -sf "$CTL_SCRIPT" "$BIN_DIR/risk-guard" 2>/dev/null && ok "risk-guard → $BIN_DIR/" || warn "请手动: ln -s $CTL_SCRIPT /usr/local/bin/risk-guard"
    else
        ln -sf "$CTL_SCRIPT" "$BIN_DIR/risk-guard" 2>/dev/null && ok "risk-guard → $BIN_DIR/" || warn "Please run: ln -s $CTL_SCRIPT /usr/local/bin/risk-guard"
    fi
else
    if is_zh; then warn "请手动: alias risk-guard='$CTL_SCRIPT'"
    else warn "Please run: alias risk-guard='$CTL_SCRIPT'"; fi
fi

# ═══ 注入 hooks 配置 ═══
if is_zh; then step "配置 Claude Code Hooks"; else step "Configure Claude Code Hooks"; fi
# Inject hook into settings if not already present
if ! jq -e '.hooks.PreToolUse[]?.hooks[]? | select(.command | contains("risk-guard.sh"))' "$SETTINGS" >/dev/null 2>&1; then
    tmp=$(jq --arg cmd "$HOOK_SCRIPT" \
      '.hooks.PreToolUse = (.hooks.PreToolUse // []) + [{matcher:"*",hooks:[{type:"command",command:$cmd,timeout:30}]}]' \
      "$SETTINGS") && echo "$tmp" > "$SETTINGS"
fi
ok "PreToolUse hook → settings.json"

echo ""
echo -e "${BOLD}  ╭───────────────────────────────────────────╮${NC}"
if is_zh; then
echo -e "${BOLD}  │  ${GREEN}安装完成！${NC}${BOLD}                                │${NC}"
else
echo -e "${BOLD}  │  ${GREEN}Install complete!${NC}${BOLD}                         │${NC}"
fi
echo -e "${BOLD}  ╰───────────────────────────────────────────╯${NC}"
echo ""
if is_zh; then
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
echo -e "    ${CYAN}risk-guard${NC} ${DIM}help${NC}           查看所有命令"
echo ""
echo -e "  ${YELLOW}▸ 新开的 Claude Code 会话自动生效${NC}"
else
echo -e "  ${BOLD}How it works${NC}"
echo ""
echo -e "    ${DIM}Read-only${NC}  Read / Grep / Glob ...  ${GREEN}━━▸${NC} Allow"
echo -e "    ${DIM}Edits   ${NC}  Normal source files      ${GREEN}━━▸${NC} Allow"
echo -e "    ${DIM}Bash    ${NC}  All commands              ${CYAN}━━▸${NC} AI assess (~7s)"
echo -e "    ${DIM}Repeat  ${NC}  Same pattern again        ${GREEN}━━▸${NC} Cache hit (0s)"
echo -e "    ${DIM}Danger  ${NC}  rm -rf / force push ...   ${RED}━━▸${NC} Notify + confirm"
echo ""
echo -e "  ${BOLD}Commands${NC}"
echo ""
echo -e "    ${CYAN}risk-guard${NC} ${DIM}help${NC}           Show all commands"
echo ""
echo -e "  ${YELLOW}▸ Takes effect in new Claude Code sessions${NC}"
fi
echo ""
