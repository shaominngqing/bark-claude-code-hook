#!/bin/bash
# =============================================================================
# Bark for Claude Code — One-line Installer
# =============================================================================
# Install:
#   curl -fsSL https://raw.githubusercontent.com/shaominngqing/bark-claude-code-hook/main/install.sh | bash
#
# Uninstall:
#   bark uninstall
# =============================================================================

set -euo pipefail

BARK_VERSION="2.1.4"
REPO="shaominngqing/bark-claude-code-hook"

NC='\033[0m'; BOLD='\033[1m'; DIM='\033[2m'
RED='\033[0;31m'; GREEN='\033[0;32m'; YELLOW='\033[1;33m'
C1='\033[38;5;39m'; C2='\033[38;5;45m'

ok()   { echo -e "  ${GREEN}✓${NC} $1"; }
warn() { echo -e "  ${YELLOW}⚠${NC} $1"; }
fail() { echo -e "  ${RED}✗${NC} $1"; exit 1; }
step() { echo -e "\n  ${C1}${BOLD}▸${NC} ${BOLD}$1${NC}"; }

# i18n
is_zh() { [[ "${LANG:-}${LC_ALL:-}" =~ zh ]]; }

# Gradient text
_gradient() {
    local text="$1" start="${2:-39}" count="${3:-6}"
    for ((i=0; i<${#text}; i++)); do
        local c=$(( start + (i % count) ))
        printf "\033[1;38;5;${c}m%s" "${text:$i:1}"
    done
    printf "${NC}"
}

# ── Banner (block pixel art) ──
echo ""
{
cat << 'BANNER'
 ███████████                      █████
░░███░░░░░███                    ░░███
 ░███    ░███  ██████   ████████  ░███ █████
 ░██████████  ░░░░░███ ░░███░░███ ░███░░███
 ░███░░░░░███  ███████  ░███ ░░░  ░██████░
 ░███    ░███ ███░░███  ░███      ░███░░███
 ███████████ ░░████████ █████     ████ █████
░░░░░░░░░░░   ░░░░░░░░ ░░░░░     ░░░░ ░░░░░
BANNER
} | while IFS= read -r line; do
    printf "  "
    _gradient "$line" 39 6
    printf "\n"
done
echo ""
printf "    ${DIM}🐕 AI-Powered Risk Assessment for Claude Code  v${BARK_VERSION}${NC}\n"
echo ""

# ── Detect platform ──
step "Detect platform"

OS=$(uname -s | tr '[:upper:]' '[:lower:]')
ARCH=$(uname -m)

case "$OS" in
    darwin) OS_LABEL="macOS" ;;
    linux)  OS_LABEL="Linux" ;;
    mingw*|msys*|cygwin*) OS="windows"; OS_LABEL="Windows" ;;
    *) fail "Unsupported OS: $OS" ;;
esac

case "$ARCH" in
    x86_64|amd64) ARCH="x86_64" ;;
    aarch64|arm64) ARCH="aarch64" ;;
    *) fail "Unsupported architecture: $ARCH" ;;
esac

TARGET="${OS}-${ARCH}"
ok "$OS_LABEL $ARCH"

command -v claude >/dev/null 2>&1 && ok "claude CLI" || warn "claude CLI not found (AI assessment will be unavailable)"

# ── Download binary ──
step "Download bark binary"

DOWNLOAD_URL="https://github.com/${REPO}/releases/download/v${BARK_VERSION}/bark-${TARGET}"

if [ "$OS" = "windows" ]; then
    DOWNLOAD_URL="${DOWNLOAD_URL}.exe"
fi

TMP_DIR=$(mktemp -d)
TMP_BIN="${TMP_DIR}/bark"
trap "rm -rf '$TMP_DIR'" EXIT

# Render a progress bar: _draw_progress <current> <total>
_draw_progress() {
    local cur="$1" total="$2" width=32
    local pct=0 filled=0 empty="$width"
    if [ "$total" -gt 0 ]; then
        pct=$(( cur * 100 / total ))
        filled=$(( cur * width / total ))
        empty=$(( width - filled ))
    fi
    # Size label
    local size_mb
    size_mb=$(echo "scale=1; $cur / 1048576" | bc 2>/dev/null || echo "?")
    local total_mb
    total_mb=$(echo "scale=1; $total / 1048576" | bc 2>/dev/null || echo "?")
    # Build bar
    local bar=""
    local i
    for ((i=0; i<filled; i++)); do bar="${bar}\033[38;5;39m━"; done
    for ((i=0; i<empty;  i++)); do bar="${bar}\033[2m━"; done
    printf "\r  ${bar}${NC} ${DIM}%s/%sMB${NC} %3d%%" "$size_mb" "$total_mb" "$pct" >&2
}

# Download with animated progress bar
DOWNLOADED=false
TOTAL_SIZE=0

if command -v curl >/dev/null 2>&1; then
    # Get file size via HEAD request
    TOTAL_SIZE=$(curl -fsSLI "$DOWNLOAD_URL" 2>/dev/null \
        | grep -i 'content-length' | tail -1 | tr -dc '0-9')
    TOTAL_SIZE="${TOTAL_SIZE:-0}"

    if [ "$TOTAL_SIZE" -gt 0 ]; then
        # Download in background, poll file size for progress
        curl -fsSL "$DOWNLOAD_URL" -o "$TMP_BIN" 2>/dev/null &
        DL_PID=$!
        while kill -0 "$DL_PID" 2>/dev/null; do
            if [ -f "$TMP_BIN" ]; then
                CUR_SIZE=$(wc -c < "$TMP_BIN" 2>/dev/null | tr -d ' ')
                _draw_progress "${CUR_SIZE:-0}" "$TOTAL_SIZE"
            fi
            sleep 0.15
        done
        wait "$DL_PID" && DOWNLOADED=true || true
        # Final 100% frame
        if [ "$DOWNLOADED" = true ] && [ -s "$TMP_BIN" ]; then
            _draw_progress "$TOTAL_SIZE" "$TOTAL_SIZE"
        fi
        printf "\n" >&2
    else
        # No content-length: silent download
        if curl -fsSL "$DOWNLOAD_URL" -o "$TMP_BIN" 2>/dev/null; then
            DOWNLOADED=true
        fi
    fi
elif command -v wget >/dev/null 2>&1; then
    if wget -q "$DOWNLOAD_URL" -O "$TMP_BIN" 2>/dev/null; then
        DOWNLOADED=true
    fi
fi

if [ "$DOWNLOADED" = true ] && [ -s "$TMP_BIN" ]; then
    chmod +x "$TMP_BIN"
    # Show size in success message
    FILESIZE=$(wc -c < "$TMP_BIN" | tr -d ' ')
    if [ "$FILESIZE" -ge 1048576 ]; then
        SIZE_LABEL="$(echo "scale=1; $FILESIZE / 1048576" | bc)MB"
    elif [ "$FILESIZE" -ge 1024 ]; then
        SIZE_LABEL="$(echo "scale=0; $FILESIZE / 1024" | bc)KB"
    else
        SIZE_LABEL="${FILESIZE}B"
    fi
    ok "bark v${BARK_VERSION} (${SIZE_LABEL})"
else
    # No pre-built binary available — try building from source
    warn "No pre-built binary for ${TARGET}, building from source..."

    if ! command -v cargo >/dev/null 2>&1; then
        fail "Rust toolchain required. Install: curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
    fi

    step "Build from source"

    BUILD_DIR="${TMP_DIR}/bark-src"

    if command -v git >/dev/null 2>&1; then
        git clone --depth 1 "https://github.com/${REPO}.git" "$BUILD_DIR" 2>/dev/null
    else
        # Fallback: download tarball
        curl -fsSL "https://github.com/${REPO}/archive/main.tar.gz" | tar -xz -C "$TMP_DIR"
        BUILD_DIR="${TMP_DIR}/bark-claude-code-hook-main"
    fi

    (cd "$BUILD_DIR" && cargo build --release 2>&1 | tail -3)
    cp "$BUILD_DIR/target/release/bark" "$TMP_BIN"
    chmod +x "$TMP_BIN"
    ok "Built from source"
fi

# ── Install binary ──
step "Install binary"

INSTALL_DIR=""
if [ -d /opt/homebrew/bin ] && echo "$PATH" | grep -q "/opt/homebrew/bin"; then
    INSTALL_DIR="/opt/homebrew/bin"
elif [ -d /usr/local/bin ] && [ -w /usr/local/bin ]; then
    INSTALL_DIR="/usr/local/bin"
elif [ -d "$HOME/.local/bin" ]; then
    INSTALL_DIR="$HOME/.local/bin"
else
    mkdir -p "$HOME/.local/bin"
    INSTALL_DIR="$HOME/.local/bin"
fi

if cp "$TMP_BIN" "$INSTALL_DIR/bark" 2>/dev/null; then
    ok "bark → $INSTALL_DIR/"
elif sudo cp "$TMP_BIN" "$INSTALL_DIR/bark" 2>/dev/null; then
    ok "bark → $INSTALL_DIR/ (sudo)"
else
    # Last resort: install to ~/.local/bin
    mkdir -p "$HOME/.local/bin"
    cp "$TMP_BIN" "$HOME/.local/bin/bark"
    INSTALL_DIR="$HOME/.local/bin"
    ok "bark → $INSTALL_DIR/"
    if ! echo "$PATH" | grep -q "$HOME/.local/bin"; then
        warn "Add to PATH: export PATH=\"\$HOME/.local/bin:\$PATH\""
    fi
fi

# ── Verify installation ──
if ! command -v bark >/dev/null 2>&1; then
    # Try with full path
    BARK_CMD="$INSTALL_DIR/bark"
else
    BARK_CMD="bark"
fi

# ── Run bark install (registers hook in settings.json) ──
"$BARK_CMD" install

# ── Optional: Install notification helper ──
if [ "$OS" = "darwin" ]; then
    echo ""
    if is_zh; then
        NOTIFIER_PROMPT="  ${C2}${BOLD}\xF0\x9F\x94\x94${NC} ${BOLD}安装 Bark 通知助手？${NC}"
        NOTIFIER_DESC="     自定义图标、允许/拒绝/跳过按钮、自动回退终端确认"
        NOTIFIER_OPT="     可选 — 不安装也可正常使用 bark"
    else
        NOTIFIER_PROMPT="  ${C2}${BOLD}\xF0\x9F\x94\x94${NC} ${BOLD}Install Bark Notifier?${NC}"
        NOTIFIER_DESC="     Custom icon, Allow/Deny/Skip buttons, auto-fallback to terminal"
        NOTIFIER_OPT="     Optional — bark works fine without it"
    fi
    echo -e "$NOTIFIER_PROMPT"
    echo -e "  ${DIM}$NOTIFIER_DESC${NC}"
    echo -e "  ${DIM}$NOTIFIER_OPT${NC}"
    echo ""
    read -p "  [y/N] " install_notifier < /dev/tty
    if [[ "$install_notifier" =~ ^[Yy] ]]; then
        "$BARK_CMD" install-notifier
    fi
elif [ "$OS" = "linux" ]; then
    echo ""
    if is_zh; then
        echo -e "  ${DIM}\xF0\x9F\x92\xA1 Linux 上 Bark 使用 D-Bus 通知，无需额外安装${NC}"
    else
        echo -e "  ${DIM}\xF0\x9F\x92\xA1 On Linux, Bark uses D-Bus notifications natively — no extra install needed${NC}"
    fi
fi
