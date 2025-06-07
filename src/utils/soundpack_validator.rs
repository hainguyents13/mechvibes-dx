use serde_json::Value;
use std::fs::File;
use std::io::Read;
use uuid::Uuid;
use zip::ZipArchive;

#[derive(Debug, Clone, PartialEq)]
pub enum SoundpackValidationStatus {
    Valid,
    InvalidVersion,
    InvalidStructure(String),
    MissingRequiredFields(Vec<String>),
    VersionOneNeedsConversion,
}

#[derive(Debug, Clone)]
pub struct SoundpackValidationResult {
    pub status: SoundpackValidationStatus,
    pub config_version: Option<u32>,
    pub detected_version: Option<String>,
    pub is_valid_v2: bool,
    pub can_be_converted: bool,
    pub message: String,
}

/// Detect and validate soundpack configuration version and structure
pub fn validate_soundpack_config(config_path: &str) -> SoundpackValidationResult {
    // Try to read and parse the config file
    let content = match crate::utils::path::read_file_contents(config_path) {
        Ok(content) => content,
        Err(e) => {
            return SoundpackValidationResult {
                status: SoundpackValidationStatus::InvalidStructure(format!(
                    "Cannot read config file: {}",
                    e
                )),
                config_version: None,
                detected_version: None,
                is_valid_v2: false,
                can_be_converted: false,
                message: format!("Failed to read config file: {}", e),
            };
        }
    };

    let config: Value = match serde_json::from_str(&content) {
        Ok(config) => config,
        Err(e) => {
            return SoundpackValidationResult {
                status: SoundpackValidationStatus::InvalidStructure(format!("Invalid JSON: {}", e)),
                config_version: None,
                detected_version: None,
                is_valid_v2: false,
                can_be_converted: false,
                message: format!("Invalid JSON format: {}", e),
            };
        }
    };

    // Extract version information
    let config_version = config
        .get("config_version")
        .and_then(|v| v.as_u64())
        .map(|v| v as u32);
    let package_version = config
        .get("version")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    // Check for V1 indicators
    let has_defines = config.get("defines").is_some();
    let has_sound_field = config.get("sound").is_some();
    let has_method_field =
        config.get("method").is_some() || config.get("key_define_type").is_some();

    // Check for V2 indicators
    let has_defs = config.get("defs").is_some();
    let _has_source_field = config.get("source").is_some();
    let has_author = config.get("author").is_some();

    // Determine version based on structure
    if config_version == Some(2) {
        // Explicitly marked as V2, validate V2 structure
        validate_v2_structure(&config, config_version, package_version)
    } else if config_version == Some(1) || (has_defines && has_sound_field && !has_defs) {
        // Explicitly V1 or has V1 structure (defines + sound + no defs)
        SoundpackValidationResult {
            status: SoundpackValidationStatus::VersionOneNeedsConversion,
            config_version: Some(1),
            detected_version: package_version,
            is_valid_v2: false,
            can_be_converted: true,
            message: if has_method_field {
                "Version 1 soundpack with method field detected, needs conversion to V2 format"
                    .to_string()
            } else {
                "Version 1 soundpack detected, needs conversion to V2 format".to_string()
            },
        }
    } else if has_defs && has_author {
        // Looks like V2 but no explicit version
        validate_v2_structure(&config, None, package_version)
    } else {
        // Unknown or invalid structure
        let mut missing_fields = Vec::new();

        if !has_defs && !has_defines {
            missing_fields.push("defs or defines".to_string());
        }

        if !config.get("name").is_some() {
            missing_fields.push("name".to_string());
        }
        SoundpackValidationResult {
            status: SoundpackValidationStatus::MissingRequiredFields(missing_fields.clone()),
            config_version: config_version,
            detected_version: package_version,
            is_valid_v2: false,
            can_be_converted: has_defines && has_sound_field, // Can convert if it looks like V1
            message: format!("Missing required fields: {}", missing_fields.join(", ")),
        }
    }
}

/// Validate V2 soundpack structure
fn validate_v2_structure(
    config: &Value,
    config_version: Option<u32>,
    package_version: Option<String>,
) -> SoundpackValidationResult {
    let mut missing_fields = Vec::new();
    let mut issues = Vec::new();

    // Check required V2 fields
    if !config.get("name").is_some() {
        missing_fields.push("name".to_string());
    }

    if !config.get("author").is_some() && !config.get("m_author").is_some() {
        missing_fields.push("author".to_string());
    }

    if !config.get("defs").is_some() {
        missing_fields.push("defs".to_string());
    }

    // Validate defs structure
    if let Some(defs) = config.get("defs") {
        if let Some(defs_obj) = defs.as_object() {
            for (key, value) in defs_obj {
                if !value.is_array() {
                    issues.push(format!("Invalid defs entry for '{}': expected array", key));
                    continue;
                }

                if let Some(arr) = value.as_array() {
                    for (i, timing) in arr.iter().enumerate() {
                        if let Some(timing_arr) = timing.as_array() {
                            if timing_arr.len() != 2 {
                                issues.push(format!(
                                    "Invalid timing array for '{}[{}]': expected [start, end]",
                                    key, i
                                ));
                            }
                        } else {
                            issues.push(format!(
                                "Invalid timing entry for '{}[{}]': expected array",
                                key, i
                            ));
                        }
                    }
                }
            }
        } else {
            issues.push("defs field should be an object".to_string());
        }
    }

    // Check mouse field
    if let Some(mouse) = config.get("mouse") {
        if !mouse.is_boolean() {
            issues.push("mouse field should be boolean".to_string());
        }
    }

    // Determine final status
    if !missing_fields.is_empty() {
        SoundpackValidationResult {
            status: SoundpackValidationStatus::MissingRequiredFields(missing_fields.clone()),
            config_version,
            detected_version: package_version,
            is_valid_v2: false,
            can_be_converted: false,
            message: format!("Missing required V2 fields: {}", missing_fields.join(", ")),
        }
    } else if !issues.is_empty() {
        SoundpackValidationResult {
            status: SoundpackValidationStatus::InvalidStructure(issues.join("; ")),
            config_version,
            detected_version: package_version,
            is_valid_v2: false,
            can_be_converted: false,
            message: format!("V2 structure issues: {}", issues.join("; ")),
        }
    } else {
        SoundpackValidationResult {
            status: SoundpackValidationStatus::Valid,
            config_version: config_version.or(Some(2)), // Default to 2 if not specified but valid
            detected_version: package_version,
            is_valid_v2: true,
            can_be_converted: false,
            message: "Valid V2 soundpack configuration".to_string(),
        }
    }
}

/// Validate ZIP file structure and basic requirements
pub async fn validate_zip_file(file_path: &str) -> Result<(), String> {
    // Check if file exists and is readable
    if !std::path::Path::new(file_path).exists() {
        return Err("File does not exist".to_string());
    }

    // Check file extension
    if !file_path.to_lowercase().ends_with(".zip") {
        return Err("File must be a ZIP archive".to_string());
    }

    // Try to open as ZIP
    let file = File::open(file_path).map_err(|e| format!("Cannot open file: {}", e))?;

    let archive = ZipArchive::new(file).map_err(|e| format!("Invalid ZIP file: {}", e))?;

    // Check if ZIP is not empty
    if archive.len() == 0 {
        return Err("ZIP file is empty".to_string());
    }

    Ok(())
}

/// Validate soundpack structure within ZIP file and return soundpack name and config content
pub async fn validate_soundpack_structure(file_path: &str) -> Result<(String, String), String> {
    let file = File::open(file_path).map_err(|e| format!("Cannot open ZIP file: {}", e))?;

    let mut archive = ZipArchive::new(file).map_err(|e| format!("Invalid ZIP archive: {}", e))?;

    let mut config_found = false;
    let mut audio_found = false;
    let mut config_method = "single".to_string();
    let mut config_content = String::new();
    let mut soundpack_name = "Unknown".to_string();

    // Check for required files
    for i in 0..archive.len() {
        let mut file = archive
            .by_index(i)
            .map_err(|e| format!("Cannot read file in archive: {}", e))?;

        let file_name = file.name().to_string();

        // Check for config.json
        if file_name.ends_with("config.json") {
            file.read_to_string(&mut config_content)
                .map_err(|e| format!("Cannot read config.json: {}", e))?;
            config_found = true;

            // Parse config to get name
            if let Ok(config) = serde_json::from_str::<Value>(&config_content) {
                if let Some(name) = config.get("name").and_then(|v| v.as_str()) {
                    soundpack_name = name.to_string();
                }
                if let Some(method) = config.get("method").and_then(|v| v.as_str()) {
                    config_method = method.to_string();
                }
            }
        }

        // Check for audio files
        let file_lower = file_name.to_lowercase();
        if file_lower.ends_with(".ogg")
            || file_lower.ends_with(".wav")
            || file_lower.ends_with(".mp3")
            || file_lower.ends_with(".flac")
        {
            audio_found = true;
        }
    }

    if !config_found {
        return Err("No config.json found in soundpack".to_string());
    }

    // If single method, ensure at least one audio file is present
    if config_method == "single" && !audio_found {
        return Err("No audio files found in soundpack".to_string());
    }

    // Use existing validator to validate config content
    let temp_config_path = format!("temp_validate_{}.json", Uuid::new_v4());
    std::fs::write(&temp_config_path, &config_content)
        .map_err(|e| format!("Cannot write temp config: {}", e))?;

    let validation_result = validate_soundpack_config(&temp_config_path);

    // Clean up temp file
    let _ = std::fs::remove_file(&temp_config_path);

    // Check validation result
    match validation_result.status {
        SoundpackValidationStatus::Valid => Ok((soundpack_name, config_content)),
        SoundpackValidationStatus::VersionOneNeedsConversion => {
            Ok((soundpack_name, config_content)) // V1 is acceptable, will be converted
        }
        SoundpackValidationStatus::InvalidStructure(msg) => {
            Err(format!("Invalid soundpack structure: {}", msg))
        }
        SoundpackValidationStatus::MissingRequiredFields(fields) => {
            Err(format!("Missing required fields: {}", fields.join(", ")))
        }
        SoundpackValidationStatus::InvalidVersion => {
            Err("Invalid or unsupported soundpack version".to_string())
        }
    }
}
