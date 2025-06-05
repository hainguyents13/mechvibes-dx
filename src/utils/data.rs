use serde::{Deserialize, Serialize};
/// Data serialization and file management utilities
use std::fs;
use std::path::Path;

/// Generic function to load JSON data from file
pub fn load_json_from_file<T>(file_path: &Path) -> Result<T, String>
where
    T: for<'de> Deserialize<'de>,
{
    let contents = fs::read_to_string(file_path)
        .map_err(|e| format!("Failed to read file '{}': {}", file_path.display(), e))?;

    serde_json::from_str::<T>(&contents)
        .map_err(|e| format!("Failed to parse JSON from '{}': {}", file_path.display(), e))
}

/// Generic function to save data as JSON to file
pub fn save_json_to_file<T>(data: &T, file_path: &Path) -> Result<(), String>
where
    T: Serialize,
{
    // Ensure parent directory exists
    if let Some(parent) = file_path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create directory '{}': {}", parent.display(), e))?;
    }

    let contents = serde_json::to_string_pretty(data)
        .map_err(|e| format!("Failed to serialize data: {}", e))?;

    fs::write(file_path, contents)
        .map_err(|e| format!("Failed to write file '{}': {}", file_path.display(), e))
}

/// Check if a file exists and is readable
pub fn is_file_accessible(file_path: &Path) -> bool {
    file_path.exists() && file_path.is_file() && fs::metadata(file_path).is_ok()
}

/// Get file modification time as Unix timestamp
pub fn get_file_modified_time(file_path: &Path) -> Result<u64, String> {
    let metadata = fs::metadata(file_path).map_err(|e| {
        format!(
            "Failed to get metadata for '{}': {}",
            file_path.display(),
            e
        )
    })?;

    let modified = metadata.modified().map_err(|e| {
        format!(
            "Failed to get modification time for '{}': {}",
            file_path.display(),
            e
        )
    })?;

    Ok(modified
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs())
}

/// Ensure directory exists, create if it doesn't
pub fn ensure_dir_exists(dir_path: &Path) -> Result<(), String> {
    if !dir_path.exists() {
        fs::create_dir_all(dir_path)
            .map_err(|e| format!("Failed to create directory '{}': {}", dir_path.display(), e))?;
    }
    Ok(())
}

/// Backup file by creating a copy with .bak extension
pub fn backup_file(file_path: &Path) -> Result<(), String> {
    if file_path.exists() {
        let backup_path = file_path.with_extension(format!(
            "{}.bak",
            file_path.extension().unwrap_or_default().to_string_lossy()
        ));
        fs::copy(file_path, &backup_path).map_err(|e| {
            format!(
                "Failed to backup '{}' to '{}': {}",
                file_path.display(),
                backup_path.display(),
                e
            )
        })?;
    }
    Ok(())
}
