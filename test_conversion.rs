use std::collections::HashMap;
use std::path::Path;

/// Test function to debug V1 to V2 conversion segment calculation
pub fn test_conversion_segment_calculation() {
    println!("🧪 Testing V1 to V2 conversion segment calculation");
    
    // Test the problematic soundpack
    let soundpack_id = "custom-sound-pack-1203000000067";
    let soundpack_dir = format!("d:\\mechvibes-dx\\soundpacks\\{}", soundpack_id);
    
    println!("📁 Testing soundpack: {}", soundpack_id);
    println!("📁 Directory: {}", soundpack_dir);
    
    // Check if combined_audio.wav exists and get its duration
    let audio_file = format!("{}\\combined_audio.wav", soundpack_dir);
    if Path::new(&audio_file).exists() {
        match get_audio_duration_seconds(&audio_file) {
            Ok(duration) => {
                println!("🎵 Audio file duration: {:.3}s ({:.1}ms)", duration, duration * 1000.0);
            }
            Err(e) => {
                println!("❌ Failed to get audio duration: {}", e);
            }
        }
    } else {
        println!("❌ Audio file not found: {}", audio_file);
    }
    
    // Load and examine the current config
    let config_file = format!("{}\\config.json", soundpack_dir);
    if Path::new(&config_file).exists() {
        match std::fs::read_to_string(&config_file) {
            Ok(content) => {
                match serde_json::from_str::<serde_json::Value>(&content) {
                    Ok(config) => {
                        println!("📋 Config loaded successfully");
                        
                        // Check if it's V1 or V2
                        if let Some(version) = config.get("config_version") {
                            println!("📋 Config version: {}", version);
                        } else {
                            println!("📋 No config_version field - likely V1 format");
                        }
                        
                        // Look at the "defines" or "defs" field
                        if let Some(defines) = config.get("defines") {
                            println!("📋 Found V1 'defines' field");
                            examine_v1_defines(defines);
                        } else if let Some(defs) = config.get("defs") {
                            println!("📋 Found V2 'defs' field");
                            examine_v2_defs(defs);
                        }
                    }
                    Err(e) => {
                        println!("❌ Failed to parse config JSON: {}", e);
                    }
                }
            }
            Err(e) => {
                println!("❌ Failed to read config file: {}", e);
            }
        }
    } else {
        println!("❌ Config file not found: {}", config_file);
    }
}

fn examine_v1_defines(defines: &serde_json::Value) {
    if let Some(defines_obj) = defines.as_object() {
        println!("🔍 V1 defines contains {} entries", defines_obj.len());
        
        // Show a few examples
        for (vk_code, value) in defines_obj.iter().take(5) {
            if let Some(sound_file) = value.as_str() {
                println!("  VK {} -> {}", vk_code, sound_file);
            } else if let Some(arr) = value.as_array() {
                println!("  VK {} -> {:?}", vk_code, arr);
            }
        }
        
        // Look for KeyK specifically
        for (vk_code, value) in defines_obj.iter() {
            if vk_code == "75" { // VK_K = 75
                println!("🔍 Found KeyK (VK 75): {:?}", value);
            }
        }
    }
}

fn examine_v2_defs(defs: &serde_json::Value) {
    if let Some(defs_obj) = defs.as_object() {
        println!("🔍 V2 defs contains {} entries", defs_obj.len());
        
        // Look for KeyK specifically and show its timing
        if let Some(keyk_timing) = defs_obj.get("KeyK") {
            println!("🔍 KeyK timing: {:?}", keyk_timing);
            
            if let Some(timing_array) = keyk_timing.as_array() {
                for (idx, timing_entry) in timing_array.iter().enumerate() {
                    if let Some(pair) = timing_entry.as_array() {
                        if pair.len() >= 2 {
                            let start_ms = pair[0].as_f64().unwrap_or(0.0);
                            let duration_ms = pair[1].as_f64().unwrap_or(0.0);
                            println!("  Entry {}: start={:.1}ms, duration={:.1}ms, end={:.1}ms", 
                                   idx, start_ms, duration_ms, start_ms + duration_ms);
                        }
                    }
                }
            }
        }
        
        // Show other problematic keys
        let problem_keys = ["KeyK", "KeyL", "KeyM", "Space", "Tab"];
        for key in &problem_keys {
            if let Some(timing) = defs_obj.get(*key) {
                if let Some(timing_array) = timing.as_array() {
                    if let Some(first_entry) = timing_array.get(0) {
                        if let Some(pair) = first_entry.as_array() {
                            if pair.len() >= 2 {
                                let start_ms = pair[0].as_f64().unwrap_or(0.0);
                                let duration_ms = pair[1].as_f64().unwrap_or(0.0);
                                println!("🔍 {}: start={:.1}ms, duration={:.1}ms, end={:.1}ms", 
                                       key, start_ms, duration_ms, start_ms + duration_ms);
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Get audio duration in seconds using Rodio
fn get_audio_duration_seconds(file_path: &str) -> Result<f64, String> {
    use rodio::{Decoder, Source};
    use std::fs::File;
    use std::io::BufReader;

    let file = File::open(file_path).map_err(|e| format!("Failed to open audio file: {}", e))?;

    let source = Decoder::new(BufReader::new(file))
        .map_err(|e| format!("Failed to decode audio file: {}", e))?;

    let sample_rate = source.sample_rate();
    let channels = source.channels();
    let samples: Vec<f32> = source.convert_samples().collect();

    let duration = (samples.len() as f64) / (sample_rate as f64) / (channels as f64);
    Ok(duration)
}

/// Test the segment calculation logic in isolation
pub fn test_segment_calculation_logic() {
    println!("\n🧪 Testing segment calculation logic");
    
    // Simulate the V1 to V2 conversion process for a few files
    let mut sound_files = HashMap::new();
    sound_files.insert("KeyA".to_string(), "key_a.wav".to_string());
    sound_files.insert("KeyB".to_string(), "key_b.wav".to_string());
    sound_files.insert("KeyK".to_string(), "key_k.wav".to_string());
    
    // Simulate file durations (in ms)
    let mut file_durations = HashMap::new();
    file_durations.insert("key_a.wav".to_string(), 150.0);
    file_durations.insert("key_b.wav".to_string(), 200.0);
    file_durations.insert("key_k.wav".to_string(), 180.0);
    
    println!("📊 Input sound files:");
    for (key, file) in &sound_files {
        if let Some(duration) = file_durations.get(file) {
            println!("  {} -> {} ({:.1}ms)", key, file, duration);
        }
    }
    
    // Simulate the segment calculation logic
    let unique_files: std::collections::BTreeSet<String> = sound_files.values().cloned().collect();
    let mut current_position = 0.0;
    let mut file_segments = HashMap::new();
    
    println!("\n📊 Calculated segments in concatenated file:");
    for sound_file in &unique_files {
        if let Some(&duration) = file_durations.get(sound_file) {
            file_segments.insert(sound_file.clone(), (current_position, duration));
            println!("  {} -> start={:.1}ms, duration={:.1}ms, end={:.1}ms", 
                   sound_file, current_position, duration, current_position + duration);
            current_position += duration;
        }
    }
    
    println!("\n📊 Final key mappings:");
    for (key_name, sound_file) in &sound_files {
        if let Some(&(start, duration)) = file_segments.get(sound_file) {
            println!("  {} -> start={:.1}ms, duration={:.1}ms, end={:.1}ms", 
                   key_name, start, duration, start + duration);
        }
    }
    
    println!("📊 Total concatenated duration: {:.1}ms", current_position);
}

fn main() {
    test_conversion_segment_calculation();
    test_segment_calculation_logic();
}
