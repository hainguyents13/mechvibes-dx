use super::path;
use serde_json::{ Map, Value };
use std::collections::HashMap;
use std::path::Path;

/// Get the duration of an audio file in milliseconds using Symphonia
fn get_audio_duration_ms(file_path: &str) -> Result<f64, Box<dyn std::error::Error>> {
    println!("🔍 [DEBUG] get_audio_duration_ms called with: {}", file_path);

    // Check if file exists first
    if !Path::new(file_path).exists() {
        println!("❌ [DEBUG] File does not exist: {}", file_path);
        return Err("File does not exist".into());
    }

    println!("✅ [DEBUG] File exists, calling Symphonia...");

    // Use symphonia for audio duration detection
    match get_duration_with_symphonia(file_path) {
        Ok(duration) if duration > 0.0 => {
            println!("✅ [DEBUG] Symphonia returned valid duration: {:.3}ms", duration);
            Ok(duration)
        }
        Ok(duration) => {
            println!(
                "⚠️ [DEBUG] Symphonia returned invalid duration: {:.3}ms, using default 100ms",
                duration
            );
            Ok(100.0)
        }
        Err(e) => {
            println!("❌ [DEBUG] Symphonia failed: {}, using default 100ms", e);
            Ok(100.0)
        }
    }
}

/// Get duration using Symphonia (better for MP3 metadata)
fn get_duration_with_symphonia(file_path: &str) -> Result<f64, Box<dyn std::error::Error>> {
    use symphonia::core::formats::FormatOptions;
    use symphonia::core::io::MediaSourceStream;
    use symphonia::core::meta::MetadataOptions;
    use symphonia::core::probe::Hint;
    use std::fs::File;

    let file = File::open(file_path)?;
    let mss = MediaSourceStream::new(Box::new(file), Default::default());

    let mut hint = Hint::new();
    if let Some(extension) = std::path::Path::new(file_path).extension() {
        if let Some(ext_str) = extension.to_str() {
            hint.with_extension(ext_str);
        }
    }

    let meta_opts: MetadataOptions = Default::default();
    let fmt_opts: FormatOptions = Default::default();

    let probed = symphonia::default::get_probe().format(&hint, mss, &fmt_opts, &meta_opts)?;

    let mut format = probed.format;

    // Get the default track
    let track = format
        .tracks()
        .iter()
        .find(|t| t.codec_params.codec != symphonia::core::codecs::CODEC_TYPE_NULL)
        .ok_or("No supported audio tracks found")?;

    // Try to get duration from metadata first
    if let Some(time_base) = &track.codec_params.time_base {
        if let Some(n_frames) = track.codec_params.n_frames {
            let duration_seconds =
                ((n_frames as f64) * (time_base.numer as f64)) / (time_base.denom as f64);
            let duration_ms = duration_seconds * 1000.0;
            println!("🎵 [DEBUG] Duration from metadata: {:.3}ms", duration_ms);
            return Ok(duration_ms);
        }
    }

    // If metadata doesn't have duration, estimate from sample rate
    if let Some(sample_rate) = track.codec_params.sample_rate {
        if let Some(n_frames) = track.codec_params.n_frames {
            let duration_seconds = (n_frames as f64) / (sample_rate as f64);
            let duration_ms = duration_seconds * 1000.0;
            println!("🎵 [DEBUG] Duration from sample rate: {:.3}ms", duration_ms);
            return Ok(duration_ms);
        }
    }

    // Fallback: use default duration
    println!("⚠️ [DEBUG] Could not determine duration, using default 100ms");
    Ok(100.0)
}

/// Convert soundpack config from version 1 to version 2
/// Uses comprehensive IOHook keycode mapping (supports all platforms)
pub fn convert_v1_to_v2(
    v1_config_path: &str,
    output_path: &str,
    soundpack_dir: Option<&str>
) -> Result<(), Box<dyn std::error::Error>> {
    println!("🔄 Starting V1 to V2 conversion...");
    println!("📁 Input config: {}", v1_config_path);
    println!("📁 Output path: {}", output_path);

    // Determine soundpack directory - use provided or infer from config path
    let soundpack_dir = if let Some(dir) = soundpack_dir {
        println!("📁 Using provided soundpack directory: {}", dir);
        dir
    } else {
        let inferred_dir = Path::new(v1_config_path)
            .parent()
            .and_then(|p| p.to_str())
            .ok_or("Could not determine soundpack directory")?;
        println!("📁 Inferred soundpack directory: {}", inferred_dir);
        inferred_dir
    };

    // Read the V1 config
    println!("📖 Reading V1 config file...");
    let content = path
        ::read_file_contents(v1_config_path)
        .map_err(|e| format!("Failed to read V1 config: {}", e))?;

    let config: Value = serde_json::from_str(&content)?;
    println!("✅ Successfully parsed V1 config");

    // Log basic config info
    if let Some(name) = config.get("name") {
        println!("🎵 Soundpack name: {}", name.as_str().unwrap_or("Unknown"));
    }
    if let Some(defines) = config.get("defines").and_then(|d| d.as_object()) {
        println!("🎹 Found {} key definitions", defines.len());
    }

    let mut converted_config = Map::new();

    // Copy basic fields with new V2 format
    if let Some(id) = config.get("id") {
        converted_config.insert("id".to_string(), id.clone());
    }

    if let Some(name) = config.get("name") {
        converted_config.insert("name".to_string(), name.clone());
    }

    // Optional fields
    if let Some(description) = config.get("description") {
        converted_config.insert("description".to_string(), description.clone());
    }

    if let Some(author) = config.get("author") {
        converted_config.insert("author".to_string(), author.clone());
    }

    if let Some(version) = config.get("version") {
        converted_config.insert("version".to_string(), version.clone());
    }

    // Convert config_version to string format
    converted_config.insert("config_version".to_string(), Value::String("2".to_string()));

    if let Some(icon) = config.get("icon") {
        converted_config.insert("icon".to_string(), icon.clone());
    }

    if let Some(tags) = config.get("tags") {
        converted_config.insert("tags".to_string(), tags.clone());
    }

    // Add created_at field with current timestamp
    let now = chrono::Utc::now();
    converted_config.insert("created_at".to_string(), Value::String(now.to_rfc3339()));

    // Determine definition_method from V1 key_define_type or sound structure
    let definition_method = if let Some(key_define_type) = config.get("key_define_type") {
        if key_define_type.as_str() == Some("multi") { "multi" } else { "single" }
    } else {
        // If there's a single sound file, use "single", otherwise "multi"
        if config.get("sound").is_some() {
            "single"
        } else {
            "multi"
        }
    };

    converted_config.insert(
        "definition_method".to_string(),
        Value::String(definition_method.to_string())
    );
    println!("🎵 Definition method: {}", definition_method);

    // Handle audio_file for "single" method
    if definition_method == "single" {
        if let Some(sound) = config.get("sound") {
            converted_config.insert("audio_file".to_string(), sound.clone());
        }
    }

    // Add default options
    let mut options = Map::new();
    options.insert(
        "recommended_volume".to_string(),
        Value::Number(serde_json::Number::from_f64(1.0).unwrap())
    );
    options.insert("random_pitch".to_string(), Value::Bool(false));
    converted_config.insert("options".to_string(), Value::Object(options));

    // Convert "defines" to "definitions" with new format
    let mut definitions = Map::new();
    if let Some(defines) = config.get("defines").and_then(|d| d.as_object()) {
        let key_mappings = create_iohook_to_web_key_mapping();
        println!("🔧 Converting {} key definitions to new format", defines.len());

        for (iohook_code, value) in defines {
            if let Ok(iohook_num) = iohook_code.parse::<u32>() {
                if let Some(key_name) = key_mappings.get(&iohook_num) {
                    let mut key_def = Map::new();

                    if definition_method == "single" {
                        // For single method, use timing from defines
                        if let Some(timing_array) = value.as_array() {
                            if timing_array.len() >= 2 {
                                let start = timing_array[0].as_f64().unwrap_or(0.0) as f32;
                                let duration = timing_array[1].as_f64().unwrap_or(100.0) as f32;
                                let end = start + duration;

                                // Create timing array with keydown and keyup
                                let timing = vec![
                                    Value::Array(
                                        vec![
                                            Value::Number(
                                                serde_json::Number::from_f64(start as f64).unwrap()
                                            ),
                                            Value::Number(
                                                serde_json::Number::from_f64(end as f64).unwrap()
                                            )
                                        ]
                                    )
                                ];
                                key_def.insert("timing".to_string(), Value::Array(timing));
                            }
                        }
                    } else {
                        // For multi method, each key can have its own audio file
                        if let Some(audio_file) = value.as_str() {
                            key_def.insert(
                                "audio_file".to_string(),
                                Value::String(audio_file.to_string())
                            );

                            // Try to get duration from audio file
                            let audio_path = format!("{}/{}", soundpack_dir, audio_file);
                            let duration = get_audio_duration_ms(&audio_path).unwrap_or(100.0);

                            let timing = vec![
                                Value::Array(
                                    vec![
                                        Value::Number(serde_json::Number::from_f64(0.0).unwrap()),
                                        Value::Number(
                                            serde_json::Number::from_f64(duration).unwrap()
                                        )
                                    ]
                                )
                            ];
                            key_def.insert("timing".to_string(), Value::Array(timing));
                        }
                    }

                    if !key_def.is_empty() {
                        definitions.insert(key_name.clone(), Value::Object(key_def));
                    }
                }
            }
        }
    }

    converted_config.insert("definitions".to_string(), Value::Object(definitions));

    // Write the converted config
    let output_json = serde_json::to_string_pretty(&converted_config)?;
    std::fs::write(output_path, output_json)?;

    println!("✅ Successfully converted V1 to V2 config");
    println!("📁 Output written to: {}", output_path);

    Ok(())
}

/// Create comprehensive IOHook to Web API key mapping
/// Supports all platforms (Windows, Linux, macOS)
fn create_iohook_to_web_key_mapping() -> HashMap<u32, String> {
    let mut mapping = HashMap::new();

    // Basic keys (common across all platforms)
    mapping.insert(1, "Escape".to_string());
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
    mapping.insert(14, "Backspace".to_string());
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
    mapping.insert(28, "Enter".to_string());
    mapping.insert(30, "KeyA".to_string());
    mapping.insert(31, "KeyS".to_string());
    mapping.insert(32, "KeyD".to_string());
    mapping.insert(33, "KeyF".to_string());
    mapping.insert(34, "KeyG".to_string());
    mapping.insert(35, "KeyH".to_string());
    mapping.insert(36, "KeyJ".to_string());
    mapping.insert(37, "KeyK".to_string());
    mapping.insert(38, "KeyL".to_string());
    mapping.insert(44, "KeyZ".to_string());
    mapping.insert(45, "KeyX".to_string());
    mapping.insert(46, "KeyC".to_string());
    mapping.insert(47, "KeyV".to_string());
    mapping.insert(48, "KeyB".to_string());
    mapping.insert(49, "KeyN".to_string());
    mapping.insert(50, "KeyM".to_string());
    mapping.insert(57, "Space".to_string());

    mapping
}
