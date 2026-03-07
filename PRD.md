# Markdown Viewer — Product Requirements Document

**Version**: 1.0
**Status**: In Progress
**Last Updated**: 2026-03-05

---

## Overview

A native Rust desktop application that renders Markdown files with a beautiful, customizable appearance. Built using the `egui` immediate-mode GUI framework with `eframe` for cross-platform windowing.

---

## Goals

- Render Markdown files accurately (CommonMark spec)
- Allow users to customize appearance (fonts, colors, theme)
- Persist user preferences across sessions
- Fast, lightweight, native feel

---

## Technology Stack

| Component        | Choice                          | Rationale                                  |
|------------------|---------------------------------|--------------------------------------------|
| GUI Framework    | `egui` + `eframe`               | Pure Rust, immediate mode, cross-platform  |
| Markdown Parser  | `pulldown-cmark`                | Fast, CommonMark compliant                 |
| Config Storage   | `serde` + `toml`                | Human-readable config files                |
| File I/O         | `std::fs`                       | Standard library                           |
| Syntax Highlight | `syntect`                       | Battle-tested Rust syntax highlighter      |

---

## Milestones

### Step 1: Project Scaffolding & Tests [x] DONE
- Initialize Cargo workspace
- Add dependencies: `eframe`, `egui`, `pulldown-cmark`, `serde`, `toml`, `syntect`
- Create project structure
- Write unit tests for markdown parsing utilities
- **Done when**: `cargo test` passes with at least 5 passing tests

### Step 2: Core Markdown Renderer [x] DONE
- Implement `MarkdownRenderer` struct that converts `pulldown-cmark` events to `egui` widgets
- Support: headings (H1-H6), paragraphs, bold, italic, code spans, code blocks, blockquotes, lists (ordered/unordered), horizontal rules, links, images
- Write tests for each block/inline element
- **Done when**: All markdown element tests pass; basic window opens with rendered content

### Step 3: File Loading [x] DONE
- Drag-and-drop file support
- File open dialog (native)
- Command-line argument: `markdown-viewer path/to/file.md`
- Recent files list (stored in config)
- **Done when**: Can open `.md` files via all three methods; recent files persist

### Step 4: Appearance Customization UI [x] DONE
- Settings panel (sidebar or modal)
- Customizable properties:
  - Theme: Light / Dark / Custom
  - Font family: Monospace / Proportional / System
  - Font size: slider (10–32pt)
  - Line spacing: slider
  - Content width: narrow / medium / wide / full
  - Heading color, body text color, link color, code background
  - Syntax highlight theme (for code blocks)
- Settings persisted to `~/.config/markdown-viewer/config.toml`
- **Done when**: All settings visible and persistent across restarts

### Step 5: Polish & UX [x] DONE
- Keyboard shortcuts (Ctrl+O open, Ctrl+, settings, Ctrl+W close)
- Scroll position memory per file
- Window title shows filename
- Empty state (drag a file here)
- Loading spinner for large files
- Error states (file not found, parse error)
- About dialog
- **Done when**: App feels complete and production-quality

---

## Project Structure

```
markdown-viewer/
├── Cargo.toml
├── PRD.md
├── src/
│   ├── main.rs              # Entry point, eframe setup
│   ├── app.rs               # Main app state & event loop
│   ├── renderer.rs          # Markdown → egui rendering
│   ├── config.rs            # AppConfig, persistence
│   ├── file_loader.rs       # File I/O, recent files
│   └── ui/
│       ├── mod.rs
│       ├── settings_panel.rs
│       └── toolbar.rs
└── tests/
    ├── renderer_tests.rs
    ├── config_tests.rs
    └── file_loader_tests.rs
```

---

## Appearance Defaults

```toml
[theme]
mode = "dark"
font_size = 16
line_spacing = 1.4
content_width = "medium"

[colors]
background = "#1e1e2e"
text = "#cdd6f4"
heading = "#89b4fa"
link = "#89dceb"
code_bg = "#313244"
blockquote_border = "#6c7086"

[syntax]
highlight_theme = "base16-ocean.dark"
```

---

## Non-Goals (v1)

- Real-time collaborative editing
- Markdown editing / WYSIWYG
- Plugin system
- Mobile support
- PDF export

---

## Completion Checklist

- [x] Step 1: Scaffolding & Tests — 41 passing tests
- [x] Step 2: Core Markdown Renderer — H1-H6, paragraphs, bold/italic, code, blockquotes, lists, HR, links
- [x] Step 3: File Loading — file dialog, CLI arg, drag-and-drop, recent files, 50MB limit, UTF-8 check
- [x] Step 4: Appearance Customization — theme toggle, font size, line spacing, content width, per-color pickers, heading scales
- [x] Step 5: Polish & UX — empty state, error state, keyboard shortcuts (Ctrl+O, Ctrl+,), settings panel, config persistence
