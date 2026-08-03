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

/// Save data as JSON, leaving either the old file or the complete new one at
/// `file_path` - never a partial document.
///
/// `fs::write` truncates first and then writes, so a crash, a full disk or a
/// power loss mid-write leaves a truncated file. For `config.json` that is not
/// a cosmetic problem: an unparseable config is what routes the load path into
/// the `.corrupt` rescue, costing the user every setting until they restore it
/// by hand. Writing a sibling temp file and renaming keeps the swap atomic on
/// both Windows (`MoveFileEx` with replace semantics, which `fs::rename` uses)
/// and POSIX.
///
/// The temp file is a sibling rather than in the system temp dir so the rename
/// stays within one filesystem, where it is atomic.
pub fn save_json_to_file_atomically<T>(data: &T, file_path: &Path) -> Result<(), String>
    where T: Serialize
{
    if let Some(parent) = file_path.parent() {
        fs
            ::create_dir_all(parent)
            .map_err(|e| format!("Failed to create directory '{}': {}", parent.display(), e))?;
    }

    let contents = serde_json
        ::to_string_pretty(data)
        .map_err(|e| format!("Failed to serialize data: {}", e))?;

    // Process id in the name so two copies of the app pointed at the same data
    // directory cannot write each other's temp file half-way through.
    let temp_path = {
        let mut name = file_path.as_os_str().to_os_string();
        name.push(format!(".{}.tmp", std::process::id()));
        std::path::PathBuf::from(name)
    };

    fs::write(&temp_path, contents).map_err(|e|
        format!("Failed to write file '{}': {}", temp_path.display(), e)
    )?;

    fs::rename(&temp_path, file_path).map_err(|e| {
        // Leaving the temp file behind would accumulate one per failed save.
        let _ = fs::remove_file(&temp_path);
        format!("Failed to replace file '{}': {}", file_path.display(), e)
    })
}
