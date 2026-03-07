#!/usr/bin/env bash
# Build MarkdownViewer.app
# Usage: bash scripts/build_app.sh [--universal]
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(dirname "$SCRIPT_DIR")"
DIST="$ROOT/dist"
APP_NAME="MarkdownViewer"
APP_BUNDLE="$DIST/$APP_NAME.app"

cd "$ROOT"
mkdir -p "$DIST"

# ── 1. Rust library ────────────────────────────────────────────────────────────
if [[ "${1:-}" == "--universal" ]]; then
    echo "==> Building Rust core (universal)…"
    bash "$SCRIPT_DIR/build_core.sh"
    # Package.swift links from core/target/release — stage the universal lib there
    cp "core/target/universal/libmarkdown_core.a" "core/target/release/libmarkdown_core.a"
    UNIVERSAL=1
else
    echo "==> Building Rust core (native)…"
    cargo build --release --manifest-path core/Cargo.toml
    UNIVERSAL=0
fi

# ── 2. Swift app ──────────────────────────────────────────────────────────────
echo "==> Building Swift app…"
if [[ "$UNIVERSAL" == "1" ]]; then
    # Build for each arch separately then lipo
    swift build -c release --arch arm64   2>&1 | grep -Ev "^(warning:|Build complete)"
    swift build -c release --arch x86_64  2>&1 | grep -Ev "^(warning:|Build complete)"

    ARM64_BIN=".build/arm64-apple-macosx/release/$APP_NAME"
    X86_BIN=".build/x86_64-apple-macosx/release/$APP_NAME"
    [[ -f "$ARM64_BIN" ]]  || { echo "ERROR: arm64 binary not found at $ARM64_BIN"; exit 1; }
    [[ -f "$X86_BIN" ]]   || { echo "ERROR: x86_64 binary not found at $X86_BIN"; exit 1; }

    BINARY="$DIST/${APP_NAME}_universal"
    echo "==> Combining arm64 + x86_64 with lipo…"
    lipo -create "$ARM64_BIN" "$X86_BIN" -output "$BINARY"
else
    swift build -c release 2>&1 | grep -v "^warning:"
    BINARY=".build/release/$APP_NAME"
fi

[[ -f "$BINARY" ]] || { echo "ERROR: binary not found at $BINARY"; exit 1; }

# ── 3. App bundle ─────────────────────────────────────────────────────────────
echo "==> Creating app bundle…"
rm -rf "$APP_BUNDLE"
mkdir -p "$APP_BUNDLE/Contents/MacOS"
mkdir -p "$APP_BUNDLE/Contents/Resources"

cp "$BINARY" "$APP_BUNDLE/Contents/MacOS/$APP_NAME"

# App icon
ICNS_SRC="$ROOT/assets/MarkdownViewer.icns"
if [[ -f "$ICNS_SRC" ]]; then
    cp "$ICNS_SRC" "$APP_BUNDLE/Contents/Resources/AppIcon.icns"
    echo "==> Copied app icon."
else
    echo "WARNING: No app icon found at $ICNS_SRC — run: swift scripts/gen_icon.swift assets/iconset"
fi

# Extract version from Cargo.toml in core
VERSION=$(grep '^version' core/Cargo.toml | head -1 | sed 's/.*"\(.*\)".*/\1/' || echo "1.0")
BUILD=$(date +%Y%m%d)

cat > "$APP_BUNDLE/Contents/Info.plist" << PLIST
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN"
  "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleExecutable</key>          <string>MarkdownViewer</string>
    <key>CFBundleIconFile</key>            <string>AppIcon</string>
    <key>CFBundleIdentifier</key>          <string>com.local.markdownish</string>
    <key>CFBundleName</key>                <string>Markdownish</string>
    <key>CFBundleDisplayName</key>         <string>Markdownish</string>
    <key>CFBundlePackageType</key>         <string>APPL</string>
    <key>CFBundleVersion</key>             <string>$BUILD</string>
    <key>CFBundleShortVersionString</key>  <string>$VERSION</string>
    <key>LSMinimumSystemVersion</key>      <string>14.0</string>
    <key>NSPrincipalClass</key>            <string>NSApplication</string>
    <key>NSHighResolutionCapable</key>     <true/>
    <key>NSSupportsAutomaticTermination</key> <false/>
    <key>CFBundleDocumentTypes</key>
    <array>
        <dict>
            <key>CFBundleTypeName</key>       <string>Markdown Document</string>
            <key>CFBundleTypeExtensions</key>
            <array>
                <string>md</string>
                <string>markdown</string>
                <string>mdown</string>
                <string>mkd</string>
            </array>
            <key>CFBundleTypeRole</key>       <string>Viewer</string>
            <key>LSHandlerRank</key>          <string>Alternate</string>
        </dict>
    </array>
    <key>UTExportedTypeDeclarations</key>  <array/>
    <key>LSApplicationCategoryType</key>   <string>public.app-category.productivity</string>
</dict>
</plist>
PLIST

# ── 4. Ad-hoc code sign ───────────────────────────────────────────────────────
echo "==> Code signing (ad-hoc)…"
codesign --force --deep --sign - "$APP_BUNDLE"

echo ""
echo "Done → $APP_BUNDLE"
echo "Launch with: open \"$APP_BUNDLE\""
