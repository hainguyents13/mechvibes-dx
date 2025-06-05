/// File system utility functions
use std::fs;
use std::process::Command;

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
