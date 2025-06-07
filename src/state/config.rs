use crate::libs::theme::{ BuiltInTheme, Theme };
use crate::state::paths;
use crate::utils::{ data, path };
use chrono::{ DateTime, Utc };
use serde::{ Deserialize, Serialize };

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LogoCustomization {
    pub border_color: String,
    pub text_color: String,
    pub shadow_color: String,
    pub background_color: String,
    pub muted_background: String,
    pub dimmed_when_muted: bool,
}

impl Default for LogoCustomization {
    fn default() -> Self {
        Self {
            border_color: "var(--color-base-content)".to_string(),
            text_color: "var(--color-base-content)".to_string(),
            shadow_color: "var(--color-base-content)".to_string(),
            background_color: "var(--color-base-200)".to_string(),
            muted_background: "var(--color-base-300)".to_string(),
            dimmed_when_muted: false,
        }
    }
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
    pub enable_mouse_sound: bool, // Enable/disable mouse sounds specifically    // UI settings
    pub theme: Theme,
    pub custom_css: String, // Legacy field for existing custom CSS
    pub logo_customization: LogoCustomization,
    pub enable_logo_customization: bool, // Enable/disable logo customization panel

    // System settings
    pub auto_start: bool,
    pub show_notifications: bool,
}

impl AppConfig {
    pub fn load() -> Self {
        let config_path = paths::data::config_json();

        // Ensure data directory exists
        if let Some(parent) = config_path.parent() {
            if let Err(_) = path::ensure_directory_exists(&parent.to_string_lossy()) {
                eprintln!("Warning: Could not create data directory");
            }
        }

        // Load config from file, falling back to defaults if it doesn't exist or is invalid
        match data::load_json_from_file::<AppConfig>(&config_path) {
            Ok(mut config) => {
                // Sync auto_start with actual registry state
                let actual_auto_start = crate::utils::auto_startup::get_auto_startup_state();
                if config.auto_start != actual_auto_start {
                    println!(
                        "🔄 Syncing auto_start config with registry: {} -> {}",
                        config.auto_start,
                        actual_auto_start
                    );
                    config.auto_start = actual_auto_start;
                    let _ = config.save(); // Save the synced state
                }
                config
            }
            Err(e) => {
                eprintln!("Warning: Failed to load config file: {}. Using defaults.", e);
                let default_config = Self::default();
                let _ = default_config.save();
                default_config
            }
        }
    }

    pub fn save(&self) -> Result<(), String> {
        let config_path = paths::data::config_json();
        data::save_json_to_file(self, &config_path)
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
            mouse_volume: 1.0, // Default mouse volume to 100%
            enable_sound: true,
            enable_keyboard_sound: true, // Default keyboard sounds enabled
            enable_mouse_sound: true, // Default mouse sounds enabled
            theme: Theme::BuiltIn(BuiltInTheme::System), // Default to System theme
            custom_css: String::new(),
            logo_customization: LogoCustomization::default(),
            enable_logo_customization: false, // Default logo customization disabled
            auto_start: false,
            show_notifications: true,
        }
    }
}
