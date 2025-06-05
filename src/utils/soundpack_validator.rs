use crate::utils::file_utils;
use serde_json::Value;

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
    let content = match file_utils::read_file_contents(config_path) {
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
    let _has_key_define_type = config.get("key_define_type").is_some();

    // Check for V2 indicators
    let has_defs = config.get("defs").is_some();
    let _has_source_field = config.get("source").is_some();
    let has_author = config.get("author").is_some();

    // Determine version based on structure
    if config_version == Some(2) {
        // Explicitly marked as V2, validate V2 structure
        validate_v2_structure(&config, config_version, package_version)
    } else if config_version == Some(1) || (has_defines && has_sound_field && !has_defs) {
        // Explicitly V1 or has V1 structure
        SoundpackValidationResult {
            status: SoundpackValidationStatus::VersionOneNeedsConversion,
            config_version: Some(1),
            detected_version: package_version,
            is_valid_v2: false,
            can_be_converted: true,
            message: "Version 1 soundpack detected, needs conversion to V2 format".to_string(),
        }
    } else if has_defs && has_author {
        // Looks like V2 but no explicit version
        validate_v2_structure(&config, None, package_version)
    } else {
        // Unknown or invalid structure
        let mut missing_fields = Vec::new();

        if !has_author {
            missing_fields.push("author".to_string());
        }

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
