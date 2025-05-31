use rodio::{Decoder, Source};
use std::fs::File;
use std::io::{BufReader, Read};

use crate::state::config::AppConfig;
use crate::state::soundpack::SoundPack;
use crate::state::soundpack_cache::{SoundpackCache, SoundpackMetadata};
use crate::state::paths;

use super::audio_context::AudioContext;

pub fn load_soundpack(context: &AudioContext) -> Result<(), String> {
    let config = AppConfig::load();
    let current_id = &config.current_soundpack;
    load_soundpack_by_id(context, current_id)
}

pub fn load_soundpack_by_id(context: &AudioContext, soundpack_id: &str) -> Result<(), String> {
    // Use optimized cache for loading
    load_soundpack_optimized(context, soundpack_id)
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

/// Direct soundpack loading (no audio caching)
pub fn load_soundpack_optimized(context: &AudioContext, soundpack_id: &str) -> Result<(), String> {
    println!("📂 Direct loading soundpack: {}", soundpack_id);

    // Load soundpack directly from filesystem
    let soundpack_path = paths::soundpacks::soundpack_dir(soundpack_id);

    // Load config.json
    let config_path = paths::soundpacks::config_json(soundpack_id);
    let config_content = std::fs::read_to_string(&config_path)
        .map_err(|e| format!("Failed to read config: {}", e))?;

    let soundpack: SoundPack = serde_json::from_str(&config_content)
        .map_err(|e| format!("Failed to parse config: {}", e))?;

    // Load audio samples directly from file
    let samples = load_audio_file(&soundpack_path, &soundpack)?;

    // Create key mappings
    let key_mappings = create_key_mappings(&soundpack, &samples.0);

    // Create mouse mappings
    let mouse_mappings = create_mouse_mappings(&soundpack, &samples.0);

    // Update audio context with new data (this replaces old data automatically)
    update_context_direct(context, samples, key_mappings, mouse_mappings, &soundpack)?;

    // Update metadata cache only (no audio caching)
    let mut cache = SoundpackCache::load();
    let metadata = create_soundpack_metadata(&soundpack_path, &soundpack)?;
    cache.add_soundpack(metadata);
    cache.save();

    println!(
        "✅ Successfully loaded soundpack: {} (direct from files)",
        soundpack.name
    );
    Ok(())
}

fn update_context_direct(
    context: &AudioContext,
    samples: (Vec<f32>, u16, u32), // (samples, channels, sample_rate)
    key_mappings: std::collections::HashMap<String, Vec<(f64, f64)>>,
    mouse_mappings: std::collections::HashMap<String, Vec<(f64, f64)>>,
    soundpack: &SoundPack,
) -> Result<(), String> {
    let (audio_samples, channels, sample_rate) = samples;
    let sample_count = audio_samples.len();
    let key_mapping_count = key_mappings.len();
    let mouse_mapping_count = mouse_mappings.len();
    let soundpack_name = soundpack.name.clone();

    // Update cached samples (replaces old data automatically for memory cleanup)
    if let Ok(mut cached) = context.cached_samples.lock() {
        // This assignment automatically drops the old Vec<f32> data
        *cached = Some((audio_samples, channels, sample_rate));
        println!("🎵 Updated cached samples: {} samples", sample_count);
    } else {
        return Err("Failed to acquire lock on cached_samples".to_string());
    }

    // Update key mappings (convert from HashMap<String, Vec<(f64, f64)>> to HashMap<String, Vec<[f32; 2]>>)
    if let Ok(mut key_map) = context.key_map.lock() {
        let old_count = key_map.len();
        // Clear old mappings to free memory
        key_map.clear();

        // Convert key mappings to the expected format
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

        // Log first few key mappings for verification
        for (key, mapping) in key_map.iter().take(3) {
            println!("   {} -> {:?}", key, mapping);
        }
    } else {
        return Err("Failed to acquire lock on key_map".to_string());
    }

    // Update mouse mappings (convert from HashMap<String, Vec<(f64, f64)>> to HashMap<String, Vec<[f32; 2]>>)
    if let Ok(mut mouse_map) = context.mouse_map.lock() {
        let old_count = mouse_map.len();
        // Clear old mappings to free memory
        mouse_map.clear();

        // Convert mouse mappings to the expected format
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

        // Log first few mouse mappings for verification
        for (button, mapping) in mouse_map.iter().take(3) {
            println!("   {} -> {:?}", button, mapping);
        }
    } else {
        return Err("Failed to acquire lock on mouse_map".to_string());
    }

    // Clear active audio state to prevent memory leaks
    if let Ok(mut sinks) = context.key_sinks.lock() {
        let old_sinks = sinks.len();
        // Drop all existing sinks to free audio resources
        sinks.clear();
        if old_sinks > 0 {
            println!("🔇 Cleared {} active key sinks", old_sinks);
        }
    }

    if let Ok(mut mouse_sinks) = context.mouse_sinks.lock() {
        let old_sinks = mouse_sinks.len();
        // Drop all existing mouse sinks to free audio resources
        mouse_sinks.clear();
        if old_sinks > 0 {
            println!("🔇 Cleared {} active mouse sinks", old_sinks);
        }
    }

    if let Ok(mut pressed) = context.key_pressed.lock() {
        let old_pressed = pressed.len();
        pressed.clear();
        if old_pressed > 0 {
            println!("⌨️ Cleared {} pressed keys", old_pressed);
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
        "✅ Successfully loaded soundpack: {} ({} key mappings, {} mouse mappings) - Memory properly cleaned",
        soundpack_name, key_mapping_count, mouse_mapping_count
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
        icon: soundpack.icon.clone(),
        last_modified,
        last_accessed: std::time::SystemTime::now()
            .duration_since(std::time::SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs(),
    })
}

fn create_key_mappings(
    soundpack: &SoundPack,
    _samples: &[f32],
) -> std::collections::HashMap<String, Vec<(f64, f64)>> {
    let mut key_mappings = std::collections::HashMap::new();

    for (key, sound_def) in &soundpack.def {
        // Convert Vec<[f32; 2]> to Vec<(f64, f64)>
        let converted_mappings: Vec<(f64, f64)> = sound_def
            .iter()
            .map(|pair| (pair[0] as f64, pair[1] as f64))
            .collect();
        key_mappings.insert(key.clone(), converted_mappings);
    }

    key_mappings
}

fn create_mouse_mappings(
    soundpack: &SoundPack,
    _samples: &[f32],
) -> std::collections::HashMap<String, Vec<(f64, f64)>> {
    let mut mouse_mappings = std::collections::HashMap::new();

    // Check if soundpack has mouse definitions
    if let Some(mouse_def) = &soundpack.mouse_def {
        for (button, sound_def) in mouse_def {
            // Convert Vec<[f32; 2]> to Vec<(f64, f64)>
            let converted_mappings: Vec<(f64, f64)> = sound_def
                .iter()
                .map(|pair| (pair[0] as f64, pair[1] as f64))
                .collect();
            mouse_mappings.insert(button.clone(), converted_mappings);
        }
    } else {
        // If no mouse definitions, create default mappings using keyboard sounds
        // This allows mouse clicks to work even with keyboard-only soundpacks
        println!("🖱️ No mouse definitions found, creating default mouse mappings from keyboard sounds");
        
        // Use common keyboard keys as fallback for mouse buttons
        let fallback_mappings = [
            ("MouseLeft", "Space"),
            ("MouseRight", "Enter"),
            ("MouseMiddle", "Tab"),
            ("MouseWheelUp", "ArrowUp"),
            ("MouseWheelDown", "ArrowDown"),
            ("Mouse4", "Backspace"),
            ("Mouse5", "Delete"),
        ];

        for (mouse_button, keyboard_key) in &fallback_mappings {
            if let Some(key_mapping) = soundpack.def.get(*keyboard_key) {
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
