#!/usr/bin/env bash
# Build the Rust core library for macOS (Apple Silicon + Intel universal)
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(dirname "$SCRIPT_DIR")"
CORE_DIR="$ROOT_DIR/core"

# Detect host architecture for cargo
HOST_ARCH=$(uname -m)

echo "==> Building markdown-core (release)…"

cd "$CORE_DIR"

# Build for arm64 (Apple Silicon)
echo "    Building arm64…"
cargo build --release --target aarch64-apple-darwin

# Build for x86_64 (Intel)
echo "    Building x86_64…"
cargo build --release --target x86_64-apple-darwin

# Create universal binary with lipo
OUTDIR="$CORE_DIR/target/universal"
mkdir -p "$OUTDIR"
echo "    Creating universal library…"
lipo -create \
    "$CORE_DIR/target/aarch64-apple-darwin/release/libmarkdown_core.a" \
    "$CORE_DIR/target/x86_64-apple-darwin/release/libmarkdown_core.a" \
    -output "$OUTDIR/libmarkdown_core.a"

echo "==> libmarkdown_core.a → $OUTDIR/libmarkdown_core.a"

# Generate C header if cbindgen is available
if command -v cbindgen &>/dev/null; then
    echo "==> Generating C header…"
    cbindgen --config "$CORE_DIR/cbindgen.toml" --crate markdown-core --output "$ROOT_DIR/viewer/Bridge/MarkdownCore.h"
    echo "==> Header → viewer/Bridge/MarkdownCore.h"
else
    echo "    (cbindgen not found — skipping header generation, using pre-written header)"
fi

echo ""
echo "Done. Library available at:"
echo "  $OUTDIR/libmarkdown_core.a"
