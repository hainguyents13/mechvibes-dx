use crate::utils::path;
use serde_json::Value;
use std::fs::File;
use std::io::Read;
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
    soundpacks: &[crate::state::soundpack::SoundpackMetadata]
) -> bool {
    soundpacks.iter().any(|pack| pack.id == id)
}

/// Extract soundpack ID from ZIP without extracting files
pub fn get_soundpack_id_from_zip(file_path: &str) -> Result<String, String> {
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

            // Extract ID from config content only
            let config: Value = serde_json
                ::from_str(&config_content)
                .map_err(|e| format!("Failed to parse config.json: {}", e))?;

            // Check if the config already contains an ID field
            if let Some(id) = config.get("id").and_then(|v| v.as_str()) {
                if !id.trim().is_empty() {
                    return Ok(id.to_string());
                }
            }

            // If no ID in config, generate a UUID-based ID
            return Ok(format!("imported-{}", Uuid::new_v4()));
        }
    }

    Err("No config.json found in ZIP file".to_string())
}

/// Extract and install soundpack from ZIP file
#[allow(dead_code)]
pub fn extract_and_install_soundpack(file_path: &str) -> Result<SoundpackInfo, String> {
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
    } // If no ID in config, generate a UUID-based ID
    if soundpack_id.is_empty() {
        soundpack_id = format!("imported-{}", Uuid::new_v4());

        // Add the generated ID to the config
        config["id"] = Value::String(soundpack_id.clone());
    } // Determine soundpack type from config
    let is_mouse_soundpack = determine_soundpack_type(&config);
    let soundpack_type = if is_mouse_soundpack { "mouse" } else { "keyboard" };

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
            let parts: Vec<&str> = file_path.split('/').collect();
            if parts.len() > 1 {
                // Skip the first part (directory) and join the rest directly to install_dir
                install_dir.join(parts[1..].join("/"))
            } else {
                install_dir.join(&file_path)
            }
        } else {
            install_dir.join(&file_path)
        };

        // Create parent directories if needed
        if let Some(parent) = output_path.parent() {
            path
                ::ensure_directory_exists(parent)
                .map_err(|e| format!("Failed to create directory: {}", e))?;
        }

        // Extract file
        let mut output_file = File::create(&output_path).map_err(|e|
            format!("Failed to create file: {}", e)
        )?;
        std::io
            ::copy(&mut file, &mut output_file)
            .map_err(|e| format!("Failed to extract file: {}", e))?;
    } // Now handle V1 to V2 conversion after files are extracted
    println!(
        "🔧 Converting config after file extraction - soundpack_dir: {}",
        install_dir.to_string_lossy()
    );
    let final_config_content = handle_config_conversion(
        &config.to_string(),
        &soundpack_id,
        &install_dir.to_string_lossy()
    )?; // Write the final config.json at the root level of the soundpack directory
    let config_path = install_dir.join("config.json");
    path
        ::write_file_contents(&config_path.to_string_lossy(), &final_config_content)
        .map_err(|e| format!("Failed to write config.json: {}", e))?;

    Ok(SoundpackInfo {
        name: soundpack_name,
        id: soundpack_id,
    })
}

/// Extract and install soundpack from ZIP file with specified target type
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
    let soundpack_type = if let Some(target) = target_type {
        match target {
            crate::state::soundpack::SoundpackType::Keyboard => "keyboard",
            crate::state::soundpack::SoundpackType::Mouse => "mouse",
        }
    } else {
        // Auto-detect from config
        let is_mouse_soundpack = determine_soundpack_type(&config);
        if is_mouse_soundpack {
            "mouse"
        } else {
            "keyboard"
        }
    };

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

    Ok(SoundpackInfo {
        name: soundpack_name,
        id: soundpack_id,
    })
}

/// Handle V1 to V2 config conversion if needed
#[allow(dead_code)]
fn handle_config_conversion(
    config_content: &str,
    soundpack_id: &str,
    soundpack_dir: &str
) -> Result<String, String> {
    // Write the config content to a temporary file so we can validate it
    let temp_validate_file = format!("temp_validate_{}.json", soundpack_id);
    std::fs
        ::write(&temp_validate_file, config_content)
        .map_err(|e| format!("Failed to write temp validation file: {}", e))?;

    let validation_result = crate::utils::soundpack_validator::validate_soundpack_config(
        &temp_validate_file
    );

    // Clean up temp validation file
    let _ = std::fs::remove_file(&temp_validate_file);

    let mut final_config_content = config_content.to_string();

    println!("⚒️ Soundpack validation result: {:?}", validation_result);

    if
        validation_result.status ==
        crate::utils::soundpack_validator::SoundpackValidationStatus::VersionOneNeedsConversion
    {
        println!("🔄 Converting V1 soundpack '{}' to V2 format during import", soundpack_id);

        // Create backup of the original V1 config before conversion
        let config_backup_path = std::path::Path::new(soundpack_dir).join("config.json.v1.backup");
        if let Err(e) = std::fs::write(&config_backup_path, config_content) {
            println!("⚠️ Failed to create V1 config backup for {}: {}", soundpack_id, e);
        } else {
            println!("💾 Created V1 config backup at: {}", config_backup_path.display());
        }

        // Convert V1 to V2 format
        let temp_input = format!("temp_v1_{}.json", soundpack_id);
        let temp_output = format!("temp_v2_{}.json", soundpack_id);

        std::fs
            ::write(&temp_input, config_content)
            .map_err(|e| format!("Failed to write temp config: {}", e))?;
        match
            crate::utils::config_converter::convert_v1_to_v2(
                &temp_input,
                &temp_output,
                Some(soundpack_dir)
            )
        {
            Ok(()) => {
                final_config_content = std::fs
                    ::read_to_string(&temp_output)
                    .map_err(|e| format!("Failed to read converted config: {}", e))?;

                println!("✅ Successfully converted {} from V1 to V2 during import", soundpack_id);

                // Clean up temp files
                let _ = std::fs::remove_file(&temp_input);
                let _ = std::fs::remove_file(&temp_output);
            }
            Err(e) => {
                // Clean up temp files on error
                let _ = std::fs::remove_file(&temp_input);
                let _ = std::fs::remove_file(&temp_output);

                // If conversion failed, also remove the backup we created
                let _ = std::fs::remove_file(&config_backup_path);

                return Err(format!("Failed to convert V1 soundpack: {}", e));
            }
        }
    }
    Ok(final_config_content)
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

/// Download a soundpack ZIP from a URL and install it
pub async fn download_and_install_soundpack(
    url: &str,
    target_type: Option<crate::state::soundpack::SoundpackType>
) -> Result<SoundpackInfo, String> {
    download_and_install_soundpack_with_progress(url, target_type, None::<fn(f64)>).await
}

/// Download a soundpack ZIP with real-time progress callbacks and install it
pub async fn download_and_install_soundpack_with_progress<F>(
    url: &str,
    target_type: Option<crate::state::soundpack::SoundpackType>,
    on_progress: Option<F>
) -> Result<SoundpackInfo, String>
where
    F: Fn(f64) + Send + Sync + 'static
{
    println!("📥 Downloading soundpack from {}...", url);
    let client = reqwest::Client::new();
    let mut response = client
        .get(url)
        .header("User-Agent", "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36")
        .send()
        .await
        .map_err(|e| format!("Network request failed: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("Download failed with status code: {}", response.status()));
    }

    let total_size = response.content_length();
    let mut bytes = Vec::new();
    if let Some(total) = total_size {
        bytes.reserve(total as usize);
    }

    let mut downloaded = 0;
    while let Some(chunk) = response.chunk().await.map_err(|e| format!("Failed to read chunk: {}", e))? {
        bytes.extend_from_slice(&chunk);
        downloaded += chunk.len();
        if let Some(total) = total_size {
            if total > 0 {
                let progress = (downloaded as f64 / total as f64) * 100.0;
                if let Some(ref cb) = on_progress {
                    cb(progress);
                }
            }
        }
    }

    // Save to temporary file in the workspace/system temp
    let temp_dir = std::env::temp_dir();
    let temp_file_path = temp_dir.join(format!("mechvibes_download_{}.zip", uuid::Uuid::new_v4()));
    std::fs::write(&temp_file_path, bytes)
        .map_err(|e| format!("Failed to write temporary file: {}", e))?;

    let file_path_str = temp_file_path.to_string_lossy().to_string();
    
    // Install using existing extractor
    let result = extract_and_install_soundpack_with_type(&file_path_str, target_type);

    // Clean up temporary file
    let _ = std::fs::remove_file(&temp_file_path);

    result
}
