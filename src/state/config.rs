use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub current_soundpack: String,
    pub volume: f32,
}

impl AppConfig {
    pub fn load() -> Self {
        let config_path = PathBuf::from("./data/config.json");
        if let Ok(contents) = fs::read_to_string(config_path) {
            serde_json::from_str(&contents).unwrap_or_else(|_| Self::default())
        } else {
            Self::default()
        }
    }

    pub fn save(&self) -> Result<(), String> {
        let config_path = PathBuf::from("./data/config.json");
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
            current_soundpack: "".to_string(),
            volume: 1.0,
        }
    }
}
