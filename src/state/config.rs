use crate::libs::theme::Theme;
use crate::state::paths;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomThemeData {
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
    pub fn save_custom_theme(&mut self, name: String, css: String) -> Result<(), String> {
        if name.trim().is_empty() {
            return Err("Theme name cannot be empty".to_string());
        }

        let now = Utc::now();
        let theme_data = CustomThemeData {
            name: name.clone(),
            css,
            created_at: self
                .custom_themes
                .get(&name)
                .map(|existing| existing.created_at)
                .unwrap_or(now),
            modified_at: now,
        };

        self.custom_themes.insert(name, theme_data);
        self.last_updated = now;
        self.save()
    }

    /// Remove a custom theme
    pub fn delete_custom_theme(&mut self, name: &str) -> Result<(), String> {
        if !self.custom_themes.contains_key(name) {
            return Err("Theme not found".to_string());
        }

        // If the current theme is the one being deleted, switch to System
        if let Theme::Custom(current_name) = &self.theme {
            if current_name == name {
                self.theme = Theme::System;
            }
        }

        self.custom_themes.remove(name);
        self.last_updated = Utc::now();
        self.save()
    }

    /// Get a custom theme by name
    pub fn get_custom_theme(&self, name: &str) -> Option<&CustomThemeData> {
        self.custom_themes.get(name)
    }
    /// List all custom theme names
    pub fn list_custom_themes(&self) -> Vec<String> {
        self.custom_themes.keys().cloned().collect()
    }

    /// Create a sample custom theme for demonstration
    pub fn create_sample_theme(&mut self, name: &str) -> Result<(), String> {
        let sample_css = match name {
            "Cyberpunk Neon" => {
                r#"/* Cyberpunk Neon Theme */
[data-theme="custom-cyberpunk-neon"] {
  color-scheme: dark;
  --color-base-100: oklch(12% 0.02 270);
  --color-base-200: oklch(8% 0.03 270);
  --color-base-300: oklch(5% 0.04 270);
  --color-base-content: oklch(90% 0.1 180);
  --color-primary: oklch(65% 0.25 240);
  --color-primary-content: oklch(10% 0.05 240);
  --color-secondary: oklch(70% 0.3 330);
  --color-secondary-content: oklch(10% 0.05 330);
  --color-accent: oklch(75% 0.25 150);
  --color-accent-content: oklch(10% 0.05 150);
  --color-neutral: oklch(20% 0.05 280);
  --color-neutral-content: oklch(85% 0.05 180);
}

[data-theme="custom-cyberpunk-neon"] .btn {
  text-transform: uppercase;
  font-weight: 600;
  letter-spacing: 0.05em;
  border: 1px solid var(--color-primary);
}

[data-theme="custom-cyberpunk-neon"] .btn-primary {
  background: linear-gradient(135deg, var(--color-primary), var(--color-secondary));
}"#
            }
            "Ocean Breeze" => {
                r#"/* Ocean Breeze Theme */
[data-theme="custom-ocean-breeze"] {
  color-scheme: light;
  --color-base-100: oklch(98% 0.01 220);
  --color-base-200: oklch(95% 0.02 220);
  --color-base-300: oklch(90% 0.03 220);
  --color-base-content: oklch(25% 0.02 220);
  --color-primary: oklch(55% 0.15 200);
  --color-primary-content: oklch(98% 0.01 200);
  --color-secondary: oklch(65% 0.12 180);
  --color-secondary-content: oklch(98% 0.01 180);
  --color-accent: oklch(70% 0.14 160);
  --color-accent-content: oklch(98% 0.01 160);
  --color-neutral: oklch(40% 0.05 220);
  --color-neutral-content: oklch(98% 0.01 220);
}

[data-theme="custom-ocean-breeze"] .btn {
  border-radius: 1rem;
  box-shadow: 0 2px 8px rgba(59, 130, 246, 0.15);
}
"#
            }
            "Sunset Glow" => {
                r#"/* Sunset Glow Theme */
[data-theme="custom-sunset-glow"] {
  color-scheme: dark;
  --color-base-100: oklch(15% 0.03 30);
  --color-base-200: oklch(12% 0.04 30);
  --color-base-300: oklch(8% 0.05 30);
  --color-base-content: oklch(90% 0.02 50);
  --color-primary: oklch(70% 0.2 40);
  --color-primary-content: oklch(15% 0.03 40);
  --color-secondary: oklch(65% 0.25 350);
  --color-secondary-content: oklch(15% 0.03 350);
  --color-accent: oklch(75% 0.18 60);
  --color-accent-content: oklch(15% 0.03 60);
  --color-neutral: oklch(25% 0.04 30);
  --color-neutral-content: oklch(90% 0.02 50);
}

[data-theme="custom-sunset-glow"] .btn {
  background: linear-gradient(135deg, var(--color-primary), var(--color-accent));
  border: none;
  text-shadow: 0 1px 2px rgba(0,0,0,0.3);
}
"#
            }
            _ => return Err("Unknown sample theme".to_string()),
        };

        self.save_custom_theme(name.to_string(), sample_css.to_string())
    }
}

impl Default for AppConfig {
    fn default() -> Self {
        let mut config = Self {
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

        // Add sample themes on first creation
        let _ = config.create_sample_theme("Cyberpunk Neon");
        let _ = config.create_sample_theme("Ocean Breeze");
        let _ = config.create_sample_theme("Sunset Glow");

        config
    }
}
