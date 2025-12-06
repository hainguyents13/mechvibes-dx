/// Centralized path definitions
///
/// ## Path Structure
/// - `data/` - Application data and configuration files (relative to app root)
/// - `soundpacks/` - Built-in soundpack directories (relative to app root)
/// - Custom soundpacks - Stored in system app data directory (e.g., %APPDATA%/Mechvibes/soundpacks)
/// - Custom images - Stored in system app data directory (e.g., %APPDATA%/Mechvibes/custom_images)
///
/// All paths are relative to the application executable directory unless specified otherwise.
use std::path::PathBuf;
use std::sync::OnceLock;

/// Get the application root directory (current working directory where data and soundpacks are located)
fn get_app_root() -> &'static PathBuf {
    static APP_ROOT: OnceLock<PathBuf> = OnceLock::new();
    APP_ROOT.get_or_init(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")))
}

/// Get the system app data directory for Mechvibes
/// Returns platform-specific app data directory:
/// - Windows: %APPDATA%/Mechvibes
/// - macOS: ~/Library/Application Support/Mechvibes
/// - Linux: ~/.local/share/mechvibes
fn get_system_app_data_dir() -> PathBuf {
    use directories::ProjectDirs;

    if let Some(proj_dirs) = ProjectDirs::from("com", "hainguyents13", "Mechvibes") {
        proj_dirs.data_dir().to_path_buf()
    } else {
        // Fallback to app root if system directories not available
        get_app_root().join("data")
    }
}

/// Application data directory paths
pub mod data {
    use super::{get_app_root, get_system_app_data_dir};
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

    /// Custom images directory for user-uploaded images
    /// Uses system app data directory (e.g., %APPDATA%/Mechvibes/custom_images on Windows)
    pub fn custom_images_dir() -> PathBuf {
        get_system_app_data_dir().join("custom_images")
    }
}

/// Soundpack directory paths
pub mod soundpacks {
    use super::{get_app_root, get_system_app_data_dir};
    use std::path::{Path, PathBuf};

    /// List of built-in soundpack IDs that ship with the app
    /// These are stored in the app root soundpacks directory
    pub const BUILTIN_SOUNDPACKS: &[&str] = &[
        "keyboard/cherrymx-black-abs",
        "keyboard/cherrymx-black-pbt",
        "keyboard/cherrymx-blue-abs",
        "keyboard/cherrymx-blue-pbt",
        "keyboard/cherrymx-brown-abs",
        "keyboard/cherrymx-brown-pbt",
        "keyboard/cherrymx-red-abs",
        "keyboard/cherrymx-red-pbt",
        "keyboard/eg-crystal-purple",
        "keyboard/eg-oreo",
        "keyboard/topre-purple-hybrid-pbt",
        "mouse/chat",
        "mouse/ping",
        "mouse/vibrate",
        "mouse/wooden",
    ];

    /// Check if a soundpack ID is a built-in soundpack
    pub fn is_builtin_soundpack(soundpack_id: &str) -> bool {
        BUILTIN_SOUNDPACKS.contains(&soundpack_id)
    }

    /// Get the base soundpacks directory for built-in soundpacks (app root)
    pub fn get_builtin_soundpacks_dir() -> PathBuf {
        get_app_root().join("soundpacks")
    }

    /// Get the base soundpacks directory for custom soundpacks (system app data)
    pub fn get_custom_soundpacks_dir() -> PathBuf {
        get_system_app_data_dir().join("soundpacks")
    }

    /// Get soundpack directory path for a specific soundpack ID
    /// Checks built-in location first, then custom location
    /// soundpack_id format: "keyboard/Soundpack Name" or "mouse/Soundpack Name"
    pub fn soundpack_dir(soundpack_id: &str) -> String {
        // Check if it's a built-in soundpack
        if is_builtin_soundpack(soundpack_id) {
            get_builtin_soundpacks_dir()
                .join(soundpack_id)
                .to_string_lossy()
                .to_string()
        } else {
            // Check custom location first
            let custom_path = get_custom_soundpacks_dir().join(soundpack_id);
            if custom_path.exists() {
                custom_path.to_string_lossy().to_string()
            } else {
                // Fallback to built-in location (for backwards compatibility)
                get_builtin_soundpacks_dir()
                    .join(soundpack_id)
                    .to_string_lossy()
                    .to_string()
            }
        }
    }

    /// Get config.json path for a specific soundpack
    /// soundpack_id format: "keyboard/Soundpack Name" or "mouse/Soundpack Name"
    pub fn config_json(soundpack_id: &str) -> String {
        Path::new(&soundpack_dir(soundpack_id))
            .join("config.json")
            .to_string_lossy()
            .to_string()
    }

    /// Get the base soundpacks directory (containing keyboard/ and mouse/ folders)
    /// Returns built-in soundpacks directory
    pub fn get_soundpacks_dir() -> String {
        get_builtin_soundpacks_dir().to_string_lossy().to_string()
    }

    /// Get keyboard soundpacks directory (built-in)
    pub fn keyboard_soundpacks_dir() -> String {
        get_builtin_soundpacks_dir()
            .join("keyboard")
            .to_string_lossy()
            .to_string()
    }

    /// Get mouse soundpacks directory (built-in)
    pub fn mouse_soundpacks_dir() -> String {
        get_builtin_soundpacks_dir()
            .join("mouse")
            .to_string_lossy()
            .to_string()
    }

    /// Get custom keyboard soundpacks directory (system app data)
    pub fn custom_keyboard_soundpacks_dir() -> String {
        get_custom_soundpacks_dir()
            .join("keyboard")
            .to_string_lossy()
            .to_string()
    }

    /// Get custom mouse soundpacks directory (system app data)
    pub fn custom_mouse_soundpacks_dir() -> String {
        get_custom_soundpacks_dir()
            .join("mouse")
            .to_string_lossy()
            .to_string()
    }

    /// Ensure soundpack directories exist (keyboard and mouse)
    /// Creates the directories if they don't exist
    pub fn ensure_soundpack_directories() -> Result<(), std::io::Error> {
        use std::fs;

        // Ensure built-in soundpack directories exist
        let builtin_soundpacks_dir = get_builtin_soundpacks_dir();
        let builtin_keyboard_dir = builtin_soundpacks_dir.join("keyboard");
        let builtin_mouse_dir = builtin_soundpacks_dir.join("mouse");

        if !builtin_soundpacks_dir.exists() {
            fs::create_dir_all(&builtin_soundpacks_dir)?;
            crate::debug_print!(
                "📁 Created built-in soundpacks directory: {}",
                builtin_soundpacks_dir.display()
            );
        }

        if !builtin_keyboard_dir.exists() {
            fs::create_dir_all(&builtin_keyboard_dir)?;
            crate::debug_print!(
                "⌨️ Created built-in keyboard soundpacks directory: {}",
                builtin_keyboard_dir.display()
            );
        }

        if !builtin_mouse_dir.exists() {
            fs::create_dir_all(&builtin_mouse_dir)?;
            crate::debug_print!(
                "🖱️ Created built-in mouse soundpacks directory: {}",
                builtin_mouse_dir.display()
            );
        }

        // Ensure custom soundpack directories exist
        let custom_soundpacks_dir = get_custom_soundpacks_dir();
        let custom_keyboard_dir = custom_soundpacks_dir.join("keyboard");
        let custom_mouse_dir = custom_soundpacks_dir.join("mouse");

        if !custom_soundpacks_dir.exists() {
            fs::create_dir_all(&custom_soundpacks_dir)?;
            crate::debug_print!(
                "📁 Created custom soundpacks directory: {}",
                custom_soundpacks_dir.display()
            );
        }

        if !custom_keyboard_dir.exists() {
            fs::create_dir_all(&custom_keyboard_dir)?;
            crate::debug_print!(
                "⌨️ Created custom keyboard soundpacks directory: {}",
                custom_keyboard_dir.display()
            );
        }

        if !custom_mouse_dir.exists() {
            fs::create_dir_all(&custom_mouse_dir)?;
            crate::debug_print!(
                "🖱️ Created custom mouse soundpacks directory: {}",
                custom_mouse_dir.display()
            );
        }

        Ok(())
    }
}
