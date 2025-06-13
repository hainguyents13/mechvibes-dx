use crate::state::paths;
use crate::state::soundpack::SoundpackMetadata;
use crate::utils::config_converter;
use crate::utils::soundpack_validator::{ validate_soundpack_config, SoundpackValidationStatus };
use std::fs;

/// Load soundpack metadata from config.json
pub fn load_soundpack_metadata(soundpack_id: &str) -> Result<SoundpackMetadata, String> {
    let config_path = paths::soundpacks::config_json(soundpack_id);
    let mut last_error: Option<String> = None; // Validate the soundpack configuration first
    let validation_result = validate_soundpack_config(&config_path);
    println!(
        "🔍 [DEBUG] Validation result for {}: status={:?}, can_convert={}",
        soundpack_id,
        validation_result.status,
        validation_result.can_be_converted
    );

    // If it's a V1 config that can be converted, auto-convert it
    if
        validation_result.status == SoundpackValidationStatus::VersionOneNeedsConversion &&
        validation_result.can_be_converted
    {
        println!("🔄 Auto-converting V1 soundpack '{}' to V2 format", soundpack_id);

        // Create backup of original config
        let backup_path = format!("{}.v1.backup", config_path);
        if let Err(e) = fs::copy(&config_path, &backup_path) {
            let error_msg = format!("Failed to create backup for {}: {}", soundpack_id, e);
            println!("⚠️  {}", error_msg);
            last_error = Some(error_msg);
        }

        // Convert V1 to V2
        match config_converter::convert_v1_to_v2(&config_path, &config_path, None) {
            Ok(()) => {
                println!("✅ Successfully converted {} from V1 to V2", soundpack_id);
            }
            Err(e) => {
                let error_msg = format!("Failed to convert {} from V1 to V2: {}", soundpack_id, e);
                println!("❌ {}", error_msg);
                // Restore backup if conversion failed
                if fs::copy(&backup_path, &config_path).is_ok() {
                    println!("🔙 Restored original config from backup");
                }
                // Return error for conversion failure
                return Err(error_msg);
            }
        }
    }

    let content = fs
        ::read_to_string(&config_path)
        .map_err(|e| format!("Failed to read config: {}", e))?;

    let config: serde_json::Value = serde_json
        ::from_str(&content)
        .map_err(|e| format!("Failed to parse config: {}", e))?;

    let name = config
        .get("name")
        .and_then(|v| v.as_str())
        .unwrap_or(soundpack_id)
        .to_string();

    let version = config
        .get("version")
        .and_then(|v| v.as_str())
        .unwrap_or("1.0.0")
        .to_string();

    let tags = config
        .get("tags")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect()
        })
        .unwrap_or_default();

    // Re-validate after potential conversion
    let final_validation = validate_soundpack_config(&config_path);

    // Get file stats
    let metadata = fs
        ::metadata(&config_path)
        .map_err(|e| format!("Failed to get metadata: {}", e))?;

    Ok(SoundpackMetadata {
        id: soundpack_id.to_string(),
        name,
        author: config
            .get("author")
            .or_else(|| config.get("m_author"))
            .and_then(|v| v.as_str())
            .map(|s| s.to_string()),
        description: config
            .get("description")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string()),
        version,
        tags,
        icon: {
            // Generate dynamic asset URL
            if let Some(icon_filename) = config.get("icon").and_then(|v| v.as_str()) {
                let icon_path = format!(
                    "{}/{}",
                    paths::soundpacks::soundpack_dir(soundpack_id),
                    icon_filename
                );
                println!(
                    "🔍 Checking icon for {}: {} -> exists: {}",
                    soundpack_id,
                    icon_path,
                    std::path::Path::new(&icon_path).exists()
                );
                if std::path::Path::new(&icon_path).exists() {
                    // Generate dynamic asset URL instead of base64 data URI
                    let asset_url = format!("/soundpack-images/{}/{}", soundpack_id, icon_filename);
                    println!("✅ Generated asset URL for {}: {}", soundpack_id, asset_url);
                    Some(asset_url)
                } else {
                    println!("❌ Icon not found for {}, setting empty string", soundpack_id);
                    Some(String::new()) // Empty string if icon file not found
                }
            } else {
                println!("ℹ️  No icon specified for {}", soundpack_id);
                Some(String::new()) // Empty string if no icon specified
            }
        },
        soundpack_type: match config.get("soundpack_type").and_then(|v| v.as_str()) {
            Some("mouse") => crate::state::soundpack::SoundpackType::Mouse,
            _ => crate::state::soundpack::SoundpackType::Keyboard, // Default to keyboard
        },
        last_modified: metadata
            .modified()
            .unwrap_or(std::time::SystemTime::UNIX_EPOCH)
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs(),
        last_accessed: 0, // Will be updated when accessed
        // Validation fields
        config_version: final_validation.config_version,
        is_valid_v2: final_validation.is_valid_v2,
        validation_status: match final_validation.status {
            SoundpackValidationStatus::Valid => "valid".to_string(),
            SoundpackValidationStatus::InvalidVersion => "invalid_version".to_string(),
            SoundpackValidationStatus::InvalidStructure(_) => "invalid_structure".to_string(),
            SoundpackValidationStatus::MissingRequiredFields(_) => "missing_fields".to_string(),
            SoundpackValidationStatus::VersionOneNeedsConversion => {
                "v1_needs_conversion".to_string()
            }
        },
        can_be_converted: final_validation.can_be_converted,
        // Error tracking - clear error if we successfully loaded metadata
        last_error: last_error,
    })
}
