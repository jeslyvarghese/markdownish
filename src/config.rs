use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ThemeMode {
    Light,
    Dark,
    Custom,
}

impl Default for ThemeMode {
    fn default() -> Self {
        ThemeMode::Dark
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ContentWidth {
    Narrow,
    Medium,
    Wide,
    Full,
}

impl Default for ContentWidth {
    fn default() -> Self {
        ContentWidth::Medium
    }
}

impl ContentWidth {
    pub fn max_pixels(&self) -> f32 {
        match self {
            ContentWidth::Narrow => 600.0,
            ContentWidth::Medium => 800.0,
            ContentWidth::Wide => 1100.0,
            ContentWidth::Full => f32::INFINITY,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColorConfig {
    pub background: [u8; 3],
    pub surface: [u8; 3],
    pub text: [u8; 3],
    pub heading: [u8; 3],
    pub link: [u8; 3],
    pub code_bg: [u8; 3],
    pub code_text: [u8; 3],
    pub blockquote_border: [u8; 3],
    pub blockquote_bg: [u8; 3],
    pub hr_color: [u8; 3],
}

impl Default for ColorConfig {
    fn default() -> Self {
        // Catppuccin Mocha inspired dark theme
        Self {
            background: [30, 30, 46],      // #1e1e2e
            surface: [49, 50, 68],          // #313244
            text: [205, 214, 244],          // #cdd6f4
            heading: [137, 180, 250],       // #89b4fa
            link: [137, 220, 235],          // #89dceb
            code_bg: [49, 50, 68],          // #313244
            code_text: [166, 227, 161],     // #a6e3a1
            blockquote_border: [108, 112, 134], // #6c7086
            blockquote_bg: [40, 41, 56],    // #282838
            hr_color: [108, 112, 134],      // #6c7086
        }
    }
}

impl ColorConfig {
    pub fn light_defaults() -> Self {
        Self {
            background: [250, 249, 247],    // warm white
            surface: [238, 236, 233],       // #eeece9
            text: [40, 36, 32],             // dark brown
            heading: [20, 100, 180],        // deep blue
            link: [0, 120, 180],            // blue
            code_bg: [238, 236, 233],       // light gray
            code_text: [60, 100, 40],       // dark green
            blockquote_border: [180, 160, 140],
            blockquote_bg: [245, 242, 238],
            hr_color: [200, 190, 180],
        }
    }

    pub fn to_egui_color32(rgb: [u8; 3]) -> egui::Color32 {
        egui::Color32::from_rgb(rgb[0], rgb[1], rgb[2])
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub theme: ThemeMode,
    pub font_size: f32,
    pub line_spacing: f32,
    pub content_width: ContentWidth,
    pub colors: ColorConfig,
    pub syntax_theme: String,
    pub recent_files: Vec<PathBuf>,
    pub show_settings: bool,
    pub heading_scale: [f32; 6], // H1..H6 multipliers
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            theme: ThemeMode::Dark,
            font_size: 16.0,
            line_spacing: 1.4,
            content_width: ContentWidth::Medium,
            colors: ColorConfig::default(),
            syntax_theme: "base16-ocean.dark".to_string(),
            recent_files: Vec::new(),
            show_settings: false,
            heading_scale: [2.0, 1.6, 1.3, 1.1, 1.0, 0.9],
        }
    }
}

impl AppConfig {
    pub fn config_path() -> Option<PathBuf> {
        dirs::config_dir().map(|d| d.join("markdown-viewer").join("config.toml"))
    }

    pub fn load() -> Self {
        let Some(path) = Self::config_path() else {
            return Self::default();
        };
        let Ok(content) = std::fs::read_to_string(&path) else {
            return Self::default();
        };
        toml::from_str(&content).unwrap_or_default()
    }

    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let Some(path) = Self::config_path() else {
            return Ok(());
        };
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let content = toml::to_string_pretty(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }

    pub fn add_recent_file(&mut self, path: PathBuf) {
        self.recent_files.retain(|p| p != &path);
        self.recent_files.insert(0, path);
        self.recent_files.truncate(10);
    }

    pub fn apply_light_theme(&mut self) {
        self.theme = ThemeMode::Light;
        self.colors = ColorConfig::light_defaults();
    }

    pub fn apply_dark_theme(&mut self) {
        self.theme = ThemeMode::Dark;
        self.colors = ColorConfig::default();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config_has_dark_theme() {
        let config = AppConfig::default();
        assert_eq!(config.theme, ThemeMode::Dark);
    }

    #[test]
    fn test_default_font_size_is_reasonable() {
        let config = AppConfig::default();
        assert!(config.font_size >= 10.0 && config.font_size <= 32.0);
    }

    #[test]
    fn test_content_width_pixels() {
        assert!(ContentWidth::Narrow.max_pixels() < ContentWidth::Medium.max_pixels());
        assert!(ContentWidth::Medium.max_pixels() < ContentWidth::Wide.max_pixels());
        assert_eq!(ContentWidth::Full.max_pixels(), f32::INFINITY);
    }

    #[test]
    fn test_add_recent_files_deduplicates() {
        let mut config = AppConfig::default();
        let path = PathBuf::from("/tmp/test.md");
        config.add_recent_file(path.clone());
        config.add_recent_file(path.clone());
        assert_eq!(config.recent_files.len(), 1);
    }

    #[test]
    fn test_add_recent_files_limits_to_ten() {
        let mut config = AppConfig::default();
        for i in 0..15 {
            config.add_recent_file(PathBuf::from(format!("/tmp/file{}.md", i)));
        }
        assert_eq!(config.recent_files.len(), 10);
    }

    #[test]
    fn test_add_recent_file_moves_to_front() {
        let mut config = AppConfig::default();
        config.add_recent_file(PathBuf::from("/tmp/a.md"));
        config.add_recent_file(PathBuf::from("/tmp/b.md"));
        config.add_recent_file(PathBuf::from("/tmp/a.md")); // re-add
        assert_eq!(config.recent_files[0], PathBuf::from("/tmp/a.md"));
        assert_eq!(config.recent_files.len(), 2);
    }

    #[test]
    fn test_apply_light_theme_changes_colors() {
        let mut config = AppConfig::default();
        config.apply_light_theme();
        assert_eq!(config.theme, ThemeMode::Light);
        // Light background should be bright
        let bg = config.colors.background;
        assert!(bg[0] > 200 && bg[1] > 200 && bg[2] > 200);
    }

    #[test]
    fn test_serialize_deserialize_roundtrip() {
        let config = AppConfig::default();
        let serialized = toml::to_string_pretty(&config).unwrap();
        let deserialized: AppConfig = toml::from_str(&serialized).unwrap();
        assert_eq!(deserialized.font_size, config.font_size);
        assert_eq!(deserialized.theme, config.theme);
    }

    #[test]
    fn test_color_to_egui() {
        let color = ColorConfig::to_egui_color32([255, 128, 0]);
        assert_eq!(color.r(), 255);
        assert_eq!(color.g(), 128);
        assert_eq!(color.b(), 0);
    }

    #[test]
    fn test_heading_scale_has_six_levels() {
        let config = AppConfig::default();
        assert_eq!(config.heading_scale.len(), 6);
        // H1 should be biggest
        assert!(config.heading_scale[0] > config.heading_scale[5]);
    }
}
