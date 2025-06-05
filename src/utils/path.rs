/// Path and file system utility functions
use crate::state::paths;
use serde::Deserialize;
use std::fs;
use std::io::Read;
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
    paths::data::config_json()
        .parent()
        .unwrap()
        .to_string_lossy()
        .to_string()
}

/// Get absolute path for config file
pub fn get_config_file_absolute() -> String {
    paths::data::config_json().to_string_lossy().to_string()
}

/// Get absolute path for soundpacks directory
pub fn get_soundpacks_dir_absolute() -> String {
    get_soundpacks_dir_path().to_string_lossy().to_string()
}

/// Get soundpacks directory path
fn get_soundpacks_dir_path() -> std::path::PathBuf {
    std::env::current_dir()
        .unwrap_or_else(|_| std::path::PathBuf::from("."))
        .join("soundpacks")
}

/// Count soundpacks in the soundpacks directory
pub fn count_soundpacks_by_type() -> (usize, usize) {
    let soundpacks_dir = get_soundpacks_dir_path();
    if !soundpacks_dir.exists() {
        return (0, 0);
    }

    let entries: Vec<_> = match fs::read_dir(&soundpacks_dir) {
        Ok(entries) => entries.filter_map(|e| e.ok()).collect(),
        Err(_) => return (0, 0),
    };

    let mut keyboard = 0;
    let mut mouse = 0;

    for entry in entries {
        let dir_path = entry.path();
        if !dir_path.is_dir() {
            continue;
        }

        let config_path = dir_path.join("config.json");
        if !config_path.exists() {
            keyboard += 1; // Assume keyboard if no config
            continue;
        }

        if let Ok(mut file) = fs::File::open(&config_path) {
            let mut contents = String::new();
            if file.read_to_string(&mut contents).is_ok() {
                #[derive(Deserialize)]
                struct Config {
                    mouse: Option<bool>,
                }

                if let Ok(cfg) = serde_json::from_str::<Config>(&contents) {
                    match cfg.mouse {
                        Some(true) => mouse += 1,
                        Some(false) | None => keyboard += 1,
                    }
                }
            }
        }
    }

    (keyboard, mouse)
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

/// Check if a file exists
pub fn file_exists(path: &str) -> bool {
    std::path::Path::new(path).is_file()
}

/// Create directory recursively if it doesn't exist
pub fn ensure_directory_exists(path: &str) -> Result<(), String> {
    fs::create_dir_all(path).map_err(|e| format!("Failed to create directory '{}': {}", path, e))
}

/// Read file contents as string
pub fn read_file_contents(path: &str) -> Result<String, String> {
    fs::read_to_string(path).map_err(|e| format!("Failed to read file '{}': {}", path, e))
}

/// Write string contents to file
pub fn write_file_contents(path: &str, contents: &str) -> Result<(), String> {
    fs::write(path, contents).map_err(|e| format!("Failed to write file '{}': {}", path, e))
}

/// Get file extension from path
pub fn get_file_extension(path: &str) -> Option<String> {
    std::path::Path::new(path)
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext.to_lowercase())
}

/// Join path components
pub fn join_paths(base: &str, component: &str) -> String {
    std::path::Path::new(base)
        .join(component)
        .to_string_lossy()
        .to_string()
}
