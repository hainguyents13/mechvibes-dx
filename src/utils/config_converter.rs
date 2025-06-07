use crate::utils::path;
use serde_json::{Map, Value};
use std::collections::HashMap;
use std::fs::File;
use std::path::Path;

/// Get the duration of an audio file in milliseconds
fn get_audio_duration_ms(file_path: &str) -> Result<f64, Box<dyn std::error::Error>> {
    use rodio::{Decoder, Source};
    use std::io::BufReader;

    let file = File::open(file_path)?;
    let reader = BufReader::new(file);
    let source = Decoder::new(reader)?;

    // Get duration from the source
    if let Some(duration) = source.total_duration() {
        Ok(duration.as_millis() as f64)
    } else {
        // Fallback: try to estimate by counting samples
        let _sample_rate = source.sample_rate() as f64;
        let _channels = source.channels() as f64;

        Err("Could not determine audio duration".into())
    }
}

/// Create concatenated audio file from multiple sound files and return segment mappings
fn create_concatenated_audio_and_segments(
    soundpack_dir: &str,
    sound_files: &HashMap<String, String>, // key_name -> sound_file_name
    output_audio_path: &str,
) -> Result<HashMap<String, (f64, f64)>, Box<dyn std::error::Error>> {
    use std::collections::BTreeSet;

    let mut segments = HashMap::new();

    println!(
        "🔗 Processing {} sound files for multi-method conversion",
        sound_files.len()
    );

    // Step 1: Collect unique sound files and their durations
    let mut unique_files: BTreeSet<String> = BTreeSet::new();
    let mut file_durations: HashMap<String, f64> = HashMap::new();

    // First pass: collect all unique sound files
    for (_, sound_file) in sound_files {
        if !sound_file.is_empty() && sound_file != "null" {
            unique_files.insert(sound_file.clone());
        }
    }

    println!(
        "📁 Found {} unique audio files to concatenate",
        unique_files.len()
    );

    // Step 2: Read durations for all unique files
    for sound_file in &unique_files {
        let file_path = format!("{}/{}", soundpack_dir, sound_file);

        if !Path::new(&file_path).exists() {
            println!("⚠️ File not found: {}, using default duration", file_path);
            file_durations.insert(sound_file.clone(), 100.0);
            continue;
        }

        match get_audio_duration_ms(&file_path) {
            Ok(duration) => {
                file_durations.insert(sound_file.clone(), duration);
                println!("📄 File '{}' -> {:.1}ms", sound_file, duration);
            }
            Err(_) => {
                file_durations.insert(sound_file.clone(), 100.0);
                println!(
                    "⚠️ Could not read duration for {}, using default 100ms",
                    file_path
                );
            }
        }
    }

    // Step 3: Create concatenated audio file and calculate segment positions
    let mut current_position = 0.0;
    let mut file_segments: HashMap<String, (f64, f64)> = HashMap::new();

    // Try to create concatenated audio using simple file concatenation approach
    if let Err(e) = create_concatenated_audio_file(
        soundpack_dir,
        &unique_files,
        &file_durations,
        output_audio_path,
    ) {
        println!("⚠️ Failed to create concatenated audio: {}", e);
        println!("📁 Falling back to using first available file");

        // Fallback: copy the first available file
        if let Some(first_file) = unique_files.iter().next() {
            let first_file_path = format!("{}/{}", soundpack_dir, first_file);
            if Path::new(&first_file_path).exists() {
                std::fs::copy(&first_file_path, output_audio_path)?;
                println!("📁 Copied '{}' as main audio source", first_file);
            }
        }

        // Use individual file approach as fallback
        for (key_name, sound_file) in sound_files {
            if let Some(&duration) = file_durations.get(sound_file) {
                segments.insert(key_name.clone(), (0.0, duration));
            }
        }
    } else {
        // Step 4: Calculate segment positions in concatenated file
        for sound_file in &unique_files {
            if let Some(&duration) = file_durations.get(sound_file) {
                file_segments.insert(sound_file.clone(), (current_position, duration));
                println!(
                    "🎵 Segment '{}': [{:.1}ms - {:.1}ms] (duration: {:.1}ms)",
                    sound_file,
                    current_position,
                    current_position + duration,
                    duration
                );
                current_position += duration;
            }
        }

        // Step 5: Map keys to their corresponding segments
        for (key_name, sound_file) in sound_files {
            if let Some(&(start, duration)) = file_segments.get(sound_file) {
                segments.insert(key_name.clone(), (start, duration));
            } else {
                // Fallback for missing files
                segments.insert(key_name.clone(), (0.0, 100.0));
            }
        }

        println!(
            "✅ Created concatenated audio file with {:.1}ms total duration",
            current_position
        );
    }

    println!(
        "✅ Created {} key segments from {} unique files",
        segments.len(),
        unique_files.len()
    );
    Ok(segments)
}

/// Create a concatenated audio file from multiple source files using WAV format
fn create_concatenated_audio_file(
    soundpack_dir: &str,
    unique_files: &std::collections::BTreeSet<String>,
    _file_durations: &HashMap<String, f64>,
    output_path: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    use rodio::{Decoder, Source};
    use std::fs::File;
    use std::io::BufReader;

    // Collect all audio samples from all files
    let mut all_samples: Vec<f32> = Vec::new();
    let mut sample_rate = 44100u32;
    let mut channels = 2u16;

    println!("🎵 Starting audio file concatenation...");

    // Process each unique file
    for (index, sound_file) in unique_files.iter().enumerate() {
        let file_path = format!("{}/{}", soundpack_dir, sound_file);

        if !Path::new(&file_path).exists() {
            println!("⚠️ Skipping missing file: {}", file_path);
            continue;
        }

        match File::open(&file_path) {
            Ok(audio_file) => {
                match Decoder::new(BufReader::new(audio_file)) {
                    Ok(source) => {
                        // Set sample rate and channels from first file
                        if index == 0 {
                            sample_rate = source.sample_rate();
                            channels = source.channels();
                            println!(
                                "📊 Using audio format: {}Hz, {} channels",
                                sample_rate, channels
                            );
                        }
                        // Convert to f32 samples and collect
                        let samples: Vec<f32> = source.convert_samples().collect();
                        let sample_count = samples.len();
                        all_samples.extend(samples);

                        println!("✅ Added {} samples from '{}'", sample_count, sound_file);
                    }
                    Err(e) => {
                        println!("⚠️ Failed to decode {}: {}", sound_file, e);
                        return Err(format!("Failed to decode {}: {}", sound_file, e).into());
                    }
                }
            }
            Err(e) => {
                println!("⚠️ Failed to open {}: {}", file_path, e);
                return Err(format!("Failed to open {}: {}", file_path, e).into());
            }
        }
    }

    if all_samples.is_empty() {
        return Err("No audio samples were collected".into());
    }

    // Write concatenated samples to WAV file using hound
    let spec = hound::WavSpec {
        channels,
        sample_rate,
        bits_per_sample: 32,
        sample_format: hound::SampleFormat::Float,
    };
    let mut writer = hound::WavWriter::create(output_path, spec)?;
    let total_samples = all_samples.len();

    for sample in &all_samples {
        writer.write_sample(*sample)?;
    }

    writer.finalize()?;

    println!(
        "🎵 Successfully created concatenated audio file: {}",
        output_path
    );
    println!(
        "📊 Total samples: {}, Duration: ~{:.1}ms",
        total_samples,
        (total_samples as f64 / channels as f64) / sample_rate as f64 * 1000.0
    );

    Ok(())
}

/// Convert soundpack config from version 1 to version 2
///
/// Version 1 format:
/// - Uses "defines" field mapping numeric IDs to sound files or timing arrays
/// - Uses "key_define_type" or "method" field to specify data format:
///   - "multi": defines contains sound file names (string values)
///   - "single": defines contains [keydown_start_ms, keydown_duration_ms] arrays
/// - Uses "sound" field for main sound file
/// - Has numeric key mappings in "defines"
///
/// Version 2 format:
/// - Uses "defs" field mapping key names to timing arrays
/// - "multi" method: [[start, duration]] timing arrays (calculated from sound file durations)
/// - "single" method: [[keydown_start, keydown_duration]] timing arrays (preserved format)
/// - Uses "source" field for main sound file  
/// - Has proper key name mappings in "defs"
/// - Has "config_version" field set to 2
/// - Has "mouse" field (defaults to false for keyboard)
/// - Has "author" field (required)
pub fn convert_v1_to_v2(
    v1_config_path: &str,
    output_path: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    // Get the directory containing the V1 config (for relative sound file paths)
    let soundpack_dir = Path::new(v1_config_path)
        .parent()
        .and_then(|p| p.to_str())
        .ok_or("Could not determine soundpack directory")?;

    // Read the V1 config
    let content = path::read_file_contents(v1_config_path)
        .map_err(|e| format!("Failed to read V1 config: {}", e))?;
    let config: Value = serde_json::from_str(&content)?;

    let mut converted_config = Map::new();

    // Copy basic fields
    if let Some(id) = config.get("id") {
        converted_config.insert("id".to_string(), id.clone());
    }

    if let Some(name) = config.get("name") {
        converted_config.insert("name".to_string(), name.clone());
    }

    // Add author field (required in V2) - use from config or default to "N/A"
    if let Some(author) = config.get("author") {
        converted_config.insert("author".to_string(), author.clone());
    } else {
        converted_config.insert("author".to_string(), Value::String("N/A".to_string()));
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
    // Convert "sound" to "source" (only if not already set by multi method processing)
    if !converted_config.contains_key("source") {
        if let Some(sound) = config.get("sound") {
            converted_config.insert("source".to_string(), sound.clone());
        }
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
    converted_config.insert(
        "config_version".to_string(),
        Value::Number(serde_json::Number::from(2)),
    );

    // Convert "defines" to "defs" with proper timing format based on method
    let mut defs = Map::new();

    if let Some(defines) = config.get("defines").and_then(|d| d.as_object()) {
        // Key mapping for common virtual key codes to Web API key names
        let key_mappings = create_vk_to_web_key_mapping();

        // Determine the method to understand the data format in defines
        let method = config
            .get("method")
            .or_else(|| config.get("key_define_type"))
            .and_then(|v| v.as_str())
            .unwrap_or("single");

        if method == "multi" {
            // Multi method: collect all sound files and create segments
            let mut sound_files = HashMap::new();

            // First pass: collect all unique sound files mapped to their keys
            for (vk_code, value) in defines {
                if let Ok(vk_num) = vk_code.parse::<u32>() {
                    if let Some(key_name) = key_mappings.get(&vk_num) {
                        if let Some(sound_file_str) = value.as_str() {
                            if !sound_file_str.is_empty() && sound_file_str != "null" {
                                sound_files.insert(key_name.clone(), sound_file_str.to_string());
                            }
                        }
                    }
                }
            }
            // Create concatenated audio and get segment mappings
            let output_dir = Path::new(output_path)
                .parent()
                .and_then(|p| p.to_str())
                .ok_or("Could not determine output directory")?;
            let output_audio_path = format!("{}/combined_audio.wav", output_dir);

            match create_concatenated_audio_and_segments(
                soundpack_dir,
                &sound_files,
                &output_audio_path,
            ) {
                Ok(segments) => {
                    // Set the source to the combined audio file
                    converted_config.insert(
                        "source".to_string(),
                        Value::String("combined_audio.wav".to_string()),
                    );

                    // Second pass: create timing definitions using calculated segments
                    for (vk_code, value) in defines {
                        if let Ok(vk_num) = vk_code.parse::<u32>() {
                            if let Some(key_name) = key_mappings.get(&vk_num) {
                                if let Some(sound_file_str) = value.as_str() {
                                    if !sound_file_str.is_empty() && sound_file_str != "null" {
                                        if let Some((start, duration)) = segments.get(key_name) {
                                            let timing = vec![Value::Array(vec![
                                                Value::from(*start),
                                                Value::from(*duration),
                                            ])];
                                            defs.insert(key_name.clone(), Value::Array(timing));
                                        }
                                    }
                                }
                            }
                        }
                    }

                    println!(
                        "✅ Created {} audio segments for multi method",
                        segments.len()
                    );
                }
                Err(e) => {
                    println!(
                        "⚠️ Warning: Could not process audio files: {}. Using default segments.",
                        e
                    );

                    // Fallback: use default timing for multi method
                    for (vk_code, value) in defines {
                        if let Ok(vk_num) = vk_code.parse::<u32>() {
                            if let Some(key_name) = key_mappings.get(&vk_num) {
                                if let Some(sound_file_str) = value.as_str() {
                                    if !sound_file_str.is_empty() && sound_file_str != "null" {
                                        let timing = vec![Value::Array(vec![
                                            Value::from(0.0),
                                            Value::from(100.0),
                                        ])];
                                        defs.insert(key_name.clone(), Value::Array(timing));
                                    }
                                }
                            }
                        }
                    }
                }
            }
        } else {
            // Single method: preserve existing logic
            for (vk_code, value) in defines {
                // Convert VK code to key name
                if let Ok(vk_num) = vk_code.parse::<u32>() {
                    if let Some(key_name) = key_mappings.get(&vk_num) {
                        let timing = if let Some(timing_array) = value.as_array() {
                            if timing_array.len() >= 2 {
                                // Keep [start_ms, duration_ms] format: [[start_ms, duration_ms]]
                                if let (Some(start), Some(duration)) =
                                    (timing_array[0].as_f64(), timing_array[1].as_f64())
                                {
                                    vec![Value::Array(vec![
                                        Value::from(start),
                                        Value::from(duration),
                                    ])]
                                } else {
                                    // Fallback to default timing if conversion fails
                                    vec![Value::Array(vec![Value::from(0.0), Value::from(100.0)])]
                                }
                            } else {
                                // Array too short, use default timing
                                vec![Value::Array(vec![Value::from(0.0), Value::from(100.0)])]
                            }
                        } else if let Some(sound_file_str) = value.as_str() {
                            // Fallback: if it's a string (like in multi), use default timing
                            if !sound_file_str.is_empty() && sound_file_str != "null" {
                                vec![Value::Array(vec![Value::from(0.0), Value::from(100.0)])]
                            } else {
                                continue; // Skip empty entries
                            }
                        } else {
                            continue; // Skip unknown types
                        };

                        defs.insert(key_name.clone(), Value::Array(timing));
                    }
                }
            }
        }
    }
    converted_config.insert("defs".to_string(), Value::Object(defs.clone()));

    // Write the converted config
    let converted_json = serde_json::to_string_pretty(&converted_config)?;
    path::write_file_contents(output_path, &converted_json)
        .map_err(|e| format!("Failed to write converted config: {}", e))?;

    println!("✅ Successfully converted config from V1 to V2");
    println!("   Input: {}", v1_config_path);
    println!("   Output: {}", output_path);
    println!("   Converted {} key mappings", defs.len());
    Ok(())
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
