use crate::libs::theme::Theme;
use crate::state::paths;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomThemeData {
    pub id: String, // Unique ID for the theme
    pub name: String,
    pub css: String,
    pub created_at: DateTime<Utc>,
    pub modified_at: DateTime<Utc>,
}

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
    pub mouse_volume: f32, // Separate volume for mouse sounds
    pub enable_sound: bool,
    pub enable_keyboard_sound: bool, // Enable/disable keyboard sounds specifically
    pub enable_mouse_sound: bool,    // Enable/disable mouse sounds specifically

    // UI settings
    pub theme: Theme,
    pub custom_css: String, // Legacy field for existing custom CSS
    pub custom_themes: HashMap<String, CustomThemeData>, // New field for custom themes

    // System settings
    pub auto_start: bool,
    pub show_notifications: bool,
}

impl AppConfig {
    pub fn load() -> Self {
        // Ensure data directory exists
        let data_dir = paths::data::config_json().parent().unwrap().to_path_buf();
        if let Err(_) = fs::create_dir_all(&data_dir) {
            eprintln!("Warning: Could not create data directory");
        }

        let config_path = paths::data::config_json();
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
        let config_path = paths::data::config_json();
        let contents = serde_json::to_string_pretty(self)
            .map_err(|e| format!("Failed to serialize config: {}", e))?;
        fs::write(config_path, contents)
            .map_err(|e| format!("Failed to write config file: {}", e))?;
        Ok(())
    }

    /// Add or update a custom theme
    pub fn save_custom_theme(
        &mut self,
        id: String,
        name: String,
        css: String,
    ) -> Result<(), String> {
        if name.trim().is_empty() {
            return Err("Theme name cannot be empty".to_string());
        }
        let now = Utc::now();

        // If updating, keep the original ID and created_at
        if let Some(existing) = self.custom_themes.get_mut(&id) {
            existing.name = name;
            existing.css = css;
            existing.modified_at = now;
        } else {
            // If new theme, create new theme data
            let theme_data = CustomThemeData {
                id: id.clone(),
                name: name,
                css: css,
                created_at: now,
                modified_at: now,
            };
            self.custom_themes.insert(id, theme_data);
        }
        self.last_updated = now;
        self.save()
    }

    /// Remove a custom theme
    pub fn delete_custom_theme(&mut self, id: &str) -> Result<(), String> {
        // If the current theme is the one being deleted, switch to System
        if let Theme::Custom(current_id) = &self.theme {
            if current_id == id {
                self.theme = Theme::System;
            }
        }

        self.custom_themes.remove(id);
        self.last_updated = Utc::now();
        self.save()
    }

    /// Get a custom theme by ID
    pub fn get_custom_theme_by_id(&self, id: &str) -> Option<&CustomThemeData> {
        self.custom_themes.values().find(|theme| theme.id == id)
    }

    /// List all custom theme data
    pub fn list_custom_theme_data(&self) -> Vec<&CustomThemeData> {
        let mut themes: Vec<_> = self.custom_themes.values().collect();
        themes.sort_by_key(|theme| theme.created_at);
        themes
    }
}

impl Default for AppConfig {
    fn default() -> Self {
        let config = Self {
            version: env!("CARGO_PKG_VERSION").to_string(),
            last_updated: Utc::now(),
            commit: option_env!("GIT_HASH").map(|s| s.to_string()),
            keyboard_soundpack: "oreo".to_string(),
            mouse_soundpack: "test-mouse".to_string(),
            volume: 1.0,
            mouse_volume: 1.0, // Default mouse volume to 100%
            enable_sound: true,
            enable_keyboard_sound: true, // Default keyboard sounds enabled
            enable_mouse_sound: true,    // Default mouse sounds enabled
            theme: Theme::System,        // Default to System theme
            custom_css: String::new(),
            custom_themes: HashMap::new(), // Empty custom themes by default
            auto_start: false,
            show_notifications: true,
        };

        config
    }
}
