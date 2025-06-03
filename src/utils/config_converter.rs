use serde_json::{Map, Value};
use std::collections::HashMap;

/// Convert soundpack config from version 1 to version 2
/// 
/// Version 1 format:
/// - Uses "defines" field mapping numeric IDs to sound files
/// - Uses "key_define_type" field
/// - Uses "sound" field for main sound file
/// - Has numeric key mappings in "defines"
/// 
/// Version 2 format:
/// - Uses "defs" field mapping key names to [start, end] timing arrays
/// - Uses "source" field for main sound file  
/// - Has proper key name mappings in "defs"
/// - Has "config_version" field set to 2
/// - Has "mouse" field (defaults to false for keyboard)
/// - Has "author" field (required)
pub fn convert_v1_to_v2(v1_config_path: &str, output_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    // Read the V1 config
    let content = std::fs::read_to_string(v1_config_path)?;
    let config: Value = serde_json::from_str(&content)?;

    let mut converted_config = Map::new();

    // Copy basic fields
    if let Some(id) = config.get("id") {
        converted_config.insert("id".to_string(), id.clone());
    }

    if let Some(name) = config.get("name") {
        converted_config.insert("name".to_string(), name.clone());
    }

    // Add author field (required in V2) - use from config or default
    if let Some(author) = config.get("m_author") {
        converted_config.insert("author".to_string(), author.clone());
    } else if let Some(author) = config.get("author") {
        converted_config.insert("author".to_string(), author.clone());
    } else {
        converted_config.insert("author".to_string(), Value::String("Unknown".to_string()));
    }

    // Optional fields
    if let Some(description) = config.get("description") {
        converted_config.insert("description".to_string(), description.clone());
    }

    if let Some(version) = config.get("version") {
        converted_config.insert("version".to_string(), version.clone());
    }

    if let Some(tags) = config.get("tags") {
        converted_config.insert("tags".to_string(), tags.clone());
    }

    if let Some(keycap) = config.get("keycap") {
        converted_config.insert("keycap".to_string(), keycap.clone());
    }

    if let Some(icon) = config.get("icon") {
        converted_config.insert("icon".to_string(), icon.clone());
    }

    // Convert "sound" to "source"
    if let Some(sound) = config.get("sound") {
        converted_config.insert("source".to_string(), sound.clone());
    }

    // Add method field - determine from key_define_type or default to "single"
    if let Some(key_define_type) = config.get("key_define_type") {
        if key_define_type.as_str() == Some("multi") {
            converted_config.insert("method".to_string(), Value::String("multi".to_string()));
        } else {
            converted_config.insert("method".to_string(), Value::String("single".to_string()));
        }
    } else {
        converted_config.insert("method".to_string(), Value::String("single".to_string()));
    }

    // Copy includes_numpad if present
    if let Some(includes_numpad) = config.get("includes_numpad") {
        converted_config.insert("includes_numpad".to_string(), includes_numpad.clone());
    }

    // Add mouse field (defaults to false for keyboard soundpacks)
    converted_config.insert("mouse".to_string(), Value::Bool(false));

    // Add config_version field set to 2
    converted_config.insert("config_version".to_string(), Value::Number(serde_json::Number::from(2)));

    // Convert "defines" to "defs" with proper timing format
    let mut defs = Map::new();

    if let Some(defines) = config.get("defines").and_then(|d| d.as_object()) {
        // Key mapping for common virtual key codes to Web API key names
        let key_mappings = create_vk_to_web_key_mapping();

        for (vk_code, sound_file) in defines {
            if let Some(sound_file_str) = sound_file.as_str() {
                if sound_file_str.is_empty() || sound_file_str == "null" {
                    continue; // Skip empty or null entries
                }

                // Convert VK code to key name
                if let Ok(vk_num) = vk_code.parse::<u32>() {
                    if let Some(key_name) = key_mappings.get(&vk_num) {
                        // Create timing array [start, end] - for now, use default timing
                        // In a real implementation, you'd extract timing from the sound file
                        let timing = vec![
                            Value::Array(vec![Value::from(0.0), Value::from(100.0)])
                        ];
                        defs.insert(key_name.clone(), Value::Array(timing));
                    }
                }
            }
        }
    }

    converted_config.insert("defs".to_string(), Value::Object(defs.clone()));

    // Write the converted config
    let converted_json = serde_json::to_string_pretty(&converted_config)?;
    std::fs::write(output_path, converted_json)?;

    println!("✅ Successfully converted config from V1 to V2");
    println!("   Input: {}", v1_config_path);
    println!("   Output: {}", output_path);
    println!("   Converted {} key mappings", defs.len());

    Ok(())
}

/// Convert V1 config JSON to V2 config JSON in memory (returns JSON string)
pub fn convert_v1_to_v2_memory(v1_config_json: &str) -> Result<String, Box<dyn std::error::Error>> {
    let config: Value = serde_json::from_str(v1_config_json)?;

    let mut converted_config = Map::new();

    // Copy basic fields
    if let Some(id) = config.get("id") {
        converted_config.insert("id".to_string(), id.clone());
    }

    if let Some(name) = config.get("name") {
        converted_config.insert("name".to_string(), name.clone());
    }

    // Add author field (required in V2) - use from config or default
    if let Some(author) = config.get("m_author") {
        converted_config.insert("author".to_string(), author.clone());
    } else if let Some(author) = config.get("author") {
        converted_config.insert("author".to_string(), author.clone());
    } else {
        converted_config.insert("author".to_string(), Value::String("Unknown".to_string()));
    }

    // Optional fields
    if let Some(description) = config.get("description") {
        converted_config.insert("description".to_string(), description.clone());
    }

    if let Some(version) = config.get("version") {
        converted_config.insert("version".to_string(), version.clone());
    }

    if let Some(tags) = config.get("tags") {
        converted_config.insert("tags".to_string(), tags.clone());
    }

    if let Some(keycap) = config.get("keycap") {
        converted_config.insert("keycap".to_string(), keycap.clone());
    }

    if let Some(icon) = config.get("icon") {
        converted_config.insert("icon".to_string(), icon.clone());
    }

    // Convert "sound" to "source"
    if let Some(sound) = config.get("sound") {
        converted_config.insert("source".to_string(), sound.clone());
    }

    // Add method field - determine from key_define_type or default to "single"
    if let Some(key_define_type) = config.get("key_define_type") {
        if key_define_type.as_str() == Some("multi") {
            converted_config.insert("method".to_string(), Value::String("multi".to_string()));
        } else {
            converted_config.insert("method".to_string(), Value::String("single".to_string()));
        }
    } else {
        converted_config.insert("method".to_string(), Value::String("single".to_string()));
    }

    // Copy includes_numpad if present
    if let Some(includes_numpad) = config.get("includes_numpad") {
        converted_config.insert("includes_numpad".to_string(), includes_numpad.clone());
    }

    // Add mouse field (defaults to false for keyboard soundpacks)
    converted_config.insert("mouse".to_string(), Value::Bool(false));

    // Add config_version field set to 2
    converted_config.insert("config_version".to_string(), Value::Number(serde_json::Number::from(2)));

    // Convert "defines" to "defs" with proper timing format
    let mut defs = Map::new();

    if let Some(defines) = config.get("defines").and_then(|d| d.as_object()) {
        // Key mapping for common virtual key codes to Web API key names
        let key_mappings = create_vk_to_web_key_mapping();

        for (vk_code, sound_file) in defines {
            if let Some(sound_file_str) = sound_file.as_str() {
                if sound_file_str.is_empty() || sound_file_str == "null" {
                    continue; // Skip empty or null entries
                }

                // Convert VK code to key name
                if let Ok(vk_num) = vk_code.parse::<u32>() {
                    if let Some(key_name) = key_mappings.get(&vk_num) {
                        // Create timing array [start, end] - for now, use default timing
                        let timing = vec![
                            Value::Array(vec![Value::from(0.0), Value::from(100.0)])
                        ];
                        defs.insert(key_name.clone(), Value::Array(timing));
                    }
                }
            }
        }
    }

    converted_config.insert("defs".to_string(), Value::Object(defs));

    // Return the converted config as JSON string
    Ok(serde_json::to_string_pretty(&converted_config)?)
}

/// Create mapping from Windows Virtual Key codes to Web API key names
fn create_vk_to_web_key_mapping() -> HashMap<u32, String> {
    let mut mapping = HashMap::new();

    // Escape, Function keys
    mapping.insert(1, "Escape".to_string());
    mapping.insert(59, "F1".to_string());
    mapping.insert(60, "F2".to_string());
    mapping.insert(61, "F3".to_string());
    mapping.insert(62, "F4".to_string());
    mapping.insert(63, "F5".to_string());
    mapping.insert(64, "F6".to_string());
    mapping.insert(65, "F7".to_string());
    mapping.insert(66, "F8".to_string());
    mapping.insert(67, "F9".to_string());
    mapping.insert(68, "F10".to_string());
    mapping.insert(87, "F11".to_string());
    mapping.insert(88, "F12".to_string());

    // Number row
    mapping.insert(2, "Digit1".to_string());
    mapping.insert(3, "Digit2".to_string());
    mapping.insert(4, "Digit3".to_string());
    mapping.insert(5, "Digit4".to_string());
    mapping.insert(6, "Digit5".to_string());
    mapping.insert(7, "Digit6".to_string());
    mapping.insert(8, "Digit7".to_string());
    mapping.insert(9, "Digit8".to_string());
    mapping.insert(10, "Digit9".to_string());
    mapping.insert(11, "Digit0".to_string());
    mapping.insert(12, "Minus".to_string());
    mapping.insert(13, "Equal".to_string());
    mapping.insert(14, "Backspace".to_string());

    // Tab and top row
    mapping.insert(15, "Tab".to_string());
    mapping.insert(16, "KeyQ".to_string());
    mapping.insert(17, "KeyW".to_string());
    mapping.insert(18, "KeyE".to_string());
    mapping.insert(19, "KeyR".to_string());
    mapping.insert(20, "KeyT".to_string());
    mapping.insert(21, "KeyY".to_string());
    mapping.insert(22, "KeyU".to_string());
    mapping.insert(23, "KeyI".to_string());
    mapping.insert(24, "KeyO".to_string());
    mapping.insert(25, "KeyP".to_string());
    mapping.insert(26, "BracketLeft".to_string());
    mapping.insert(27, "BracketRight".to_string());
    mapping.insert(28, "Enter".to_string());

    // Caps Lock and home row
    mapping.insert(58, "CapsLock".to_string());
    mapping.insert(30, "KeyA".to_string());
    mapping.insert(31, "KeyS".to_string());
    mapping.insert(32, "KeyD".to_string());
    mapping.insert(33, "KeyF".to_string());
    mapping.insert(34, "KeyG".to_string());
    mapping.insert(35, "KeyH".to_string());
    mapping.insert(36, "KeyJ".to_string());
    mapping.insert(37, "KeyK".to_string());
    mapping.insert(38, "KeyL".to_string());
    mapping.insert(39, "Semicolon".to_string());
    mapping.insert(40, "Quote".to_string());
    mapping.insert(41, "Backquote".to_string());

    // Shift and bottom row
    mapping.insert(42, "ShiftLeft".to_string());
    mapping.insert(43, "Backslash".to_string());
    mapping.insert(44, "KeyZ".to_string());
    mapping.insert(45, "KeyX".to_string());
    mapping.insert(46, "KeyC".to_string());
    mapping.insert(47, "KeyV".to_string());
    mapping.insert(48, "KeyB".to_string());
    mapping.insert(49, "KeyN".to_string());
    mapping.insert(50, "KeyM".to_string());
    mapping.insert(51, "Comma".to_string());
    mapping.insert(52, "Period".to_string());
    mapping.insert(53, "Slash".to_string());
    mapping.insert(54, "ShiftRight".to_string());

    // Control keys
    mapping.insert(29, "ControlLeft".to_string());
    mapping.insert(56, "AltLeft".to_string());
    mapping.insert(57, "Space".to_string());
    mapping.insert(3640, "AltRight".to_string());
    mapping.insert(3613, "ControlRight".to_string());

    // Arrow keys
    mapping.insert(57416, "ArrowUp".to_string());
    mapping.insert(57424, "ArrowLeft".to_string());
    mapping.insert(57421, "ArrowDown".to_string());
    mapping.insert(57419, "ArrowRight".to_string());

    // Insert/Delete cluster
    mapping.insert(3597, "Insert".to_string());
    mapping.insert(3639, "Delete".to_string());
    mapping.insert(61001, "Home".to_string());
    mapping.insert(61007, "End".to_string());
    mapping.insert(61009, "PageUp".to_string());
    mapping.insert(61003, "PageDown".to_string());

    // Numpad
    mapping.insert(61010, "Numpad0".to_string());
    mapping.insert(61011, "Numpad1".to_string());
    mapping.insert(61000, "Numpad2".to_string());
    mapping.insert(61005, "Numpad3".to_string());
    mapping.insert(60999, "Numpad4".to_string());
    mapping.insert(61001, "Numpad5".to_string());
    mapping.insert(61003, "Numpad6".to_string());
    mapping.insert(61007, "Numpad7".to_string());
    mapping.insert(61008, "Numpad8".to_string());
    mapping.insert(61009, "Numpad9".to_string());
    mapping.insert(3677, "NumpadMultiply".to_string());
    mapping.insert(3675, "NumpadAdd".to_string());
    mapping.insert(3676, "NumpadSubtract".to_string());
    mapping.insert(3667, "NumpadDecimal".to_string());
    mapping.insert(3665, "NumpadDivide".to_string());
    mapping.insert(3612, "NumpadEnter".to_string());

    mapping
}
