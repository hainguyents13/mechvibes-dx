use rodio::{Decoder, Source};
use std::fs::File;
use std::io::{BufReader, Read};

use crate::state::config::AppConfig;
use crate::state::soundpack::SoundPack;
use crate::state::soundpack_cache::SoundpackCache;

use super::audio_context::AudioContext;

#[allow(dead_code)]
pub fn load_soundpack(context: &AudioContext) -> Result<(), String> {
    let config = AppConfig::load();
    let current_id = &config.current_soundpack; // Try to load from cache first
    let cache = SoundpackCache::load();
    if let Some(pack_item) = cache.get_soundpack_by_id(current_id) {
        println!(
            "🚀 Loading soundpack '{}' from cache",
            pack_item.soundpack.name
        );

        // Sử dụng đường dẫn tương đối từ cache
        let soundpack_path = &pack_item.relative_path;
        let cached_samples = load_audio_file(soundpack_path, &pack_item.soundpack)?;

        // Update context with cached data
        update_context(context, cached_samples, pack_item.soundpack.clone())?;

        return Ok(());
    }

    // Fallback to file-based loading if not in cache
    println!(
        "⚠️ Soundpack '{}' not found in cache, falling back to file loading",
        current_id
    );
    load_soundpack_from_files(context)
}

fn load_soundpack_from_files(context: &AudioContext) -> Result<(), String> {
    // Load soundpacks from directory
    let soundpacks = find_soundpacks()?;
    let available_packs = load_available_packs(&soundpacks)?;

    // Get current or default soundpack
    let (soundpack_path, soundpack) = get_current_soundpack(&available_packs)?;

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

fn get_current_soundpack(
    available_packs: &[(String, SoundPack)],
) -> Result<(String, SoundPack), String> {
    let config = AppConfig::load();
    let current_id = &config.current_soundpack;

    match available_packs
        .iter()
        .find(|(_, pack)| pack.id == *current_id)
    {
        Some((path, pack)) => Ok((path.clone(), pack.clone())),
        None => {
            let (path, pack) = available_packs[0].clone();
            println!(
                "⚠️ Soundpack '{}' not found, using '{}' instead",
                current_id, pack.name
            );
            Ok((path, pack))
        }
    }
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
    // Update samples
    if let Ok(mut cached) = context.cached_samples.lock() {
        *cached = Some((samples, channels, sample_rate));
    }

    // Log key mappings
    println!("🔍 Loaded soundpack: {}", soundpack.name,);

    // Update key mappings
    if let Ok(mut key_map) = context.key_map.lock() {
        key_map.clear();
        key_map.extend(soundpack.def);
    } else {
        return Err("Failed to acquire lock on key_map".to_string());
    }

    // Clear state
    if let Ok(mut sinks) = context.key_sinks.lock() {
        sinks.clear();
    }
    if let Ok(mut pressed) = context.key_pressed.lock() {
        pressed.clear();
    }

    println!(
        "✅ Loaded soundpack: {} by {}",
        soundpack.name, soundpack.author
    );
    Ok(())
}
