#!/bin/bash
# =============================================================================
# Bark - Core Logic Tests
# =============================================================================
# Tests cache_key normalization, fast_check, output format, and custom rules.
# Does NOT require claude CLI (no AI layer tests).
# =============================================================================

set -uo pipefail

PASS=0
FAIL=0
HOOK_SCRIPT="/tmp/bark.sh"

RED='\033[0;31m'; GREEN='\033[0;32m'; BOLD='\033[1m'; DIM='\033[2m'; NC='\033[0m'

assert_eq() {
    local desc="$1" expected="$2" actual="$3"
    if [ "$expected" = "$actual" ]; then
        echo -e "  ${GREEN}PASS${NC}  $desc"
        PASS=$((PASS+1))
    else
        echo -e "  ${RED}FAIL${NC}  $desc"
        echo -e "        ${DIM}expected: ${expected}${NC}"
        echo -e "        ${DIM}  actual: ${actual}${NC}"
        FAIL=$((FAIL+1))
    fi
}

assert_contains() {
    local desc="$1" needle="$2" haystack="$3"
    if echo "$haystack" | grep -q "$needle"; then
        echo -e "  ${GREEN}PASS${NC}  $desc"
        PASS=$((PASS+1))
    else
        echo -e "  ${RED}FAIL${NC}  $desc"
        echo -e "        ${DIM}expected to contain: ${needle}${NC}"
        echo -e "        ${DIM}  actual: ${haystack}${NC}"
        FAIL=$((FAIL+1))
    fi
}

# Helper: run hook with a given tool_name and input, return JSON output
run_hook() {
    local tool="$1" input="$2"
    echo "$input" | jq -n --arg tool "$tool" --argjson input "$(cat)" \
      '{tool_name: $tool, tool_input: $input}' \
      | "$HOOK_SCRIPT" 2>/dev/null
}

run_hook_bash() {
    local cmd="$1"
    echo "{\"tool_name\":\"Bash\",\"tool_input\":{\"command\":\"$cmd\"}}" \
      | "$HOOK_SCRIPT" 2>/dev/null
}

run_hook_tool() {
    local tool="$1"
    echo "{\"tool_name\":\"$tool\",\"tool_input\":{}}" \
      | "$HOOK_SCRIPT" 2>/dev/null
}

run_hook_file() {
    local tool="$1" path="$2"
    echo "{\"tool_name\":\"$tool\",\"tool_input\":{\"file_path\":\"$path\"}}" \
      | "$HOOK_SCRIPT" 2>/dev/null
}

get_decision() {
    echo "$1" | jq -r '.hookSpecificOutput.permissionDecision // empty'
}

get_reason() {
    echo "$1" | jq -r '.hookSpecificOutput.permissionDecisionReason // empty'
}

# Ensure hook script exists
if [ ! -f "$HOOK_SCRIPT" ]; then
    echo "Extracting hook script..."
    sed -n "/^cat > \"\$HOOK_SCRIPT\" << 'HOOK__EOF'/,/^HOOK__EOF/p" install.sh \
      | tail -n +2 | sed '$d' > "$HOOK_SCRIPT"
    chmod +x "$HOOK_SCRIPT"
fi

# =============================================================================
echo -e "\n${BOLD}Fast Check - Read-only Tools${NC}\n"
# =============================================================================

for tool in Read Glob Grep Agent WebFetch Skill AskUserQuestion; do
    result=$(run_hook_tool "$tool")
    decision=$(get_decision "$result")
    assert_eq "tool=$tool → allow" "allow" "$decision"
done

# =============================================================================
echo -e "\n${BOLD}Fast Check - Task Tools${NC}\n"
# =============================================================================

for tool in TaskCreate TaskGet TaskList TaskOutput TaskUpdate TaskStop; do
    result=$(run_hook_tool "$tool")
    decision=$(get_decision "$result")
    assert_eq "tool=$tool → allow" "allow" "$decision"
done

# =============================================================================
echo -e "\n${BOLD}Fast Check - Normal File Edits${NC}\n"
# =============================================================================

for path in "src/main.ts" "lib/utils.py" "README.md" "Makefile"; do
    result=$(run_hook_file "Edit" "$path")
    decision=$(get_decision "$result")
    assert_eq "Edit $path → allow" "allow" "$decision"
done

# =============================================================================
echo -e "\n${BOLD}Fast Check - Sensitive File Edits (should NOT fast-allow)${NC}\n"
# =============================================================================

# These should NOT be fast-allowed (they go to cache/AI)
# Since there's no claude CLI in CI, they'll fail to AI assess and fallback to level 1
# We just check they don't get "Low" fast-pass reason
for path in ".env" "credentials.json" ".github/workflows/ci.yml" "id_rsa"; do
    result=$(run_hook_file "Edit" "$path")
    reason=$(get_reason "$result")
    # Should NOT contain "Normal file edit" which is the fast-check reason
    if echo "$reason" | grep -q "Normal file edit\|普通文件编辑"; then
        echo -e "  ${RED}FAIL${NC}  Edit $path should NOT be fast-allowed"
        FAIL=$((FAIL+1))
    else
        echo -e "  ${GREEN}PASS${NC}  Edit $path not fast-allowed"
        PASS=$((PASS+1))
    fi
done

# =============================================================================
echo -e "\n${BOLD}Output Format${NC}\n"
# =============================================================================

result=$(run_hook_tool "Read")
# Verify JSON structure
hook_event=$(echo "$result" | jq -r '.hookSpecificOutput.hookEventName // empty')
assert_eq "hookEventName=PreToolUse" "PreToolUse" "$hook_event"

decision=$(get_decision "$result")
assert_eq "permissionDecision exists" "allow" "$decision"

reason=$(get_reason "$result")
assert_contains "permissionDecisionReason non-empty" "." "$reason"

# =============================================================================
echo -e "\n${BOLD}Custom Rules${NC}\n"
# =============================================================================

CONF_FILE="$HOME/.claude/hooks/bark.conf"
mkdir -p "$HOME/.claude/hooks"

# Backup existing conf if any
[ -f "$CONF_FILE" ] && cp "$CONF_FILE" "$CONF_FILE.bak"

# Write test rules
cat > "$CONF_FILE" << 'EOF'
allow: make build
allow: npm test
block: rm -rf /
notify: docker push *
EOF

result=$(run_hook_bash "make build")
decision=$(get_decision "$result")
reason=$(get_reason "$result")
assert_eq "custom rule: make build → allow" "allow" "$decision"
assert_contains "custom rule reason mentions allow" "allow\|放行" "$reason"

result=$(run_hook_bash "npm test")
decision=$(get_decision "$result")
assert_eq "custom rule: npm test → allow" "allow" "$decision"

result=$(run_hook_bash "rm -rf /")
decision=$(get_decision "$result")
assert_eq "custom rule: rm -rf / → ask (block)" "ask" "$decision"

# Restore conf
if [ -f "$CONF_FILE.bak" ]; then
    mv "$CONF_FILE.bak" "$CONF_FILE"
else
    rm -f "$CONF_FILE"
fi

# =============================================================================
echo -e "\n${BOLD}Dry-run Mode${NC}\n"
# =============================================================================

# Write a block rule, then test dry-run overrides it
cat > "$CONF_FILE" << 'EOF'
block: echo danger
EOF

result=$(echo '{"tool_name":"Bash","tool_input":{"command":"echo danger"}}' \
  | BARK_DRY_RUN=1 "$HOOK_SCRIPT" 2>/dev/null)
decision=$(get_decision "$result")
assert_eq "dry-run: block rule → allow" "allow" "$decision"

# Restore
rm -f "$CONF_FILE"

# =============================================================================
# Summary
# =============================================================================
echo ""
echo -e "${BOLD}Results: ${GREEN}${PASS} passed${NC}, ${RED}${FAIL} failed${NC}"
echo ""

[ "$FAIL" -gt 0 ] && exit 1
exit 0
