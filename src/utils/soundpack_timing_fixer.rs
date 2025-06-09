use crate::libs::audio::soundpack_loader;
use crate::state::soundpack::SoundPack;
use crate::state::paths::soundpacks;
use crate::utils::path;
use serde_json::Value;
use std::collections::HashMap;
use std::fs;

/// Validate soundpack timing issues without fixing them
pub fn validate_soundpack_timing(soundpack_id: &str) -> Result<Vec<String>, String> {
    let config_path = soundpacks::config_json(soundpack_id);
    let soundpack_dir = soundpacks::soundpack_dir(soundpack_id);

    // Read current config
    let config_content = fs
        ::read_to_string(&config_path)
        .map_err(|e| format!("Failed to read config: {}", e))?;

    let config: Value = serde_json
        ::from_str(&config_content)
        .map_err(|e| format!("Failed to parse config: {}", e))?;

    // Parse as SoundPack to get audio info
    let soundpack: SoundPack = serde_json
        ::from_str(&config_content)
        .map_err(|e| format!("Failed to parse soundpack: {}", e))?;

    // Check if source file exists and get its duration
    let source_file = soundpack.source.as_ref().ok_or("No source field found")?;
    let audio_path = format!("{}/{}", soundpack_dir, source_file);

    if !std::path::Path::new(&audio_path).exists() {
        return Err(format!("Audio file not found: {}", audio_path));
    }

    // Get audio duration using our existing logic
    let audio_duration = get_audio_duration_seconds(&audio_path)?;
    let audio_duration_ms = audio_duration * 1000.0;

    println!(
        "🎵 Soundpack: {} - Audio file: {} ({:.3}s / {:.1}ms)",
        soundpack_id,
        source_file,
        audio_duration,
        audio_duration_ms
    );

    // Check timing in defs - only report issues, don't fix
    let mut timing_issues = Vec::new();
    if let Some(defs) = config.get("defs").and_then(|d| d.as_object()) {
        for (key, value) in defs.iter() {
            if let Some(timing_array) = value.as_array() {
                for (idx, timing_entry) in timing_array.iter().enumerate() {
                    if let Some(pair) = timing_entry.as_array() {
                        if pair.len() >= 2 {
                            let start_ms = pair[0].as_f64().unwrap_or(0.0);
                            let duration_ms = pair[1].as_f64().unwrap_or(100.0);
                            let end_ms = start_ms + duration_ms;

                            // Check for timing issues
                            if start_ms >= audio_duration_ms {
                                let issue = format!(
                                    "❌ Key '{}' [{}]: START EXCEEDS AUDIO - start: {:.1}ms >= duration: {:.1}ms",
                                    key,
                                    idx,
                                    start_ms,
                                    audio_duration_ms
                                );
                                println!("{}", issue);
                                timing_issues.push(issue);
                            } else if end_ms > audio_duration_ms {
                                let issue = format!(
                                    "⚠️ Key '{}' [{}]: END EXCEEDS AUDIO - end: {:.1}ms > duration: {:.1}ms (overage: {:.1}ms)",
                                    key,
                                    idx,
                                    end_ms,
                                    audio_duration_ms,
                                    end_ms - audio_duration_ms
                                );
                                println!("{}", issue);
                                timing_issues.push(issue);
                            } else {
                                // Timing is valid
                                println!(
                                    "✅ Key '{}' [{}]: OK - {:.1}ms to {:.1}ms ({:.1}ms duration)",
                                    key,
                                    idx,
                                    start_ms,
                                    end_ms,
                                    duration_ms
                                );
                            }
                        }
                    }
                }
            }
        }
    }

    if timing_issues.is_empty() {
        println!("✅ No timing issues found in soundpack '{}'", soundpack_id);
    } else {
        println!("❌ Found {} timing issues in soundpack '{}'", timing_issues.len(), soundpack_id);
    }

    Ok(timing_issues)
}

/// Get audio duration in seconds using Rodio
fn get_audio_duration_seconds(file_path: &str) -> Result<f64, String> {
    use rodio::{ Decoder, Source };
    use std::fs::File;
    use std::io::BufReader;

    let file = File::open(file_path).map_err(|e| format!("Failed to open audio file: {}", e))?;

    let source = Decoder::new(BufReader::new(file)).map_err(|e|
        format!("Failed to decode audio file: {}", e)
    )?;

    let sample_rate = source.sample_rate();
    let channels = source.channels();
    let samples: Vec<f32> = source.convert_samples().collect();

    let duration = (samples.len() as f64) / (sample_rate as f64) / (channels as f64);
    Ok(duration)
}

/// Check all soundpacks for timing issues without fixing them
pub fn check_all_soundpacks_timing() -> Result<HashMap<String, Vec<String>>, String> {    let soundpacks_dir = path::get_soundpacks_dir_absolute();
    let mut soundpack_issues = HashMap::new();

    if !std::path::Path::new(&soundpacks_dir).exists() {
        return Ok(soundpack_issues);
    }

    let entries = fs
        ::read_dir(&soundpacks_dir)
        .map_err(|e| format!("Failed to read soundpacks directory: {}", e))?;

    for entry in entries {
        if let Ok(entry) = entry {
            let path = entry.path();
            if path.is_dir() {
                if let Some(soundpack_id) = path.file_name().and_then(|n| n.to_str()) {
                    match validate_soundpack_timing(soundpack_id) {
                        Ok(issues) => {
                            if !issues.is_empty() {
                                soundpack_issues.insert(soundpack_id.to_string(), issues);
                            }
                        }
                        Err(e) => {
                            println!("❌ Error checking soundpack '{}': {}", soundpack_id, e);
                            soundpack_issues.insert(
                                soundpack_id.to_string(), 
                                vec![format!("Validation error: {}", e)]
                            );
                        }
                    }
                }
            }
        }
    }

    // Summary
    if soundpack_issues.is_empty() {
        println!("\n✅ All soundpacks have valid timing!");
    } else {
        println!("\n📊 TIMING VALIDATION SUMMARY:");
        for (soundpack_id, issues) in &soundpack_issues {
            println!("  🔍 {}: {} issues", soundpack_id, issues.len());
        }
    }

    Ok(soundpack_issues)
}
