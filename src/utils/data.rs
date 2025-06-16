use serde::{ Deserialize, Serialize };
/// Data serialization and file management utilities
use std::fs;
use std::path::Path;

/// Generic function to load JSON data from file
pub fn load_json_from_file<T>(file_path: &Path) -> Result<T, String>
    where T: for<'de> Deserialize<'de>
{
    let contents = fs
        ::read_to_string(file_path)
        .map_err(|e| format!("Failed to read file '{}': {}", file_path.display(), e))?;

    serde_json
        ::from_str::<T>(&contents)
        .map_err(|e| format!("Failed to parse JSON from '{}': {}", file_path.display(), e))
}

/// Generic function to save data as JSON to file
pub fn save_json_to_file<T>(data: &T, file_path: &Path) -> Result<(), String> where T: Serialize {
    // Ensure parent directory exists
    if let Some(parent) = file_path.parent() {
        fs
            ::create_dir_all(parent)
            .map_err(|e| format!("Failed to create directory '{}': {}", parent.display(), e))?;
    }

    let contents = serde_json
        ::to_string_pretty(data)
        .map_err(|e| format!("Failed to serialize data: {}", e))?;

    fs::write(file_path, contents).map_err(|e|
        format!("Failed to write file '{}': {}", file_path.display(), e)
    )
}
