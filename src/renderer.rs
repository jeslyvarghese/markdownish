use std::collections::HashMap;
use std::sync::Arc;

use egui::{
    Color32, FontId, RichText, Stroke, Ui,
    text::{LayoutJob, TextFormat},
};
use egui_extras::{Column, TableBuilder};

use crate::config::{AppConfig, ColorConfig};
use crate::parser::{Block, ColumnAlign, Inline, ListItem, MarkdownParser};

/// Result of rendering — carries back any anchor that was clicked
/// and the y-positions of each heading anchor in the scroll content.
#[derive(Default)]
pub struct RenderOutput {
    /// Anchor slug of any in-document link the user clicked this frame.
    pub anchor_clicked: Option<String>,
    /// Map of anchor slug → y offset from the top of the scroll content.
    pub anchor_positions: HashMap<String, f32>,
}

pub struct MarkdownRenderer<'a> {
    config: &'a AppConfig,
}

// ── colours that get computed once per render call ─────────────────────────
struct Palette {
    text: Color32,
    text_strong: Color32,
    link: Color32,
    code_fg: Color32,
    code_bg: Color32,
    heading: Color32,
    blockquote_border: Color32,
    blockquote_bg: Color32,
    hr: Color32,
}

impl Palette {
    fn from_config(config: &AppConfig) -> Self {
        let c = &config.colors;
        Self {
            text:             ColorConfig::to_egui_color32(c.text),
            // "strong" = slightly lighter than text for dark themes
            text_strong:      Color32::WHITE,
            link:             ColorConfig::to_egui_color32(c.link),
            code_fg:          ColorConfig::to_egui_color32(c.code_text),
            code_bg:          ColorConfig::to_egui_color32(c.code_bg),
            heading:          ColorConfig::to_egui_color32(c.heading),
            blockquote_border:ColorConfig::to_egui_color32(c.blockquote_border),
            blockquote_bg:    ColorConfig::to_egui_color32(c.blockquote_bg),
            hr:               ColorConfig::to_egui_color32(c.hr_color),
        }
    }
}

impl<'a> MarkdownRenderer<'a> {
    pub fn new(config: &'a AppConfig) -> Self {
        Self { config }
    }

    /// Render the full markdown string.  Returns `RenderOutput` with anchor info.
    pub fn render(&self, ui: &mut Ui, markdown: &str) -> RenderOutput {
        let blocks = MarkdownParser::parse(markdown);
        let pal = Palette::from_config(self.config);
        let mut out = RenderOutput::default();
        // y-origin of the scroll content at the time render() was called
        let origin_y = ui.cursor().top();
        self.render_blocks(ui, &blocks, 0, &pal, &mut out, origin_y);
        out
    }

    fn render_blocks(
        &self,
        ui: &mut Ui,
        blocks: &[Block],
        depth: usize,
        pal: &Palette,
        out: &mut RenderOutput,
        origin_y: f32,
    ) {
        for block in blocks {
            self.render_block(ui, block, depth, pal, out, origin_y);
            ui.add_space(self.config.font_size * 0.4);
        }
    }

    fn render_block(
        &self,
        ui: &mut Ui,
        block: &Block,
        depth: usize,
        pal: &Palette,
        out: &mut RenderOutput,
        origin_y: f32,
    ) {
        match block {
            Block::Heading { level, content, anchor } => {
                // Record y-position of this heading for anchor navigation
                let y = ui.cursor().top() - origin_y;
                out.anchor_positions.insert(anchor.clone(), y);

                self.render_heading(ui, *level, content, anchor, pal, out);
            }
            Block::Paragraph(inlines) => {
                self.render_paragraph(ui, inlines, pal, out);
            }
            Block::CodeBlock { language, code } => {
                self.render_code_block(ui, language.as_deref(), code, pal);
            }
            Block::BlockQuote(inner) => {
                self.render_blockquote(ui, inner, depth, pal, out, origin_y);
            }
            Block::BulletList(items) => {
                self.render_bullet_list(ui, items, depth, pal, out, origin_y);
            }
            Block::OrderedList { start, items } => {
                self.render_ordered_list(ui, *start, items, depth, pal, out, origin_y);
            }
            Block::HorizontalRule => {
                ui.add_space(6.0);
                let rect = ui.available_rect_before_wrap();
                ui.painter().line_segment(
                    [
                        egui::pos2(rect.left(), rect.top()),
                        egui::pos2(rect.right(), rect.top()),
                    ],
                    Stroke::new(1.0, pal.hr),
                );
                // Allocate space so the cursor advances past the line
                ui.allocate_space(egui::vec2(rect.width(), 1.0));
                ui.add_space(6.0);
            }
            Block::Table { alignments, headers, rows } => {
                self.render_table(ui, alignments, headers, rows, pal, out);
            }
        }
    }

    // ── Heading ────────────────────────────────────────────────────────────

    fn render_heading(
        &self,
        ui: &mut Ui,
        level: u8,
        content: &[Inline],
        _anchor: &str,
        pal: &Palette,
        _out: &mut RenderOutput,
    ) {
        let idx = (level as usize).saturating_sub(1).min(5);
        let scale = self.config.heading_scale[idx];
        let size = self.config.font_size * scale;

        // Add top spacing for headings (more space above H1/H2)
        let top_space = match level {
            1 => size * 0.8,
            2 => size * 0.6,
            _ => size * 0.3,
        };
        ui.add_space(top_space);

        let mut job = LayoutJob::default();
        job.wrap.max_width = ui.available_width();
        self.append_inlines(
            &mut job,
            content,
            size,
            pal.heading,
            pal,
            InlineFmt { bold: true, italic: false, strikethrough: false, is_link: false },
            &mut vec![],
        );

        let galley = ui.fonts(|f| f.layout_job(job));
        let response = ui.label(Arc::clone(&galley));

        // Draw underline beneath H1 and H2
        if level <= 2 {
            let rect = response.rect;
            let y = rect.bottom() + 3.0;
            let alpha = if level == 1 { 120u8 } else { 60u8 };
            ui.painter().line_segment(
                [egui::pos2(rect.left(), y), egui::pos2(rect.right(), y)],
                Stroke::new(if level == 1 { 2.0 } else { 1.0 }, pal.heading.gamma_multiply(alpha as f32 / 255.0)),
            );
        }

        // Check for any anchor link click in this heading (unlikely but consistent)
        if response.clicked() {
            // headings themselves aren't links, nothing to do
        }

        ui.add_space(size * 0.15);
    }

    // ── Paragraph ──────────────────────────────────────────────────────────

    fn render_paragraph(&self, ui: &mut Ui, inlines: &[Inline], pal: &Palette, out: &mut RenderOutput) {
        let mut job = LayoutJob::default();
        job.wrap.max_width = ui.available_width();
        job.wrap.break_anywhere = false;

        let mut link_ranges: Vec<(std::ops::Range<usize>, String)> = Vec::new();

        self.append_inlines(
            &mut job,
            inlines,
            self.config.font_size,
            pal.text,
            pal,
            InlineFmt::default(),
            &mut link_ranges,
        );

        let galley = ui.fonts(|f| f.layout_job(job));
        let response = ui.label(Arc::clone(&galley));

        // Handle link clicks via galley cursor hit-test
        if response.clicked() {
            if let Some(click_pos) = response.interact_pointer_pos() {
                let local = click_pos - response.rect.min;
                let cursor = galley.cursor_from_pos(local);
                let byte_offset = cursor.pcursor.offset;

                for (range, url) in &link_ranges {
                    if range.contains(&byte_offset) {
                        handle_link_click(url, out);
                        break;
                    }
                }

                // Fallback: if only one link in paragraph, clicking anywhere fires it
                if out.anchor_clicked.is_none() && link_ranges.len() == 1 {
                    handle_link_click(&link_ranges[0].1, out);
                }
            }
        }

        // Hover cursor
        if response.hovered() && !link_ranges.is_empty() {
            ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
        }
    }

    // ── Code block ─────────────────────────────────────────────────────────

    fn render_code_block(&self, ui: &mut Ui, _language: Option<&str>, code: &str, pal: &Palette) {
        egui::Frame::new()
            .fill(pal.code_bg)
            .inner_margin(egui::Margin::same(12i8))
            .corner_radius(egui::CornerRadius::same(6))
            .show(ui, |ui| {
                // Code blocks should scroll horizontally rather than wrap
                ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Extend);
                ui.label(
                    RichText::new(code.trim_end())
                        .font(FontId::monospace(self.config.font_size - 1.0))
                        .color(pal.code_fg),
                );
            });
    }

    // ── Blockquote ─────────────────────────────────────────────────────────

    fn render_blockquote(
        &self,
        ui: &mut Ui,
        inner: &[Block],
        depth: usize,
        pal: &Palette,
        out: &mut RenderOutput,
        origin_y: f32,
    ) {
        // Use horizontal layout to draw the left accent bar alongside the content
        ui.horizontal_top(|ui| {
            // Left accent bar — drawn as a coloured narrow widget
            let bar_width = 3.0;
            let (_, bar_rect) = ui.allocate_space(egui::vec2(bar_width + 8.0, 1.0));
            // We'll paint the bar after we know the content height — use painter
            let painter = ui.painter().clone();
            let bar_x = bar_rect.left() + 2.0;

            let before_y = ui.cursor().top();

            egui::Frame::new()
                .fill(pal.blockquote_bg)
                .inner_margin(egui::Margin { left: 8i8, right: 8i8, top: 6i8, bottom: 6i8 })
                .show(ui, |ui| {
                    self.render_blocks(ui, inner, depth + 1, pal, out, origin_y);
                });

            let after_y = ui.cursor().top();
            let height = (after_y - before_y).max(8.0);
            painter.line_segment(
                [
                    egui::pos2(bar_x, before_y),
                    egui::pos2(bar_x, before_y + height),
                ],
                Stroke::new(bar_width, pal.blockquote_border),
            );
        });
    }

    // ── Table ──────────────────────────────────────────────────────────────

    fn render_table(
        &self,
        ui: &mut Ui,
        alignments: &[ColumnAlign],
        headers: &[Vec<Inline>],
        rows: &[Vec<Vec<Inline>>],
        pal: &Palette,
        out: &mut RenderOutput,
    ) {
        let col_count = headers.len().max(rows.iter().map(|r| r.len()).max().unwrap_or(0));
        if col_count == 0 { return; }

        let font_size = self.config.font_size;

        // Fix column widths BEFORE building the TableBuilder.
        // Column::exact avoids the two-pass width-negotiation that Column::remainder triggers,
        // which is the root cause of per-frame flickering.
        //
        // TableBuilder deducts (col_w + spacing_x) per column from available_width (line 906
        // in egui_extras table.rs). So for N columns to fill available_w exactly:
        //   N * (col_w + spacing_x) = available_w  →  col_w = available_w/N - spacing_x
        let available_w = ui.available_width();
        let spacing_x = ui.spacing().item_spacing.x;
        let col_w = (available_w / col_count as f32 - spacing_x).max(50.0);

        let row_h = font_size * 1.4 + 8.0;
        let header_h = font_size * 1.4 + 10.0;

        // Pre-compute body row heights using the same col_w that rendering will use.
        // Uses &Ui (not &mut Ui) so there's no borrow conflict with the TableBuilder below.
        let body_heights: Vec<f32> = rows
            .iter()
            .map(|row| self.table_row_height(ui, row, false, col_w, font_size, pal, row_h))
            .collect();

        // Collect any link-click from inside cell closures.
        let clicked_url: std::cell::RefCell<Option<String>> = std::cell::RefCell::new(None);
        let clicked_url_ref = &clicked_url;

        let mut builder = TableBuilder::new(ui)
            .striped(true)
            .resizable(false)
            // No internal scroll: table grows vertically without limit inside the
            // parent pane's ScrollArea.
            .vscroll(false)
            // Default cell layout: bottom-up so content baseline-aligns within each row.
            // Per-column horizontal alignment is applied via ui.with_layout() inside each cell.
            .cell_layout(egui::Layout::bottom_up(egui::Align::LEFT));

        for _ in 0..col_count {
            builder = builder.column(Column::exact(col_w));
        }

        if !headers.is_empty() {
            builder
                .header(header_h, |mut header| {
                    for (col_idx, cell) in headers.iter().enumerate() {
                        let align = alignments.get(col_idx).copied().unwrap_or(ColumnAlign::None);
                        header.col(|ui| {
                            let layout = col_align_layout(align);
                            ui.with_layout(layout, |ui| {
                                if let Some(url) = self.table_cell(ui, cell, font_size, pal.text_strong, true, align, pal) {
                                    *clicked_url_ref.borrow_mut() = Some(url);
                                }
                            });
                        });
                    }
                })
                .body(|body| {
                    let mut row_idx = 0usize;
                    body.heterogeneous_rows(body_heights.into_iter(), |mut row| {
                        let data_row = &rows[row_idx];
                        row_idx += 1;
                        for col_idx in 0..col_count {
                            let align = alignments.get(col_idx).copied().unwrap_or(ColumnAlign::None);
                            let cell = data_row.get(col_idx).map(|c| c.as_slice()).unwrap_or(&[]);
                            row.col(|ui| {
                                let layout = col_align_layout(align);
                                ui.with_layout(layout, |ui| {
                                    if let Some(url) = self.table_cell(ui, cell, font_size, pal.text, false, align, pal) {
                                        *clicked_url_ref.borrow_mut() = Some(url);
                                    }
                                });
                            });
                        }
                    });
                });
        } else {
            builder.body(|body| {
                let mut row_idx = 0usize;
                body.heterogeneous_rows(body_heights.into_iter(), |mut row| {
                    let data_row = &rows[row_idx];
                    row_idx += 1;
                    for col_idx in 0..col_count {
                        let align = alignments.get(col_idx).copied().unwrap_or(ColumnAlign::None);
                        let cell = data_row.get(col_idx).map(|c| c.as_slice()).unwrap_or(&[]);
                        row.col(|ui| {
                            let layout = col_align_layout(align);
                            ui.with_layout(layout, |ui| {
                                if let Some(url) = self.table_cell(ui, cell, font_size, pal.text, false, align, pal) {
                                    *clicked_url_ref.borrow_mut() = Some(url);
                                }
                            });
                        });
                    }
                });
            });
        }

        if let Some(url) = clicked_url.into_inner() {
            handle_link_click(&url, out);
        }
    }

    /// Pre-measure a row's height (using &Ui, no mutation) so TableBuilder
    /// can allocate the right amount of vertical space before rendering.
    fn table_row_height(
        &self,
        ui: &egui::Ui,
        row: &[Vec<Inline>],
        bold: bool,
        col_w: f32,
        font_size: f32,
        pal: &Palette,
        min_h: f32,
    ) -> f32 {
        // Use exactly col_w as the wrap limit — Column::exact(col_w) means
        // ui.available_width() inside the cell equals col_w exactly (egui_extras
        // sets max_rect.width = column_width with no additional padding).
        // Any divergence here vs table_cell causes a height mismatch → repaint loop.
        let wrap_w = col_w.max(20.0);
        let color = if bold { pal.text_strong } else { pal.text };
        let fmt = InlineFmt { bold, ..InlineFmt::default() };

        let max_text_h = row
            .iter()
            .map(|cell| {
                let mut job = LayoutJob::default();
                job.wrap.max_width = wrap_w;
                self.append_inlines(&mut job, cell, font_size, color, pal, fmt.clone(), &mut vec![]);
                if job.text.is_empty() {
                    job.append(" ", 0.0, TextFormat {
                        font_id: FontId::proportional(font_size),
                        color: Color32::TRANSPARENT,
                        ..Default::default()
                    });
                }
                ui.fonts(|f| f.layout_job(job).size().y)
            })
            .fold(0.0f32, f32::max);

        (max_text_h + 10.0).max(min_h)
    }

    /// Render the content of one table cell.  Returns the URL of any clicked link.
    fn table_cell(
        &self,
        ui: &mut Ui,
        inlines: &[Inline],
        size: f32,
        color: Color32,
        bold: bool,
        align: ColumnAlign,
        pal: &Palette,
    ) -> Option<String> {
        // Inside a TableBuilder column cell, ui.available_width() is the true column width.
        let col_w = ui.available_width().max(20.0);

        let mut job = LayoutJob::default();
        job.wrap.max_width = col_w;
        job.wrap.break_anywhere = false; // wrap at word boundaries only
        job.halign = match align {
            ColumnAlign::Center => egui::Align::Center,
            ColumnAlign::Right  => egui::Align::RIGHT,
            _                   => egui::Align::LEFT,
        };

        let mut link_ranges: Vec<(std::ops::Range<usize>, String)> = Vec::new();
        let fmt = InlineFmt { bold, ..InlineFmt::default() };
        self.append_inlines(&mut job, inlines, size, color, pal, fmt, &mut link_ranges);

        if job.text.is_empty() {
            job.append(" ", 0.0, TextFormat {
                font_id: FontId::proportional(size),
                color: Color32::TRANSPARENT,
                ..Default::default()
            });
        }

        let galley = ui.fonts(|f| f.layout_job(job));
        let response = ui.add(
            egui::Label::new(Arc::clone(&galley))
                .sense(egui::Sense::click())
                .wrap_mode(egui::TextWrapMode::Wrap),
        );

        if response.hovered() && !link_ranges.is_empty() {
            ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
        }

        if response.clicked() {
            if let Some(pos) = response.interact_pointer_pos() {
                let cursor = galley.cursor_from_pos(pos - response.rect.min);
                let byte_offset = cursor.pcursor.offset;
                for (range, url) in &link_ranges {
                    if range.contains(&byte_offset) {
                        return Some(url.clone());
                    }
                }
            }
        }
        None
    }

    // ── Lists ──────────────────────────────────────────────────────────────

    fn render_bullet_list(
        &self,
        ui: &mut Ui,
        items: &[ListItem],
        depth: usize,
        pal: &Palette,
        out: &mut RenderOutput,
        origin_y: f32,
    ) {
        let bullet = match depth % 3 {
            0 => "•",
            1 => "◦",
            _ => "▸",
        };
        let indent = depth as f32 * 20.0;

        for item in items {
            ui.horizontal_top(|ui| {
                ui.add_space(indent);
                ui.label(
                    RichText::new(bullet)
                        .font(FontId::proportional(self.config.font_size))
                        .color(pal.text),
                );
                ui.add_space(4.0);
                ui.vertical(|ui| {
                    self.render_blocks(ui, &item.content, depth + 1, pal, out, origin_y);
                });
            });
        }
    }

    fn render_ordered_list(
        &self,
        ui: &mut Ui,
        start: u64,
        items: &[ListItem],
        depth: usize,
        pal: &Palette,
        out: &mut RenderOutput,
        origin_y: f32,
    ) {
        let indent = depth as f32 * 20.0;

        for (i, item) in items.iter().enumerate() {
            let number = start + i as u64;
            ui.horizontal_top(|ui| {
                ui.add_space(indent);
                ui.label(
                    RichText::new(format!("{}.", number))
                        .font(FontId::proportional(self.config.font_size))
                        .color(pal.text),
                );
                ui.add_space(4.0);
                ui.vertical(|ui| {
                    self.render_blocks(ui, &item.content, depth + 1, pal, out, origin_y);
                });
            });
        }
    }

    // ── LayoutJob inline builder ───────────────────────────────────────────

    fn append_inlines(
        &self,
        job: &mut LayoutJob,
        inlines: &[Inline],
        size: f32,
        color: Color32,
        pal: &Palette,
        fmt: InlineFmt,
        link_ranges: &mut Vec<(std::ops::Range<usize>, String)>,
    ) {
        for inline in inlines {
            match inline {
                Inline::Text(t) => {
                    let byte_start = job.text.len();
                    let tf = self.text_format(size, color, pal, &fmt);
                    job.append(t, 0.0, tf);
                    if fmt.is_link {
                        // caller will have set current url via link_ranges tracking
                        let _ = byte_start; // used by the Link branch below
                    }
                }

                Inline::SoftBreak => {
                    job.append(" ", 0.0, self.text_format(size, color, pal, &InlineFmt::default()));
                }
                Inline::HardBreak => {
                    job.append("\n", 0.0, self.text_format(size, color, pal, &InlineFmt::default()));
                }

                Inline::Bold(inner) => {
                    self.append_inlines(
                        job, inner, size, pal.text_strong, pal,
                        fmt.merge(InlineFmt { bold: true, ..InlineFmt::default() }),
                        link_ranges,
                    );
                }
                Inline::Italic(inner) => {
                    self.append_inlines(
                        job, inner, size, color, pal,
                        fmt.merge(InlineFmt { italic: true, ..InlineFmt::default() }),
                        link_ranges,
                    );
                }
                Inline::Strikethrough(inner) => {
                    self.append_inlines(
                        job, inner, size, color, pal,
                        fmt.merge(InlineFmt { strikethrough: true, ..InlineFmt::default() }),
                        link_ranges,
                    );
                }

                Inline::Code(c) => {
                    let tf = TextFormat {
                        font_id: FontId::monospace(size - 1.0),
                        color: pal.code_fg,
                        background: pal.code_bg,
                        strikethrough: if fmt.strikethrough {
                            Stroke::new(1.0, pal.code_fg)
                        } else {
                            Stroke::NONE
                        },
                        ..Default::default()
                    };
                    job.append(c, 0.0, tf);
                }

                Inline::Link { text, url, .. } => {
                    let byte_start = job.text.len();
                    let link_fmt = fmt.merge(InlineFmt { is_link: true, ..InlineFmt::default() });
                    self.append_inlines(
                        job, text, size, pal.link, pal,
                        link_fmt,
                        &mut vec![], // don't pass link_ranges through — we record them here
                    );
                    // Re-append with underline for the text we just wrote
                    // We already appended, so record the range
                    let byte_end = job.text.len();
                    if byte_end > byte_start {
                        // Retroactively mark these bytes as a link (already written as pal.link colour)
                        // Also apply underline to each section in this range
                        for section in job.sections.iter_mut() {
                            if section.byte_range.start >= byte_start
                                && section.byte_range.end <= byte_end
                            {
                                section.format.underline = Stroke::new(1.0, pal.link);
                            }
                        }
                        link_ranges.push((byte_start..byte_end, url.clone()));
                    }
                }

                Inline::Image { alt, .. } => {
                    let tf = TextFormat {
                        font_id: FontId::proportional(size),
                        color: pal.blockquote_border,
                        italics: true,
                        ..Default::default()
                    };
                    job.append(&format!("[{}]", alt), 0.0, tf);
                }
            }
        }
    }

    fn text_format(&self, size: f32, color: Color32, pal: &Palette, fmt: &InlineFmt) -> TextFormat {
        TextFormat {
            font_id: FontId::proportional(size),
            color,
            italics: fmt.italic,
            extra_letter_spacing: if fmt.bold { 0.6 } else { 0.0 },
            strikethrough: if fmt.strikethrough {
                Stroke::new(1.5, color)
            } else {
                Stroke::NONE
            },
            underline: if fmt.is_link {
                Stroke::new(1.0, pal.link)
            } else {
                Stroke::NONE
            },
            ..Default::default()
        }
    }
}

/// Inline formatting state (composable)
#[derive(Clone, Default)]
struct InlineFmt {
    bold: bool,
    italic: bool,
    strikethrough: bool,
    is_link: bool,
}

impl InlineFmt {
    /// OR two formats together
    fn merge(&self, other: InlineFmt) -> InlineFmt {
        InlineFmt {
            bold: self.bold || other.bold,
            italic: self.italic || other.italic,
            strikethrough: self.strikethrough || other.strikethrough,
            is_link: self.is_link || other.is_link,
        }
    }
}

/// Map a column alignment to an egui Layout for use inside TableBuilder cells.
/// bottom_up: content sits at the row's baseline; extra row height adds space above.
fn col_align_layout(align: ColumnAlign) -> egui::Layout {
    match align {
        ColumnAlign::Center => egui::Layout::bottom_up(egui::Align::Center),
        ColumnAlign::Right  => egui::Layout::bottom_up(egui::Align::RIGHT),
        _                   => egui::Layout::bottom_up(egui::Align::LEFT),
    }
}

/// Dispatch a link click — anchor links signal in-doc navigation, external ones open browser.
fn handle_link_click(url: &str, out: &mut RenderOutput) {
    if let Some(anchor) = url.strip_prefix('#') {
        out.anchor_clicked = Some(anchor.to_string());
    } else {
        let _ = open::that(url);
    }
}

// ── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::MarkdownParser;

    #[test]
    fn test_renderer_creates_without_panic() {
        let config = AppConfig::default();
        let _renderer = MarkdownRenderer::new(&config);
    }

    #[test]
    fn test_markdown_parses_to_blocks_for_rendering() {
        let md = "# Title\n\nSome paragraph text.\n\n```rust\nlet x = 1;\n```";
        let blocks = MarkdownParser::parse(md);
        assert!(blocks.len() >= 3);
    }

    #[test]
    fn test_heading_level_scale() {
        let config = AppConfig::default();
        assert!(config.heading_scale[0] > config.heading_scale[5]);
    }

    #[test]
    fn test_content_width_constrains_rendering() {
        use crate::config::ContentWidth;
        assert!(ContentWidth::Narrow.max_pixels() < ContentWidth::Wide.max_pixels());
    }

    #[test]
    fn test_inline_fmt_merge() {
        let a = InlineFmt { bold: true, ..InlineFmt::default() };
        let b = InlineFmt { italic: true, ..InlineFmt::default() };
        let merged = a.merge(b);
        assert!(merged.bold);
        assert!(merged.italic);
        assert!(!merged.strikethrough);
    }

    #[test]
    fn test_handle_anchor_link() {
        let mut out = RenderOutput::default();
        handle_link_click("#introduction", &mut out);
        assert_eq!(out.anchor_clicked, Some("introduction".to_string()));
    }

    #[test]
    fn test_handle_external_link_no_anchor_click() {
        let mut out = RenderOutput::default();
        handle_link_click("https://example.com", &mut out);
        assert!(out.anchor_clicked.is_none());
    }
}
