/// Path and file system utility functions
use crate::state::paths;
use std::fs;
use std::process::Command;

/// Check if data directory exists
pub fn data_dir_exists() -> bool {
    paths::data::config_json().parent().unwrap().exists()
}

/// Check if config file exists
pub fn config_file_exists() -> bool {
    paths::data::config_json().exists()
}

/// Get absolute path for data directory
pub fn get_data_dir_absolute() -> String {
    paths::data::config_json().parent().unwrap().to_string_lossy().to_string()
}

/// Get absolute path for config file
pub fn get_config_file_absolute() -> String {
    paths::data::config_json().to_string_lossy().to_string()
}

/// Get absolute path for soundpacks directory (built-in soundpacks)
pub fn get_soundpacks_dir_absolute() -> String {
    paths::soundpacks::get_builtin_soundpacks_dir()
        .to_string_lossy()
        .to_string()
}

/// Get absolute path for custom soundpacks directory (system app data)
pub fn get_custom_soundpacks_dir_absolute() -> String {
    paths::soundpacks::get_custom_soundpacks_dir()
        .to_string_lossy()
        .to_string()
}

// ===== FILE SYSTEM UTILITIES =====

/// Open a path in the system file manager
pub fn open_path(path_to_open: &str) -> Result<(), String> {
    let result = if cfg!(target_os = "windows") {
        Command::new("explorer").arg(&path_to_open).spawn()
    } else if cfg!(target_os = "macos") {
        Command::new("open").arg(&path_to_open).spawn()
    } else {
        // Linux and other Unix-like systems
        Command::new("xdg-open").arg(&path_to_open).spawn()
    };

    match result {
        Ok(_) => Ok(()),
        Err(e) => Err(format!("Failed to open path: {}", e)),
    }
}

/// Check if a directory exists
pub fn directory_exists(path: &str) -> bool {
    std::path::Path::new(path).exists()
}

/// Create directory recursively if it doesn't exist
pub fn ensure_directory_exists(path: impl AsRef<std::path::Path>) -> Result<(), String> {
    let path_ref = path.as_ref();
    fs::create_dir_all(path_ref).map_err(|e| {
        format!(
            "Failed to create directory '{}': {}",
            path_ref.display(),
            e
        )
    })
}

/// Read file contents as string
pub fn read_file_contents(path: &str) -> Result<String, String> {
    fs::read_to_string(path).map_err(|e| format!("Failed to read file '{}': {}", path, e))
}

/// Write string contents to file
pub fn write_file_contents(path: &str, contents: &str) -> Result<(), String> {
    fs::write(path, contents).map_err(|e| format!("Failed to write file '{}': {}", path, e))
}

/// Copy a file to the custom images directory and return the asset URL
/// Returns a URL in the format: /custom-images/{filename}
pub fn copy_to_custom_images(source_path: &str) -> Result<String, String> {
    use std::path::Path;

    let source = Path::new(source_path);

    // Validate source file exists
    if !source.exists() {
        return Err(format!("Source file does not exist: {}", source_path));
    }

    // Get file extension
    let extension = source
        .extension()
        .and_then(|ext| ext.to_str())
        .ok_or_else(|| "File has no extension".to_string())?;

    // Generate unique filename using timestamp and original filename
    let original_filename = source
        .file_stem()
        .and_then(|name| name.to_str())
        .ok_or_else(|| "Invalid filename".to_string())?;

    // Normalize filename to safe characters (alphanumeric, dash, underscore)
    // This prevents issues with spaces/special characters in CSS URLs
    let safe_filename: String = original_filename
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect();

    // Generate timestamp, with fallback for systems with misconfigured clocks
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_else(|_| {
            // Fallback: use a random number if system time is before epoch
            use std::collections::hash_map::RandomState;
            use std::hash::{BuildHasher, Hash, Hasher};
            let mut hasher = RandomState::new().build_hasher();
            std::time::SystemTime::now().hash(&mut hasher);
            std::time::Duration::from_secs(hasher.finish())
        })
        .as_secs();

    let new_filename = format!("{}_{}.{}", safe_filename, timestamp, extension);

    // Ensure custom images directory exists
    let custom_images_dir = paths::data::custom_images_dir();
    ensure_directory_exists(&custom_images_dir)?;

    // Copy file to custom images directory
    let dest_path = custom_images_dir.join(&new_filename);
    fs::copy(source, &dest_path)
        .map_err(|e| format!("Failed to copy file: {}", e))?;

    // Return asset URL
    Ok(format!("/custom-images/{}", new_filename))
}
