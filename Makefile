.PHONY: all core app app-universal dmg dmg-only icon clean test

CORE_DIR   := core
VIEWER_DIR := viewer
LIB_UNIVERSAL := core/target/universal/libmarkdown_core.a

# ── Default: build app ────────────────────────────────────────────────────────
all: app

# ── App icon ──────────────────────────────────────────────────────────────────
icon:
	@swift scripts/gen_icon.swift /tmp/md_iconset_tmp
	@mkdir -p /tmp/MarkdownViewer_iconset_tmp
	@cp /tmp/md_iconset_tmp/icon_16x16.png    /tmp/MarkdownViewer_iconset_tmp/
	@cp /tmp/md_iconset_tmp/icon_16x16@2x.png /tmp/MarkdownViewer_iconset_tmp/
	@cp /tmp/md_iconset_tmp/icon_32x32.png    /tmp/MarkdownViewer_iconset_tmp/
	@cp /tmp/md_iconset_tmp/icon_32x32@2x.png /tmp/MarkdownViewer_iconset_tmp/
	@cp /tmp/md_iconset_tmp/icon_128x128.png    /tmp/MarkdownViewer_iconset_tmp/
	@cp /tmp/md_iconset_tmp/icon_128x128@2x.png /tmp/MarkdownViewer_iconset_tmp/
	@cp /tmp/md_iconset_tmp/icon_256x256.png    /tmp/MarkdownViewer_iconset_tmp/
	@cp /tmp/md_iconset_tmp/icon_256x256@2x.png /tmp/MarkdownViewer_iconset_tmp/
	@cp /tmp/md_iconset_tmp/icon_512x512.png    /tmp/MarkdownViewer_iconset_tmp/
	@cp /tmp/md_iconset_tmp/icon_512x512@2x.png /tmp/MarkdownViewer_iconset_tmp/
	@mkdir -p assets
	@iconutil -c icns /tmp/MarkdownViewer_iconset_tmp -o assets/MarkdownViewer.icns
	@cp /tmp/md_iconset_tmp/icon_256x256.png assets/icon-256.png
	@cp /tmp/md_iconset_tmp/icon_512x512.png assets/icon-512.png
	@cp /tmp/md_iconset_tmp/icon_32x32.png   assets/favicon-32.png
	@cp assets/favicon-32.png landing/favicon.png
	@cp assets/icon-256.png   landing/app-icon.png
	@rm -rf /tmp/md_iconset_tmp /tmp/MarkdownViewer_iconset_tmp
	@echo "Icon generated → assets/MarkdownViewer.icns"

# ── App bundle (Rust + Swift + bundle + sign) ─────────────────────────────────
app:
	@bash scripts/build_app.sh

# Universal app bundle (arm64 + x86_64)
app-universal:
	@bash scripts/build_app.sh --universal

# ── DMG installer ─────────────────────────────────────────────────────────────
dmg:
	@bash scripts/make_dmg.sh

# DMG without rebuilding (if dist/MarkdownViewer.app is already fresh)
dmg-only:
	@bash scripts/make_dmg.sh --skip-build

# ── Rust library ──────────────────────────────────────────────────────────────
core:
	@bash scripts/build_core.sh

# Quick single-arch build for the current machine (faster in development)
core-fast:
	cd $(CORE_DIR) && cargo build --release
	@echo "Single-arch lib: $(CORE_DIR)/target/release/libmarkdown_core.a"

# ── Tests ─────────────────────────────────────────────────────────────────────
test:
	cd $(CORE_DIR) && cargo test

# ── Clean ─────────────────────────────────────────────────────────────────────
clean:
	cd $(CORE_DIR) && cargo clean

# ── Xcode setup instructions ──────────────────────────────────────────────────
xcode-setup:
	@echo ""
	@echo "Xcode project setup:"
	@echo "  1. File → New → Project → macOS App"
	@echo "  2. Product Name: MarkdownViewer"
	@echo "  3. Interface: SwiftUI, Language: Swift"
	@echo "  4. Add all *.swift from viewer/ to the target"
	@echo "  5. Build Phases → Link Binary With Libraries → Add $(LIB_UNIVERSAL)"
	@echo "  6. Build Settings:"
	@echo "       SWIFT_OBJC_BRIDGING_HEADER = viewer/Bridge/MarkdownViewer-Bridging-Header.h"
	@echo "       OTHER_LDFLAGS = -lmarkdown_core"
	@echo "       LIBRARY_SEARCH_PATHS = \$$(PROJECT_DIR)/core/target/universal"
	@echo "  7. Add viewer/Renderer/Shaders.metal to the target"
	@echo ""
