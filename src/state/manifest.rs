use crate::state::paths;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

// User-defined configuration (app.config.json)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AppConfig {
    pub app: AppConfigInfo,
    pub compatibility: Compatibility,
    pub paths: AppPaths,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AppConfigInfo {
    pub name: String,
    pub version: String,
    pub description: String,
}

// Generated manifest (data/manifest.json)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AppInfo {
    pub name: String,
    pub version: String,
    pub description: String,
    pub build_date: DateTime<Utc>,
    pub git_commit: Option<String>,
    pub git_branch: String,
    pub build_type: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Compatibility {
    pub config_version: String,
    pub soundpack_version: String,
    pub cache_version: String,
    pub minimum_app_version: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AppPaths {
    pub config_file: String,
    pub soundpack_cache: String,
    pub soundpacks_dir: String,
    pub data_dir: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Metadata {
    pub created_at: DateTime<Utc>,
    pub last_updated: DateTime<Utc>,
    pub platform: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AppManifest {
    pub app: AppInfo,
    pub compatibility: Compatibility,
    pub paths: AppPaths,
    pub metadata: Metadata,
}

impl AppManifest {
    const MANIFEST_FILE: &'static str = paths::data::MANIFEST_JSON;
    const CONFIG_FILE: &'static str = "./app.config.json";

    pub fn load() -> Self {
        // Ensure data directory exists
        if let Err(_) = fs::create_dir_all(paths::data::DIR) {
            eprintln!("Warning: Could not create data directory");
        }

        // Load user configuration
        let config = Self::load_config();

        // Check if manifest exists and load it
        let manifest_path = PathBuf::from(Self::MANIFEST_FILE);
        if let Ok(contents) = fs::read_to_string(manifest_path) {
            match serde_json::from_str::<AppManifest>(&contents) {
                Ok(mut manifest) => {
                    // Update from config
                    manifest.app.name = config.app.name.clone();
                    manifest.app.version = config.app.version.clone();
                    manifest.app.description = config.app.description.clone();
                    manifest.compatibility = config.compatibility.clone();
                    manifest.paths = config.paths.clone();

                    // Update runtime info
                    manifest.metadata.last_updated = Utc::now();
                    manifest.metadata.platform = Self::get_platform();
                    manifest.app.build_type = if cfg!(debug_assertions) {
                        "debug".to_string()
                    } else {
                        "release".to_string()
                    };

                    // Save updated manifest
                    let _ = manifest.save();

                    println!(
                        "📋 Loaded app manifest: {} v{}",
                        manifest.app.name, manifest.app.version
                    );

                    manifest
                }
                Err(e) => {
                    eprintln!(
                        "Warning: Failed to parse manifest file: {}. Creating new.",
                        e
                    );
                    Self::create_from_config(config)
                }
            }
        } else {
            println!("📋 Creating new app manifest from config...");
            Self::create_from_config(config)
        }
    }

    fn load_config() -> AppConfig {
        let config_path = PathBuf::from(Self::CONFIG_FILE);
        if let Ok(contents) = fs::read_to_string(config_path) {
            match serde_json::from_str::<AppConfig>(&contents) {
                Ok(config) => {
                    println!("📋 Loaded app configuration from {}", Self::CONFIG_FILE);
                    config
                }
                Err(e) => {
                    eprintln!(
                        "Warning: Failed to parse config file: {}. Using default.",
                        e
                    );
                    Self::create_default_config()
                }
            }
        } else {
            println!("📋 Config file not found, creating default...");
            let config = Self::create_default_config();
            let _ = Self::save_config(&config);
            config
        }
    }

    fn create_default_config() -> AppConfig {
        AppConfig {
            app: AppConfigInfo {
                name: "MechvibesDX".to_string(),
                version: "0.1.0".to_string(),
                description: "Enhanced mechanical keyboard sound simulator".to_string(),
            },
            compatibility: Compatibility {
                config_version: "1.0".to_string(),
                soundpack_version: "1.0".to_string(),
                cache_version: "1.0".to_string(),
                minimum_app_version: "0.1.0".to_string(),
            },
            paths: AppPaths {
                config_file: paths::data::CONFIG_JSON.to_string(),
                soundpack_cache: paths::data::SOUNDPACK_METADATA_CACHE_JSON.to_string(),
                soundpacks_dir: paths::soundpacks::DIR.to_string(),
                data_dir: paths::data::DIR.to_string(),
            },
        }
    }

    fn save_config(config: &AppConfig) -> Result<(), String> {
        let contents = serde_json::to_string_pretty(config)
            .map_err(|e| format!("Failed to serialize config: {}", e))?;
        fs::write(Self::CONFIG_FILE, contents)
            .map_err(|e| format!("Failed to write config file: {}", e))?;
        Ok(())
    }

    fn create_from_config(config: AppConfig) -> Self {
        let manifest = Self {
            app: AppInfo {
                name: config.app.name,
                version: config.app.version,
                description: config.app.description,
                build_date: Utc::now(),
                git_commit: option_env!("GIT_HASH").map(|s| s.to_string()),
                git_branch: "main".to_string(),
                build_type: if cfg!(debug_assertions) {
                    "debug".to_string()
                } else {
                    "release".to_string()
                },
            },
            compatibility: config.compatibility,
            paths: config.paths,
            metadata: Metadata {
                created_at: Utc::now(),
                last_updated: Utc::now(),
                platform: Self::get_platform(),
            },
        };

        let _ = manifest.save();
        manifest
    }

    fn get_platform() -> String {
        if cfg!(target_os = "windows") {
            "windows".to_string()
        } else if cfg!(target_os = "macos") {
            "macos".to_string()
        } else if cfg!(target_os = "linux") {
            "linux".to_string()
        } else {
            "unknown".to_string()
        }
    }

    pub fn save(&self) -> Result<(), String> {
        let contents = serde_json::to_string_pretty(self)
            .map_err(|e| format!("Failed to serialize manifest: {}", e))?;
        fs::write(Self::MANIFEST_FILE, contents)
            .map_err(|e| format!("Failed to write manifest file: {}", e))?;
        Ok(())
    }

    #[allow(dead_code)]
    pub fn is_compatible_config(&self, config_version: &str) -> bool {
        self.compatibility.config_version == config_version
    }

    #[allow(dead_code)]
    pub fn is_compatible_soundpack(&self, soundpack_version: &str) -> bool {
        self.compatibility.soundpack_version == soundpack_version
    }

    #[allow(dead_code)]
    pub fn get_app_info(&self) -> String {
        format!(
            "{} v{} ({})",
            self.app.name, self.app.version, self.app.build_type
        )
    }
}

impl Default for AppManifest {
    fn default() -> Self {
        let config = AppConfig::default();
        Self::create_from_config(config)
    }
}

impl Default for AppConfig {
    fn default() -> Self {
        AppManifest::create_default_config()
    }
}
