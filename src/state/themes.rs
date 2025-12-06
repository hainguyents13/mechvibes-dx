use crate::state::paths;
use crate::utils::{data, path};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CustomThemeData {
    pub id: String,
    pub name: String,
    pub description: String,
    pub css: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub is_built_in: bool, // Indicates if this is a built-in theme
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ThemesConfig {
    pub version: String,
    pub custom_themes: HashMap<String, CustomThemeData>,
    pub last_updated: DateTime<Utc>,
}

impl Default for ThemesConfig {
    fn default() -> Self {
        Self {
            version: "1.0.0".to_string(),
            custom_themes: HashMap::new(),
            last_updated: Utc::now(),
        }
    }
}

impl ThemesConfig {
    pub fn load() -> Self {
        let themes_path = paths::data::themes_json();

        // Ensure data directory exists
        if let Some(parent) = themes_path.parent() {
            if let Err(e) = path::ensure_directory_exists(parent) {
                eprintln!("Warning: Could not create themes data directory: {}", e);
            }
        }

        match data::load_json_from_file::<ThemesConfig>(&themes_path) {
            Ok(config) => {
                println!(
                    "✅ Loaded themes configuration from {}",
                    themes_path.display()
                );
                config
            }
            Err(e) => {
                eprintln!("❌ Failed to load themes.json: {}", e);
                println!("📝 Creating new themes configuration");
                let config = Self::default();
                if let Err(e) = config.save() {
                    eprintln!("❌ Failed to create initial themes.json: {}", e);
                }
                config
            }
        }
    }

    pub fn save(&self) -> Result<(), String> {
        let themes_path = paths::data::themes_json();

        // Ensure the data directory exists
        if let Some(parent) = themes_path.parent() {
            path::ensure_directory_exists(parent)
                .map_err(|e| format!("Failed to create data directory: {}", e))?;
        }

        data::save_json_to_file(self, &themes_path)?;
        println!("💾 Saved themes configuration to {}", themes_path.display());
        Ok(())
    }
    pub fn add_theme(
        &mut self,
        name: String,
        description: String,
        css: String,
    ) -> Result<String, String> {
        if name.trim().is_empty() {
            return Err("Theme name cannot be empty".to_string());
        }

        // Check for duplicate names
        for theme in self.custom_themes.values() {
            if theme.name.to_lowercase() == name.to_lowercase() {
                return Err("A theme with this name already exists".to_string());
            }
        }

        let id = uuid::Uuid::new_v4().to_string();
        let now = Utc::now();

        let theme_data = CustomThemeData {
            id: id.clone(),
            name,
            description,
            css,
            created_at: now,
            updated_at: now,
            is_built_in: false,
        };

        self.custom_themes.insert(id.clone(), theme_data);
        self.last_updated = now;

        Ok(id)
    }
    pub fn update_theme(
        &mut self,
        id: &str,
        name: String,
        description: String,
        css: String,
    ) -> Result<(), String> {
        if name.trim().is_empty() {
            return Err("Theme name cannot be empty".to_string());
        }

        // Check for duplicate names (excluding current theme)
        for (theme_id, theme) in &self.custom_themes {
            if theme_id != id && theme.name.to_lowercase() == name.to_lowercase() {
                return Err("A theme with this name already exists".to_string());
            }
        }

        if let Some(theme) = self.custom_themes.get_mut(id) {
            theme.name = name;
            theme.description = description;
            theme.css = css;
            theme.updated_at = Utc::now();
            self.last_updated = Utc::now();
            Ok(())
        } else {
            Err("Theme not found".to_string())
        }
    }

    pub fn delete_theme(&mut self, id: &str) -> Result<(), String> {
        if self.custom_themes.remove(id).is_some() {
            self.last_updated = Utc::now();
            Ok(())
        } else {
            Err("Theme not found".to_string())
        }
    }

    pub fn get_theme_by_id(&self, id: &str) -> Option<&CustomThemeData> {
        self.custom_themes.get(id)
    }

    pub fn list_themes(&self) -> Vec<&CustomThemeData> {
        let mut themes: Vec<&CustomThemeData> = self.custom_themes.values().collect();
        themes.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
        // Sort by most recently updated
        themes
    }
}
