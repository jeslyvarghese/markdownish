use egui::{Color32, RichText, Ui};

use crate::config::{AppConfig, ColorConfig, ContentWidth, ThemeMode};

pub struct SettingsPanel;

impl SettingsPanel {
    pub fn show(ui: &mut Ui, config: &mut AppConfig) {
        ui.heading(
            RichText::new("Appearance Settings")
                .size(18.0)
                .color(ColorConfig::to_egui_color32(config.colors.heading)),
        );
        ui.add_space(12.0);

        Self::show_theme_section(ui, config);
        ui.separator();

        Self::show_typography_section(ui, config);
        ui.separator();

        Self::show_layout_section(ui, config);
        ui.separator();

        Self::show_colors_section(ui, config);
    }

    fn show_theme_section(ui: &mut Ui, config: &mut AppConfig) {
        ui.label(RichText::new("Theme").strong());
        ui.add_space(4.0);

        ui.horizontal(|ui| {
            let is_dark = config.theme == ThemeMode::Dark;
            let is_light = config.theme == ThemeMode::Light;

            if ui.selectable_label(is_dark, "Dark").clicked() {
                config.apply_dark_theme();
            }
            if ui.selectable_label(is_light, "Light").clicked() {
                config.apply_light_theme();
            }
            if ui.selectable_label(config.theme == ThemeMode::Custom, "Custom").clicked() {
                config.theme = ThemeMode::Custom;
            }
        });
        ui.add_space(8.0);
    }

    fn show_typography_section(ui: &mut Ui, config: &mut AppConfig) {
        ui.label(RichText::new("Typography").strong());
        ui.add_space(4.0);

        ui.horizontal(|ui| {
            ui.label("Font Size:");
            ui.add(egui::Slider::new(&mut config.font_size, 10.0..=32.0).suffix("pt"));
        });

        ui.horizontal(|ui| {
            ui.label("Line Spacing:");
            ui.add(egui::Slider::new(&mut config.line_spacing, 1.0..=2.5).step_by(0.1));
        });

        ui.add_space(4.0);
        ui.label(RichText::new("Heading Scales").size(12.0).weak());
        for (i, scale) in config.heading_scale.iter_mut().enumerate() {
            ui.horizontal(|ui| {
                ui.label(format!("H{}:", i + 1));
                ui.add(egui::Slider::new(scale, 0.8..=3.0).step_by(0.1));
            });
        }
        ui.add_space(8.0);
    }

    fn show_layout_section(ui: &mut Ui, config: &mut AppConfig) {
        ui.label(RichText::new("Layout").strong());
        ui.add_space(4.0);

        ui.horizontal(|ui| {
            ui.label("Content Width:");
            egui::ComboBox::from_id_salt("content_width")
                .selected_text(match &config.content_width {
                    ContentWidth::Narrow => "Narrow (600px)",
                    ContentWidth::Medium => "Medium (800px)",
                    ContentWidth::Wide => "Wide (1100px)",
                    ContentWidth::Full => "Full Width",
                })
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut config.content_width, ContentWidth::Narrow, "Narrow (600px)");
                    ui.selectable_value(&mut config.content_width, ContentWidth::Medium, "Medium (800px)");
                    ui.selectable_value(&mut config.content_width, ContentWidth::Wide, "Wide (1100px)");
                    ui.selectable_value(&mut config.content_width, ContentWidth::Full, "Full Width");
                });
        });
        ui.add_space(8.0);
    }

    fn show_colors_section(ui: &mut Ui, config: &mut AppConfig) {
        ui.label(RichText::new("Colors").strong());
        ui.add_space(4.0);

        let color_entries: &mut [(&str, &mut [u8; 3])] = &mut [
            ("Background", &mut config.colors.background),
            ("Surface", &mut config.colors.surface),
            ("Text", &mut config.colors.text),
            ("Headings", &mut config.colors.heading),
            ("Links", &mut config.colors.link),
            ("Code Background", &mut config.colors.code_bg),
            ("Code Text", &mut config.colors.code_text),
            ("Blockquote Border", &mut config.colors.blockquote_border),
        ];

        for (label, rgb) in color_entries.iter_mut() {
            ui.horizontal(|ui| {
                ui.label(*label);
                let mut color = Color32::from_rgb(rgb[0], rgb[1], rgb[2]);
                if ui.color_edit_button_srgba(&mut color).changed() {
                    rgb[0] = color.r();
                    rgb[1] = color.g();
                    rgb[2] = color.b();
                }
            });
        }
        ui.add_space(8.0);

        if ui.button("Reset to Defaults").clicked() {
            match config.theme {
                ThemeMode::Light => config.apply_light_theme(),
                _ => config.apply_dark_theme(),
            }
        }
    }
}
