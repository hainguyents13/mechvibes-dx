use crate::state::paths;
use crate::{ debug_print, always_eprint };
use crate::utils::platform;
use crate::utils;
use crate::utils::constants::{ APP_NAME_DISPLAY, APP_VERSION };
use chrono::{ DateTime, Utc };
use serde::{ Deserialize, Serialize };
use std::fs;

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
pub struct CompatibilityInfo {
    pub min_os_version: String,
    pub supported_architectures: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AppPaths {
    pub config: String,
    pub themes: String,
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
    pub compatibility: CompatibilityInfo,
    pub paths: AppPaths,
    pub metadata: Metadata,
}

impl AppManifest {
    pub fn new() -> Self {
        Self {
            app: AppInfo {
                name: APP_NAME_DISPLAY.to_string(),
                version: APP_VERSION.to_string(),
                description: "Mechanical keyboard sound simulator".to_string(),
                build_date: Utc::now(),
                git_commit: option_env!("GIT_HASH").map(|s| s.to_string()),
                git_branch: "main".to_string(),
                build_type: platform::get_build_type(),
            },
            compatibility: CompatibilityInfo {
                min_os_version: platform::get_min_os_version(),
                supported_architectures: platform::get_supported_architectures(),
            },
            paths: AppPaths {
                config: paths::data::config_json().to_string_lossy().to_string(),
                themes: paths::data::themes_json().to_string_lossy().to_string(),
                soundpack_cache: paths::data::soundpack_cache_json().to_string_lossy().to_string(),
                soundpacks_dir: utils::path::get_soundpacks_dir_absolute(),
                data_dir: utils::path::get_data_dir_absolute(),
            },
            metadata: Metadata {
                created_at: Utc::now(),
                last_updated: Utc::now(),
                platform: platform::get_platform(),
            },
        }
    }

    pub fn load() -> Self {
        let manifest_path = paths::data::manifest_json();

        if manifest_path.exists() {
            match fs::read_to_string(&manifest_path) {
                Ok(content) =>
                    match serde_json::from_str::<AppManifest>(&content) {
                        Ok(manifest) => {
                            println!("✅ Loaded app manifest from {}", manifest_path.display());
                            manifest
                        }
                        Err(e) => {
                            eprintln!("❌ Failed to parse manifest.json: {}", e);
                            let new_manifest = Self::new();
                            if let Err(e) = new_manifest.save() {
                                eprintln!("❌ Failed to create new manifest: {}", e);
                            }
                            new_manifest
                        }
                    }
                Err(e) => {
                    always_eprint!("❌ Failed to read manifest.json: {}", e);
                    let new_manifest = Self::new();
                    if let Err(e) = new_manifest.save() {
                        always_eprint!("❌ Failed to create new manifest: {}", e);
                    }
                    new_manifest
                }
            }
        } else {
            debug_print!("📝 Creating new app manifest");
            let new_manifest = Self::new();
            if let Err(e) = new_manifest.save() {
                always_eprint!("❌ Failed to create manifest.json: {}", e);
            }
            new_manifest
        }
    }

    pub fn save(&self) -> Result<(), String> {
        // Ensure data directory exists
        if let Some(parent) = paths::data::manifest_json().parent() {
            if let Err(e) = fs::create_dir_all(parent) {
                return Err(format!("Failed to create data directory: {}", e));
            }
        }

        let contents = serde_json
            ::to_string_pretty(self)
            .map_err(|e| format!("Failed to serialize manifest: {}", e))?;
        fs
            ::write(paths::data::manifest_json(), contents)
            .map_err(|e| format!("Failed to write manifest file: {}", e))?;
        Ok(())
    }
}

impl Default for AppManifest {
    fn default() -> Self {
        Self::new()
    }
}
