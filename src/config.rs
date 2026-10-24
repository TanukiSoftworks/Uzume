use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

pub const DEEZER_CLIENT_ID: &str = "1258131430928547880";
pub const MUSIC_CLIENT_ID: &str = "1205619376275980288";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Language {
    Auto,
    Es,
    En,
    ZhHans,
    ZhHant,
    Ja,
    Ko,
}

impl Default for Language {
    fn default() -> Self {
        Self::Auto
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StatusDisplayMode {
    ListeningSong,
    CustomText,
    AppDefault,
}

impl Default for StatusDisplayMode {
    fn default() -> Self {
        Self::ListeningSong
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AppPreset {
    Music,
    Deezer,
    Custom,
}

impl Default for AppPreset {
    fn default() -> Self {
        Self::Music
    }
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub language: Language,
    #[serde(default)]
    pub preset: AppPreset,
    #[serde(default)]
    pub custom_client_id: String,
    #[serde(default)]
    pub status_mode: StatusDisplayMode,
    #[serde(default)]
    pub custom_text: String,
    #[serde(default = "default_true")]
    pub show_progress_bar: bool,
    #[serde(default = "default_true")]
    pub show_cover_art: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            language: Language::Auto,
            preset: AppPreset::Music,
            custom_client_id: String::new(),
            status_mode: StatusDisplayMode::ListeningSong,
            custom_text: String::new(),
            show_progress_bar: true,
            show_cover_art: true,
        }
    }
}

impl Config {
    pub fn get_active_client_id(&self) -> String {
        match self.preset {
            AppPreset::Music => MUSIC_CLIENT_ID.to_string(),
            AppPreset::Deezer => DEEZER_CLIENT_ID.to_string(),
            AppPreset::Custom => {
                let clean = self.custom_client_id.trim();
                if !clean.is_empty() {
                    clean.to_string()
                } else {
                    MUSIC_CLIENT_ID.to_string()
                }
            }
        }
    }

    fn config_path() -> Option<PathBuf> {
        let appdata = std::env::var_os("APPDATA")?;
        let dir = PathBuf::from(appdata).join("Uzume");
        let _ = fs::create_dir_all(&dir);
        Some(dir.join("config.json"))
    }

    pub fn load() -> Self {
        if let Some(path) = Self::config_path() {
            if let Ok(content) = fs::read_to_string(&path) {
                if let Ok(config) = serde_json::from_str(&content) {
                    return config;
                }
            }
        }
        Self::default()
    }

    pub fn save(&self) {
        if let Some(path) = Self::config_path() {
            if let Ok(content) = serde_json::to_string_pretty(self) {
                let _ = fs::write(path, content);
            }
        }
    }
}
