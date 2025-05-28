use rodio::{Decoder, Source};
use std::fs::File;
use std::io::{BufReader, Read};

use crate::state::config::AppConfig;
use crate::state::soundpack::SoundPack;
use crate::state::soundpack_cache::SoundpackCache;

use super::audio_context::AudioContext;

pub fn load_soundpack(context: &AudioContext) -> Result<(), String> {
    let config = AppConfig::load();
    let current_id = &config.current_soundpack;
    load_soundpack_by_id(context, current_id)
}

pub fn load_soundpack_by_id(context: &AudioContext, soundpack_id: &str) -> Result<(), String> {
    // Try to load from cache first
    let cache = SoundpackCache::load();
    if let Some(pack_item) = cache.get_soundpack_by_id(soundpack_id) {
        println!(
            "🚀 Loading soundpack '{}' from cache",
            pack_item.soundpack.name
        );

        // Sử dụng đường dẫn tương đối từ cache
        let soundpack_path = &pack_item.relative_path;
        let cached_samples = load_audio_file(soundpack_path, &pack_item.soundpack)?;

        // Update context with cached data
        update_context(context, cached_samples, pack_item.soundpack.clone())?;

        // Update config
        update_config(&pack_item.soundpack)?;

        return Ok(());
    }

    // Fallback to file-based loading if not in cache
    println!(
        "⚠️ Soundpack '{}' not found in cache, falling back to file loading",
        soundpack_id
    );
    load_soundpack_from_files_by_id(context, soundpack_id)
}

fn load_soundpack_from_files(context: &AudioContext) -> Result<(), String> {
    let config = AppConfig::load();
    load_soundpack_from_files_by_id(context, &config.current_soundpack)
}

fn load_soundpack_from_files_by_id(
    context: &AudioContext,
    soundpack_id: &str,
) -> Result<(), String> {
    // Load soundpacks from directory
    let soundpacks = find_soundpacks()?;
    let available_packs = load_available_packs(&soundpacks)?;

    // Get specified soundpack or fallback to default
    let (soundpack_path, soundpack) = get_soundpack_by_id(&available_packs, soundpack_id)?;

    // Load and decode audio file
    let cached_samples = load_audio_file(&soundpack_path, &soundpack)?;

    // Update configuration
    update_config(&soundpack)?;

    // Update context
    update_context(context, cached_samples, soundpack)?;

    Ok(())
}

fn find_soundpacks() -> Result<Vec<String>, String> {
    let soundpacks = std::fs::read_dir("./soundpacks")
        .map_err(|_| "Failed to read soundpacks directory".to_string())?
        .filter_map(|entry| {
            entry.ok().and_then(|e| {
                let path = e.path();
                if path.join("config.json").exists() {
                    Some(path.to_string_lossy().into_owned())
                } else {
                    None
                }
            })
        })
        .collect::<Vec<_>>();
    if soundpacks.is_empty() {
        Err("No soundpacks found in ./soundpacks directory".to_string())
    } else {
        Ok(soundpacks)
    }
}

fn load_available_packs(soundpacks: &[String]) -> Result<Vec<(String, SoundPack)>, String> {
    let packs: Vec<_> = soundpacks
        .iter()
        .filter_map(|path| {
            std::fs::read_to_string(format!("{}/config.json", path))
                .ok()
                .and_then(|content| {
                    serde_json::from_str::<SoundPack>(&content)
                        .ok()
                        .map(|pack| (path.clone(), pack))
                })
        })
        .collect();

    if packs.is_empty() {
        Err("No valid soundpacks found".to_string())
    } else {
        Ok(packs)
    }
}

fn get_soundpack_by_id(
    available_packs: &[(String, SoundPack)],
    soundpack_id: &str,
) -> Result<(String, SoundPack), String> {
    match available_packs
        .iter()
        .find(|(_, pack)| pack.id == soundpack_id)
    {
        Some((path, pack)) => Ok((path.clone(), pack.clone())),
        None => {
            let (path, pack) = available_packs[0].clone();
            println!(
                "⚠️ Soundpack '{}' not found, using '{}' instead",
                soundpack_id, pack.name
            );
            Ok((path, pack))
        }
    }
}

fn get_current_soundpack(
    available_packs: &[(String, SoundPack)],
) -> Result<(String, SoundPack), String> {
    let config = AppConfig::load();
    let current_id = &config.current_soundpack;
    get_soundpack_by_id(available_packs, current_id)
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

fn update_config(soundpack: &SoundPack) -> Result<(), String> {
    let mut config = AppConfig::load();
    config.current_soundpack = soundpack.id.clone();
    config
        .save()
        .map_err(|e| format!("Failed to save config: {}", e))
}

fn update_context(
    context: &AudioContext,
    (samples, channels, sample_rate): (Vec<f32>, u16, u32),
    soundpack: SoundPack,
) -> Result<(), String> {
    println!(
        "🔄 Updating audio context for soundpack: {}",
        soundpack.name
    );

    // Update samples
    if let Ok(mut cached) = context.cached_samples.lock() {
        *cached = Some((samples.clone(), channels, sample_rate));
        println!(
            "✅ Updated cached samples: {} samples, {} channels, {} Hz",
            samples.len(),
            channels,
            sample_rate
        );
    } else {
        return Err("Failed to acquire lock on cached_samples".to_string());
    }

    // Update key mappings
    if let Ok(mut key_map) = context.key_map.lock() {
        let old_count = key_map.len();
        key_map.clear();
        key_map.extend(soundpack.def.clone());
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

    // Clear state
    if let Ok(mut sinks) = context.key_sinks.lock() {
        let old_sinks = sinks.len();
        sinks.clear();
        println!("🔇 Cleared {} active sinks", old_sinks);
    }
    if let Ok(mut pressed) = context.key_pressed.lock() {
        let old_pressed = pressed.len();
        pressed.clear();
        println!("⌨️ Cleared {} pressed keys", old_pressed);
    }

    println!(
        "✅ Successfully loaded soundpack: {} by {} ({} key mappings)",
        soundpack.name,
        soundpack.author,
        soundpack.def.len()
    );
    Ok(())
}
