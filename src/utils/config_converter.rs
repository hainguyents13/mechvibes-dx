use crate::utils::path;
use serde_json::{ Map, Value };
use std::collections::HashMap;
use std::fs::File;
use std::path::Path;

/// Get the duration of an audio file in milliseconds
fn get_audio_duration_ms(file_path: &str) -> Result<f64, Box<dyn std::error::Error>> {
    // Debug log the file being processed
    println!("🎵 Attempting to get duration for: {}", file_path);

    // Try symphonia first for better metadata support
    if let Ok(duration) = get_duration_with_symphonia(file_path) {
        if duration > 0.0 {
            println!("✅ Symphonia duration for {}: {:.1}ms", file_path, duration);
            return Ok(duration);
        }
    }

    // Fallback to rodio if symphonia fails
    if let Ok(duration) = get_duration_with_rodio(file_path) {
        if duration > 0.0 {
            println!("✅ Rodio duration for {}: {:.1}ms", file_path, duration);
            return Ok(duration);
        }
    }

    println!("⚠️ No duration metadata available for {}, using fallback", file_path);
    Ok(100.0)
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
    let format = probed.format;

    // Get the default track
    let track = format
        .tracks()
        .iter()
        .find(|t| t.codec_params.codec != symphonia::core::codecs::CODEC_TYPE_NULL)
        .ok_or("No valid audio track found")?;

    // Calculate duration from time base and frames
    if
        let (Some(time_base), Some(n_frames)) = (
            track.codec_params.time_base,
            track.codec_params.n_frames,
        )
    {
        let duration_seconds =
            ((n_frames as f64) * (time_base.numer as f64)) / (time_base.denom as f64);
        let duration_ms = duration_seconds * 1000.0;
        return Ok(duration_ms);
    }

    Err("No duration information available in track".into())
}

/// Get duration using Rodio (fallback method)
fn get_duration_with_rodio(file_path: &str) -> Result<f64, Box<dyn std::error::Error>> {
    use rodio::{ Decoder, Source };
    use std::io::BufReader;

    let file = File::open(file_path)?;
    let reader = BufReader::new(file);
    let source = Decoder::new(reader)?;

    if let Some(duration) = source.total_duration() {
        let duration_ms = duration.as_millis() as f64;
        if duration_ms > 0.0 {
            return Ok(duration_ms);
        }
    }

    Err("No duration available from rodio".into())
}

/// Create concatenated audio file from multiple sound files and return segment mappings
fn create_concatenated_audio_and_segments(
    soundpack_dir: &str,
    sound_files: &HashMap<String, String>,
    // key_name -> sound_file_name
    output_audio_path: &str
) -> Result<HashMap<String, (f64, f64)>, Box<dyn std::error::Error>> {
    use std::collections::BTreeSet;

    println!("\n🔗 === SEGMENT CREATION DEBUG ===");
    println!("Processing {} sound files for multi-method conversion", sound_files.len());

    let mut segments = HashMap::new();

    println!("🔗 Processing {} sound files for multi-method conversion", sound_files.len());

    // Step 1: Collect unique sound files and their durations
    let mut unique_files: BTreeSet<String> = BTreeSet::new();
    let mut file_durations: HashMap<String, f64> = HashMap::new();

    // First pass: collect all unique sound files
    for (_, sound_file) in sound_files {
        if !sound_file.is_empty() && sound_file != "null" {
            unique_files.insert(sound_file.clone());
        }
    }

    println!("📁 Found {} unique audio files to concatenate", unique_files.len());

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
                println!("⚠️ Could not read duration for {}, using default 100ms", file_path);
            }
        }
    }

    // Step 3: Create concatenated audio file and calculate segment positions
    let mut current_position = 0.0;
    let mut file_segments: HashMap<String, (f64, f64)> = HashMap::new();

    // Try to create concatenated audio using simple file concatenation approach
    if
        let Err(e) = create_concatenated_audio_file(
            soundpack_dir,
            &unique_files,
            &file_durations,
            output_audio_path
        )
    {
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
                current_position += duration;
            }
        }

        // Step 5: Map keys to their corresponding segments
        for (key_name, sound_file) in sound_files {
            if let Some(&(start, duration)) = file_segments.get(sound_file) {
                segments.insert(key_name.clone(), (start, duration));
            } else {
                // Fallback for missing files
                println!("⚠️ Fallback for missing file '{}', using default (0.0, 100.0)", sound_file);
                segments.insert(key_name.clone(), (0.0, 100.0));
            }
        }

        println!(
            "✅ Created concatenated audio file with {:.1}ms total duration",
            current_position
        );
    }

    println!("✅ Created {} key segments from {} unique files", segments.len(), unique_files.len());

    Ok(segments)
}

/// Create a concatenated audio file from multiple source files using WAV format
fn create_concatenated_audio_file(
    soundpack_dir: &str,
    unique_files: &std::collections::BTreeSet<String>,
    _file_durations: &HashMap<String, f64>,
    output_path: &str
) -> Result<(), Box<dyn std::error::Error>> {
    use rodio::{ Decoder, Source };
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
        println!("🔍 Processing file:  {}", file_path);

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
                                sample_rate,
                                channels
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

    println!("🚧 Creating WAV file at: {}", output_path);

    let mut writer = match hound::WavWriter::create(output_path, spec) {
        Ok(writer) => writer,
        Err(e) => {
            println!("❌ Failed to create WAV writer for {}: {}", output_path, e);
            return Err(format!("Failed to create WAV writer: {}", e).into());
        }
    };

    let total_samples = all_samples.len();
    println!("📝 Writing {} samples to WAV file...", total_samples);

    // Write samples with error handling
    for (index, sample) in all_samples.iter().enumerate() {
        if let Err(e) = writer.write_sample(*sample) {
            println!("❌ Failed to write sample {} to WAV file: {}", index, e);
            return Err(format!("Failed to write sample {} to WAV file: {}", index, e).into());
        }

        // Progress indicator for large files
        if index > 0 && index % 100000 == 0 {
            println!(
                "📝 Written {}/{} samples ({:.1}%)",
                index,
                total_samples,
                ((index as f64) / (total_samples as f64)) * 100.0
            );
        }
    }

    // Finalize the WAV file with error handling
    if let Err(e) = writer.finalize() {
        println!("❌ Failed to finalize WAV file {}: {}", output_path, e);
        return Err(format!("Failed to finalize WAV file: {}", e).into());
    }

    println!("🎵 Successfully created concatenated audio file: {}", output_path);
    println!(
        "📊 Total samples: {}, Duration: ~{:.1}ms",
        total_samples,
        ((total_samples as f64) / (channels as f64) / (sample_rate as f64)) * 1000.0
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
///
/// Parameters:
/// - `v1_config_path`: Path to the V1 config file to convert
/// - `output_path`: Path where the converted V2 config will be written
/// - `soundpack_dir`: Optional explicit soundpack directory. If None, uses the parent directory of v1_config_path
pub fn convert_v1_to_v2(
    v1_config_path: &str,
    output_path: &str,
    soundpack_dir: Option<&str>
) -> Result<(), Box<dyn std::error::Error>> {
    println!("⚒️ Converting V1 soundpack config to V2 format");

    // Determine soundpack directory - use provided or infer from config path
    let soundpack_dir = if let Some(dir) = soundpack_dir {
        println!("🔍 Using explicit soundpack directory: {}", dir);
        dir
    } else {
        let inferred_dir = Path::new(v1_config_path)
            .parent()
            .and_then(|p| p.to_str())
            .ok_or("Could not determine soundpack directory")?;
        println!("🔍 Inferred soundpack directory from config path: {}", inferred_dir);
        inferred_dir
    };

    println!("🔍 Config path: {}", v1_config_path);
    println!("🔍 Soundpack directory: {}", soundpack_dir);

    // Read the V1 config
    let content = path
        ::read_file_contents(v1_config_path)
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

    // Add method field - determine from key_define_type or default to "single"
    let method = if let Some(key_define_type) = config.get("key_define_type") {
        if key_define_type.as_str() == Some("multi") {
            converted_config.insert("method".to_string(), Value::String("multi".to_string()));
            "multi"
        } else {
            converted_config.insert("method".to_string(), Value::String("single".to_string()));
            "single"
        }
    } else {
        converted_config.insert("method".to_string(), Value::String("single".to_string()));
        "single"
    };

    // Copy includes_numpad if present
    if let Some(includes_numpad) = config.get("includes_numpad") {
        converted_config.insert("includes_numpad".to_string(), includes_numpad.clone());
    }

    // Add mouse field (defaults to false for keyboard soundpacks)
    converted_config.insert("mouse".to_string(), Value::Bool(false));

    // Add config_version field set to 2
    converted_config.insert(
        "config_version".to_string(),
        Value::Number(serde_json::Number::from(2))
    );

    // Handle source field and multi-method audio processing
    let output_audio_filename = if method == "multi" {
        // For multi method, always use combined_audio.wav
        "combined_audio.wav".to_string()
    } else {
        // For single method, preserve the original sound field
        if let Some(sound) = config.get("sound") {
            converted_config.insert("source".to_string(), sound.clone());
        }
        String::new() // Not used for single method
    };

    // Convert "defines" to "defs" with proper timing format based on method
    let mut defs = Map::new();

    if let Some(defines) = config.get("defines").and_then(|d| d.as_object()) {
        // Key mapping for common virtual key codes to Web API key names
        let key_mappings = create_vk_to_web_key_mapping();

        if method == "multi" {
            // Multi method: collect all sound files and create segments
            let mut sound_files = HashMap::new(); // First pass: collect all unique sound files mapped to their keys
            for (vk_code, value) in defines {
                if let Ok(vk_num) = vk_code.parse::<u32>() {
                    if let Some(key_name) = key_mappings.get(&vk_num) {
                        // Skip null values and only process actual string sound file names
                        if !value.is_null() {
                            if let Some(sound_file_str) = value.as_str() {
                                if !sound_file_str.is_empty() && sound_file_str != "null" {
                                    sound_files.insert(
                                        key_name.clone(),
                                        sound_file_str.to_string()
                                    );
                                    println!(
                                        "✅ Added mapping: {} -> {}",
                                        key_name,
                                        sound_file_str
                                    );
                                }
                            }
                        } else {
                            println!("⏭️ Skipping null mapping for VK {} ({})", vk_code, key_name);
                        }
                    }
                }
            }

            // Create concatenated audio and get segment mappings
            // Write the audio file to the soundpack directory
            let output_audio_path = format!("{}/{}", soundpack_dir, output_audio_filename);

            match
                create_concatenated_audio_and_segments(
                    soundpack_dir,
                    &sound_files,
                    &output_audio_path
                )
            {
                Ok(segments) => {
                    // Set the source to the audio file (using preserved filename)
                    converted_config.insert(
                        "source".to_string(),
                        Value::String(output_audio_filename)
                    ); // Second pass: create timing definitions using calculated segments
                    // Only process keys that were included in sound_files (i.e., not null)
                    for (key_name, _sound_file) in &sound_files {
                        if let Some((start, duration)) = segments.get(key_name) {
                            let timing = vec![
                                Value::Array(vec![Value::from(*start), Value::from(*duration)])
                            ];
                            defs.insert(key_name.clone(), Value::Array(timing));
                        } else {
                            println!("⚠️ No segment found for key: {}", key_name);
                        }
                    }

                    println!("✅ Created {} audio segments for multi method", segments.len());
                }
                Err(e) => {
                    println!("⚠️ Warning: Could not process audio files: {}. Using default segments.", e);

                    // Set source to the preserved filename even if processing failed
                    converted_config.insert(
                        "source".to_string(),
                        Value::String(output_audio_filename)
                    ); // Fallback: use default timing for multi method
                    // Only process keys that were included in sound_files (i.e., not null)
                    for (key_name, _sound_file) in &sound_files {
                        let timing = vec![Value::Array(vec![Value::from(0.0), Value::from(100.0)])];
                        defs.insert(key_name.clone(), Value::Array(timing));
                    }
                }
            }
        } else {
            // Single method: preserve existing logic with null filtering
            for (vk_code, value) in defines {
                // Convert VK code to key name
                if let Ok(vk_num) = vk_code.parse::<u32>() {
                    if let Some(key_name) = key_mappings.get(&vk_num) {
                        // Skip null values in single method too
                        if value.is_null() {
                            println!(
                                "⏭️ Skipping null mapping for VK {} ({}) in single method",
                                vk_code,
                                key_name
                            );
                            continue;
                        }

                        let timing = if let Some(timing_array) = value.as_array() {
                            if timing_array.len() >= 2 {
                                // Keep [start_ms, duration_ms] format: [[start_ms, duration_ms]]
                                if
                                    let (Some(start), Some(duration)) = (
                                        timing_array[0].as_f64(),
                                        timing_array[1].as_f64(),
                                    )
                                {
                                    println!(
                                        "✅ Added single method mapping: {} -> [{:.1}ms, {:.1}ms]",
                                        key_name,
                                        start,
                                        duration
                                    );
                                    vec![
                                        Value::Array(
                                            vec![Value::from(start), Value::from(duration)]
                                        )
                                    ]
                                } else {
                                    // Fallback to default timing if conversion fails
                                    println!("⚠️ Using default timing for {}: invalid array values", key_name);
                                    vec![Value::Array(vec![Value::from(0.0), Value::from(100.0)])]
                                }
                            } else {
                                // Array too short, use default timing
                                println!("⚠️ Using default timing for {}: array too short", key_name);
                                vec![Value::Array(vec![Value::from(0.0), Value::from(100.0)])]
                            }
                        } else if let Some(sound_file_str) = value.as_str() {
                            // Fallback: if it's a string (like in multi), use default timing
                            if !sound_file_str.is_empty() && sound_file_str != "null" {
                                println!("✅ Added single method mapping: {} -> [0.0ms, 100.0ms] (from string)", key_name);
                                vec![Value::Array(vec![Value::from(0.0), Value::from(100.0)])]
                            } else {
                                println!(
                                    "⏭️ Skipping empty/null string mapping for VK {} ({})",
                                    vk_code,
                                    key_name
                                );
                                continue; // Skip empty entries
                            }
                        } else {
                            println!(
                                "⏭️ Skipping unknown type mapping for VK {} ({})",
                                vk_code,
                                key_name
                            );
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
    path
        ::write_file_contents(output_path, &converted_json)
        .map_err(|e| format!("Failed to write converted config: {}", e))?;

    println!("✅ Successfully converted config from V1 to V2");
    println!("   Input: {}", v1_config_path);
    println!("   Output: {}", output_path);
    println!("   Soundpack dir: {}", soundpack_dir);
    println!("   Converted {} key mappings", defs.len());
    Ok(())
}

/// Create mapping from Windows Virtual Key codes to Web API key names
fn create_vk_to_web_key_mapping() -> HashMap<u32, String> {
    let mut mapping = HashMap::new();

    // Function keys
    mapping.insert(112, "F1".to_string()); // VK_F1
    mapping.insert(113, "F2".to_string()); // VK_F2
    mapping.insert(114, "F3".to_string()); // VK_F3
    mapping.insert(115, "F4".to_string()); // VK_F4
    mapping.insert(116, "F5".to_string()); // VK_F5
    mapping.insert(117, "F6".to_string()); // VK_F6
    mapping.insert(118, "F7".to_string()); // VK_F7
    mapping.insert(119, "F8".to_string()); // VK_F8
    mapping.insert(120, "F9".to_string()); // VK_F9
    mapping.insert(121, "F10".to_string()); // VK_F10
    mapping.insert(122, "F11".to_string()); // VK_F11
    mapping.insert(123, "F12".to_string()); // VK_F12

    // Number row
    mapping.insert(48, "Digit0".to_string()); // VK_0
    mapping.insert(49, "Digit1".to_string()); // VK_1
    mapping.insert(50, "Digit2".to_string()); // VK_2
    mapping.insert(51, "Digit3".to_string()); // VK_3
    mapping.insert(52, "Digit4".to_string()); // VK_4
    mapping.insert(53, "Digit5".to_string()); // VK_5
    mapping.insert(54, "Digit6".to_string()); // VK_6
    mapping.insert(55, "Digit7".to_string()); // VK_7
    mapping.insert(56, "Digit8".to_string()); // VK_8
    mapping.insert(57, "Digit9".to_string()); // VK_9

    // Letter keys A-Z
    mapping.insert(65, "KeyA".to_string()); // VK_A
    mapping.insert(66, "KeyB".to_string()); // VK_B
    mapping.insert(67, "KeyC".to_string()); // VK_C
    mapping.insert(68, "KeyD".to_string()); // VK_D
    mapping.insert(69, "KeyE".to_string()); // VK_E
    mapping.insert(70, "KeyF".to_string()); // VK_F
    mapping.insert(71, "KeyG".to_string()); // VK_G
    mapping.insert(72, "KeyH".to_string()); // VK_H
    mapping.insert(73, "KeyI".to_string()); // VK_I
    mapping.insert(74, "KeyJ".to_string()); // VK_J
    mapping.insert(75, "KeyK".to_string()); // VK_K - CORRECT VK CODE!
    mapping.insert(76, "KeyL".to_string()); // VK_L
    mapping.insert(77, "KeyM".to_string()); // VK_M
    mapping.insert(78, "KeyN".to_string()); // VK_N
    mapping.insert(79, "KeyO".to_string()); // VK_O
    mapping.insert(80, "KeyP".to_string()); // VK_P
    mapping.insert(81, "KeyQ".to_string()); // VK_Q
    mapping.insert(82, "KeyR".to_string()); // VK_R
    mapping.insert(83, "KeyS".to_string()); // VK_S
    mapping.insert(84, "KeyT".to_string()); // VK_T
    mapping.insert(85, "KeyU".to_string()); // VK_U
    mapping.insert(86, "KeyV".to_string()); // VK_V
    mapping.insert(87, "KeyW".to_string()); // VK_W
    mapping.insert(88, "KeyX".to_string()); // VK_X
    mapping.insert(89, "KeyY".to_string()); // VK_Y
    mapping.insert(90, "KeyZ".to_string()); // VK_Z

    // Special keys
    mapping.insert(27, "Escape".to_string()); // VK_ESCAPE
    mapping.insert(9, "Tab".to_string()); // VK_TAB
    mapping.insert(20, "CapsLock".to_string()); // VK_CAPITAL
    mapping.insert(16, "ShiftLeft".to_string()); // VK_SHIFT (we'll treat as left)
    mapping.insert(17, "ControlLeft".to_string()); // VK_CONTROL (we'll treat as left)
    mapping.insert(18, "AltLeft".to_string()); // VK_MENU (Alt key)
    mapping.insert(32, "Space".to_string()); // VK_SPACE
    mapping.insert(13, "Enter".to_string()); // VK_RETURN
    mapping.insert(8, "Backspace".to_string()); // VK_BACK

    // Punctuation keys (common US layout)
    mapping.insert(189, "Minus".to_string()); // VK_OEM_MINUS
    mapping.insert(187, "Equal".to_string()); // VK_OEM_PLUS
    mapping.insert(219, "BracketLeft".to_string()); // VK_OEM_4
    mapping.insert(221, "BracketRight".to_string()); // VK_OEM_6
    mapping.insert(220, "Backslash".to_string()); // VK_OEM_5
    mapping.insert(186, "Semicolon".to_string()); // VK_OEM_1
    mapping.insert(222, "Quote".to_string()); // VK_OEM_7
    mapping.insert(192, "Backquote".to_string()); // VK_OEM_3
    mapping.insert(188, "Comma".to_string()); // VK_OEM_COMMA
    mapping.insert(190, "Period".to_string()); // VK_OEM_PERIOD
    mapping.insert(191, "Slash".to_string()); // VK_OEM_2

    // Arrow keys
    mapping.insert(37, "ArrowLeft".to_string()); // VK_LEFT - CORRECT: VK 37 is LEFT ARROW!
    mapping.insert(38, "ArrowUp".to_string()); // VK_UP
    mapping.insert(39, "ArrowRight".to_string()); // VK_RIGHT
    mapping.insert(40, "ArrowDown".to_string()); // VK_DOWN    // Insert/Delete cluster
    mapping.insert(45, "Insert".to_string()); // VK_INSERT
    mapping.insert(46, "Delete".to_string()); // VK_DELETE
    mapping.insert(36, "Home".to_string()); // VK_HOME
    mapping.insert(35, "End".to_string()); // VK_END
    mapping.insert(33, "PageUp".to_string()); // VK_PRIOR
    mapping.insert(34, "PageDown".to_string()); // VK_NEXT

    // Numpad
    mapping.insert(96, "Numpad0".to_string()); // VK_NUMPAD0
    mapping.insert(97, "Numpad1".to_string()); // VK_NUMPAD1
    mapping.insert(98, "Numpad2".to_string()); // VK_NUMPAD2
    mapping.insert(99, "Numpad3".to_string()); // VK_NUMPAD3
    mapping.insert(100, "Numpad4".to_string()); // VK_NUMPAD4
    mapping.insert(101, "Numpad5".to_string()); // VK_NUMPAD5
    mapping.insert(102, "Numpad6".to_string()); // VK_NUMPAD6
    mapping.insert(103, "Numpad7".to_string()); // VK_NUMPAD7
    mapping.insert(104, "Numpad8".to_string()); // VK_NUMPAD8
    mapping.insert(105, "Numpad9".to_string()); // VK_NUMPAD9
    mapping.insert(106, "NumpadMultiply".to_string()); // VK_MULTIPLY
    mapping.insert(107, "NumpadAdd".to_string()); // VK_ADD
    mapping.insert(109, "NumpadSubtract".to_string()); // VK_SUBTRACT
    mapping.insert(110, "NumpadDecimal".to_string()); // VK_DECIMAL
    mapping.insert(111, "NumpadDivide".to_string()); // VK_DIVIDE

    mapping
}
