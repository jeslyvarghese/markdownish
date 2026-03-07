#!/usr/bin/env bash
# Build MarkdownViewer and package it into a distributable DMG
# Usage: bash scripts/make_dmg.sh [--skip-build]
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(dirname "$SCRIPT_DIR")"
DIST="$ROOT/dist"
APP_NAME="MarkdownViewer"
APP_BUNDLE="$DIST/$APP_NAME.app"
DMG_NAME="Markdownish.dmg"
DMG_PATH="$DIST/$DMG_NAME"
VOLUME_NAME="Markdownish"

# ── 1. Build universal app bundle ────────────────────────────────────────────
if [[ "${1:-}" != "--skip-build" ]]; then
    echo "==> Building universal app bundle…"
    bash "$SCRIPT_DIR/build_app.sh" --universal
fi

[[ -d "$APP_BUNDLE" ]] || { echo "ERROR: $APP_BUNDLE not found. Run without --skip-build."; exit 1; }

# ── 2. Prepare staging area ───────────────────────────────────────────────────
echo "==> Staging DMG contents…"
STAGING="$(mktemp -d)"
trap 'rm -rf "$STAGING"' EXIT

cp -R "$APP_BUNDLE" "$STAGING/$APP_NAME.app"
ln -s /Applications "$STAGING/Applications"

# Copy volume icon (.VolumeIcon.icns) so the DMG has a custom icon
ICNS_SRC="$ROOT/assets/MarkdownViewer.icns"
if [[ -f "$ICNS_SRC" ]]; then
    cp "$ICNS_SRC" "$STAGING/.VolumeIcon.icns"
    # Tag the staging dir so macOS picks up the custom icon
    /usr/bin/SetFile -a C "$STAGING" 2>/dev/null || true
fi

# ── 3. Create compressed DMG ─────────────────────────────────────────────────
echo "==> Creating DMG…"
rm -f "$DMG_PATH"

hdiutil create \
    -volname "$VOLUME_NAME" \
    -srcfolder "$STAGING" \
    -ov \
    -format UDZO \
    -imagekey zlib-level=9 \
    -fs HFS+ \
    "$DMG_PATH"

# ── 4. Ad-hoc sign the DMG ───────────────────────────────────────────────────
echo "==> Signing DMG (ad-hoc)…"
codesign --force --sign - "$DMG_PATH"

# ── 5. Summary ────────────────────────────────────────────────────────────────
SIZE=$(du -sh "$DMG_PATH" | cut -f1)
echo ""
echo "✓ DMG ready → $DMG_PATH  ($SIZE)"
echo ""
echo "To install: open \"$DMG_PATH\""
echo "            drag Markdownish → Applications"
