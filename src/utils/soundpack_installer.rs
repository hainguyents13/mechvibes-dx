use crate::utils::path_utils;
use serde_json::Value;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use uuid::Uuid;
use zip::ZipArchive;

// Structure to hold soundpack information after extraction
#[derive(Debug, Clone)]
pub struct SoundpackInfo {
    pub name: String,
    pub id: String,
}

/// Check if a soundpack ID already exists in the app state
pub fn check_soundpack_id_conflict(
    id: &str,
    soundpacks: &[crate::state::soundpack::SoundpackMetadata],
) -> bool {
    soundpacks.iter().any(|pack| pack.id == id)
}

/// Extract soundpack ID from ZIP without extracting files
pub fn get_soundpack_id_from_zip(file_path: &str) -> Result<String, String> {
    let file = File::open(file_path).map_err(|e| format!("Failed to open ZIP file: {}", e))?;
    let mut archive =
        ZipArchive::new(file).map_err(|e| format!("Failed to read ZIP archive: {}", e))?;

    // Find config.json to determine soundpack ID
    for i in 0..archive.len() {
        let mut file = archive
            .by_index(i)
            .map_err(|e| format!("Failed to read archive entry: {}", e))?;
        let file_path = file.name().to_string();

        if file_path.ends_with("config.json") {
            let mut config_content = String::new();
            file.read_to_string(&mut config_content)
                .map_err(|e| format!("Failed to read config.json: {}", e))?;

            // Extract ID from config content only
            let config: Value = serde_json::from_str(&config_content)
                .map_err(|e| format!("Failed to parse config.json: {}", e))?; // Check if the config already contains an ID field
            if let Some(id) = config.get("id").and_then(|v| v.as_str()) {
                if !id.trim().is_empty() {
                    return Ok(id.to_string());
                }
            }

            // If no ID in config, generate a UUID-based ID
            return Ok(format!("imported_{}", Uuid::new_v4()));
        }
    }

    Err("No config.json found in ZIP file".to_string())
}

/// Extract and install soundpack from ZIP file
pub fn extract_and_install_soundpack(file_path: &str) -> Result<SoundpackInfo, String> {
    // Open ZIP file
    let file = File::open(file_path).map_err(|e| format!("Failed to open ZIP file: {}", e))?;
    let mut archive =
        ZipArchive::new(file).map_err(|e| format!("Failed to read ZIP archive: {}", e))?;

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
            file.read_to_string(&mut config_content)
                .map_err(|e| format!("Failed to read config.json: {}", e))?;
            found_config = true;
            break;
        }
    }

    if !found_config {
        return Err("No config.json found in ZIP file".to_string());
    }

    // Parse config to get soundpack info
    let mut config: Value = serde_json::from_str(&config_content)
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
        soundpack_id = format!("imported_{}", Uuid::new_v4());

        // Add the generated ID to the config
        config["id"] = Value::String(soundpack_id.clone());
    }

    // Handle V1 to V2 conversion if needed
    let final_config_content = handle_config_conversion(&config.to_string(), &soundpack_id)?;    // Determine installation directory using soundpack ID
    let soundpacks_dir = crate::state::paths::utils::get_soundpacks_dir_absolute();
    let install_dir = Path::new(&soundpacks_dir).join(&soundpack_id);    // Create installation directory
    path_utils::ensure_directory_exists(&install_dir.to_string_lossy())
        .map_err(|e| format!("Failed to create soundpack directory: {}", e))?;

    // Second pass: extract all files
    let mut archive =
        ZipArchive::new(File::open(file_path).map_err(|e| format!("Failed to reopen ZIP: {}", e))?)
            .map_err(|e| format!("Failed to reread ZIP archive: {}", e))?;

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
            let parts: Vec<&str> = file_path.split('/').collect();
            if parts.len() > 1 {
                // Skip the first part (directory) and join the rest directly to install_dir
                install_dir.join(parts[1..].join("/"))
            } else {
                install_dir.join(&file_path)
            }
        } else {
            install_dir.join(&file_path)
        };        // Create parent directories if needed
        if let Some(parent) = output_path.parent() {
            path_utils::ensure_directory_exists(&parent.to_string_lossy())
                .map_err(|e| format!("Failed to create directory: {}", e))?;
        }

        // Extract file
        let mut output_file =
            File::create(&output_path).map_err(|e| format!("Failed to create file: {}", e))?;
        std::io::copy(&mut file, &mut output_file)
            .map_err(|e| format!("Failed to extract file: {}", e))?;
    }    // Write the final config.json at the root level of the soundpack directory
    let config_path = install_dir.join("config.json");
    path_utils::write_file_contents(&config_path.to_string_lossy(), &final_config_content)
        .map_err(|e| format!("Failed to write config.json: {}", e))?;

    Ok(SoundpackInfo {
        name: soundpack_name,
        id: soundpack_id,
    })
}

/// Handle V1 to V2 config conversion if needed
fn handle_config_conversion(config_content: &str, soundpack_id: &str) -> Result<String, String> {
    let validation_result = crate::utils::soundpack_validator::validate_soundpack_config(&format!(
        "temp_config_{}.json",
        soundpack_id
    ));

    let mut final_config_content = config_content.to_string();

    if validation_result.status
        == crate::utils::soundpack_validator::SoundpackValidationStatus::VersionOneNeedsConversion
    {
        // Convert V1 to V2 format
        let temp_input = format!("temp_v1_{}.json", soundpack_id);
        let temp_output = format!("temp_v2_{}.json", soundpack_id);

        std::fs::write(&temp_input, config_content)
            .map_err(|e| format!("Failed to write temp config: {}", e))?;

        match crate::utils::config_converter::convert_v1_to_v2(&temp_input, &temp_output) {
            Ok(()) => {
                final_config_content = std::fs::read_to_string(&temp_output)
                    .map_err(|e| format!("Failed to read converted config: {}", e))?;

                // Clean up temp files
                let _ = std::fs::remove_file(&temp_input);
                let _ = std::fs::remove_file(&temp_output);
            }
            Err(e) => {
                // Clean up temp files on error
                let _ = std::fs::remove_file(&temp_input);
                let _ = std::fs::remove_file(&temp_output);
                return Err(format!("Failed to convert V1 soundpack: {}", e));
            }
        }
    }
    Ok(final_config_content)
}
