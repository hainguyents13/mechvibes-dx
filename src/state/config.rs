use crate::libs::theme::Theme;
use crate::state::paths;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    // Metadata
    pub version: String,
    pub last_updated: DateTime<Utc>,
    pub commit: Option<String>,

    // Audio settings
    pub keyboard_soundpack: String,
    pub mouse_soundpack: String,
    pub volume: f32,
    pub enable_sound: bool,

    // UI settings
    pub theme: Theme,

    // System settings
    pub auto_start: bool,
    pub show_notifications: bool,
}

impl AppConfig {
    pub fn load() -> Self {
        // Ensure data directory exists
        if let Err(_) = fs::create_dir_all(paths::data::DIR) {
            eprintln!("Warning: Could not create data directory");
        }

        let config_path = PathBuf::from(paths::data::CONFIG_JSON);
        if let Ok(contents) = fs::read_to_string(config_path) {
            match serde_json::from_str::<AppConfig>(&contents) {
                Ok(config) => {
                    // Don't update version and last_updated when only reading config
                    // Don't save file when only reading config
                    config
                }
                Err(e) => {
                    eprintln!(
                        "Warning: Failed to parse config file: {}. Using defaults.",
                        e
                    );
                    Self::default()
                }
            }
        } else {
            let default_config = Self::default();
            let _ = default_config.save();
            default_config
        }
    }

    pub fn save(&self) -> Result<(), String> {
        let config_path = PathBuf::from(paths::data::CONFIG_JSON);
        let contents = serde_json::to_string_pretty(self)
            .map_err(|e| format!("Failed to serialize config: {}", e))?;
        fs::write(config_path, contents)
            .map_err(|e| format!("Failed to write config file: {}", e))?;
        Ok(())
    }
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            version: env!("CARGO_PKG_VERSION").to_string(),
            last_updated: Utc::now(),
            commit: option_env!("GIT_HASH").map(|s| s.to_string()),
            keyboard_soundpack: "oreo".to_string(),
            mouse_soundpack: "test-mouse".to_string(),
            volume: 1.0,
            enable_sound: true,
            theme: Theme::System, // Default to System theme
            auto_start: false,
            show_notifications: true,
        }
    }
}
