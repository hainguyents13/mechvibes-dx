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

    /// Custom themes configuration file
    pub fn themes_json() -> PathBuf {
        get_app_root().join("data").join("themes.json")
    }

    /// Soundpack cache file
    pub fn soundpack_cache_json() -> PathBuf {
        get_app_root().join("data").join("soundpack_cache.json")
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

/// Legacy compatibility functions - delegate to new utility modules
pub mod utils {    // Delegate to new utility modules for backward compatibility
    pub fn count_soundpacks_by_type() -> (usize, usize) {
        crate::utils::path::count_soundpacks_by_type()
    }

    pub fn data_dir_exists() -> bool {
        crate::utils::path::data_dir_exists()
    }

    pub fn config_file_exists() -> bool {
        crate::utils::path::config_file_exists()
    }
    pub fn open_path(path_to_open: &str) -> Result<(), String> {
        crate::utils::path::open_path(path_to_open)
    }

    pub fn get_data_dir_absolute() -> String {
        crate::utils::path::get_data_dir_absolute()
    }

    pub fn get_config_file_absolute() -> String {
        crate::utils::path::get_config_file_absolute()
    }

    pub fn get_soundpacks_dir_absolute() -> String {
        crate::utils::path::get_soundpacks_dir_absolute()
    }
}
