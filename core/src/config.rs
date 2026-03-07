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
        // Catppuccin Mocha dark theme
        Self {
            background: [30, 30, 46],
            surface: [49, 50, 68],
            text: [205, 214, 244],
            heading: [137, 180, 250],
            link: [137, 220, 235],
            code_bg: [49, 50, 68],
            code_text: [166, 227, 161],
            blockquote_border: [108, 112, 134],
            blockquote_bg: [40, 41, 56],
            hr_color: [108, 112, 134],
        }
    }
}

impl ColorConfig {
    pub fn light_defaults() -> Self {
        Self {
            background: [250, 249, 247],
            surface: [238, 236, 233],
            text: [40, 36, 32],
            heading: [20, 100, 180],
            link: [0, 120, 180],
            code_bg: [238, 236, 233],
            code_text: [60, 100, 40],
            blockquote_border: [180, 160, 140],
            blockquote_bg: [245, 242, 238],
            hr_color: [200, 190, 180],
        }
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
    pub heading_scale: [f32; 6],
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
