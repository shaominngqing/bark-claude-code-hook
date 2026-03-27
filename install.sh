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

BARK_VERSION="2.0.1"
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

# ── Banner ──
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
    printf "  "
    _gradient "$line" 39 6
    printf "\n"
done
echo ""
printf "  ${DIM}  🐕 AI-Powered Risk Assessment for Claude Code  v${BARK_VERSION}${NC}\n"
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

# ── Check dependencies ──
step "Check environment"

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

# Try downloading from GitHub Releases
DOWNLOADED=false

if command -v curl >/dev/null 2>&1; then
    if curl -fsSL "$DOWNLOAD_URL" -o "$TMP_BIN" 2>/dev/null; then
        DOWNLOADED=true
    fi
elif command -v wget >/dev/null 2>&1; then
    if wget -q "$DOWNLOAD_URL" -O "$TMP_BIN" 2>/dev/null; then
        DOWNLOADED=true
    fi
fi

if [ "$DOWNLOADED" = true ] && [ -s "$TMP_BIN" ]; then
    chmod +x "$TMP_BIN"
    ok "Downloaded from GitHub Releases"
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
