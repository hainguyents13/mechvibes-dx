use rodio::{Decoder, Source};
use std::fs::File;
use std::io::{BufReader, Read};

use crate::libs::audio::compressed_audio::CompressedAudioData;
use crate::state::config::AppConfig;
use crate::state::optimized_soundpack_cache::{OptimizedSoundpackCache, SoundpackMetadata};
use crate::state::soundpack::SoundPack;

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

/// Optimized soundpack loading with individual caching
pub fn load_soundpack_optimized(context: &AudioContext, soundpack_id: &str) -> Result<(), String> {
    let cache = OptimizedSoundpackCache::load();

    // Check if audio cache exists
    if cache.has_audio_cache(soundpack_id) {
        println!("🚀 Loading {} from optimized audio cache", soundpack_id);

        if let Some(cached_data) = cache.load_audio_cache(soundpack_id) {
            // Deserialize compressed audio data
            match CompressedAudioData::decompress(&cached_data) {
                Ok(audio_data) => {
                    // Capture data before moving it
                    let sample_count = audio_data.samples.len();

                    // Update audio context with cached data
                    update_context_optimized(context, audio_data)?;
                    println!(
                        "✅ Loaded {} from cache ({} samples)",
                        soundpack_id, sample_count
                    );
                    return Ok(());
                }
                Err(e) => {
                    eprintln!("⚠️  Failed to deserialize audio cache: {}", e);
                    // Fall through to fresh load
                }
            }
        }
    }

    // Fresh load if cache miss or invalid
    println!("📂 Fresh loading soundpack: {}", soundpack_id);
    load_and_cache_soundpack_optimized(context, soundpack_id, cache)
}

fn load_and_cache_soundpack_optimized(
    context: &AudioContext,
    soundpack_id: &str,
    mut cache: OptimizedSoundpackCache,
) -> Result<(), String> {
    // Load soundpack directly from filesystem
    let soundpack_path = format!("./soundpacks/{}", soundpack_id);

    // Load config.json
    let config_path = format!("{}/config.json", soundpack_path);
    let config_content = std::fs::read_to_string(&config_path)
        .map_err(|e| format!("Failed to read config: {}", e))?;

    let soundpack: SoundPack = serde_json::from_str(&config_content)
        .map_err(|e| format!("Failed to parse config: {}", e))?;

    // Load audio samples
    let samples = load_audio_file(&soundpack_path, &soundpack)?;

    // Create key mappings
    let key_mappings = create_key_mappings(&soundpack, &samples.0);

    // Create compressed audio data for caching
    let compressed_data = CompressedAudioData::new(
        samples.0.clone(),
        samples.2, // Use actual sample rate
        samples.1, // Use actual channels
        key_mappings.clone(),
        soundpack_id.to_string(),
        soundpack.name.clone(),
        soundpack
            .version
            .clone()
            .unwrap_or_else(|| "1.0".to_string()),
    );

    // Serialize and save to individual cache file
    match compressed_data.compress() {
        Ok(serialized) => {
            cache.save_audio_cache(soundpack_id, &serialized);

            // Update metadata cache
            let metadata = create_soundpack_metadata(&soundpack_path, &soundpack)?;
            cache.add_soundpack(metadata);
            cache.save();

            let stats = compressed_data.get_stats();
            println!(
                "💾 Cached {} - Original: {}, Compressed: {}, Savings: {}",
                soundpack_id,
                stats.format_original_size(),
                stats.format_compressed_size(),
                stats.format_savings()
            );
        }
        Err(e) => {
            eprintln!("⚠️  Failed to cache audio data: {}", e);
        }
    }

    // Update audio context
    update_context_optimized(context, compressed_data)?;

    Ok(())
}

fn update_context_optimized(
    context: &AudioContext,
    audio_data: CompressedAudioData,
) -> Result<(), String> {
    // Capture data before moving it
    let sample_count = audio_data.samples.len();
    let key_mapping_count = audio_data.key_mappings.len();
    let soundpack_name = audio_data.soundpack_info.name.clone();

    // Update cached samples (using the existing field structure)
    if let Ok(mut cached) = context.cached_samples.lock() {
        *cached = Some((
            audio_data.samples,
            audio_data.channels,
            audio_data.sample_rate,
        ));
        println!("🎵 Updated cached samples: {} samples", sample_count);
    } else {
        return Err("Failed to acquire lock on cached_samples".to_string());
    }

    // Update key mappings (convert from HashMap<String, Vec<(f64, f64)>> to HashMap<String, Vec<[f32; 2]>>)
    if let Ok(mut key_map) = context.key_map.lock() {
        let old_count = key_map.len();
        key_map.clear();

        // Convert key mappings to the expected format
        for (key, mappings) in audio_data.key_mappings {
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
        "✅ Successfully loaded optimized soundpack: {} ({} key mappings)",
        soundpack_name, key_mapping_count
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
    let (last_modified, file_size) = match std::fs::metadata(soundpack_path) {
        Ok(metadata) => {
            let modified = metadata
                .modified()
                .unwrap_or(std::time::SystemTime::UNIX_EPOCH)
                .duration_since(std::time::SystemTime::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();
            (modified, metadata.len())
        }
        Err(_) => (0, 0),
    };

    Ok(SoundpackMetadata {
        id,
        name: soundpack.name.clone(),
        description: soundpack.description.clone(),
        version: soundpack
            .version
            .clone()
            .unwrap_or_else(|| "1.0".to_string()),
        tags: soundpack.tags.clone().unwrap_or_default(),
        keycap: soundpack.keycap.clone(),
        icon: soundpack.icon.clone(),
        last_modified,
        file_size,
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

/// Get cache statistics for display
pub fn get_cache_statistics() -> Result<String, String> {
    let cache = OptimizedSoundpackCache::load();
    let stats = cache.get_cache_stats();

    Ok(format!(
        "• Metadata entries: {}\n\
         • Audio cache files: {}\n\
         • Total cache size: {}\n",
        stats.metadata_count,
        stats.file_count,
        stats.format_size()
    ))
}

/// Clean up old cache files manually
pub fn cleanup_cache(keep_recent: usize) -> Result<String, String> {
    let mut cache = OptimizedSoundpackCache::load();
    cache.cleanup_old_caches(keep_recent);
    cache.save();

    Ok(format!(
        "🧹 Cache cleanup completed, kept {} most recent files",
        keep_recent
    ))
}

/// Start background cache cleanup task
pub fn start_cache_maintenance() {
    crate::state::optimized_soundpack_cache::start_cache_cleanup_task();
    println!("🔄 Started background cache maintenance task");
}
