use crate::state::config::AppConfig;
use crate::state::paths;
use crate::state::soundpack::SoundPack;
use crate::state::soundpack::{ SoundpackCache, SoundpackMetadata };

use super::audio_context::AudioContext;

pub fn load_soundpack(context: &AudioContext) -> Result<(), String> {
    let config = AppConfig::load();
    // Load both keyboard and mouse soundpacks
    load_keyboard_soundpack(context, &config.keyboard_soundpack)?;
    load_mouse_soundpack(context, &config.mouse_soundpack)?;
    Ok(())
}

pub fn load_keyboard_soundpack(context: &AudioContext, soundpack_id: &str) -> Result<(), String> {
    println!("🎹 Loading keyboard soundpack: {}", soundpack_id);
    match load_keyboard_soundpack_optimized(context, soundpack_id) {
        Ok(()) => Ok(()),
        Err(e) => {
            // Capture the error in cache
            capture_soundpack_loading_error(soundpack_id, &e);
            Err(e)
        }
    }
}

pub fn load_mouse_soundpack(context: &AudioContext, soundpack_id: &str) -> Result<(), String> {
    println!("🖱️ Loading mouse soundpack: {}", soundpack_id);
    match load_mouse_soundpack_optimized(context, soundpack_id) {
        Ok(()) => Ok(()),
        Err(e) => {
            // Capture the error in cache
            capture_soundpack_loading_error(soundpack_id, &e);
            Err(e)
        }
    }
}

fn load_audio_file(
    soundpack_path: &str,
    soundpack: &SoundPack
) -> Result<(Vec<f32>, u16, u32), String> {
    println!("🔍 [DEBUG] load_audio_file called");
    println!("🔍 [DEBUG] soundpack_path: {}", soundpack_path);
    println!("🔍 [DEBUG] soundpack.audio_file: {:?}", soundpack.audio_file);

    let sound_file_path = soundpack.audio_file
        .as_ref()
        .map(|src| format!("{}/{}", soundpack_path, src.trim_start_matches("./")))
        .ok_or_else(|| "No audio_file field in soundpack config".to_string())?;

    println!("🔍 [DEBUG] Full sound file path: {}", sound_file_path);

    if !std::path::Path::new(&sound_file_path).exists() {
        println!("❌ [DEBUG] Sound file not found: {}", sound_file_path);
        return Err(format!("Sound file not found: {}", sound_file_path));
    }

    println!("✅ [DEBUG] Sound file exists, loading with Symphonia...");

    // Use Symphonia for audio loading instead of Rodio
    match load_audio_with_symphonia(&sound_file_path) {
        Ok((samples, channels, sample_rate)) => {
            println!(
                "✅ [DEBUG] Audio loaded with Symphonia: {} samples, {} channels, {} Hz",
                samples.len(),
                channels,
                sample_rate
            );
            Ok((samples, channels, sample_rate))
        }
        Err(e) => {
            println!("❌ [DEBUG] Failed to load audio with Symphonia: {}", e);
            Err(format!("Failed to load audio: {}", e))
        }
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
        .map_err(|e| format!("Failed to probe format: {}", e))?;

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
                        for &sample in buf.chan(0) {
                            samples.push(sample);
                        }
                        // Handle additional channels if stereo
                        if channels > 1 && buf.spec().channels.count() > 1 {
                            for &sample in buf.chan(1) {
                                samples.push(sample);
                            }
                        }
                    }
                    AudioBufferRef::S32(buf) => {
                        for &sample in buf.chan(0) {
                            samples.push((sample as f32) / (i32::MAX as f32));
                        }
                        if channels > 1 && buf.spec().channels.count() > 1 {
                            for &sample in buf.chan(1) {
                                samples.push((sample as f32) / (i32::MAX as f32));
                            }
                        }
                    }
                    AudioBufferRef::S16(buf) => {
                        for &sample in buf.chan(0) {
                            samples.push((sample as f32) / (i16::MAX as f32));
                        }
                        if channels > 1 && buf.spec().channels.count() > 1 {
                            for &sample in buf.chan(1) {
                                samples.push((sample as f32) / (i16::MAX as f32));
                            }
                        }
                    }
                    AudioBufferRef::U32(buf) => {
                        for &sample in buf.chan(0) {
                            samples.push(
                                ((sample as f32) - (u32::MAX as f32) / 2.0) /
                                    ((u32::MAX as f32) / 2.0)
                            );
                        }
                        if channels > 1 && buf.spec().channels.count() > 1 {
                            for &sample in buf.chan(1) {
                                samples.push(
                                    ((sample as f32) - (u32::MAX as f32) / 2.0) /
                                        ((u32::MAX as f32) / 2.0)
                                );
                            }
                        }
                    }
                    AudioBufferRef::U16(buf) => {
                        for &sample in buf.chan(0) {
                            samples.push(
                                ((sample as f32) - (u16::MAX as f32) / 2.0) /
                                    ((u16::MAX as f32) / 2.0)
                            );
                        }
                        if channels > 1 && buf.spec().channels.count() > 1 {
                            for &sample in buf.chan(1) {
                                samples.push(
                                    ((sample as f32) - (u16::MAX as f32) / 2.0) /
                                        ((u16::MAX as f32) / 2.0)
                                );
                            }
                        }
                    }
                    AudioBufferRef::U8(buf) => {
                        for &sample in buf.chan(0) {
                            samples.push(((sample as f32) - 128.0) / 128.0);
                        }
                        if channels > 1 && buf.spec().channels.count() > 1 {
                            for &sample in buf.chan(1) {
                                samples.push(((sample as f32) - 128.0) / 128.0);
                            }
                        }
                    }
                    AudioBufferRef::S8(buf) => {
                        for &sample in buf.chan(0) {
                            samples.push((sample as f32) / (i8::MAX as f32));
                        }
                        if channels > 1 && buf.spec().channels.count() > 1 {
                            for &sample in buf.chan(1) {
                                samples.push((sample as f32) / (i8::MAX as f32));
                            }
                        }
                    }
                    AudioBufferRef::F64(buf) => {
                        for &sample in buf.chan(0) {
                            samples.push(sample as f32);
                        }
                        if channels > 1 && buf.spec().channels.count() > 1 {
                            for &sample in buf.chan(1) {
                                samples.push(sample as f32);
                            }
                        }
                    }
                    AudioBufferRef::U24(buf) => {
                        for &sample in buf.chan(0) {
                            let sample_f32 = ((sample.inner() as f32) - 8388608.0) / 8388608.0; // 2^23
                            samples.push(sample_f32);
                        }
                        if channels > 1 && buf.spec().channels.count() > 1 {
                            for &sample in buf.chan(1) {
                                let sample_f32 = ((sample.inner() as f32) - 8388608.0) / 8388608.0;
                                samples.push(sample_f32);
                            }
                        }
                    }
                    AudioBufferRef::S24(buf) => {
                        for &sample in buf.chan(0) {
                            let sample_f32 = (sample.inner() as f32) / 8388607.0; // 2^23 - 1
                            samples.push(sample_f32);
                        }
                        if channels > 1 && buf.spec().channels.count() > 1 {
                            for &sample in buf.chan(1) {
                                let sample_f32 = (sample.inner() as f32) / 8388607.0;
                                samples.push(sample_f32);
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
    soundpack_id: &str
) -> Result<(), String> {
    println!("📂 Direct loading keyboard soundpack: {}", soundpack_id);

    // Load soundpack directly from filesystem
    let soundpack_path = paths::soundpacks::soundpack_dir(soundpack_id);

    // Load config.json
    let config_path = paths::soundpacks::config_json(soundpack_id);
    let config_content = std::fs
        ::read_to_string(&config_path)
        .map_err(|e| format!("Failed to read config: {}", e))?;

    // First try to parse as V2, if fails try V1 format
    let soundpack: SoundPack = match serde_json::from_str(&config_content) {
        Ok(sp) => sp,
        Err(_) => {
            // Try to parse as V1 and convert on-the-fly for loading only
            println!("⚠️ Config appears to be V1 format, converting for loading (not saving)");
            convert_v1_for_loading(&config_content, soundpack_id)?
        }
    }; // Verify this is a keyboard soundpack
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
            // Create minimal metadata with error information
            let error_metadata = SoundpackMetadata {
                id: soundpack_id.to_string(),
                name: soundpack.name.clone(),
                author: soundpack.author.clone(),
                description: soundpack.description.clone(),
                version: soundpack.version.clone().unwrap_or_else(|| "1.0".to_string()),
                tags: soundpack.tags.clone().unwrap_or_default(),
                icon: soundpack.icon.clone(),
                soundpack_type: crate::state::soundpack::SoundpackType::Keyboard,
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
    cache.save();

    println!("✅ Successfully loaded keyboard soundpack: {} (direct from files)", soundpack.name);
    Ok(())
}

/// Direct mouse soundpack loading
pub fn load_mouse_soundpack_optimized(
    context: &AudioContext,
    soundpack_id: &str
) -> Result<(), String> {
    println!("📂 Direct loading mouse soundpack: {}", soundpack_id);

    // Load soundpack directly from filesystem
    let soundpack_path = paths::soundpacks::soundpack_dir(soundpack_id);

    // Load config.json
    let config_path = paths::soundpacks::config_json(soundpack_id);
    let config_content = std::fs
        ::read_to_string(&config_path)
        .map_err(|e| format!("Failed to read config: {}", e))?;

    let soundpack: SoundPack = serde_json
        ::from_str(&config_content)
        .map_err(|e| format!("Failed to parse config: {}", e))?; // Verify this is a mouse soundpack
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
            // Create minimal metadata with error information
            let error_metadata = SoundpackMetadata {
                id: soundpack_id.to_string(),
                name: soundpack.name.clone(),
                author: soundpack.author.clone(),
                description: soundpack.description.clone(),
                version: soundpack.version.clone().unwrap_or_else(|| "1.0".to_string()),
                tags: soundpack.tags.clone().unwrap_or_default(),
                icon: soundpack.icon.clone(),
                soundpack_type: crate::state::soundpack::SoundpackType::Mouse,
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
    let path = std::path::Path::new(soundpack_path);
    let id = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("unknown")
        .to_string();

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
        id,
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
                    Some(
                        format!(
                            "/soundpack-images/{}/{}",
                            std::path::Path
                                ::new(soundpack_path)
                                .file_name()
                                .and_then(|name| name.to_str())
                                .unwrap_or("unknown"),
                            icon_filename
                        )
                    )
                } else {
                    Some(String::new()) // Empty string if icon file not found
                }
            } else {
                Some(String::new()) // Empty string if no icon specified
            }
        },
        soundpack_type: soundpack.soundpack_type.clone(), // Include the mouse field
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
            soundpack_type: crate::state::soundpack::SoundpackType::Keyboard,
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

/// Convert V1 config to SoundPack struct for loading only (doesn't save to file)
fn convert_v1_for_loading(config_content: &str, soundpack_id: &str) -> Result<SoundPack, String> {
    println!("🔍 [DEBUG] convert_v1_for_loading called for {}", soundpack_id);

    let v1_config: serde_json::Value = serde_json
        ::from_str(config_content)
        .map_err(|e| format!("Failed to parse V1 config: {}", e))?;

    // Extract basic fields from V1
    let name = v1_config
        .get("name")
        .and_then(|v| v.as_str())
        .unwrap_or(soundpack_id)
        .to_string();

    let source = v1_config
        .get("sound")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    println!("🔍 [DEBUG] V1 source field: {:?}", source); // Convert V1 defines to V2 definitions format
    let mut defs: std::collections::HashMap<
        String,
        crate::state::soundpack::KeyDefinition
    > = std::collections::HashMap::new();

    if let Some(defines) = v1_config.get("defines").and_then(|d| d.as_object()) {
        println!("🔍 [DEBUG] Found {} defines in V1 config", defines.len());

        // Simple IOHook to Web key mapping for common keys
        let key_mappings = create_simple_key_mapping();

        for (iohook_code, timing) in defines {
            if let Ok(iohook_num) = iohook_code.parse::<u32>() {
                if let Some(key_name) = key_mappings.get(&iohook_num) {
                    if let Some(timing_array) = timing.as_array() {
                        if timing_array.len() >= 2 {
                            let start = timing_array[0].as_f64().unwrap_or(0.0) as f32;
                            let duration = timing_array[1].as_f64().unwrap_or(100.0) as f32;
                            println!(
                                "🔍 [DEBUG] Key {} (IOHook {}): [{}, {}]",
                                key_name,
                                iohook_code,
                                start,
                                duration
                            );
                            defs.insert(key_name.clone(), crate::state::soundpack::KeyDefinition {
                                timing: vec![[start, duration]],
                                audio_file: None, // V1 configs use single audio file
                            });
                        }
                    }
                }
            }
        }
    }
    println!("🔍 [DEBUG] Converted {} key mappings to V2 format", defs.len());

    // Create SoundPack struct with V2 format
    Ok(SoundPack {
        id: v1_config
            .get("id")
            .and_then(|v| v.as_str())
            .unwrap_or(soundpack_id)
            .to_string(),
        name,
        author: Some("N/A".to_string()), // Default for V1 configs
        description: v1_config
            .get("description")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string()),
        version: v1_config
            .get("version")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string()),
        config_version: Some("1".to_string()), // Mark as V1 so we know this needs proper conversion later
        icon: v1_config
            .get("icon")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string()),
        audio_file: source, // V1 single audio file becomes audio_file
        license: None,
        tags: v1_config
            .get("tags")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect()
            }),
        created_at: None,
        definition_method: v1_config
            .get("key_define_type")
            .and_then(|v| v.as_str())
            .unwrap_or("single")
            .to_string(),
        options: crate::state::soundpack::SoundpackOptions {
            recommended_volume: 1.0,
            random_pitch: false,
        },
        soundpack_type: crate::state::soundpack::SoundpackType::Keyboard, // V1 configs are keyboard soundpacks
        config_version_num: 1, // Internal version number
        definitions: defs,
    })
}

/// Create simple IOHook to Web key mapping for basic conversion
fn create_simple_key_mapping() -> std::collections::HashMap<u32, String> {
    let mut mapping = std::collections::HashMap::new();

    // Common keys only - enough for basic loading
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
    mapping.insert(28, "Enter".to_string());
    mapping.insert(14, "Backspace".to_string());
    mapping.insert(15, "Tab".to_string());

    mapping
}
