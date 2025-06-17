use crate::state::config::AppConfig;
use crate::state::paths;
use crate::state::soundpack::SoundPack;
use crate::state::soundpack::{ SoundpackCache, SoundpackMetadata };

use super::audio_context::AudioContext;

/// Determine soundpack type based on the soundpack path
fn determine_soundpack_type(soundpack_id: &str) -> crate::state::soundpack::SoundpackType {
    if soundpack_id.starts_with("keyboard/") || soundpack_id.starts_with("keyboard\\") {
        crate::state::soundpack::SoundpackType::Keyboard
    } else if soundpack_id.starts_with("mouse/") || soundpack_id.starts_with("mouse\\") {
        crate::state::soundpack::SoundpackType::Mouse
    } else {
        // Default to keyboard for backwards compatibility
        crate::state::soundpack::SoundpackType::Keyboard
    }
}

pub fn load_soundpack(context: &AudioContext) -> Result<(), String> {
    let config = AppConfig::load();
    // Load both keyboard and mouse soundpacks
    load_keyboard_soundpack(context, &config.keyboard_soundpack)?;
    load_mouse_soundpack(context, &config.mouse_soundpack)?;
    Ok(())
}

pub fn load_keyboard_soundpack(context: &AudioContext, soundpack_id: &str) -> Result<(), String> {
    load_keyboard_soundpack_with_cache_control(context, soundpack_id, true)
}

pub fn load_keyboard_soundpack_with_cache_control(
    context: &AudioContext,
    soundpack_id: &str,
    update_cache_on_error: bool
) -> Result<(), String> {
    // Skip loading if soundpack ID is empty
    if soundpack_id.is_empty() {
        println!("⚠️ Skipping keyboard soundpack loading: empty soundpack ID");
        return Ok(()); // Return success to avoid error handling
    }

    println!("🎹 Loading keyboard soundpack: {}", soundpack_id);
    match load_keyboard_soundpack_optimized(context, soundpack_id, update_cache_on_error) {
        Ok(()) => Ok(()),
        Err(e) => {
            // Capture the error in cache
            capture_soundpack_loading_error(soundpack_id, &e);
            Err(e)
        }
    }
}

pub fn load_mouse_soundpack(context: &AudioContext, soundpack_id: &str) -> Result<(), String> {
    load_mouse_soundpack_with_cache_control(context, soundpack_id, true)
}

pub fn load_mouse_soundpack_with_cache_control(
    context: &AudioContext,
    soundpack_id: &str,
    update_cache_on_error: bool
) -> Result<(), String> {
    // Skip loading if soundpack ID is empty
    if soundpack_id.is_empty() {
        println!("⚠️ Skipping mouse soundpack loading: empty soundpack ID");
        return Ok(()); // Return success to avoid error handling
    }

    println!("🖱️ Loading mouse soundpack: {}", soundpack_id);
    match load_mouse_soundpack_optimized(context, soundpack_id, update_cache_on_error) {
        Ok(()) => Ok(()),
        Err(e) => {
            // Capture the error in cache only if requested
            if update_cache_on_error {
                capture_soundpack_loading_error(soundpack_id, &e);
            }
            Err(e)
        }
    }
}

fn load_audio_file(
    soundpack_path: &str,
    soundpack: &SoundPack
) -> Result<(Vec<f32>, u16, u32), String> {
    let sound_file_path = soundpack.audio_file
        .as_ref()
        .map(|src| format!("{}/{}", soundpack_path, src.trim_start_matches("./")))
        .ok_or_else(|| "No audio_file field in soundpack config".to_string())?;

    if !std::path::Path::new(&sound_file_path).exists() {
        return Err(format!("Sound file not found: {}", sound_file_path));
    }

    // Use Symphonia for audio loading instead of Rodio
    match load_audio_with_symphonia(&sound_file_path) {
        Ok((samples, channels, sample_rate)) => { Ok((samples, channels, sample_rate)) }
        Err(e) => { Err(format!("Failed to load audio: {}", e)) }
    }
}

/// Load audio file using Symphonia for consistent duration detection
fn load_audio_with_symphonia(file_path: &str) -> Result<(Vec<f32>, u16, u32), String> {
    use symphonia::core::audio::{ AudioBufferRef, Signal };
    use symphonia::core::codecs::{ DecoderOptions, CODEC_TYPE_NULL };
    use symphonia::core::formats::FormatOptions;
    use symphonia::core::io::MediaSourceStream;
    use symphonia::core::meta::MetadataOptions;
    use symphonia::core::probe::Hint;
    use std::fs::File;

    // First, check if file exists and has content
    let metadata = std::fs
        ::metadata(file_path)
        .map_err(|e| format!("Failed to get file metadata: {}", e))?;
    if metadata.len() == 0 {
        return Err(format!("Audio file is empty: {}", file_path));
    }

    let file = File::open(file_path).map_err(|e| format!("Failed to open file: {}", e))?;
    let mss = MediaSourceStream::new(Box::new(file), Default::default());
    let mut hint = Hint::new();
    if let Some(extension) = std::path::Path::new(file_path).extension() {
        if let Some(ext_str) = extension.to_str() {
            hint.with_extension(ext_str);
        }
    }

    let meta_opts: MetadataOptions = Default::default();
    let fmt_opts: FormatOptions = Default::default();

    let probed = symphonia::default
        ::get_probe()
        .format(&hint, mss, &fmt_opts, &meta_opts)
        .map_err(|e| {
            format!(
                "Failed to probe format for '{}': {} (file size: {} bytes)",
                file_path,
                e,
                metadata.len()
            )
        })?;

    let mut format = probed.format;
    let track = format
        .tracks()
        .iter()
        .find(|t| t.codec_params.codec != CODEC_TYPE_NULL)
        .ok_or("No supported audio tracks found")?;

    let dec_opts: DecoderOptions = Default::default();
    let mut decoder = symphonia::default
        ::get_codecs()
        .make(&track.codec_params, &dec_opts)
        .map_err(|e| format!("Failed to create decoder: {}", e))?;

    let track_id = track.id;
    let mut samples = Vec::new();
    let mut sample_rate = 44100u32;
    let mut channels = 2u16;

    // Decode audio packets
    loop {
        let packet = match format.next_packet() {
            Ok(packet) => packet,
            Err(_) => {
                break;
            } // End of stream
        };

        if packet.track_id() != track_id {
            continue;
        }

        match decoder.decode(&packet) {
            Ok(decoded) => {
                if samples.is_empty() {
                    // Get format info from first decoded buffer
                    sample_rate = decoded.spec().rate;
                    channels = decoded.spec().channels.count() as u16;
                } // Convert audio buffer to f32 samples
                match decoded {
                    AudioBufferRef::F32(buf) => {
                        if channels == 1 {
                            // Mono audio
                            for &sample in buf.chan(0) {
                                samples.push(sample);
                            }
                        } else {
                            // Stereo audio - interleave samples correctly
                            let left_chan = buf.chan(0);
                            let right_chan = if buf.spec().channels.count() > 1 {
                                buf.chan(1)
                            } else {
                                buf.chan(0)
                            };
                            for (left, right) in left_chan.iter().zip(right_chan.iter()) {
                                samples.push(*left);
                                samples.push(*right);
                            }
                        }
                    }
                    AudioBufferRef::S32(buf) => {
                        if channels == 1 {
                            for &sample in buf.chan(0) {
                                samples.push((sample as f32) / (i32::MAX as f32));
                            }
                        } else {
                            let left_chan = buf.chan(0);
                            let right_chan = if buf.spec().channels.count() > 1 {
                                buf.chan(1)
                            } else {
                                buf.chan(0)
                            };
                            for (left, right) in left_chan.iter().zip(right_chan.iter()) {
                                samples.push((*left as f32) / (i32::MAX as f32));
                                samples.push((*right as f32) / (i32::MAX as f32));
                            }
                        }
                    }
                    AudioBufferRef::S16(buf) => {
                        if channels == 1 {
                            for &sample in buf.chan(0) {
                                samples.push((sample as f32) / (i16::MAX as f32));
                            }
                        } else {
                            let left_chan = buf.chan(0);
                            let right_chan = if buf.spec().channels.count() > 1 {
                                buf.chan(1)
                            } else {
                                buf.chan(0)
                            };
                            for (left, right) in left_chan.iter().zip(right_chan.iter()) {
                                samples.push((*left as f32) / (i16::MAX as f32));
                                samples.push((*right as f32) / (i16::MAX as f32));
                            }
                        }
                    }
                    AudioBufferRef::U32(buf) => {
                        if channels == 1 {
                            for &sample in buf.chan(0) {
                                samples.push(
                                    ((sample as f32) - (u32::MAX as f32) / 2.0) /
                                        ((u32::MAX as f32) / 2.0)
                                );
                            }
                        } else {
                            let left_chan = buf.chan(0);
                            let right_chan = if buf.spec().channels.count() > 1 {
                                buf.chan(1)
                            } else {
                                buf.chan(0)
                            };
                            for (left, right) in left_chan.iter().zip(right_chan.iter()) {
                                samples.push(
                                    ((*left as f32) - (u32::MAX as f32) / 2.0) /
                                        ((u32::MAX as f32) / 2.0)
                                );
                                samples.push(
                                    ((*right as f32) - (u32::MAX as f32) / 2.0) /
                                        ((u32::MAX as f32) / 2.0)
                                );
                            }
                        }
                    }
                    AudioBufferRef::U16(buf) => {
                        if channels == 1 {
                            for &sample in buf.chan(0) {
                                samples.push(
                                    ((sample as f32) - (u16::MAX as f32) / 2.0) /
                                        ((u16::MAX as f32) / 2.0)
                                );
                            }
                        } else {
                            let left_chan = buf.chan(0);
                            let right_chan = if buf.spec().channels.count() > 1 {
                                buf.chan(1)
                            } else {
                                buf.chan(0)
                            };
                            for (left, right) in left_chan.iter().zip(right_chan.iter()) {
                                samples.push(
                                    ((*left as f32) - (u16::MAX as f32) / 2.0) /
                                        ((u16::MAX as f32) / 2.0)
                                );
                                samples.push(
                                    ((*right as f32) - (u16::MAX as f32) / 2.0) /
                                        ((u16::MAX as f32) / 2.0)
                                );
                            }
                        }
                    }
                    AudioBufferRef::U8(buf) => {
                        if channels == 1 {
                            for &sample in buf.chan(0) {
                                samples.push(((sample as f32) - 128.0) / 128.0);
                            }
                        } else {
                            let left_chan = buf.chan(0);
                            let right_chan = if buf.spec().channels.count() > 1 {
                                buf.chan(1)
                            } else {
                                buf.chan(0)
                            };
                            for (left, right) in left_chan.iter().zip(right_chan.iter()) {
                                samples.push(((*left as f32) - 128.0) / 128.0);
                                samples.push(((*right as f32) - 128.0) / 128.0);
                            }
                        }
                    }
                    AudioBufferRef::S8(buf) => {
                        if channels == 1 {
                            for &sample in buf.chan(0) {
                                samples.push((sample as f32) / (i8::MAX as f32));
                            }
                        } else {
                            let left_chan = buf.chan(0);
                            let right_chan = if buf.spec().channels.count() > 1 {
                                buf.chan(1)
                            } else {
                                buf.chan(0)
                            };
                            for (left, right) in left_chan.iter().zip(right_chan.iter()) {
                                samples.push((*left as f32) / (i8::MAX as f32));
                                samples.push((*right as f32) / (i8::MAX as f32));
                            }
                        }
                    }
                    AudioBufferRef::F64(buf) => {
                        if channels == 1 {
                            for &sample in buf.chan(0) {
                                samples.push(sample as f32);
                            }
                        } else {
                            let left_chan = buf.chan(0);
                            let right_chan = if buf.spec().channels.count() > 1 {
                                buf.chan(1)
                            } else {
                                buf.chan(0)
                            };
                            for (left, right) in left_chan.iter().zip(right_chan.iter()) {
                                samples.push(*left as f32);
                                samples.push(*right as f32);
                            }
                        }
                    }
                    AudioBufferRef::U24(buf) => {
                        if channels == 1 {
                            for &sample in buf.chan(0) {
                                let sample_f32 = ((sample.inner() as f32) - 8388608.0) / 8388608.0; // 2^23
                                samples.push(sample_f32);
                            }
                        } else {
                            let left_chan = buf.chan(0);
                            let right_chan = if buf.spec().channels.count() > 1 {
                                buf.chan(1)
                            } else {
                                buf.chan(0)
                            };
                            for (left, right) in left_chan.iter().zip(right_chan.iter()) {
                                let left_f32 = ((left.inner() as f32) - 8388608.0) / 8388608.0;
                                let right_f32 = ((right.inner() as f32) - 8388608.0) / 8388608.0;
                                samples.push(left_f32);
                                samples.push(right_f32);
                            }
                        }
                    }
                    AudioBufferRef::S24(buf) => {
                        if channels == 1 {
                            for &sample in buf.chan(0) {
                                let sample_f32 = (sample.inner() as f32) / 8388607.0; // 2^23 - 1
                                samples.push(sample_f32);
                            }
                        } else {
                            let left_chan = buf.chan(0);
                            let right_chan = if buf.spec().channels.count() > 1 {
                                buf.chan(1)
                            } else {
                                buf.chan(0)
                            };
                            for (left, right) in left_chan.iter().zip(right_chan.iter()) {
                                let left_f32 = (left.inner() as f32) / 8388607.0;
                                let right_f32 = (right.inner() as f32) / 8388607.0;
                                samples.push(left_f32);
                                samples.push(right_f32);
                            }
                        }
                    }
                }
            }
            Err(e) => {
                println!("⚠️ [DEBUG] Decode error (continuing): {}", e);
                continue;
            }
        }
    }

    if samples.is_empty() {
        return Err("No audio data decoded".to_string());
    }

    Ok((samples, channels, sample_rate))
}

/// Direct keyboard soundpack loading
pub fn load_keyboard_soundpack_optimized(
    context: &AudioContext,
    soundpack_id: &str,
    update_cache_on_error: bool
) -> Result<(), String> {
    println!("📂 Direct loading keyboard soundpack: {}", soundpack_id);

    // Load soundpack directly from filesystem
    let soundpack_path = paths::soundpacks::soundpack_dir(soundpack_id);

    // Load config.json
    let config_path = paths::soundpacks::config_json(soundpack_id);
    let config_content = std::fs
        ::read_to_string(&config_path)
        .map_err(|e| format!("Failed to read config: {}", e))?; // Only load V2 config format - V1 configs must be converted first
    let mut soundpack: SoundPack = serde_json::from_str(&config_content).map_err(|e| {
        // Check if this might be a V1 config
        if config_content.contains("\"key_define_type\"") || config_content.contains("\"defines\"") {
            format!("This appears to be a V1 soundpack config. Please convert it to V2 format first using the refresh/convert function. Parse error: {}", e)
        } else {
            format!("Failed to parse V2 soundpack config: {}", e)
        }
    })?;

    // Override soundpack_type based on folder path (more reliable than JSON content)
    soundpack.soundpack_type = determine_soundpack_type(soundpack_id);

    // Verify this is a keyboard soundpack
    if soundpack.soundpack_type != crate::state::soundpack::SoundpackType::Keyboard {
        return Err("This is a mouse soundpack, not a keyboard soundpack".to_string());
    }

    // Load audio samples directly from file
    let samples = load_audio_file(&soundpack_path, &soundpack)?;

    // Create key mappings (only for keyboard soundpacks)
    let key_mappings = create_key_mappings(&soundpack, &samples.0); // Update audio context with keyboard data
    update_keyboard_context(context, samples, key_mappings, &soundpack)?;

    // Update metadata cache - create metadata with no error since loading succeeded
    let mut cache = SoundpackCache::load();
    match create_soundpack_metadata(&soundpack_path, &soundpack) {
        Ok(metadata) => {
            cache.add_soundpack(metadata);
        }
        Err(e) => {
            println!("⚠️ Failed to create metadata for {}: {}", soundpack_id, e);

            // Only add error metadata to cache if requested (not during startup)
            if update_cache_on_error {
                // Create minimal metadata with error information
                let error_metadata = SoundpackMetadata {
                    id: soundpack.id.clone(), // Use original ID from config
                    name: soundpack.name.clone(),
                    author: soundpack.author.clone(),
                    description: soundpack.description.clone(),
                    version: soundpack.version.clone().unwrap_or_else(|| "1.0".to_string()),
                    tags: soundpack.tags.clone().unwrap_or_default(),
                    icon: soundpack.icon.clone(),
                    soundpack_type: determine_soundpack_type(soundpack_id),
                    folder_path: soundpack_id.to_string(), // Add folder_path for correct path resolution
                    last_modified: 0,
                    last_accessed: std::time::SystemTime
                        ::now()
                        .duration_since(std::time::SystemTime::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_secs(),
                    config_version: soundpack.config_version
                        .as_ref()
                        .and_then(|v| v.parse::<u32>().ok()),
                    is_valid_v2: true,
                    validation_status: "loaded_with_metadata_error".to_string(),
                    can_be_converted: false,
                    last_error: Some(format!("Metadata creation failed: {}", e)),
                };
                cache.add_soundpack(error_metadata);
            }
        }
    }
    cache.save();

    println!("✅ Successfully loaded keyboard soundpack: {} (direct from files)", soundpack.name);
    Ok(())
}

/// Direct mouse soundpack loading
pub fn load_mouse_soundpack_optimized(
    context: &AudioContext,
    soundpack_id: &str,
    update_cache_on_error: bool
) -> Result<(), String> {
    println!("📂 Direct loading mouse soundpack: {}", soundpack_id);

    // Load soundpack directly from filesystem
    let soundpack_path = paths::soundpacks::soundpack_dir(soundpack_id);

    // Load config.json
    let config_path = paths::soundpacks::config_json(soundpack_id);
    let config_content = std::fs
        ::read_to_string(&config_path)
        .map_err(|e| format!("Failed to read config: {}", e))?;
    let mut soundpack: SoundPack = serde_json
        ::from_str(&config_content)
        .map_err(|e| format!("Failed to parse config: {}", e))?;

    // Override soundpack_type based on folder path (more reliable than JSON content)
    soundpack.soundpack_type = determine_soundpack_type(soundpack_id);

    // Verify this is a mouse soundpack
    if soundpack.soundpack_type != crate::state::soundpack::SoundpackType::Mouse {
        return Err("This is a keyboard soundpack, not a mouse soundpack".to_string());
    }

    // Load audio samples directly from file
    let samples = load_audio_file(&soundpack_path, &soundpack)?;

    // Create mouse mappings (only for mouse soundpacks)
    let mouse_mappings = create_mouse_mappings(&soundpack, &samples.0); // Update audio context with mouse data
    update_mouse_context(context, samples, mouse_mappings, &soundpack)?;

    // Update metadata cache - create metadata with no error since loading succeeded
    let mut cache = SoundpackCache::load();
    match create_soundpack_metadata(&soundpack_path, &soundpack) {
        Ok(metadata) => {
            cache.add_soundpack(metadata);
        }
        Err(e) => {
            println!("⚠️ Failed to create metadata for {}: {}", soundpack_id, e);

            // Only add error metadata to cache if requested (not during startup)
            if update_cache_on_error {
                // Create minimal metadata with error information
                let error_metadata = SoundpackMetadata {
                    id: soundpack.id.clone(), // Use original ID from config
                    name: soundpack.name.clone(),
                    author: soundpack.author.clone(),
                    description: soundpack.description.clone(),
                    version: soundpack.version.clone().unwrap_or_else(|| "1.0".to_string()),
                    tags: soundpack.tags.clone().unwrap_or_default(),
                    icon: soundpack.icon.clone(),
                    soundpack_type: determine_soundpack_type(soundpack_id),
                    folder_path: soundpack_id.to_string(), // Add folder_path for correct path resolution
                    last_modified: 0,
                    last_accessed: std::time::SystemTime
                        ::now()
                        .duration_since(std::time::SystemTime::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_secs(),
                    config_version: soundpack.config_version
                        .as_ref()
                        .and_then(|v| v.parse::<u32>().ok()),
                    is_valid_v2: true,
                    validation_status: "loaded_with_metadata_error".to_string(),
                    can_be_converted: false,
                    last_error: Some(format!("Metadata creation failed: {}", e)),
                };
                cache.add_soundpack(error_metadata);
            }
        }
    }
    cache.save();

    println!("✅ Successfully loaded mouse soundpack: {} (direct from files)", soundpack.name);
    Ok(())
}

fn update_keyboard_context(
    context: &AudioContext,
    samples: (Vec<f32>, u16, u32), // (samples, channels, sample_rate)
    key_mappings: std::collections::HashMap<String, Vec<(f64, f64)>>,
    soundpack: &SoundPack
) -> Result<(), String> {
    let (audio_samples, channels, sample_rate) = samples;
    let sample_count = audio_samples.len();
    let key_mapping_count = key_mappings.len();
    let soundpack_name = soundpack.name.clone();

    // Update keyboard samples
    if let Ok(mut cached) = context.keyboard_samples.lock() {
        *cached = Some((audio_samples, channels, sample_rate));
        println!("🎹 Updated keyboard samples: {} samples", sample_count);
    } else {
        return Err("Failed to acquire lock on keyboard_samples".to_string());
    }

    // Update key mappings
    if let Ok(mut key_map) = context.key_map.lock() {
        let old_count = key_map.len();
        key_map.clear();

        for (key, mappings) in key_mappings {
            let converted_mappings: Vec<[f32; 2]> = mappings
                .into_iter()
                .map(|(start, end)| [start as f32, end as f32])
                .collect();
            key_map.insert(key.clone(), converted_mappings);
        }

        println!("🗝️ Updated key mappings: {} -> {} keys", old_count, key_map.len());
    } else {
        return Err("Failed to acquire lock on key_map".to_string());
    }

    // Clear active keyboard audio state
    if let Ok(mut sinks) = context.key_sinks.lock() {
        let old_sinks = sinks.len();
        sinks.clear();
        if old_sinks > 0 {
            println!("🔇 Cleared {} active key sinks", old_sinks);
        }
    }

    if let Ok(mut pressed) = context.key_pressed.lock() {
        let old_pressed = pressed.len();
        pressed.clear();
        if old_pressed > 0 {
            println!("⌨️ Cleared {} pressed keys", old_pressed);
        }
    }

    println!(
        "✅ Successfully loaded keyboard soundpack: {} ({} key mappings) - Memory properly cleaned",
        soundpack_name,
        key_mapping_count
    );
    Ok(())
}

fn update_mouse_context(
    context: &AudioContext,
    samples: (Vec<f32>, u16, u32), // (samples, channels, sample_rate)
    mouse_mappings: std::collections::HashMap<String, Vec<(f64, f64)>>,
    soundpack: &SoundPack
) -> Result<(), String> {
    let (audio_samples, channels, sample_rate) = samples;
    let sample_count = audio_samples.len();
    let mouse_mapping_count = mouse_mappings.len();
    let soundpack_name = soundpack.name.clone();

    // Update mouse samples
    if let Ok(mut cached) = context.mouse_samples.lock() {
        *cached = Some((audio_samples, channels, sample_rate));
        println!("🖱️ Updated mouse samples: {} samples", sample_count);
    } else {
        return Err("Failed to acquire lock on mouse_samples".to_string());
    }

    // Update mouse mappings
    if let Ok(mut mouse_map) = context.mouse_map.lock() {
        let old_count = mouse_map.len();
        mouse_map.clear();

        for (button, mappings) in mouse_mappings {
            let converted_mappings: Vec<[f32; 2]> = mappings
                .into_iter()
                .map(|(start, end)| [start as f32, end as f32])
                .collect();
            mouse_map.insert(button.clone(), converted_mappings);
        }

        println!("🖱️ Updated mouse mappings: {} -> {} buttons", old_count, mouse_map.len());
    } else {
        return Err("Failed to acquire lock on mouse_map".to_string());
    }

    // Clear active mouse audio state
    if let Ok(mut mouse_sinks) = context.mouse_sinks.lock() {
        let old_sinks = mouse_sinks.len();
        mouse_sinks.clear();
        if old_sinks > 0 {
            println!("🔇 Cleared {} active mouse sinks", old_sinks);
        }
    }

    if let Ok(mut mouse_pressed) = context.mouse_pressed.lock() {
        let old_pressed = mouse_pressed.len();
        mouse_pressed.clear();
        if old_pressed > 0 {
            println!("🖱️ Cleared {} pressed mouse buttons", old_pressed);
        }
    }

    println!(
        "✅ Successfully loaded mouse soundpack: {} ({} mouse mappings) - Memory properly cleaned",
        soundpack_name,
        mouse_mapping_count
    );
    Ok(())
}

fn create_soundpack_metadata(
    soundpack_path: &str,
    soundpack: &SoundPack
) -> Result<SoundpackMetadata, String> {
    // Extract the soundpack ID from the full path
    // e.g., "/path/to/soundpacks/keyboard/Apex by teia" -> "keyboard/Apex by teia"
    let soundpacks_dir = crate::utils::path::get_soundpacks_dir_absolute();
    let id = if
        let Ok(relative_path) = std::path::Path::new(soundpack_path).strip_prefix(&soundpacks_dir)
    {
        relative_path.to_string_lossy().replace('\\', "/")
    } else {
        // Fallback to just the folder name if we can't get relative path
        std::path::Path
            ::new(soundpack_path)
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("unknown")
            .to_string()
    };

    // Get file metadata
    let last_modified = match std::fs::metadata(soundpack_path) {
        Ok(metadata) =>
            metadata
                .modified()
                .unwrap_or(std::time::SystemTime::UNIX_EPOCH)
                .duration_since(std::time::SystemTime::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        Err(_) => 0,
    };

    Ok(SoundpackMetadata {
        id: id.clone(), // Use calculated relative path ID instead of config ID
        name: soundpack.name.clone(),
        author: soundpack.author.clone(),
        description: soundpack.description.clone(),
        version: soundpack.version.clone().unwrap_or_else(|| "1.0".to_string()),
        tags: soundpack.tags.clone().unwrap_or_default(),
        icon: {
            // Generate dynamic URL for icon instead of base64 conversion
            if let Some(icon_filename) = &soundpack.icon {
                let icon_path = format!("{}/{}", soundpack_path, icon_filename);
                if std::path::Path::new(&icon_path).exists() {
                    // Create dynamic URL that will be served by the asset handler
                    Some(format!("/soundpack-images/{}/{}", id, icon_filename))
                } else {
                    Some(String::new()) // Empty string if icon file not found
                }
            } else {
                Some(String::new()) // Empty string if no icon specified
            }
        },
        soundpack_type: soundpack.soundpack_type.clone(), // Include the mouse field
        folder_path: id, // Use the derived folder path for loading
        last_modified,
        last_accessed: std::time::SystemTime
            ::now()
            .duration_since(std::time::SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs(), // Add validation fields with default values
        config_version: Some(soundpack.config_version_num),
        is_valid_v2: true, // Assume valid since it loaded successfully
        validation_status: "valid".to_string(),
        can_be_converted: false,
        // Error tracking - None since we successfully created metadata
        last_error: None,
    })
}

fn create_key_mappings(
    soundpack: &SoundPack,
    _samples: &[f32]
) -> std::collections::HashMap<String, Vec<(f64, f64)>> {
    let mut key_mappings = std::collections::HashMap::new(); // For keyboard soundpacks, use the definitions field for keyboard mappings
    // For mouse soundpacks, return empty key mappings
    if soundpack.soundpack_type == crate::state::soundpack::SoundpackType::Keyboard {
        for (key, key_def) in &soundpack.definitions {
            // Convert KeyDefinition timing to Vec<(f64, f64)>
            let converted_mappings: Vec<(f64, f64)> = key_def.timing
                .iter()
                .map(|pair| (pair[0] as f64, pair[1] as f64))
                .collect();
            key_mappings.insert(key.clone(), converted_mappings);
        }
    }

    key_mappings
}

fn create_mouse_mappings(
    soundpack: &SoundPack,
    _samples: &[f32]
) -> std::collections::HashMap<String, Vec<(f64, f64)>> {
    let mut mouse_mappings = std::collections::HashMap::new(); // For mouse soundpacks, use the definitions field directly
    if soundpack.soundpack_type == crate::state::soundpack::SoundpackType::Mouse {
        // This is a mouse soundpack, use definitions field for mouse mappings
        for (button, key_def) in &soundpack.definitions {
            // Convert KeyDefinition timing to Vec<(f64, f64)>
            let converted_mappings: Vec<(f64, f64)> = key_def.timing
                .iter()
                .map(|pair| (pair[0] as f64, pair[1] as f64))
                .collect();
            mouse_mappings.insert(button.clone(), converted_mappings);
        }
    } else {
        // This is a keyboard soundpack, create default mouse mappings from keyboard sounds
        println!(
            "🖱️ No mouse definitions found, creating default mouse mappings from keyboard sounds"
        );

        // Use common keyboard keys as fallback for mouse buttons
        let fallback_mappings = [
            ("MouseLeft", "Space"),
            ("MouseRight", "Enter"),
            ("MouseMiddle", "Tab"),
            ("MouseWheelUp", "ArrowUp"),
            ("MouseWheelDown", "ArrowDown"),
            ("Mouse4", "Backspace"),
            ("Mouse5", "Delete"),
            ("Mouse6", "Home"),
            ("Mouse7", "End"),
            ("Mouse8", "PageUp"),
        ];
        for (mouse_button, keyboard_key) in &fallback_mappings {
            if let Some(key_def) = soundpack.definitions.get(*keyboard_key) {
                let converted_mappings: Vec<(f64, f64)> = key_def.timing
                    .iter()
                    .map(|pair| (pair[0] as f64, pair[1] as f64))
                    .collect();
                mouse_mappings.insert(mouse_button.to_string(), converted_mappings);
            }
        }
    }

    mouse_mappings
}

/// Capture soundpack loading error and update the cache
fn capture_soundpack_loading_error(soundpack_id: &str, error: &str) {
    // Skip creating cache entries for empty soundpack IDs
    if soundpack_id.is_empty() {
        println!("⚠️ Skipping cache entry for empty soundpack ID: {}", error);
        return;
    }

    println!("📝 Capturing loading error for {}: {}", soundpack_id, error);

    let mut cache = SoundpackCache::load();

    // Check if we already have metadata for this soundpack
    if let Some(existing_metadata) = cache.soundpacks.get_mut(soundpack_id) {
        // Update existing metadata with error
        existing_metadata.last_error = Some(error.to_string());
        existing_metadata.validation_status = "loading_error".to_string();
    } else {
        // Create minimal metadata entry with error information
        let error_metadata = SoundpackMetadata {
            id: soundpack_id.to_string(),
            name: format!("Error: {}", soundpack_id),
            author: None,
            description: Some(format!("Loading failed: {}", error)),
            version: "unknown".to_string(),
            tags: vec!["error".to_string()],
            icon: None,
            soundpack_type: determine_soundpack_type(soundpack_id),
            folder_path: soundpack_id.to_string(), // Add folder_path for error entries
            last_modified: 0,
            last_accessed: std::time::SystemTime
                ::now()
                .duration_since(std::time::SystemTime::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            config_version: None,
            is_valid_v2: false,
            validation_status: "loading_error".to_string(),
            can_be_converted: false,
            last_error: Some(error.to_string()),
        };

        cache.soundpacks.insert(soundpack_id.to_string(), error_metadata);
    }

    cache.save();
    println!("💾 Updated cache with error information for {}", soundpack_id);
}
