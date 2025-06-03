use rodio::{Decoder, Source};
use std::fs::File;
use std::io::{BufReader, Read};

use crate::state::config::AppConfig;
use crate::state::paths;
use crate::state::soundpack::SoundPack;
use crate::state::soundpack_cache::{SoundpackCache, SoundpackMetadata};

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
    load_keyboard_soundpack_optimized(context, soundpack_id)
}

pub fn load_mouse_soundpack(context: &AudioContext, soundpack_id: &str) -> Result<(), String> {
    println!("🖱️ Loading mouse soundpack: {}", soundpack_id);
    load_mouse_soundpack_optimized(context, soundpack_id)
}

fn load_audio_file(
    soundpack_path: &str,
    soundpack: &SoundPack,
) -> Result<(Vec<f32>, u16, u32), String> {
    let sound_file_path = soundpack
        .source
        .as_ref()
        .map(|src| format!("{}/{}", soundpack_path, src.trim_start_matches("./")))
        .ok_or_else(|| "No source field in soundpack config".to_string())?;

    if !std::path::Path::new(&sound_file_path).exists() {
        return Err(format!("Sound file not found: {}", sound_file_path));
    }

    let file =
        File::open(&sound_file_path).map_err(|e| format!("Failed to open sound file: {}", e))?;

    let mut buf = Vec::new();
    file.take(10_000_000)
        .read_to_end(&mut buf)
        .map_err(|e| format!("Failed to read sound file: {}", e))?;

    let cursor = std::io::Cursor::new(buf);
    let decoder = Decoder::new(BufReader::new(cursor))
        .map_err(|e| format!("Failed to decode audio: {}", e))?;

    let sample_rate = decoder.sample_rate();
    let channels = decoder.channels();
    let samples: Vec<f32> = decoder.convert_samples().collect();

    Ok((samples, channels, sample_rate))
}

/// Direct keyboard soundpack loading
pub fn load_keyboard_soundpack_optimized(
    context: &AudioContext,
    soundpack_id: &str,
) -> Result<(), String> {
    println!("📂 Direct loading keyboard soundpack: {}", soundpack_id);

    // Load soundpack directly from filesystem
    let soundpack_path = paths::soundpacks::soundpack_dir(soundpack_id);

    // Load config.json
    let config_path = paths::soundpacks::config_json(soundpack_id);
    let config_content = std::fs::read_to_string(&config_path)
        .map_err(|e| format!("Failed to read config: {}", e))?;

    let soundpack: SoundPack = serde_json::from_str(&config_content)
        .map_err(|e| format!("Failed to parse config: {}", e))?;

    // Verify this is a keyboard soundpack
    if soundpack.mouse {
        return Err("This is a mouse soundpack, not a keyboard soundpack".to_string());
    }

    // Load audio samples directly from file
    let samples = load_audio_file(&soundpack_path, &soundpack)?;

    // Create key mappings (only for keyboard soundpacks)
    let key_mappings = create_key_mappings(&soundpack, &samples.0);

    // Update audio context with keyboard data
    update_keyboard_context(context, samples, key_mappings, &soundpack)?;

    // Update metadata cache only (no audio caching)
    let mut cache = SoundpackCache::load();
    let metadata = create_soundpack_metadata(&soundpack_path, &soundpack)?;
    cache.add_soundpack(metadata);
    cache.save();

    println!(
        "✅ Successfully loaded keyboard soundpack: {} (direct from files)",
        soundpack.name
    );
    Ok(())
}

/// Direct mouse soundpack loading
pub fn load_mouse_soundpack_optimized(
    context: &AudioContext,
    soundpack_id: &str,
) -> Result<(), String> {
    println!("📂 Direct loading mouse soundpack: {}", soundpack_id);

    // Load soundpack directly from filesystem
    let soundpack_path = paths::soundpacks::soundpack_dir(soundpack_id);

    // Load config.json
    let config_path = paths::soundpacks::config_json(soundpack_id);
    let config_content = std::fs::read_to_string(&config_path)
        .map_err(|e| format!("Failed to read config: {}", e))?;

    let soundpack: SoundPack = serde_json::from_str(&config_content)
        .map_err(|e| format!("Failed to parse config: {}", e))?;

    // Verify this is a mouse soundpack
    if !soundpack.mouse {
        return Err("This is a keyboard soundpack, not a mouse soundpack".to_string());
    }

    // Load audio samples directly from file
    let samples = load_audio_file(&soundpack_path, &soundpack)?;

    // Create mouse mappings (only for mouse soundpacks)
    let mouse_mappings = create_mouse_mappings(&soundpack, &samples.0);

    // Update audio context with mouse data
    update_mouse_context(context, samples, mouse_mappings, &soundpack)?;

    // Update metadata cache only (no audio caching)
    let mut cache = SoundpackCache::load();
    let metadata = create_soundpack_metadata(&soundpack_path, &soundpack)?;
    cache.add_soundpack(metadata);
    cache.save();

    println!(
        "✅ Successfully loaded mouse soundpack: {} (direct from files)",
        soundpack.name
    );
    Ok(())
}

fn update_keyboard_context(
    context: &AudioContext,
    samples: (Vec<f32>, u16, u32), // (samples, channels, sample_rate)
    key_mappings: std::collections::HashMap<String, Vec<(f64, f64)>>,
    soundpack: &SoundPack,
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

        println!(
            "🗝️ Updated key mappings: {} -> {} keys",
            old_count,
            key_map.len()
        );
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
        soundpack_name, key_mapping_count
    );
    Ok(())
}

fn update_mouse_context(
    context: &AudioContext,
    samples: (Vec<f32>, u16, u32), // (samples, channels, sample_rate)
    mouse_mappings: std::collections::HashMap<String, Vec<(f64, f64)>>,
    soundpack: &SoundPack,
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

        println!(
            "🖱️ Updated mouse mappings: {} -> {} buttons",
            old_count,
            mouse_map.len()
        );
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
        soundpack_name, mouse_mapping_count
    );
    Ok(())
}

fn create_soundpack_metadata(
    soundpack_path: &str,
    soundpack: &SoundPack,
) -> Result<SoundpackMetadata, String> {
    let path = std::path::Path::new(soundpack_path);
    let id = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("unknown")
        .to_string();

    // Get file metadata
    let last_modified = match std::fs::metadata(soundpack_path) {
        Ok(metadata) => metadata
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
        author: Some(soundpack.author.clone()),
        description: soundpack.description.clone(),
        version: soundpack
            .version
            .clone()
            .unwrap_or_else(|| "1.0".to_string()),
        tags: soundpack.tags.clone().unwrap_or_default(),
        keycap: soundpack.keycap.clone(),
        icon: {
            // Convert icon to base64 data URI, similar to soundpack_cache.rs
            if let Some(icon_filename) = &soundpack.icon {
                let icon_path = format!("{}/{}", soundpack_path, icon_filename);
                if std::path::Path::new(&icon_path).exists() {
                    // Convert to base64 data URI for Dioxus WebView
                    match convert_image_to_data_uri(&icon_path) {
                        Ok(data_uri) => Some(data_uri),
                        Err(_) => Some(String::new()),
                    }
                } else {
                    Some(String::new()) // Empty string if icon file not found
                }
            } else {
                Some(String::new()) // Empty string if no icon specified
            }
        },
        mouse: soundpack.mouse, // Include the mouse field
        last_modified,
        last_accessed: std::time::SystemTime::now()
            .duration_since(std::time::SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs(),
        // Add validation fields with default values
        config_version: Some(soundpack.config_version),
        is_valid_v2: true, // Assume valid since it loaded successfully
        validation_status: "valid".to_string(),
        can_be_converted: false,
    })
}

// Helper function to convert image files to base64 data URIs for WebView compatibility
fn convert_image_to_data_uri(image_path: &str) -> Result<String, String> {
    // Read the image file
    let image_data =
        std::fs::read(image_path).map_err(|e| format!("Failed to read image file: {}", e))?;

    // Determine MIME type based on file extension
    let mime_type = match std::path::Path::new(image_path)
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext.to_lowercase())
        .as_deref()
    {
        Some("jpg") | Some("jpeg") => "image/jpeg",
        Some("png") => "image/png",
        Some("gif") => "image/gif",
        Some("webp") => "image/webp",
        Some("avif") => "image/avif",
        Some("svg") => "image/svg+xml",
        Some("bmp") => "image/bmp",
        Some("ico") => "image/x-icon",
        _ => "image/png", // Default fallback
    };

    // Convert to base64
    let base64_data =
        base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &image_data);

    // Create data URI
    Ok(format!("data:{};base64,{}", mime_type, base64_data))
}

fn create_key_mappings(
    soundpack: &SoundPack,
    _samples: &[f32],
) -> std::collections::HashMap<String, Vec<(f64, f64)>> {
    let mut key_mappings = std::collections::HashMap::new();

    // For keyboard soundpacks, use the defs field for keyboard mappings
    // For mouse soundpacks, return empty key mappings
    if !soundpack.mouse {
        for (key, sound_def) in &soundpack.defs {
            // Convert Vec<[f32; 2]> to Vec<(f64, f64)>
            let converted_mappings: Vec<(f64, f64)> = sound_def
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
    _samples: &[f32],
) -> std::collections::HashMap<String, Vec<(f64, f64)>> {
    let mut mouse_mappings = std::collections::HashMap::new();

    // For mouse soundpacks, use the defs field directly
    if soundpack.mouse {
        // This is a mouse soundpack, use defs field for mouse mappings
        for (button, sound_def) in &soundpack.defs {
            // Convert Vec<[f32; 2]> to Vec<(f64, f64)>
            let converted_mappings: Vec<(f64, f64)> = sound_def
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
            if let Some(key_mapping) = soundpack.defs.get(*keyboard_key) {
                let converted_mappings: Vec<(f64, f64)> = key_mapping
                    .iter()
                    .map(|pair| (pair[0] as f64, pair[1] as f64))
                    .collect();
                mouse_mappings.insert(mouse_button.to_string(), converted_mappings);
            }
        }
    }

    mouse_mappings
}
