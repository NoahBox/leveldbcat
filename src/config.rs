use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

const DEFAULT_FONT_SIZE_PX: f32 = 16.0;
const MIN_FONT_SIZE_PX: f32 = 12.0;
const MAX_FONT_SIZE_PX: f32 = 28.0;
const DEFAULT_JSON_INDENT_SPACES: u8 = 2;
const MIN_JSON_INDENT_SPACES: u8 = 1;
const MAX_JSON_INDENT_SPACES: u8 = 8;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VisualMode {
    Light,
    Dark,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AppLanguage {
    English,
    Chinese,
    TraditionalChinese,
    Japanese,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct AppConfig {
    pub font_size_px: f32,
    pub monospace_font_family: Option<String>,
    pub json_indent_spaces: u8,
    pub visual_mode: VisualMode,
    pub language: AppLanguage,
    pub last_workdir: Option<PathBuf>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            font_size_px: DEFAULT_FONT_SIZE_PX,
            monospace_font_family: None,
            json_indent_spaces: DEFAULT_JSON_INDENT_SPACES,
            visual_mode: VisualMode::Light,
            language: detect_system_language(),
            last_workdir: None,
        }
    }
}

impl AppConfig {
    pub fn load(path: &Path) -> Self {
        let Ok(raw) = fs::read_to_string(path) else {
            return Self::default();
        };

        serde_json::from_str::<Self>(&raw)
            .map(Self::sanitized)
            .unwrap_or_default()
    }

    pub fn save(&self, path: &Path) -> Result<(), String> {
        let config = self.clone().sanitized();
        let parent = path.parent().ok_or_else(|| {
            format!(
                "Failed to determine config directory for {}",
                path.display()
            )
        })?;

        fs::create_dir_all(parent).map_err(|error| {
            format!(
                "Failed to create config directory {}: {error}",
                parent.display()
            )
        })?;

        let serialized = serde_json::to_string_pretty(&config)
            .map_err(|error| format!("Failed to serialize config: {error}"))?;

        fs::write(path, serialized)
            .map_err(|error| format!("Failed to write config {}: {error}", path.display()))
    }

    pub fn sanitized(mut self) -> Self {
        self.font_size_px = self.font_size_px.clamp(MIN_FONT_SIZE_PX, MAX_FONT_SIZE_PX);
        self.json_indent_spaces = self
            .json_indent_spaces
            .clamp(MIN_JSON_INDENT_SPACES, MAX_JSON_INDENT_SPACES);
        self
    }
}

pub fn default_config_path() -> PathBuf {
    let base = dirs::config_dir()
        .or_else(|| std::env::current_dir().ok())
        .unwrap_or_else(|| PathBuf::from("."));

    base.join("LeveldbCat").join("config.json")
}

pub fn min_font_size_px() -> f32 {
    MIN_FONT_SIZE_PX
}

pub fn max_font_size_px() -> f32 {
    MAX_FONT_SIZE_PX
}

pub fn min_json_indent_spaces() -> u8 {
    MIN_JSON_INDENT_SPACES
}

pub fn max_json_indent_spaces() -> u8 {
    MAX_JSON_INDENT_SPACES
}

fn detect_system_language() -> AppLanguage {
    let Some(locale) = sys_locale::get_locale() else {
        return AppLanguage::English;
    };

    let locale = locale.replace('_', "-").to_ascii_lowercase();

    if locale.starts_with("ja") {
        return AppLanguage::Japanese;
    }

    if locale.starts_with("zh") {
        if locale.contains("hant")
            || locale.contains("tw")
            || locale.contains("hk")
            || locale.contains("mo")
        {
            return AppLanguage::TraditionalChinese;
        }

        return AppLanguage::Chinese;
    }

    AppLanguage::English
}
