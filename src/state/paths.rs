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
    /// soundpack_id format: "keyboard/Soundpack Name" or "mouse/Soundpack Name"
    pub fn soundpack_dir(soundpack_id: &str) -> String {
        get_app_root().join("soundpacks").join(soundpack_id).to_string_lossy().to_string()
    }

    /// Get config.json path for a specific soundpack
    /// soundpack_id format: "keyboard/Soundpack Name" or "mouse/Soundpack Name"
    pub fn config_json(soundpack_id: &str) -> String {
        get_app_root()
            .join("soundpacks")
            .join(soundpack_id)
            .join("config.json")
            .to_string_lossy()
            .to_string()
    }

    /// Get the base soundpacks directory (containing keyboard/ and mouse/ folders)
    pub fn get_soundpacks_dir() -> String {
        get_app_root().join("soundpacks").to_string_lossy().to_string()
    }

    /// Get keyboard soundpacks directory
    pub fn keyboard_soundpacks_dir() -> String {
        get_app_root().join("soundpacks").join("keyboard").to_string_lossy().to_string()
    }

    /// Get mouse soundpacks directory
    pub fn mouse_soundpacks_dir() -> String {
        get_app_root().join("soundpacks").join("mouse").to_string_lossy().to_string()
    }

    /// Ensure soundpack directories exist (keyboard and mouse)
    /// Creates the directories if they don't exist
    pub fn ensure_soundpack_directories() -> Result<(), std::io::Error> {
        use std::fs;

        let soundpacks_dir = get_app_root().join("soundpacks");
        let keyboard_dir = soundpacks_dir.join("keyboard");
        let mouse_dir = soundpacks_dir.join("mouse");

        // Create soundpacks directory if it doesn't exist
        if !soundpacks_dir.exists() {
            fs::create_dir_all(&soundpacks_dir)?;
            crate::debug_print!("📁 Created soundpacks directory: {}", soundpacks_dir.display());
        }

        // Create keyboard directory if it doesn't exist
        if !keyboard_dir.exists() {
            fs::create_dir_all(&keyboard_dir)?;
            crate::debug_print!(
                "⌨️ Created keyboard soundpacks directory: {}",
                keyboard_dir.display()
            );
        }

        // Create mouse directory if it doesn't exist
        if !mouse_dir.exists() {
            fs::create_dir_all(&mouse_dir)?;
            crate::debug_print!("🖱️ Created mouse soundpacks directory: {}", mouse_dir.display());
        }

        Ok(())
    }
}
