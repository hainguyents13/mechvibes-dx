use super::path;
use serde_json::{ Map, Value };
use std::collections::HashMap;
use std::path::Path;

/// Get the duration of an audio file in milliseconds using Symphonia
fn get_audio_duration_ms(file_path: &str) -> Result<f64, Box<dyn std::error::Error>> {
    // Check if file exists first
    if !Path::new(file_path).exists() {
        return Err("File does not exist".into());
    } // Use symphonia for audio duration detection
    match get_duration_with_symphonia(file_path) {
        Ok(duration) if duration > 0.0 => { Ok(duration) }
        Ok(_) => { Ok(100.0) }
        Err(_) => { Ok(100.0) }
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
            return Ok(duration_ms);
        }
    }

    // If metadata doesn't have duration, estimate from sample rate
    if let Some(sample_rate) = track.codec_params.sample_rate {
        if let Some(n_frames) = track.codec_params.n_frames {
            let duration_seconds = (n_frames as f64) / (sample_rate as f64);
            let duration_ms = duration_seconds * 1000.0;
            return Ok(duration_ms);
        }
    }

    // Fallback: use default duration
    Ok(100.0)
}

/// Convert soundpack config from version 1 to version 2
/// Uses comprehensive IOHook keycode mapping (supports all platforms)
pub fn convert_v1_to_v2(
    v1_config_path: &str,
    output_path: &str,
    soundpack_dir: Option<&str>
) -> Result<(), Box<dyn std::error::Error>> {
    // Determine soundpack directory - use provided or infer from config path
    let soundpack_dir = if let Some(dir) = soundpack_dir {
        dir
    } else {
        let inferred_dir = Path::new(v1_config_path)
            .parent()
            .and_then(|p| p.to_str())
            .ok_or("Could not determine soundpack directory")?;
        inferred_dir
    };

    // Read the V1 config
    let content = path
        ::read_file_contents(v1_config_path)
        .map_err(|e| format!("Failed to read V1 config: {}", e))?;
    let config: Value = serde_json::from_str(&content)?;

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
    converted_config.insert("created_at".to_string(), Value::String(now.to_rfc3339())); // Determine definition_method from V1 key_define_type or sound structure
    // IMPORTANT: Always convert to "single" method for V2
    // This means we'll use one main audio file with timing segments
    let v1_define_type = config
        .get("key_define_type")
        .and_then(|v| v.as_str())
        .unwrap_or("single");
    let definition_method = "single";

    converted_config.insert(
        "definition_method".to_string(),
        Value::String(definition_method.to_string())
    ); // Handle audio_file for "single" method
    let (audio_file_name, audio_file_info) = if v1_define_type == "multi" {
        // For V1 multi method, we need to create a concatenated audio file
        // First, collect all unique audio files from defines
        let mut audio_files_ordered = Vec::new();
        let mut seen_files = std::collections::HashSet::new();

        if let Some(defines) = config.get("defines").and_then(|d| d.as_object()) {
            // Sort keys to ensure consistent order
            let mut sorted_keys: Vec<_> = defines.keys().collect();
            sorted_keys.sort_by_key(|k| k.parse::<u32>().unwrap_or(0));

            for key in sorted_keys {
                if let Some(value) = defines.get(key) {
                    if let Some(filename) = value.as_str() {
                        if
                            !filename.is_empty() &&
                            filename != "null" &&
                            !seen_files.contains(filename)
                        {
                            // Just collect the files, we'll get timing from concatenation
                            audio_files_ordered.push(filename.to_string());
                            seen_files.insert(filename.to_string());
                        }
                    }
                }
            }
        }

        println!("🔧 Found {} unique audio files in V1 multi method", audio_files_ordered.len());

        // Create a concatenated audio file name
        let concat_filename = "concatenated_audio.wav";
        println!("🎵 Creating concatenated audio file: {}", concat_filename);

        // Actually concatenate the audio files and get accurate timing
        let audio_file_info = match
            concatenate_audio_files_with_timing(
                &audio_files_ordered,
                soundpack_dir,
                &concat_filename
            )
        {
            Ok(timing_info) => timing_info,
            Err(e) => {
                println!("❌ Failed to create concatenated audio file: {}", e);
                return Err(format!("Audio concatenation failed: {}", e).into());
            }
        };

        (concat_filename.to_string(), audio_file_info)
    } else {
        // For V1 single method, use the main sound file
        let main_file = if let Some(sound) = config.get("sound") {
            if let Some(sound_str) = sound.as_str() {
                println!("🎵 Using main audio file from V1 single method: {}", sound_str);
                sound_str.to_string()
            } else {
                return Err("Invalid sound field in V1 config".into());
            }
        } else {
            // If no main sound file, we need to find one from the soundpack directory
            let audio_extensions = ["ogg", "mp3", "wav", "flac"];
            let mut found_audio = None;

            if let Ok(entries) = std::fs::read_dir(soundpack_dir) {
                for entry in entries.filter_map(|e| e.ok()) {
                    if let Some(filename) = entry.file_name().to_str() {
                        let filename_lower = filename.to_lowercase();
                        for ext in &audio_extensions {
                            if filename_lower.ends_with(&format!(".{}", ext)) {
                                found_audio = Some(filename.to_string());
                                break;
                            }
                        }
                        if found_audio.is_some() {
                            break;
                        }
                    }
                }
            }

            if let Some(audio_file) = found_audio {
                println!("🎵 Found audio file in directory: {}", audio_file);
                audio_file
            } else {
                return Err("No audio file found for single method conversion".into());
            }
        };

        (main_file, std::collections::HashMap::new())
    };

    converted_config.insert("audio_file".to_string(), Value::String(audio_file_name.clone()));

    // Add default options
    let mut options = Map::new();
    options.insert(
        "recommended_volume".to_string(),
        Value::Number(serde_json::Number::from_f64(1.0).unwrap())
    );
    options.insert("random_pitch".to_string(), Value::Bool(false));
    converted_config.insert("options".to_string(), Value::Object(options)); // Convert "defines" to "definitions" with new format
    let mut definitions = Map::new();
    if let Some(defines) = config.get("defines").and_then(|d| d.as_object()) {
        let key_mappings = create_iohook_to_web_key_mapping();
        println!("🔧 Converting {} key definitions to new format (single method)", defines.len());
        if v1_define_type == "multi" {
            // V1 multi method: defines contains IOHook code -> audio filename mappings
            // We need to create timing based on concatenated audio file offsets
            println!("🔧 Processing V1 multi method defines");

            for (iohook_code, value) in defines {
                if let Ok(iohook_num) = iohook_code.parse::<u32>() {
                    if let Some(key_name) = key_mappings.get(&iohook_num) {
                        if let Some(audio_filename) = value.as_str() {
                            if !audio_filename.is_empty() && audio_filename != "null" {
                                let mut key_def = Map::new();

                                // Get offset and duration for this audio file
                                if
                                    let Some(&(offset, duration)) =
                                        audio_file_info.get(audio_filename)
                                {
                                    let end_time = offset + duration;

                                    // Special debug for Enter key
                                    if key_name == "Enter" {
                                        println!("🔍 [ENTER DEBUG] Key: {}", key_name);
                                        println!("🔍 [ENTER DEBUG] IOHook code: {}", iohook_num);
                                        println!("🔍 [ENTER DEBUG] Audio file: {}", audio_filename);
                                        println!("🔍 [ENTER DEBUG] Offset: {}ms", offset);
                                        println!("🔍 [ENTER DEBUG] Duration: {}ms", duration);
                                        println!("🔍 [ENTER DEBUG] End time: {}ms", end_time);

                                        // Check concatenated audio file duration
                                        let concat_path =
                                            format!("{}/concatenated_audio.wav", soundpack_dir);
                                        if
                                            let Ok(concat_duration) = get_audio_duration_ms(
                                                &concat_path
                                            )
                                        {
                                            println!("🔍 [ENTER DEBUG] Concatenated audio duration: {}ms", concat_duration);
                                            if end_time > concat_duration {
                                                println!(
                                                    "❌ [ENTER DEBUG] ERROR: End time ({}) > Concat duration ({})",
                                                    end_time,
                                                    concat_duration
                                                );
                                            }
                                        }
                                    }

                                    // Create timing based on offset in concatenated file
                                    let timing = vec![
                                        Value::Array(
                                            vec![
                                                Value::Number(
                                                    serde_json::Number::from_f64(offset).unwrap()
                                                ),
                                                Value::Number(
                                                    serde_json::Number::from_f64(end_time).unwrap()
                                                )
                                            ]
                                        )
                                    ];
                                    key_def.insert("timing".to_string(), Value::Array(timing));

                                    definitions.insert(key_name.clone(), Value::Object(key_def));
                                    println!(
                                        "   ✅ Key '{}' -> {} [offset: {}ms, end: {}ms]",
                                        key_name,
                                        audio_filename,
                                        offset,
                                        end_time
                                    );
                                } else {
                                    println!("   ⚠️ No offset found for audio file: {}", audio_filename);
                                }
                            } else {
                                println!("   ⚠️ Key IOHook {} has empty/null audio file", iohook_code);
                            }
                        }
                    } else {
                        println!("   ⚠️ No key mapping found for IOHook code: {}", iohook_code);
                    }
                }
            }
        } else {
            // V1 single method: defines contains IOHook code -> timing array mappings
            println!("🔧 Processing V1 single method defines");

            for (iohook_code, value) in defines {
                if let Ok(iohook_num) = iohook_code.parse::<u32>() {
                    if let Some(key_name) = key_mappings.get(&iohook_num) {
                        let mut key_def = Map::new();

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

                                definitions.insert(key_name.clone(), Value::Object(key_def));
                                println!("   ✅ Key '{}' -> timing [{}, {}]", key_name, start, end);
                            }
                        } else {
                            println!("   ⚠️ Key '{}' has invalid timing format", key_name);
                        }
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

/// Convert V2 config from multi method to single method
/// This ensures all V2 configs use the single method format
pub fn convert_v2_multi_to_single(
    config_path: &str,
    soundpack_dir: &str
) -> Result<(), Box<dyn std::error::Error>> {
    println!("🔄 Converting V2 multi method to single method...");

    // Read the existing V2 config
    let content = std::fs::read_to_string(config_path)?;
    let mut config: Value = serde_json::from_str(&content)?;

    // Check if this is already single method
    if let Some(definition_method) = config.get("definition_method").and_then(|v| v.as_str()) {
        if definition_method == "single" {
            println!("✅ Already using single method, no conversion needed");
            return Ok(());
        }
    }

    println!("🔧 Converting from multi method to single method");

    // Analyze audio files used in definitions to find the most common one
    let mut audio_file_usage = std::collections::HashMap::new();
    if let Some(definitions) = config.get("definitions").and_then(|d| d.as_object()) {
        for (key_name, key_def) in definitions {
            if let Some(key_obj) = key_def.as_object() {
                if let Some(audio_file) = key_obj.get("audio_file").and_then(|v| v.as_str()) {
                    *audio_file_usage.entry(audio_file.to_string()).or_insert(0) += 1;
                    println!("🔍 Key '{}' uses audio file: {}", key_name, audio_file);
                }
            }
        }
    }
    // Find the most commonly used audio file
    let main_audio_file = if
        let Some((audio_file, count)) = audio_file_usage.iter().max_by_key(|(_, count)| *count)
    {
        println!("🎵 Most used audio file: {} (used by {} keys)", audio_file, count);
        audio_file.clone()
    } else {
        // Fallback: find any audio file in the directory
        let audio_extensions = ["ogg", "mp3", "wav", "flac"];
        let mut found_audio = None;

        if let Ok(entries) = std::fs::read_dir(soundpack_dir) {
            for entry in entries.filter_map(|e| e.ok()) {
                if let Some(filename) = entry.file_name().to_str() {
                    let filename_lower = filename.to_lowercase();
                    for ext in &audio_extensions {
                        if filename_lower.ends_with(&format!(".{}", ext)) {
                            found_audio = Some(filename.to_string());
                            break;
                        }
                    }
                    if found_audio.is_some() {
                        break;
                    }
                }
            }
        }

        found_audio.ok_or("No audio file found in soundpack directory")?
    };

    println!("🎵 Using main audio file for single method: {}", main_audio_file);

    // Update config to single method
    config
        .as_object_mut()
        .unwrap()
        .insert("definition_method".to_string(), Value::String("single".to_string()));

    config
        .as_object_mut()
        .unwrap()
        .insert("audio_file".to_string(), Value::String(main_audio_file.clone()));

    // Convert definitions from multi to single format
    if
        let Some(definitions) = config
            .get("definitions")
            .and_then(|d| d.as_object())
            .cloned()
    {
        let mut new_definitions = serde_json::Map::new();

        for (key_name, key_def) in definitions {
            if let Some(key_obj) = key_def.as_object() {
                let mut new_key_def = serde_json::Map::new();

                // Check if this key was using the main audio file
                let key_audio_file = key_obj
                    .get("audio_file")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");

                if key_audio_file == main_audio_file {
                    // This key uses the main audio file, keep its timing
                    if let Some(timing) = key_obj.get("timing") {
                        new_key_def.insert("timing".to_string(), timing.clone());
                        println!("✅ Key '{}' kept timing (uses main audio file)", key_name);
                    } else {
                        // Create default timing for the whole audio file
                        let audio_path = format!("{}/{}", soundpack_dir, main_audio_file);
                        let duration = get_audio_duration_ms(&audio_path).unwrap_or(100.0);

                        let timing = vec![
                            Value::Array(
                                vec![
                                    Value::Number(serde_json::Number::from_f64(0.0).unwrap()),
                                    Value::Number(serde_json::Number::from_f64(duration).unwrap())
                                ]
                            )
                        ];
                        new_key_def.insert("timing".to_string(), Value::Array(timing));
                        println!("⚠️ Key '{}' got default timing (no timing specified)", key_name);
                    }

                    new_definitions.insert(key_name, Value::Object(new_key_def));
                } else if !key_audio_file.is_empty() {
                    // This key uses a different audio file, we'll skip it in single method
                    println!(
                        "⚠️ Key '{}' uses different audio file '{}', skipping in single method conversion",
                        key_name,
                        key_audio_file
                    );
                } else {
                    // No audio_file specified, create default timing
                    let audio_path = format!("{}/{}", soundpack_dir, main_audio_file);
                    let duration = get_audio_duration_ms(&audio_path).unwrap_or(100.0);

                    let timing = vec![
                        Value::Array(
                            vec![
                                Value::Number(serde_json::Number::from_f64(0.0).unwrap()),
                                Value::Number(serde_json::Number::from_f64(duration).unwrap())
                            ]
                        )
                    ];
                    new_key_def.insert("timing".to_string(), Value::Array(timing));
                    println!("⚠️ Key '{}' got default timing (no audio_file specified)", key_name);
                    new_definitions.insert(key_name, Value::Object(new_key_def));
                }
            }
        }

        config
            .as_object_mut()
            .unwrap()
            .insert("definitions".to_string(), Value::Object(new_definitions));
    }

    // Write the converted config back
    let output_json = serde_json::to_string_pretty(&config)?;
    std::fs::write(config_path, output_json)?;

    println!("✅ Successfully converted to single method");
    Ok(())
}

/// Concatenate multiple audio files into one file
fn concatenate_audio_files(
    audio_files: &[(String, f64)], // (filename, duration)
    soundpack_dir: &str,
    output_filename: &str
) -> Result<(), Box<dyn std::error::Error>> {
    println!("🔧 Concatenating {} audio files...", audio_files.len());

    let mut all_samples = Vec::new();
    let mut sample_rate = 44100u32; // Default sample rate
    let mut channels = 2u16; // Default to stereo

    for (i, (filename, _duration)) in audio_files.iter().enumerate() {
        let file_path = format!("{}/{}", soundpack_dir, filename);
        println!("   📁 Loading audio file {}/{}: {}", i + 1, audio_files.len(), filename);

        if !Path::new(&file_path).exists() {
            println!("   ⚠️ Audio file not found, skipping: {}", file_path);
            continue;
        }

        // Load audio file using Symphonia
        match load_audio_file_samples(&file_path) {
            Ok((samples, file_channels, file_sample_rate)) => {
                // Use the first file's format as reference
                if i == 0 {
                    sample_rate = file_sample_rate;
                    channels = file_channels;
                    println!("   🎵 Using format: {}Hz, {} channels", sample_rate, channels);
                }

                // Convert to target format if needed
                let converted_samples = if
                    file_sample_rate != sample_rate ||
                    file_channels != channels
                {
                    println!(
                        "   🔄 Converting from {}Hz {} channels to {}Hz {} channels",
                        file_sample_rate,
                        file_channels,
                        sample_rate,
                        channels
                    );
                    convert_audio_format(
                        &samples,
                        file_channels,
                        file_sample_rate,
                        channels,
                        sample_rate
                    )
                } else {
                    samples
                };

                // Special debug for Enter audio file
                if filename == "SPMEnter.wav" {
                    let actual_duration_ms =
                        ((converted_samples.len() as f64) /
                            ((sample_rate as f64) * (channels as f64))) *
                        1000.0;
                    println!("🔍 [ENTER CONCAT DEBUG] File: {}", filename);
                    println!("🔍 [ENTER CONCAT DEBUG] Samples: {}", converted_samples.len());
                    println!("🔍 [ENTER CONCAT DEBUG] Sample rate: {}Hz", sample_rate);
                    println!("🔍 [ENTER CONCAT DEBUG] Channels: {}", channels);
                    println!(
                        "🔍 [ENTER CONCAT DEBUG] Actual duration: {:.2}ms",
                        actual_duration_ms
                    );
                    println!(
                        "🔍 [ENTER CONCAT DEBUG] Current position in concat: {:.2}ms",
                        ((all_samples.len() as f64) / ((sample_rate as f64) * (channels as f64))) *
                            1000.0
                    );
                }

                all_samples.extend(&converted_samples);
                println!("   ✅ Added {} samples from {}", converted_samples.len(), filename);
            }
            Err(e) => {
                println!("   ❌ Failed to load {}: {}", filename, e);
                // Continue with other files
            }
        }
    }

    if all_samples.is_empty() {
        return Err("No audio samples were loaded".into());
    } // Save concatenated audio file
    let output_path = format!("{}/{}", soundpack_dir, output_filename);
    save_audio_file(&all_samples, channels, sample_rate, &output_path)?;

    let final_duration_ms =
        ((all_samples.len() as f64) / ((sample_rate as f64) * (channels as f64))) * 1000.0;

    println!("✅ Successfully concatenated audio to: {}", output_path);
    println!("🎵 Total samples: {}, Duration: {:.2}ms", all_samples.len(), final_duration_ms);

    // Special debug output for comparison
    println!("🔍 [CONCAT FINAL DEBUG] Final concatenated duration: {:.2}ms", final_duration_ms);

    Ok(())
}

/// Concatenate multiple audio files and return timing information
/// Returns HashMap with (filename -> (offset_ms, duration_ms))
fn concatenate_audio_files_with_timing(
    audio_files: &[String], // just filenames
    soundpack_dir: &str,
    output_filename: &str
) -> Result<std::collections::HashMap<String, (f64, f64)>, Box<dyn std::error::Error>> {
    println!("🔧 Concatenating {} audio files with timing...", audio_files.len());

    let mut all_samples = Vec::new();
    let mut sample_rate = 44100u32; // Default sample rate
    let mut channels = 2u16; // Default to stereo
    let mut timing_info = std::collections::HashMap::new();

    for (i, filename) in audio_files.iter().enumerate() {
        let file_path = format!("{}/{}", soundpack_dir, filename);
        println!("   📁 Loading audio file {}/{}: {}", i + 1, audio_files.len(), filename);

        if !Path::new(&file_path).exists() {
            println!("   ⚠️ Audio file not found, skipping: {}", file_path);
            continue;
        }

        // Record the current position as the offset for this file
        let current_offset_ms =
            ((all_samples.len() as f64) / ((sample_rate as f64) * (channels as f64))) * 1000.0;

        // Load audio file using Symphonia
        match load_audio_file_samples(&file_path) {
            Ok((samples, file_channels, file_sample_rate)) => {
                // Use the first file's format as reference
                if i == 0 {
                    sample_rate = file_sample_rate;
                    channels = file_channels;
                    println!("   🎵 Using format: {}Hz, {} channels", sample_rate, channels);
                    // Recalculate offset for first file with correct sample rate
                    let corrected_offset =
                        ((all_samples.len() as f64) / ((sample_rate as f64) * (channels as f64))) *
                        1000.0;
                    // Update if needed
                }

                // Convert to target format if needed
                let converted_samples = if
                    file_sample_rate != sample_rate ||
                    file_channels != channels
                {
                    println!(
                        "   🔄 Converting from {}Hz {} channels to {}Hz {} channels",
                        file_sample_rate,
                        file_channels,
                        sample_rate,
                        channels
                    );
                    convert_audio_format(
                        &samples,
                        file_channels,
                        file_sample_rate,
                        channels,
                        sample_rate
                    )
                } else {
                    samples
                };

                // Calculate the actual duration of this file after conversion
                let actual_duration_ms =
                    ((converted_samples.len() as f64) /
                        ((sample_rate as f64) * (channels as f64))) *
                    1000.0;

                // Store timing info for this file
                timing_info.insert(filename.clone(), (current_offset_ms, actual_duration_ms));

                // Special debug for Enter audio file
                if filename == "SPMEnter.wav" {
                    println!("🔍 [ENTER TIMING DEBUG] File: {}", filename);
                    println!("🔍 [ENTER TIMING DEBUG] Offset: {:.2}ms", current_offset_ms);
                    println!("🔍 [ENTER TIMING DEBUG] Duration: {:.2}ms", actual_duration_ms);
                    println!(
                        "🔍 [ENTER TIMING DEBUG] End time: {:.2}ms",
                        current_offset_ms + actual_duration_ms
                    );
                    println!("🔍 [ENTER TIMING DEBUG] Samples: {}", converted_samples.len());
                }

                all_samples.extend(&converted_samples);
                println!(
                    "   ✅ Added {} samples from {} (offset: {:.2}ms, duration: {:.2}ms)",
                    converted_samples.len(),
                    filename,
                    current_offset_ms,
                    actual_duration_ms
                );
            }
            Err(e) => {
                println!("   ❌ Failed to load {}: {}", filename, e);
                // Continue with other files
            }
        }
    }

    if all_samples.is_empty() {
        return Err("No audio samples were loaded".into());
    }

    // Save concatenated audio file
    let output_path = format!("{}/{}", soundpack_dir, output_filename);
    save_audio_file(&all_samples, channels, sample_rate, &output_path)?;

    let final_duration_ms =
        ((all_samples.len() as f64) / ((sample_rate as f64) * (channels as f64))) * 1000.0;

    println!("✅ Successfully concatenated audio to: {}", output_path);
    println!("🎵 Total samples: {}, Final duration: {:.2}ms", all_samples.len(), final_duration_ms);

    // Debug output for Enter file timing
    if let Some((offset, duration)) = timing_info.get("SPMEnter.wav") {
        println!(
            "🔍 [FINAL TIMING DEBUG] SPMEnter.wav: offset={:.2}ms, duration={:.2}ms, end={:.2}ms",
            offset,
            duration,
            offset + duration
        );
        println!("🔍 [FINAL TIMING DEBUG] Concatenated total: {:.2}ms", final_duration_ms);
        if offset + duration > final_duration_ms {
            println!("❌ [FINAL TIMING DEBUG] ERROR: End time exceeds total duration!");
        } else {
            println!("✅ [FINAL TIMING DEBUG] Timing looks correct!");
        }
    }

    Ok(timing_info)
}

/// Load audio file and return samples
fn load_audio_file_samples(
    file_path: &str
) -> Result<(Vec<f32>, u16, u32), Box<dyn std::error::Error>> {
    use symphonia::core::formats::FormatOptions;
    use symphonia::core::io::MediaSourceStream;
    use symphonia::core::meta::MetadataOptions;
    use symphonia::core::probe::Hint;
    use symphonia::core::audio::SampleBuffer;
    use std::fs::File;

    let file = File::open(file_path)?;
    let mss = MediaSourceStream::new(Box::new(file), Default::default());

    let mut hint = Hint::new();
    if let Some(extension) = Path::new(file_path).extension() {
        if let Some(ext_str) = extension.to_str() {
            hint.with_extension(ext_str);
        }
    }

    let meta_opts: MetadataOptions = Default::default();
    let fmt_opts: FormatOptions = Default::default();

    let probed = symphonia::default::get_probe().format(&hint, mss, &fmt_opts, &meta_opts)?;
    let mut format = probed.format;

    let track = format
        .tracks()
        .iter()
        .find(|t| t.codec_params.codec != symphonia::core::codecs::CODEC_TYPE_NULL)
        .ok_or("No supported audio tracks found")?;

    let mut decoder = symphonia::default
        ::get_codecs()
        .make(&track.codec_params, &Default::default())?;

    let track_id = track.id;
    let sample_rate = track.codec_params.sample_rate.unwrap_or(44100);
    let channels = track.codec_params.channels.map(|c| c.count()).unwrap_or(2) as u16;

    let mut samples = Vec::new();
    let mut sample_buf = None;

    loop {
        let packet = match format.next_packet() {
            Ok(packet) => packet,
            Err(_) => {
                break;
            }
        };

        if packet.track_id() != track_id {
            continue;
        }

        match decoder.decode(&packet) {
            Ok(decoded) => {
                if sample_buf.is_none() {
                    let spec = *decoded.spec();
                    let duration = decoded.capacity() as u64;
                    sample_buf = Some(SampleBuffer::<f32>::new(duration, spec));
                }

                if let Some(buf) = &mut sample_buf {
                    buf.copy_interleaved_ref(decoded);
                    samples.extend_from_slice(buf.samples());
                }
            }
            Err(_) => {
                break;
            }
        }
    }

    Ok((samples, channels, sample_rate))
}

/// Convert audio format (sample rate and channel count)
fn convert_audio_format(
    samples: &[f32],
    from_channels: u16,
    from_sample_rate: u32,
    to_channels: u16,
    to_sample_rate: u32
) -> Vec<f32> {
    // Simple conversion - just handle channel conversion for now
    // Sample rate conversion would require more complex resampling

    if from_channels == to_channels && from_sample_rate == to_sample_rate {
        return samples.to_vec();
    }

    // Convert channels
    let channel_converted = if from_channels == 1 && to_channels == 2 {
        // Mono to stereo: duplicate each sample
        samples
            .iter()
            .flat_map(|&sample| vec![sample, sample])
            .collect()
    } else if from_channels == 2 && to_channels == 1 {
        // Stereo to mono: average each pair
        samples
            .chunks(2)
            .map(|chunk| {
                if chunk.len() == 2 { (chunk[0] + chunk[1]) / 2.0 } else { chunk[0] }
            })
            .collect()
    } else {
        samples.to_vec()
    };

    // For now, ignore sample rate conversion (would need proper resampling)
    channel_converted
}

/// Save audio samples to file
fn save_audio_file(
    samples: &[f32],
    channels: u16,
    sample_rate: u32,
    output_path: &str
) -> Result<(), Box<dyn std::error::Error>> {
    use hound;

    let spec = hound::WavSpec {
        channels,
        sample_rate,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };

    let mut writer = hound::WavWriter::create(output_path, spec)?;

    for &sample in samples {
        // Convert f32 to i16
        let sample_i16 = (sample * (i16::MAX as f32)) as i16;
        writer.write_sample(sample_i16)?;
    }

    writer.finalize()?;
    Ok(())
}

/// Create comprehensive IOHook to Web API key mapping
/// Supports all platforms (Windows, Linux, macOS)
fn create_iohook_to_web_key_mapping() -> HashMap<u32, String> {
    let mut mapping = HashMap::new();

    // Basic keys (VC_* constants from IOHook)
    mapping.insert(1, "Escape".to_string()); // VC_ESCAPE = 0x0001
    mapping.insert(2, "Digit1".to_string()); // VC_1 = 0x0002
    mapping.insert(3, "Digit2".to_string()); // VC_2 = 0x0003
    mapping.insert(4, "Digit3".to_string()); // VC_3 = 0x0004
    mapping.insert(5, "Digit4".to_string()); // VC_4 = 0x0005
    mapping.insert(6, "Digit5".to_string()); // VC_5 = 0x0006
    mapping.insert(7, "Digit6".to_string()); // VC_6 = 0x0007
    mapping.insert(8, "Digit7".to_string()); // VC_7 = 0x0008
    mapping.insert(9, "Digit8".to_string()); // VC_8 = 0x0009
    mapping.insert(10, "Digit9".to_string()); // VC_9 = 0x000A
    mapping.insert(11, "Digit0".to_string()); // VC_0 = 0x000B
    mapping.insert(12, "Minus".to_string()); // VC_MINUS = 0x000C
    mapping.insert(13, "Equal".to_string()); // VC_EQUALS = 0x000D
    mapping.insert(14, "Backspace".to_string()); // VC_BACKSPACE = 0x000E
    mapping.insert(15, "Tab".to_string()); // VC_TAB = 0x000F
    mapping.insert(16, "KeyQ".to_string()); // VC_Q = 0x0010
    mapping.insert(17, "KeyW".to_string()); // VC_W = 0x0011
    mapping.insert(18, "KeyE".to_string()); // VC_E = 0x0012
    mapping.insert(19, "KeyR".to_string()); // VC_R = 0x0013
    mapping.insert(20, "KeyT".to_string()); // VC_T = 0x0014
    mapping.insert(21, "KeyY".to_string()); // VC_Y = 0x0015
    mapping.insert(22, "KeyU".to_string()); // VC_U = 0x0016
    mapping.insert(23, "KeyI".to_string()); // VC_I = 0x0017
    mapping.insert(24, "KeyO".to_string()); // VC_O = 0x0018
    mapping.insert(25, "KeyP".to_string()); // VC_P = 0x0019
    mapping.insert(26, "BracketLeft".to_string()); // VC_OPEN_BRACKET = 0x001A
    mapping.insert(27, "BracketRight".to_string()); // VC_CLOSE_BRACKET = 0x001B
    mapping.insert(28, "Enter".to_string()); // VC_ENTER = 0x001C
    mapping.insert(29, "ControlLeft".to_string()); // VC_CONTROL_L = 0x001D
    mapping.insert(30, "KeyA".to_string()); // VC_A = 0x001E
    mapping.insert(31, "KeyS".to_string()); // VC_S = 0x001F
    mapping.insert(32, "KeyD".to_string()); // VC_D = 0x0020
    mapping.insert(33, "KeyF".to_string()); // VC_F = 0x0021
    mapping.insert(34, "KeyG".to_string()); // VC_G = 0x0022
    mapping.insert(35, "KeyH".to_string()); // VC_H = 0x0023
    mapping.insert(36, "KeyJ".to_string()); // VC_J = 0x0024
    mapping.insert(37, "KeyK".to_string()); // VC_K = 0x0025
    mapping.insert(38, "KeyL".to_string()); // VC_L = 0x0026
    mapping.insert(39, "Semicolon".to_string()); // VC_SEMICOLON = 0x0027
    mapping.insert(40, "Quote".to_string()); // VC_QUOTE = 0x0028
    mapping.insert(41, "Backquote".to_string()); // VC_BACKQUOTE = 0x0029
    mapping.insert(42, "ShiftLeft".to_string()); // VC_SHIFT_L = 0x002A
    mapping.insert(43, "Backslash".to_string()); // VC_BACK_SLASH = 0x002B
    mapping.insert(44, "KeyZ".to_string()); // VC_Z = 0x002C
    mapping.insert(45, "KeyX".to_string()); // VC_X = 0x002D
    mapping.insert(46, "KeyC".to_string()); // VC_C = 0x002E
    mapping.insert(47, "KeyV".to_string()); // VC_V = 0x002F
    mapping.insert(48, "KeyB".to_string()); // VC_B = 0x0030
    mapping.insert(49, "KeyN".to_string()); // VC_N = 0x0031
    mapping.insert(50, "KeyM".to_string()); // VC_M = 0x0032
    mapping.insert(51, "Comma".to_string()); // VC_COMMA = 0x0033
    mapping.insert(52, "Period".to_string()); // VC_PERIOD = 0x0034
    mapping.insert(53, "Slash".to_string()); // VC_SLASH = 0x0035
    mapping.insert(54, "ShiftRight".to_string()); // VC_SHIFT_R = 0x0036
    mapping.insert(55, "NumpadMultiply".to_string()); // VC_KP_MULTIPLY = 0x0037
    mapping.insert(56, "AltLeft".to_string()); // VC_ALT_L = 0x0038
    mapping.insert(57, "Space".to_string()); // VC_SPACE = 0x0039
    mapping.insert(58, "CapsLock".to_string()); // VC_CAPS_LOCK = 0x003A

    // Function keys F1-F12
    mapping.insert(59, "F1".to_string()); // VC_F1 = 0x003B
    mapping.insert(60, "F2".to_string()); // VC_F2 = 0x003C
    mapping.insert(61, "F3".to_string()); // VC_F3 = 0x003D
    mapping.insert(62, "F4".to_string()); // VC_F4 = 0x003E
    mapping.insert(63, "F5".to_string()); // VC_F5 = 0x003F
    mapping.insert(64, "F6".to_string()); // VC_F6 = 0x0040
    mapping.insert(65, "F7".to_string()); // VC_F7 = 0x0041
    mapping.insert(66, "F8".to_string()); // VC_F8 = 0x0042
    mapping.insert(67, "F9".to_string()); // VC_F9 = 0x0043
    mapping.insert(68, "F10".to_string()); // VC_F10 = 0x0044
    mapping.insert(69, "NumLock".to_string()); // VC_NUM_LOCK = 0x0045
    mapping.insert(70, "ScrollLock".to_string()); // VC_SCROLL_LOCK = 0x0046

    // Numpad keys
    mapping.insert(71, "Numpad7".to_string()); // VC_KP_7 = 0x0047
    mapping.insert(72, "Numpad8".to_string()); // VC_KP_8 = 0x0048
    mapping.insert(73, "Numpad9".to_string()); // VC_KP_9 = 0x0049
    mapping.insert(74, "NumpadSubtract".to_string()); // VC_KP_SUBTRACT = 0x004A
    mapping.insert(75, "Numpad4".to_string()); // VC_KP_4 = 0x004B
    mapping.insert(76, "Numpad5".to_string()); // VC_KP_5 = 0x004C
    mapping.insert(77, "Numpad6".to_string()); // VC_KP_6 = 0x004D
    mapping.insert(78, "NumpadAdd".to_string()); // VC_KP_ADD = 0x004E
    mapping.insert(79, "Numpad1".to_string()); // VC_KP_1 = 0x004F
    mapping.insert(80, "Numpad2".to_string()); // VC_KP_2 = 0x0050
    mapping.insert(81, "Numpad3".to_string()); // VC_KP_3 = 0x0051
    mapping.insert(82, "Numpad0".to_string()); // VC_KP_0 = 0x0052
    mapping.insert(83, "NumpadDecimal".to_string()); // VC_KP_SEPARATOR = 0x0053

    // Function keys F11-F24
    mapping.insert(87, "F11".to_string()); // VC_F11 = 0x0057
    mapping.insert(88, "F12".to_string()); // VC_F12 = 0x0058
    mapping.insert(91, "F13".to_string()); // VC_F13 = 0x005B
    mapping.insert(92, "F14".to_string()); // VC_F14 = 0x005C
    mapping.insert(93, "F15".to_string()); // VC_F15 = 0x005D
    mapping.insert(99, "F16".to_string()); // VC_F16 = 0x0063
    mapping.insert(100, "F17".to_string()); // VC_F17 = 0x0064
    mapping.insert(101, "F18".to_string()); // VC_F18 = 0x0065
    mapping.insert(102, "F19".to_string()); // VC_F19 = 0x0066
    mapping.insert(103, "F20".to_string()); // VC_F20 = 0x0067
    mapping.insert(104, "F21".to_string()); // VC_F21 = 0x0068
    mapping.insert(105, "F22".to_string()); // VC_F22 = 0x0069
    mapping.insert(106, "F23".to_string()); // VC_F23 = 0x006A
    mapping.insert(107, "F24".to_string()); // VC_F24 = 0x006B

    // Japanese language keys
    mapping.insert(112, "Convert".to_string()); // VC_KATAKANA = 0x0070
    mapping.insert(115, "Lang1".to_string()); // VC_UNDERSCORE = 0x0073
    mapping.insert(119, "Lang2".to_string()); // VC_FURIGANA = 0x0077
    mapping.insert(121, "KanaMode".to_string()); // VC_KANJI = 0x0079
    mapping.insert(123, "HiraganaKatakana".to_string()); // VC_HIRAGANA = 0x007B
    mapping.insert(125, "IntlYen".to_string()); // VC_YEN = 0x007D
    mapping.insert(126, "NumpadComma".to_string()); // VC_KP_COMMA = 0x007E

    // Extended keys (proper extended scancode values)
    // Extended numpad and control keys
    mapping.insert(3637, "NumpadDivide".to_string()); // VC_KP_DIVIDE = 0x0E35
    mapping.insert(3612, "NumpadEnter".to_string()); // VC_KP_ENTER = 0x0E1C
    mapping.insert(3597, "ControlRight".to_string()); // VC_CONTROL_R = 0x0E1D
    mapping.insert(3645, "NumpadEquals".to_string()); // VC_KP_EQUALS = 0x0E0D    // Navigation cluster - using CORRECT 0xE0xx values (fixed from incorrect mapping)
    mapping.insert(57399, "PrintScreen".to_string()); // VC_PRINTSCREEN = 0xE037 = 57399
    mapping.insert(58437, "Pause".to_string()); // VC_PAUSE = 0xE045 = 57413 (keeping old for compatibility)
    mapping.insert(57415, "Home".to_string()); // VC_HOME = 0xE047 = 57415
    mapping.insert(57416, "ArrowUp".to_string()); // VC_UP = 0xE048 = 57416 ✓ CORRECT
    mapping.insert(57417, "PageUp".to_string()); // VC_PAGE_UP = 0xE049 = 57417
    mapping.insert(57419, "ArrowLeft".to_string()); // VC_LEFT = 0xE04B = 57419 ✓ CORRECT
    mapping.insert(57421, "ArrowRight".to_string()); // VC_RIGHT = 0xE04D = 57421 ✓ CORRECT
    mapping.insert(57423, "End".to_string()); // VC_END = 0xE04F = 57423
    mapping.insert(57424, "ArrowDown".to_string()); // VC_DOWN = 0xE050 = 57424 ✓ CORRECT
    mapping.insert(57425, "PageDown".to_string()); // VC_PAGE_DOWN = 0xE051 = 57425
    mapping.insert(57426, "Insert".to_string()); // VC_INSERT = 0xE052 = 57426
    mapping.insert(57427, "Delete".to_string()); // VC_DELETE = 0xE053 = 57427    // Extended modifier keys - using CORRECT IOHook values
    mapping.insert(57400, "AltRight".to_string()); // VC_ALT_R = 0xE038 = 57400
    mapping.insert(57435, "MetaLeft".to_string()); // VC_META_L = 0xE05B = 57435
    mapping.insert(57436, "MetaRight".to_string()); // VC_META_R = 0xE05C = 57436
    mapping.insert(57437, "ContextMenu".to_string()); // VC_CONTEXT_MENU = 0xE05D = 57437    // Power and sleep keys - using CORRECT IOHook values
    mapping.insert(57438, "Power".to_string()); // VC_POWER = 0xE05E = 57438
    mapping.insert(57439, "Sleep".to_string()); // VC_SLEEP = 0xE05F = 57439
    mapping.insert(57443, "WakeUp".to_string()); // VC_WAKE = 0xE063 = 57443

    // Media keys (correct 0xE0xx values)
    mapping.insert(57360, "MediaTrackPrevious".to_string()); // VC_MEDIA_PREVIOUS = 0xE010
    mapping.insert(57369, "MediaTrackNext".to_string()); // VC_MEDIA_NEXT = 0xE019
    mapping.insert(57376, "AudioVolumeMute".to_string()); // VC_VOLUME_MUTE = 0xE020
    mapping.insert(57377, "LaunchApp2".to_string()); // VC_APP_CALCULATOR = 0xE021
    mapping.insert(57378, "MediaPlayPause".to_string()); // VC_MEDIA_PLAY = 0xE022
    mapping.insert(57380, "MediaStop".to_string()); // VC_MEDIA_STOP = 0xE024
    mapping.insert(57390, "AudioVolumeDown".to_string()); // VC_VOLUME_DOWN = 0xE02E
    mapping.insert(57392, "AudioVolumeUp".to_string()); // VC_VOLUME_UP = 0xE030
    mapping.insert(57394, "BrowserHome".to_string()); // VC_BROWSER_HOME = 0xE032
    mapping.insert(57404, "LaunchApp1".to_string()); // VC_APP_MUSIC = 0xE03C
    mapping.insert(57444, "LaunchApp3".to_string()); // VC_APP_PICTURES = 0xE064
    mapping.insert(57445, "BrowserSearch".to_string()); // VC_BROWSER_SEARCH = 0xE065
    mapping.insert(57446, "BrowserFavorites".to_string()); // VC_BROWSER_FAVORITES = 0xE066
    mapping.insert(57447, "BrowserRefresh".to_string()); // VC_BROWSER_REFRESH = 0xE067
    mapping.insert(57448, "BrowserStop".to_string()); // VC_BROWSER_STOP = 0xE068
    mapping.insert(57449, "BrowserForward".to_string()); // VC_BROWSER_FORWARD = 0xE069
    mapping.insert(57450, "BrowserBack".to_string()); // VC_BROWSER_BACK = 0xE06A
    mapping.insert(57452, "LaunchMail".to_string()); // VC_APP_MAIL = 0xE06C
    mapping.insert(57453, "MediaSelect".to_string()); // VC_MEDIA_SELECT = 0xE06D

    // Alternate keycode ranges for compatibility
    // Some systems may report different keycode values for extended keys

    // Clear key and additional special keys
    mapping.insert(58444, "Clear".to_string()); // VC_CLEAR = 0xE04C (alternate)
    mapping.insert(58470, "IntlBackslash".to_string()); // VC_LESSER_GREATER = 0xE046

    // Legacy compatibility mappings for V1 configs and alternative implementations
    // These handle cases where different IOHook implementations use different ranges

    // Alternative numpad mappings (some implementations use these ranges)
    mapping.insert(3597, "NumLock".to_string()); // Alternative range
    mapping.insert(3612, "NumpadDivide".to_string()); // Alternative range
    mapping.insert(3613, "NumpadMultiply".to_string()); // Alternative range
    mapping.insert(3639, "Numpad7".to_string()); // Alternative range
    mapping.insert(3640, "Numpad8".to_string()); // Alternative range
    mapping.insert(3653, "Numpad9".to_string()); // Alternative range
    mapping.insert(3655, "NumpadAdd".to_string()); // Alternative range
    mapping.insert(3657, "Numpad4".to_string()); // Alternative range
    mapping.insert(3663, "Numpad5".to_string()); // Alternative range
    mapping.insert(3665, "Numpad6".to_string()); // Alternative range
    mapping.insert(3666, "Numpad1".to_string()); // Alternative range
    mapping.insert(3667, "Numpad2".to_string()); // Alternative range
    mapping.insert(3675, "Numpad3".to_string()); // Alternative range
    mapping.insert(3676, "NumpadEnter".to_string()); // Alternative range
    mapping.insert(3677, "Numpad0".to_string()); // Alternative range

    // Alternative extended key mappings for broader compatibility
    mapping.insert(60999, "Insert".to_string()); // V1 compatibility
    mapping.insert(61000, "Delete".to_string()); // V1 compatibility
    mapping.insert(61001, "Home".to_string()); // V1 compatibility
    mapping.insert(61003, "End".to_string()); // V1 compatibility
    mapping.insert(61005, "PageUp".to_string()); // V1 compatibility
    mapping.insert(61007, "PageDown".to_string()); // V1 compatibility
    mapping.insert(61008, "PrintScreen".to_string()); // V1 compatibility
    mapping.insert(61009, "ScrollLock".to_string()); // V1 compatibility
    mapping.insert(61010, "Pause".to_string()); // V1 compatibility
    mapping.insert(61011, "NumpadDecimal".to_string()); // V1 compatibility

    // Additional platform-specific keycodes that might appear
    mapping.insert(94, "IntlBackslash".to_string()); // Less/Greater key on some keyboards
    mapping.insert(95, "Fn".to_string()); // Function key modifier
    mapping.insert(96, "Clear".to_string()); // Clear key on some keyboards

    // Handle potential Sun keyboard extensions (rarely used but in IOHook)
    mapping.insert(65397, "Help".to_string()); // VC_SUN_HELP = 0xFF75
    mapping.insert(65398, "Props".to_string()); // VC_SUN_PROPS = 0xFF76
    mapping.insert(65399, "Front".to_string()); // VC_SUN_FRONT = 0xFF77
    mapping.insert(65400, "Stop".to_string()); // VC_SUN_STOP = 0xFF78
    mapping.insert(65401, "Again".to_string()); // VC_SUN_AGAIN = 0xFF79
    mapping.insert(65402, "Undo".to_string()); // VC_SUN_UNDO = 0xFF7A
    mapping.insert(65403, "Cut".to_string()); // VC_SUN_CUT = 0xFF7B
    mapping.insert(65404, "Copy".to_string()); // VC_SUN_COPY = 0xFF7C
    mapping.insert(65405, "Paste".to_string()); // VC_SUN_INSERT = 0xFF7D
    mapping.insert(65406, "Find".to_string()); // VC_SUN_FIND = 0xFF7E

    mapping
}
