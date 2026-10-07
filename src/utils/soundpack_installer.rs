use crate::utils::path;
use serde_json::Value;
use std::fs::File;
use std::io::Read;
use std::path::{ Path, PathBuf };
use uuid::Uuid;
use zip::ZipArchive;

/// Read and parse the `config.json` inside a soundpack ZIP.
fn read_config_from_zip(file_path: &str) -> Result<Value, String> {
    let file = File::open(file_path).map_err(|e| format!("Failed to open ZIP file: {}", e))?;
    let mut archive = ZipArchive::new(file).map_err(|e|
        format!("Failed to read ZIP archive: {}", e)
    )?;

    // Find config.json to determine soundpack ID
    for i in 0..archive.len() {
        let mut file = archive
            .by_index(i)
            .map_err(|e| format!("Failed to read archive entry: {}", e))?;
        let file_path = file.name().to_string();

        if file_path.ends_with("config.json") {
            let mut config_content = String::new();
            file
                .read_to_string(&mut config_content)
                .map_err(|e| format!("Failed to read config.json: {}", e))?;

            let config: Value = serde_json
                ::from_str(&config_content)
                .map_err(|e| format!("Failed to parse config.json: {}", e))?;

            return Ok(config);
        }
    }

    Err("No config.json found in ZIP file".to_string())
}

/// The raw ID stored in `config.json`, generating one when it is absent or blank.
fn soundpack_id_from_config(config: &Value) -> String {
    match config.get("id").and_then(|v| v.as_str()) {
        Some(id) if !id.trim().is_empty() => id.to_string(),
        _ => format!("imported-{}", Uuid::new_v4()),
    }
}

/// Extract soundpack ID from ZIP without extracting files
pub fn get_soundpack_id_from_zip(file_path: &str) -> Result<String, String> {
    let config = read_config_from_zip(file_path)?;
    Ok(soundpack_id_from_config(&config))
}

/// Resolve the soundpack type exactly as the installer does.
fn resolve_soundpack_type(
    config: &Value,
    target_type: Option<crate::state::soundpack::SoundpackType>
) -> &'static str {
    if let Some(target) = target_type {
        match target {
            crate::state::soundpack::SoundpackType::Keyboard => "keyboard",
            crate::state::soundpack::SoundpackType::Mouse => "mouse",
        }
    } else if determine_soundpack_type(config) {
        "mouse"
    } else {
        "keyboard"
    }
}

/// Resolve the directory an import of `file_path` would install into.
///
/// Reads the ZIP's `config.json` and resolves the type the same way
/// `extract_and_install_soundpack_with_type` does, then joins the raw ID.
/// `base` is normally `get_custom_soundpacks_dir()`, so the returned path is
/// exactly what a repeated import would overwrite.
pub fn resolve_soundpack_install_dir(
    file_path: &str,
    target_type: Option<crate::state::soundpack::SoundpackType>,
    base: &Path
) -> Result<PathBuf, String> {
    let config = read_config_from_zip(file_path)?;
    let soundpack_id = soundpack_id_from_config(&config);
    let soundpack_type = resolve_soundpack_type(&config, target_type);
    Ok(base.join(soundpack_type).join(&soundpack_id))
}

/// Check whether installing `file_path` would overwrite an existing pack.
///
/// Resolves the destination directory the installer would use and reports
/// whether it already exists, so a repeated ID cannot silently replace it.
/// This also catches a directory left on disk that is not in the loaded list.
pub fn check_soundpack_conflict(
    file_path: &str,
    target_type: Option<crate::state::soundpack::SoundpackType>,
    base: &Path
) -> Result<bool, String> {
    Ok(resolve_soundpack_install_dir(file_path, target_type, base)?.exists())
}

/// Extract and install soundpack from ZIP file with specified target type
// Structure to hold soundpack information after extraction
#[derive(Debug, Clone)]
pub struct SoundpackInfo {
    pub name: String,
    pub id: String,
}

pub fn extract_and_install_soundpack_with_type(
    file_path: &str,
    target_type: Option<crate::state::soundpack::SoundpackType>
) -> Result<SoundpackInfo, String> {
    // Open ZIP file
    let file = File::open(file_path).map_err(|e| format!("Failed to open ZIP file: {}", e))?;
    let mut archive = ZipArchive::new(file).map_err(|e|
        format!("Failed to read ZIP archive: {}", e)
    )?;

    // Find config.json to determine soundpack info
    let mut config_content = String::new();
    let mut soundpack_id = String::new();
    let mut found_config = false;

    // First pass: find and read config.json
    for i in 0..archive.len() {
        let mut file = archive
            .by_index(i)
            .map_err(|e| format!("Failed to read archive entry: {}", e))?;
        let file_path = file.name().to_string();

        // Look for config.json in any directory level
        if file_path.ends_with("config.json") {
            file
                .read_to_string(&mut config_content)
                .map_err(|e| format!("Failed to read config.json: {}", e))?;
            found_config = true;
            break;
        }
    }

    if !found_config {
        return Err("No config.json found in ZIP file".to_string());
    }

    // Parse config to get soundpack info
    let mut config: Value = serde_json
        ::from_str(&config_content)
        .map_err(|e| format!("Failed to parse config.json: {}", e))?;

    let soundpack_name = config
        .get("name")
        .and_then(|v| v.as_str())
        .unwrap_or("Unknown Soundpack")
        .to_string();

    // Extract ID from config content only
    if let Some(id) = config.get("id").and_then(|v| v.as_str()) {
        if !id.trim().is_empty() {
            soundpack_id = id.to_string();
        }
    }

    // If no ID in config, generate a UUID-based ID
    if soundpack_id.is_empty() {
        soundpack_id = format!("imported-{}", Uuid::new_v4());
        // Add the generated ID to the config
        config["id"] = Value::String(soundpack_id.clone());
    }

    // Determine soundpack type - use target type if provided, otherwise auto-detect
    let soundpack_type = resolve_soundpack_type(&config, target_type);

    // Determine installation directory using soundpack type and ID
    // Custom soundpacks go to system app data directory
    let soundpacks_dir = crate::state::paths::soundpacks::get_custom_soundpacks_dir();
    let install_dir = soundpacks_dir.join(soundpack_type).join(&soundpack_id);

    // Create installation directory
    path
        ::ensure_directory_exists(&install_dir)
        .map_err(|e| format!("Failed to create soundpack directory: {}", e))?;

    // Extract all files
    let mut archive = ZipArchive::new(
        File::open(file_path).map_err(|e| format!("Failed to reopen ZIP: {}", e))?
    ).map_err(|e| format!("Failed to reread ZIP archive: {}", e))?;

    for i in 0..archive.len() {
        let mut file = archive
            .by_index(i)
            .map_err(|e| format!("Failed to read archive entry: {}", e))?;
        let file_path = file.name().to_string();

        // Skip directories
        if file_path.ends_with('/') {
            continue;
        }

        // Determine output path - strip the first directory level if it exists and place all files at root
        let output_path = if file_path.contains('/') {
            // Get the filename only (remove directory structure)
            let filename = file_path.split('/').last().unwrap_or(&file_path);
            install_dir.join(filename)
        } else {
            install_dir.join(&file_path)
        };

        // Create parent directory if needed
        if let Some(parent) = output_path.parent() {
            path
                ::ensure_directory_exists(parent)
                .map_err(|e| format!("Failed to create parent directory: {}", e))?;
        }

        // Extract file
        let mut output_file = File::create(&output_path).map_err(|e|
            format!("Failed to create output file: {}", e)
        )?;
        std::io
            ::copy(&mut file, &mut output_file)
            .map_err(|e| format!("Failed to extract file: {}", e))?;
    }

    // Write updated config.json with ID if it was generated
    let config_path = install_dir.join("config.json");
    let updated_config = serde_json
        ::to_string_pretty(&config)
        .map_err(|e| format!("Failed to serialize updated config: {}", e))?;
    std::fs
        ::write(&config_path, updated_config)
        .map_err(|e| format!("Failed to write updated config.json: {}", e))?;

    // Stamp the arrival so the lists can sort by it and badge it as new. The
    // key is `{type}/{id}`, the same folder_path the scanner records.
    let folder_path = format!("{}/{}", soundpack_type, soundpack_id);
    crate::state::config_writer::apply(|config| {
        crate::state::soundpack_library::mark_added(&mut config.soundpack_added_at, &folder_path);
    });

    Ok(SoundpackInfo {
        name: soundpack_name,
        id: soundpack_id,
    })
}

fn determine_soundpack_type(config: &serde_json::Value) -> bool {
    // Check for explicit type field
    if let Some(soundpack_type) = config.get("type") {
        if let Some(type_str) = soundpack_type.as_str() {
            return type_str == "mouse";
        }
    }

    // Check if defs contain mouse-specific keys
    if let Some(defs) = config.get("defs") {
        if let Some(defs_obj) = defs.as_object() {
            for key in defs_obj.keys() {
                if
                    key.starts_with("Mouse") ||
                    key.starts_with("Button") ||
                    key.starts_with("Wheel")
                {
                    return true;
                }
            }
        }
    }

    // Default to keyboard
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::soundpack::SoundpackType;
    use std::io::Write;

    const MOUSE_CONFIG: &str =
        r#"{"id":"custom-sound-pack-77777732","defs":{"MouseLeft":{"type":"single","file":"a.ogg"}}}"#;
    const KEYBOARD_CONFIG: &str =
        r#"{"id":"thocky","defs":{"KeyA":{"type":"single","file":"a.ogg"}}}"#;

    /// A unique scratch directory per test, so the real data dir is never read.
    fn scratch_dir() -> PathBuf {
        let dir = std::env::temp_dir().join(
            format!("mechvibes-soundpack-test-{}", Uuid::new_v4())
        );
        std::fs::create_dir_all(&dir).expect("scratch dir");
        dir
    }

    /// Build a ZIP in memory and write it to `dir`, returning its path.
    fn zip_with_config(dir: &Path, name: &str, config: &str) -> String {
        let zip_path = dir.join(name);
        let file = File::create(&zip_path).expect("zip file");
        let mut writer = zip::ZipWriter::new(file);
        writer
            .start_file("config.json", zip::write::SimpleFileOptions::default())
            .expect("start config entry");
        writer.write_all(config.as_bytes()).expect("write config");
        writer.finish().expect("finish zip");
        zip_path.to_string_lossy().to_string()
    }

    #[test]
    fn mouse_config_without_target_type_resolves_to_mouse_dir() {
        let base = scratch_dir();
        let pack_zip = zip_with_config(&base, "pack.zip", MOUSE_CONFIG);
        let install_dir = resolve_soundpack_install_dir(&pack_zip, None, &base).expect(
            "resolve"
        );
        assert_eq!(install_dir, base.join("mouse").join("custom-sound-pack-77777732"));
    }

    #[test]
    fn explicit_target_type_wins_for_mouse_and_keyboard() {
        let base = scratch_dir();
        let pack_zip = zip_with_config(&base, "pack.zip", MOUSE_CONFIG);

        let mouse_dir = resolve_soundpack_install_dir(
            &pack_zip,
            Some(SoundpackType::Mouse),
            &base
        ).expect("resolve mouse");
        assert_eq!(mouse_dir, base.join("mouse").join("custom-sound-pack-77777732"));

        let keyboard_dir = resolve_soundpack_install_dir(
            &pack_zip,
            Some(SoundpackType::Keyboard),
            &base
        ).expect("resolve keyboard");
        assert_eq!(keyboard_dir, base.join("keyboard").join("custom-sound-pack-77777732"));
    }

    #[test]
    fn keyboard_config_auto_detects_keyboard() {
        let base = scratch_dir();
        let pack_zip = zip_with_config(&base, "pack.zip", KEYBOARD_CONFIG);
        let install_dir = resolve_soundpack_install_dir(&pack_zip, None, &base).expect(
            "resolve"
        );
        assert_eq!(install_dir, base.join("keyboard").join("thocky"));
    }

    #[test]
    fn conflict_is_reported_only_when_destination_exists() {
        let base = scratch_dir();
        let pack_zip = zip_with_config(&base, "pack.zip", MOUSE_CONFIG);
        let install_dir = resolve_soundpack_install_dir(&pack_zip, None, &base).expect(
            "resolve"
        );

        assert!(!check_soundpack_conflict(&pack_zip, None, &base).expect("free check"));

        std::fs::create_dir_all(&install_dir).expect("existing pack dir");
        assert!(check_soundpack_conflict(&pack_zip, None, &base).expect("taken check"));
    }
}
