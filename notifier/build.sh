#!/bin/bash
# Build BarkNotifier.app without Xcode — uses swiftc directly.
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
SRC_DIR="$SCRIPT_DIR/BarkNotifier"
BUILD_DIR="$SCRIPT_DIR/build"
APP_DIR="$BUILD_DIR/BarkNotifier.app"
CONTENTS="$APP_DIR/Contents"
MACOS="$CONTENTS/MacOS"
RESOURCES="$CONTENTS/Resources"

echo "Building BarkNotifier.app..."

# Clean
rm -rf "$BUILD_DIR"
mkdir -p "$MACOS" "$RESOURCES"

# Compile Swift sources
SWIFT_FILES=(
    "$SRC_DIR/main.swift"
    "$SRC_DIR/AppDelegate.swift"
    "$SRC_DIR/SocketServer.swift"
    "$SRC_DIR/Theme.swift"
    "$SRC_DIR/SQLiteReader.swift"
    "$SRC_DIR/DataStore.swift"
    "$SRC_DIR/TabBarView.swift"
    "$SRC_DIR/PopoverViewController.swift"
    "$SRC_DIR/DashboardTabView.swift"
    "$SRC_DIR/ActivityTabView.swift"
    "$SRC_DIR/RulesTabView.swift"
    "$SRC_DIR/SettingsTabView.swift"
)

swiftc \
    -o "$MACOS/BarkNotifier" \
    -target "$(uname -m)-apple-macosx12.0" \
    -import-objc-header "$SRC_DIR/BridgingHeader.h" \
    -framework Cocoa \
    -framework UserNotifications \
    -lsqlite3 \
    "${SWIFT_FILES[@]}"

# Copy Info.plist
cp "$SRC_DIR/Info.plist" "$CONTENTS/Info.plist"

# Copy menu bar icon resources
if [ -d "$SRC_DIR/Resources" ]; then
    cp "$SRC_DIR/Resources/"*.png "$RESOURCES/" 2>/dev/null || true
fi

# Copy icon if it exists
if [ -f "$SRC_DIR/Assets.xcassets/AppIcon.appiconset/icon_512x512.png" ]; then
    # Convert PNG to icns using sips + iconutil
    ICONSET_DIR="$BUILD_DIR/BarkNotifier.iconset"
    mkdir -p "$ICONSET_DIR"

    ICON_SRC="$SRC_DIR/Assets.xcassets/AppIcon.appiconset/icon_512x512.png"

    sips -z 16 16     "$ICON_SRC" --out "$ICONSET_DIR/icon_16x16.png"      2>/dev/null
    sips -z 32 32     "$ICON_SRC" --out "$ICONSET_DIR/icon_16x16@2x.png"   2>/dev/null
    sips -z 32 32     "$ICON_SRC" --out "$ICONSET_DIR/icon_32x32.png"      2>/dev/null
    sips -z 64 64     "$ICON_SRC" --out "$ICONSET_DIR/icon_32x32@2x.png"   2>/dev/null
    sips -z 128 128   "$ICON_SRC" --out "$ICONSET_DIR/icon_128x128.png"    2>/dev/null
    sips -z 256 256   "$ICON_SRC" --out "$ICONSET_DIR/icon_128x128@2x.png" 2>/dev/null
    sips -z 256 256   "$ICON_SRC" --out "$ICONSET_DIR/icon_256x256.png"    2>/dev/null
    sips -z 512 512   "$ICON_SRC" --out "$ICONSET_DIR/icon_256x256@2x.png" 2>/dev/null
    cp "$ICON_SRC"               "$ICONSET_DIR/icon_512x512.png"
    sips -z 1024 1024 "$ICON_SRC" --out "$ICONSET_DIR/icon_512x512@2x.png" 2>/dev/null

    iconutil -c icns "$ICONSET_DIR" -o "$RESOURCES/AppIcon.icns"
    rm -rf "$ICONSET_DIR"

    # Add icon reference to Info.plist
    /usr/libexec/PlistBuddy -c "Add :CFBundleIconFile string AppIcon" "$CONTENTS/Info.plist" 2>/dev/null || \
    /usr/libexec/PlistBuddy -c "Set :CFBundleIconFile AppIcon" "$CONTENTS/Info.plist"
fi

# Ad-hoc code sign (required for UNUserNotificationCenter permission prompt)
codesign --force --deep --sign - "$APP_DIR"

echo "Built: $APP_DIR"
echo ""
echo "To install:"
echo "  cp -r $APP_DIR ~/Applications/"
echo "  xattr -cr ~/Applications/BarkNotifier.app"
