/// Centralized path definitions
///
/// ## Path Structure
/// - `data/` - Application data and configuration files
/// - `soundpacks/` - Soundpack directories containing audio files and metadata
///
/// All paths are relative to the application root directory unless specified otherwise.

/// Application data directory paths
pub mod data {
    /// Main data directory
    pub const DIR: &str = "./data";

    /// Application configuration file
    pub const CONFIG_JSON: &str = "./data/config.json";

    /// Application manifest file  
    pub const MANIFEST_JSON: &str = "./data/manifest.json";
    /// Soundpack metadata cache file
    pub const SOUNDPACK_METADATA_CACHE_JSON: &str = "data/soundpack_metadata_cache.json";
}

/// Soundpack directory paths
pub mod soundpacks {
    /// Main soundpacks directory
    pub const DIR: &str = "./soundpacks";

    /// Get soundpack directory path for a specific soundpack ID
    pub fn soundpack_dir(soundpack_id: &str) -> String {
        format!("./soundpacks/{}", soundpack_id)
    }
    /// Get config.json path for a specific soundpack
    pub fn config_json(soundpack_id: &str) -> String {
        format!("./soundpacks/{}/config.json", soundpack_id)
    }
}

/// Utility functions for path operations
pub mod utils {
    use std::path::Path;

    /// Check if data directory exists
    pub fn data_dir_exists() -> bool {
        Path::new(super::data::DIR).exists()
    }

    /// Check if config file exists
    pub fn config_file_exists() -> bool {
        Path::new(super::data::CONFIG_JSON).exists()
    }

    /// Check if soundpacks directory exists
    pub fn soundpacks_dir_exists() -> bool {
        Path::new(super::soundpacks::DIR).exists()
    }

    /// Count soundpacks in the soundpacks directory
    pub fn count_soundpacks() -> usize {
        if soundpacks_dir_exists() {
            std::fs::read_dir(super::soundpacks::DIR)
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
}
