/// Centralized path definitions
///
/// ## Path Structure
/// - `data/` - Application data and configuration files
/// - `soundpacks/` - Soundpack directories containing audio files and metadata
///
/// All paths are relative to the application executable directory unless specified otherwise.
use std::path::PathBuf;
use std::sync::OnceLock;

/// Get the application root directory (current working directory where data and soundpacks are located)
fn get_app_root() -> &'static PathBuf {
    static APP_ROOT: OnceLock<PathBuf> = OnceLock::new();
    APP_ROOT.get_or_init(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")))
}

/// Application data directory paths
pub mod data {
    use super::get_app_root;
    use std::path::PathBuf;

    /// Application configuration file
    pub fn config_json() -> PathBuf {
        get_app_root().join("data").join("config.json")
    }

    /// Application manifest file  
    pub fn manifest_json() -> PathBuf {
        get_app_root().join("data").join("manifest.json")
    }

    /// Soundpack metadata cache file
    pub fn soundpack_metadata_cache_json() -> PathBuf {
        get_app_root()
            .join("data")
            .join("soundpack_metadata_cache.json")
    }
}

/// Soundpack directory paths
pub mod soundpacks {
    use super::get_app_root;

    /// Get soundpack directory path for a specific soundpack ID
    pub fn soundpack_dir(soundpack_id: &str) -> String {
        get_app_root()
            .join("soundpacks")
            .join(soundpack_id)
            .to_string_lossy()
            .to_string()
    }

    /// Get config.json path for a specific soundpack
    pub fn config_json(soundpack_id: &str) -> String {
        get_app_root()
            .join("soundpacks")
            .join(soundpack_id)
            .join("config.json")
            .to_string_lossy()
            .to_string()
    }
}

/// Utility functions for path operations
pub mod utils {
    use super::get_app_root;

    /// Check if data directory exists
    pub fn data_dir_exists() -> bool {
        get_app_root().join("data").exists()
    }

    /// Check if config file exists
    pub fn config_file_exists() -> bool {
        get_app_root().join("data").join("config.json").exists()
    }

    /// Check if soundpacks directory exists
    pub fn soundpacks_dir_exists() -> bool {
        get_app_root().join("soundpacks").exists()
    }

    /// Count soundpacks in the soundpacks directory
    pub fn count_soundpacks() -> usize {
        let soundpacks_dir = get_app_root().join("soundpacks");
        if soundpacks_dir.exists() {
            std::fs::read_dir(&soundpacks_dir)
                .map(|entries| {
                    entries
                        .filter_map(|e| e.ok())
                        .filter(|e| e.path().is_dir())
                        .count()
                })
                .unwrap_or(0)
        } else {
            0
        }
    }

    /// Get absolute path for data directory
    pub fn get_data_dir_absolute() -> String {
        get_app_root().join("data").to_string_lossy().to_string()
    }

    /// Get absolute path for config file
    pub fn get_config_file_absolute() -> String {
        get_app_root()
            .join("data")
            .join("config.json")
            .to_string_lossy()
            .to_string()
    }

    /// Get absolute path for soundpacks directory
    pub fn get_soundpacks_dir_absolute() -> String {
        get_app_root()
            .join("soundpacks")
            .to_string_lossy()
            .to_string()
    }
}
