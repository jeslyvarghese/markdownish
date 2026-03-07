use std::collections::HashMap;
use std::path::PathBuf;

use eframe::egui::{self, Color32, Context, Key, RichText};

use crate::config::{AppConfig, ColorConfig};
use crate::file_browser::{BrowserPalette, FileBrowser};
use crate::file_loader::FileLoader;
use crate::renderer::MarkdownRenderer;
use crate::ui::{SettingsPanel, ToolbarAction};

// ── Pane ─────────────────────────────────────────────────────────────────────

#[derive(Default)]
pub struct Pane {
    pub path: Option<PathBuf>,
    pub content: Option<String>,
    pub error: Option<String>,
    pub anchor_positions: HashMap<String, f32>,
    pub pending_scroll_y: Option<f32>,
}

impl Pane {
    fn title(&self) -> &str {
        self.path
            .as_deref()
            .and_then(|p| p.file_name())
            .and_then(|n| n.to_str())
            .unwrap_or("Untitled")
    }

    pub fn load(&mut self, path: PathBuf) {
        match FileLoader::load_file(&path) {
            Ok(content) => {
                self.path = Some(path);
                self.content = Some(content);
                self.error = None;
                self.anchor_positions.clear();
                self.pending_scroll_y = None;
            }
            Err(e) => {
                self.error = Some(e.to_string());
                self.content = None;
            }
        }
    }
}

// ── View mode ────────────────────────────────────────────────────────────────

/// Whether to show one pane at a time or all panes side by side.
#[derive(Debug, Clone, PartialEq)]
pub enum ViewMode {
    /// Show only the active pane
    Single,
    /// Show all panes in equal-width columns
    Split,
}

// ── App ───────────────────────────────────────────────────────────────────────

pub struct MarkdownViewerApp {
    config: AppConfig,
    panes: Vec<Pane>,
    active_pane: usize,
    view_mode: ViewMode,
    file_browser: FileBrowser,
    pending_open_for_pane: Option<usize>,
}

impl MarkdownViewerApp {
    pub fn new(cc: &eframe::CreationContext<'_>, initial_file: Option<PathBuf>) -> Self {
        let config = AppConfig::load();
        Self::apply_egui_theme(&cc.egui_ctx, &config);

        let mut browser = FileBrowser::default();
        let mut pane = Pane::default();

        if let Some(path) = initial_file {
            // Add the parent dir to the browser
            if let Some(parent) = path.parent() {
                browser.add_path(parent.to_path_buf());
            }
            pane.load(path);
        }

        Self {
            config,
            panes: vec![pane],
            active_pane: 0,
            view_mode: ViewMode::Single,
            file_browser: browser,
            pending_open_for_pane: None,
        }
    }

    fn apply_egui_theme(ctx: &Context, config: &AppConfig) {
        let bg = ColorConfig::to_egui_color32(config.colors.background);
        let text = ColorConfig::to_egui_color32(config.colors.text);
        let mut visuals = if matches!(config.theme, crate::config::ThemeMode::Light) {
            egui::Visuals::light()
        } else {
            egui::Visuals::dark()
        };
        visuals.panel_fill = bg;
        visuals.window_fill = bg;
        visuals.override_text_color = Some(text);
        ctx.set_visuals(visuals);
    }

    fn open_file_dialog_for_pane(&mut self, pane_idx: usize) {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("Markdown", &["md", "markdown", "mdown", "mkd", "mdx"])
            .add_filter("All Files", &["*"])
            .pick_file()
        {
            self.load_path_into_pane(path, pane_idx);
        }
    }

    fn open_dir_dialog(&mut self) {
        if let Some(path) = rfd::FileDialog::new().pick_folder() {
            self.file_browser.add_path(path);
        }
    }

    /// Load a file into the given pane index, adding it to recent files and browser.
    fn load_path_into_pane(&mut self, path: PathBuf, pane_idx: usize) {
        // Add parent dir to browser if not already present
        if let Some(parent) = path.parent() {
            self.file_browser.add_path(parent.to_path_buf());
        }
        if let Some(pane) = self.panes.get_mut(pane_idx) {
            pane.load(path.clone());
            self.config.add_recent_file(path);
            let _ = self.config.save();
        }
    }

    fn add_pane_with_file(&mut self, path: PathBuf) {
        let mut pane = Pane::default();
        pane.load(path.clone());
        self.config.add_recent_file(path);
        self.panes.push(pane);
        self.active_pane = self.panes.len() - 1;
        self.view_mode = ViewMode::Split;
    }

    fn add_empty_pane(&mut self) {
        self.panes.push(Pane::default());
        self.active_pane = self.panes.len() - 1;
        self.view_mode = ViewMode::Split;
    }

    fn close_pane(&mut self, idx: usize) {
        if self.panes.len() > 1 {
            self.panes.remove(idx);
            self.active_pane = self.active_pane.min(self.panes.len() - 1);
            if self.panes.len() == 1 {
                self.view_mode = ViewMode::Single;
            }
        }
    }

    fn handle_keyboard(&mut self, ctx: &Context) {
        if ctx.input(|i| i.key_pressed(Key::Comma) && i.modifiers.command) {
            self.config.show_settings = !self.config.show_settings;
        }
        if ctx.input(|i| i.key_pressed(Key::O) && i.modifiers.command) {
            self.pending_open_for_pane = Some(self.active_pane);
        }
        if ctx.input(|i| i.key_pressed(Key::T) && i.modifiers.command) {
            self.add_empty_pane();
        }
        if ctx.input(|i| i.key_pressed(Key::W) && i.modifiers.command) {
            let idx = self.active_pane;
            self.close_pane(idx);
        }
        // Ctrl+B toggle file browser
        if ctx.input(|i| i.key_pressed(Key::B) && i.modifiers.command) {
            self.file_browser.visible = !self.file_browser.visible;
        }
    }

    fn handle_dropped_files(&mut self, ctx: &Context) {
        let dropped: Vec<PathBuf> = ctx.input(|i| {
            i.raw.dropped_files
                .iter()
                .filter_map(|f| f.path.clone())
                .collect()
        });

        for (i, path) in dropped.into_iter().enumerate() {
            if path.is_dir() {
                self.file_browser.add_path(path);
            } else if FileLoader::is_markdown_file(&path) {
                if i == 0 {
                    self.load_path_into_pane(path, self.active_pane);
                } else {
                    self.add_pane_with_file(path);
                }
            }
        }
    }

    fn browser_palette(&self) -> BrowserPalette {
        let c = &self.config.colors;
        BrowserPalette {
            heading:  ColorConfig::to_egui_color32(c.heading),
            file:     ColorConfig::to_egui_color32(c.text),
            dir:      ColorConfig::to_egui_color32(c.link),
            muted:    ColorConfig::to_egui_color32(c.blockquote_border),
        }
    }

    fn show_toolbar(&mut self, ui: &mut egui::Ui) -> ToolbarAction {
        let surface   = ColorConfig::to_egui_color32(self.config.colors.surface);
        let heading_c = ColorConfig::to_egui_color32(self.config.colors.heading);
        let muted     = ColorConfig::to_egui_color32(self.config.colors.blockquote_border);

        let mut action = ToolbarAction::None;

        egui::Frame::new()
            .fill(surface)
            .inner_margin(egui::Margin::symmetric(12, 6))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    // File browser toggle
                    let browser_label = if self.file_browser.visible { "◀ Files" } else { "▶ Files" };
                    if ui.selectable_label(self.file_browser.visible,
                        RichText::new(browser_label).color(heading_c).size(13.0)).clicked()
                    {
                        self.file_browser.visible = !self.file_browser.visible;
                    }

                    ui.separator();

                    // View mode toggle (only relevant with >1 pane)
                    if self.panes.len() > 1 {
                        if ui.selectable_label(self.view_mode == ViewMode::Single,
                            RichText::new("Single").color(muted).size(12.0)).clicked()
                        {
                            self.view_mode = ViewMode::Single;
                        }
                        if ui.selectable_label(self.view_mode == ViewMode::Split,
                            RichText::new("Split").color(muted).size(12.0)).clicked()
                        {
                            self.view_mode = ViewMode::Split;
                        }
                        ui.separator();
                    }

                    // Pane tabs
                    let pane_count = self.panes.len();
                    let mut close_idx: Option<usize> = None;
                    let mut switch_idx: Option<usize> = None;

                    for i in 0..pane_count {
                        let title = self.panes[i].title().to_owned();
                        let is_active = i == self.active_pane;
                        let col = if is_active { heading_c } else { muted };

                        if ui.selectable_label(is_active,
                            RichText::new(&title).color(col).size(13.0)).clicked()
                        {
                            switch_idx = Some(i);
                        }
                        if pane_count > 1 {
                            if ui.small_button(RichText::new("✕").size(10.0).color(muted)).clicked() {
                                close_idx = Some(i);
                            }
                            ui.add_space(4.0);
                        }
                    }

                    if let Some(i) = switch_idx { self.active_pane = i; }
                    if let Some(i) = close_idx  { self.close_pane(i); }

                    // Right-side controls
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let sl = if self.config.show_settings { "✕ Settings" } else { "⚙ Settings" };
                        if ui.button(sl).clicked() {
                            self.config.show_settings = !self.config.show_settings;
                        }
                        if ui.button("Open Folder…").clicked() {
                            action = ToolbarAction::OpenFolder;
                        }
                        if ui.button("Open File…").clicked() {
                            action = ToolbarAction::OpenFile;
                        }
                        if ui.button("+ Pane").clicked() {
                            self.add_empty_pane();
                        }
                    });
                });
            });

        action
    }
}

impl eframe::App for MarkdownViewerApp {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        self.handle_keyboard(ctx);
        self.handle_dropped_files(ctx);
        Self::apply_egui_theme(ctx, &self.config);

        // File-dialog deferred until outside input closure
        if let Some(pane_idx) = self.pending_open_for_pane.take() {
            self.open_file_dialog_for_pane(pane_idx);
        }

        let bg = ColorConfig::to_egui_color32(self.config.colors.background);
        let surface = ColorConfig::to_egui_color32(self.config.colors.surface);

        // ── Toolbar ──────────────────────────────────────────────────────────
        egui::TopBottomPanel::top("toolbar").show(ctx, |ui| {
            let action = self.show_toolbar(ui);
            match action {
                ToolbarAction::OpenFile => {
                    let idx = self.active_pane;
                    self.open_file_dialog_for_pane(idx);
                }
                ToolbarAction::OpenFolder => {
                    self.open_dir_dialog();
                }
                ToolbarAction::None => {}
            }
        });

        // ── Settings sidebar ─────────────────────────────────────────────────
        if self.config.show_settings {
            egui::SidePanel::right("settings")
                .min_width(280.0)
                .max_width(380.0)
                .show(ctx, |ui| {
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        SettingsPanel::show(ui, &mut self.config);
                        ui.add_space(20.0);
                        if ui.button("Save Settings").clicked() {
                            let _ = self.config.save();
                        }
                    });
                });
        }

        // ── File browser side panel ──────────────────────────────────────────
        if self.file_browser.visible {
            let palette = self.browser_palette();
            let mut open_in_active: Option<PathBuf> = None;
            let mut open_in_new: Option<PathBuf> = None;

            egui::SidePanel::left("file_browser")
                .min_width(180.0)
                .max_width(320.0)
                .default_width(220.0)
                .show(ctx, |ui| {
                    egui::Frame::new()
                        .fill(surface)
                        .inner_margin(egui::Margin::same(0i8))
                        .show(ui, |ui| {
                            // Browser header
                            egui::Frame::new()
                                .fill(surface)
                                .inner_margin(egui::Margin { left: 10i8, right: 6i8, top: 8i8, bottom: 8i8 })
                                .show(ui, |ui| {
                                    ui.horizontal(|ui| {
                                        ui.label(
                                            RichText::new("Files")
                                                .size(12.0)
                                                .color(palette.muted)
                                                .strong(),
                                        );
                                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                            if ui.small_button(
                                                RichText::new("+ Folder").size(10.0).color(palette.dir)
                                            ).clicked() {
                                                open_in_active = Some(PathBuf::from("__open_dir__"));
                                            }
                                            if ui.small_button(
                                                RichText::new("+ File").size(10.0).color(palette.file)
                                            ).clicked() {
                                                open_in_active = Some(PathBuf::from("__open_file__"));
                                            }
                                        });
                                    });
                                });

                            ui.separator();

                            egui::ScrollArea::vertical().show(ui, |ui| {
                                ui.add_space(4.0);
                                // Each item in the browser is clicked → decide: single click opens in active,
                                // Ctrl+click opens in new pane
                                if let Some(path) = self.file_browser.show(ui, &palette) {
                                    let ctrl_held = ui.input(|i| i.modifiers.command);
                                    if ctrl_held {
                                        open_in_new = Some(path);
                                    } else {
                                        open_in_active = Some(path);
                                    }
                                }
                                ui.add_space(8.0);
                            });
                        });
                });

            // Act on browser clicks (outside the panel closure to avoid borrow conflict)
            if let Some(path) = open_in_active {
                if path == PathBuf::from("__open_dir__") {
                    self.open_dir_dialog();
                } else if path == PathBuf::from("__open_file__") {
                    let idx = self.active_pane;
                    self.open_file_dialog_for_pane(idx);
                } else {
                    let idx = self.active_pane;
                    self.load_path_into_pane(path, idx);
                }
            }
            if let Some(path) = open_in_new {
                self.add_pane_with_file(path);
            }
        }

        // ── Central content area ─────────────────────────────────────────────
        egui::CentralPanel::default()
            .frame(egui::Frame::new().fill(bg))
            .show(ctx, |ui| {
                let view_mode = self.view_mode.clone();
                let active = self.active_pane;

                match view_mode {
                    ViewMode::Single => {
                        // Show only the active pane
                        let pending = self.panes[active].pending_scroll_y.take();
                        let content = self.panes[active].content.clone();
                        let error   = self.panes[active].error.clone();
                        let path    = self.panes[active].path.clone();

                        let ro = Self::show_pane_content(
                            ui, bg, &content, &error, &path, pending, true, &self.config,
                        );
                        if let Some(ro) = ro {
                            Self::apply_render_output(&mut self.panes[active], ro);
                        }
                    }
                    ViewMode::Split => {
                        let pane_count = self.panes.len();

                        // Extract mutable state before columns closure
                        let states: Vec<_> = self.panes.iter_mut().map(|p| {
                            (p.pending_scroll_y.take(), p.content.clone(), p.error.clone(), p.path.clone())
                        }).collect();

                        let mut render_outputs: Vec<Option<crate::renderer::RenderOutput>> =
                            (0..pane_count).map(|_| None).collect();

                        let config = &self.config;

                        ui.columns(pane_count, |cols| {
                            for (i, col) in cols.iter_mut().enumerate() {
                                let (pending, ref content, ref error, ref path) = states[i];
                                let is_active = i == active;

                                if i > 0 {
                                    let r = col.available_rect_before_wrap();
                                    col.painter().line_segment(
                                        [egui::pos2(r.left(), r.top()), egui::pos2(r.left(), r.bottom())],
                                        egui::Stroke::new(1.0, ColorConfig::to_egui_color32([50, 52, 70])),
                                    );
                                }

                                let ro = Self::show_pane_content(
                                    col, bg, content, error, path, pending, is_active, config,
                                );
                                render_outputs[i] = ro;
                            }
                        });

                        for (i, ro) in render_outputs.into_iter().enumerate() {
                            if let Some(ro) = ro {
                                Self::apply_render_output(&mut self.panes[i], ro);
                            }
                        }
                    }
                }
            });
    }
}

impl MarkdownViewerApp {
    fn show_pane_content(
        ui: &mut egui::Ui,
        bg: Color32,
        content: &Option<String>,
        error: &Option<String>,
        path: &Option<PathBuf>,
        pending_scroll_y: Option<f32>,
        is_active: bool,
        config: &AppConfig,
    ) -> Option<crate::renderer::RenderOutput> {
        if is_active {
            let rect = ui.available_rect_before_wrap();
            let accent = ColorConfig::to_egui_color32(config.colors.heading);
            ui.painter().line_segment(
                [egui::pos2(rect.left(), rect.top()), egui::pos2(rect.right(), rect.top())],
                egui::Stroke::new(2.0, accent),
            );
        }

        egui::Frame::new().fill(bg).show(ui, |ui| {
            if let Some(err) = error {
                ui.centered_and_justified(|ui| {
                    ui.label(
                        RichText::new(format!("Error: {}", err))
                            .color(Color32::from_rgb(243, 139, 168))
                            .size(14.0),
                    );
                });
                None
            } else if let Some(content) = content {
                let max_width = config.content_width.max_pixels();

                let mut scroll_area = egui::ScrollArea::vertical()
                    .id_salt(format!("scroll_{:?}", path))
                    .auto_shrink([false; 2]);

                if let Some(y) = pending_scroll_y {
                    scroll_area = scroll_area.vertical_scroll_offset(y);
                }

                let inner_out = scroll_area.show(ui, |ui| {
                    let available_w = ui.available_width();
                    let content_w = max_width.min(available_w);
                    let h_pad = ((available_w - content_w) / 2.0).max(0.0);

                    let frame_margin = egui::Margin {
                        left:   (h_pad + 32.0).min(127.0) as i8,
                        right:  (h_pad + 32.0).min(127.0) as i8,
                        top:    24i8,
                        bottom: 48i8,
                    };

                    egui::Frame::new()
                        .inner_margin(frame_margin)
                        .show(ui, |ui| {
                            let wrap_width = (content_w - 64.0).max(200.0);
                            ui.set_max_width(wrap_width);
                            let renderer = MarkdownRenderer::new(config);
                            renderer.render(ui, content)
                        })
                        .inner
                });

                Some(inner_out.inner)
            } else {
                ui.centered_and_justified(|ui| {
                    ui.vertical_centered(|ui| {
                        let muted = ColorConfig::to_egui_color32(config.colors.blockquote_border);
                        let tc    = ColorConfig::to_egui_color32(config.colors.text);
                        ui.add_space(60.0);
                        ui.label(RichText::new("Drop a file here").size(16.0).color(tc));
                        ui.add_space(6.0);
                        ui.label(RichText::new("or press Ctrl+O").size(12.0).color(muted));
                    });
                });
                None
            }
        }).inner
    }

    fn apply_render_output(pane: &mut Pane, ro: crate::renderer::RenderOutput) {
        pane.anchor_positions = ro.anchor_positions;

        if let Some(anchor) = ro.anchor_clicked {
            if let Some(&y) = pane.anchor_positions.get(&anchor) {
                pane.pending_scroll_y = Some((y - 16.0).max(0.0));
            }
        }
    }
}
