use super::path;
use serde_json::{ Map, Value };
use std::collections::HashMap;
use std::path::Path;

/// Get the duration of an audio file in milliseconds using Symphonia
fn get_audio_duration_ms(file_path: &str) -> Result<f64, Box<dyn std::error::Error>> {
    println!("🔍 [DEBUG] get_audio_duration_ms called with: {}", file_path);

    // Check if file exists first
    if !Path::new(file_path).exists() {
        println!("❌ [DEBUG] File does not exist: {}", file_path);
        return Err("File does not exist".into());
    }

    println!("✅ [DEBUG] File exists, calling Symphonia...");

    // Use symphonia for audio duration detection
    match get_duration_with_symphonia(file_path) {
        Ok(duration) if duration > 0.0 => {
            println!("✅ [DEBUG] Symphonia returned valid duration: {:.3}ms", duration);
            Ok(duration)
        }
        Ok(duration) => {
            println!(
                "⚠️ [DEBUG] Symphonia returned invalid duration: {:.3}ms, using default 100ms",
                duration
            );
            Ok(100.0)
        }
        Err(e) => {
            println!("❌ [DEBUG] Symphonia failed: {}, using default 100ms", e);
            Ok(100.0)
        }
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
    let format = probed.format;

    // Get the default track
    let track = format
        .tracks()
        .iter()
        .find(|t| t.codec_params.codec != symphonia::core::codecs::CODEC_TYPE_NULL)
        .ok_or("No valid audio track found")?;

    // Calculate duration from time base and frames
    if
        let (Some(time_base), Some(n_frames)) = (
            track.codec_params.time_base,
            track.codec_params.n_frames,
        )
    {
        let duration_seconds =
            ((n_frames as f64) * (time_base.numer as f64)) / (time_base.denom as f64);
        let duration_ms = duration_seconds * 1000.0;
        return Ok(duration_ms);
    }
    Err("No duration information available in track".into())
}

/// Round timing data to ensure proper alignment and avoid gaps
fn round_timing_data(segments: &mut HashMap<String, (f64, f64)>, _total_duration: f64) {
    for (_key, (start, duration)) in segments.iter_mut() {
        // Round to nearest millisecond to avoid floating point precision issues
        *start = start.round();
        *duration = duration.round();

        // Ensure minimum duration of 1ms
        if *duration < 1.0 {
            *duration = 1.0;
        }
    }
}

/// Create concatenated audio file with precise timing using sample-based calculation
fn create_concatenated_audio_file_with_precise_timing(
    soundpack_dir: &str,
    unique_files: &std::collections::BTreeSet<String>,
    output_path: &str
) -> Result<HashMap<String, f64>, Box<dyn std::error::Error>> {
    use symphonia::core::audio::{ AudioBufferRef, Signal };
    use symphonia::core::codecs::{ DecoderOptions, CODEC_TYPE_NULL };
    use symphonia::core::formats::FormatOptions;
    use symphonia::core::io::MediaSourceStream;
    use symphonia::core::meta::MetadataOptions;
    use symphonia::core::probe::Hint;
    use std::fs::File;

    let mut all_samples: Vec<f32> = Vec::new();
    let mut sample_rate = 44100u32;
    let mut channels = 2u16;
    let mut file_durations = HashMap::new();

    // Process each unique file and track precise durations
    for (index, sound_file) in unique_files.iter().enumerate() {
        let file_path = format!("{}/{}", soundpack_dir, sound_file);

        if !std::path::Path::new(&file_path).exists() {
            file_durations.insert(sound_file.clone(), 100.0);
            continue;
        }

        match File::open(&file_path) {
            Ok(audio_file) => {
                let mss = MediaSourceStream::new(Box::new(audio_file), Default::default());

                let mut hint = Hint::new();
                if let Some(extension) = std::path::Path::new(&file_path).extension() {
                    if let Some(ext_str) = extension.to_str() {
                        hint.with_extension(ext_str);
                    }
                }

                let meta_opts: MetadataOptions = Default::default();
                let fmt_opts: FormatOptions = Default::default();

                match symphonia::default::get_probe().format(&hint, mss, &fmt_opts, &meta_opts) {
                    Ok(probed) => {
                        let mut format = probed.format;
                        let track = format
                            .tracks()
                            .iter()
                            .find(|t| t.codec_params.codec != CODEC_TYPE_NULL);

                        if let Some(track) = track {
                            let dec_opts: DecoderOptions = Default::default();
                            match
                                symphonia::default
                                    ::get_codecs()
                                    .make(&track.codec_params, &dec_opts)
                            {
                                Ok(mut decoder) => {
                                    let track_id = track.id;
                                    let mut file_samples = Vec::new();

                                    // Set sample rate and channels from first file
                                    if index == 0 {
                                        sample_rate =
                                            track.codec_params.sample_rate.unwrap_or(44100);
                                        channels = track.codec_params.channels
                                            .map(|ch| ch.count() as u16)
                                            .unwrap_or(2);
                                    }

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
                                                // Convert audio buffer to f32 samples
                                                match decoded {
                                                    AudioBufferRef::F32(buf) => {
                                                        for &sample in buf.chan(0) {
                                                            file_samples.push(sample);
                                                        }
                                                        if
                                                            channels > 1 &&
                                                            buf.spec().channels.count() > 1
                                                        {
                                                            for &sample in buf.chan(1) {
                                                                file_samples.push(sample);
                                                            }
                                                        }
                                                    }
                                                    AudioBufferRef::S16(buf) => {
                                                        for &sample in buf.chan(0) {
                                                            file_samples.push(
                                                                (sample as f32) / (i16::MAX as f32)
                                                            );
                                                        }
                                                        if
                                                            channels > 1 &&
                                                            buf.spec().channels.count() > 1
                                                        {
                                                            for &sample in buf.chan(1) {
                                                                file_samples.push(
                                                                    (sample as f32) /
                                                                        (i16::MAX as f32)
                                                                );
                                                            }
                                                        }
                                                    }
                                                    _ => {
                                                        // Handle other formats with basic conversion
                                                        // Skip complex conversion for concatenation
                                                        continue;
                                                    }
                                                }
                                            }
                                            Err(_) => {
                                                continue;
                                            }
                                        }
                                    }

                                    // Calculate precise duration from sample count
                                    let duration_ms = if sample_rate > 0 && channels > 0 {
                                        ((file_samples.len() as f64) /
                                            (sample_rate as f64) /
                                            (channels as f64)) *
                                            1000.0
                                    } else {
                                        100.0
                                    };

                                    file_durations.insert(sound_file.clone(), duration_ms);
                                    all_samples.extend(file_samples);
                                }
                                Err(_) => {
                                    file_durations.insert(sound_file.clone(), 100.0);
                                }
                            }
                        } else {
                            file_durations.insert(sound_file.clone(), 100.0);
                        }
                    }
                    Err(_) => {
                        file_durations.insert(sound_file.clone(), 100.0);
                    }
                }
            }
            Err(_) => {
                file_durations.insert(sound_file.clone(), 100.0);
            }
        }
    }

    if all_samples.is_empty() {
        return Err("No audio samples were collected".into());
    }

    // Write concatenated samples to WAV file using hound
    let spec = hound::WavSpec {
        channels,
        sample_rate,
        bits_per_sample: 32,
        sample_format: hound::SampleFormat::Float,
    };

    let mut writer = hound::WavWriter
        ::create(output_path, spec)
        .map_err(|e| format!("Failed to create WAV writer: {}", e))?;

    // Write samples
    for sample in all_samples.iter() {
        writer.write_sample(*sample).map_err(|e| format!("Failed to write sample: {}", e))?;
    }

    // Finalize the WAV file
    writer.finalize().map_err(|e| format!("Failed to finalize WAV file: {}", e))?;

    Ok(file_durations)
}

/// Create concatenated audio file from multiple sound files and return segment mappings
fn create_concatenated_audio_and_segments(
    soundpack_dir: &str,
    sound_files: &HashMap<String, String>,
    output_audio_path: &str
) -> Result<HashMap<String, (f64, f64)>, Box<dyn std::error::Error>> {
    use std::collections::BTreeSet;

    println!("🔧 Creating concatenated audio from {} sound mappings", sound_files.len());
    let mut segments = HashMap::new();

    // Step 1: Collect unique sound files and their durations
    let mut unique_files: BTreeSet<String> = BTreeSet::new();
    let mut file_durations: HashMap<String, f64> = HashMap::new();

    // First pass: collect all unique sound files
    for (key, sound_file) in sound_files {
        if !sound_file.is_empty() && sound_file != "null" {
            println!("📁 Found sound file: {} -> {}", key, sound_file);
            unique_files.insert(sound_file.clone());
        }
    }

    println!("📊 Found {} unique sound files", unique_files.len()); // Step 2: Read durations for all unique files
    for sound_file in &unique_files {
        let file_path = format!("{}/{}", soundpack_dir, sound_file);

        if !Path::new(&file_path).exists() {
            println!("❌ File not found: {} (using default 100ms duration)", file_path);
            file_durations.insert(sound_file.clone(), 100.0);
            continue;
        }

        match get_audio_duration_ms(&file_path) {
            Ok(duration) => {
                println!("⏱️ File {}: duration = {:.3}ms", sound_file, duration);
                file_durations.insert(sound_file.clone(), duration);
            }
            Err(e) => {
                println!(
                    "⚠️ Failed to get duration for {}: {} (using default 100ms)",
                    sound_file,
                    e
                );
                file_durations.insert(sound_file.clone(), 100.0);
            }
        }
    }

    // Step 3: Create concatenated audio file and calculate segment positions
    let mut file_segments: HashMap<String, (f64, f64)> = HashMap::new();

    // Try to create concatenated audio using precise sample-based method
    match
        create_concatenated_audio_file_with_precise_timing(
            soundpack_dir,
            &unique_files,
            output_audio_path
        )
    {
        Ok(precise_file_durations) => {
            // Step 4: Calculate segment positions using PRECISE durations from samples
            let mut current_position = 0.0;
            let mut total_duration = 0.0;

            for sound_file in &unique_files {
                if let Some(&precise_duration) = precise_file_durations.get(sound_file) {
                    file_segments.insert(sound_file.clone(), (current_position, precise_duration));
                    current_position += precise_duration;
                    total_duration = current_position; // Track total concatenated duration
                } else {
                    if let Some(&fallback_duration) = file_durations.get(sound_file) {
                        file_segments.insert(sound_file.clone(), (
                            current_position,
                            fallback_duration,
                        ));
                        current_position += fallback_duration;
                        total_duration = current_position; // Track total concatenated duration
                    }
                }
            }

            println!("📏 Total concatenated audio duration: {:.3}ms", total_duration); // Step 5: Map keys to their corresponding PRECISE segments with validation
            for (key_name, sound_file) in sound_files {
                if let Some(&(start, duration)) = file_segments.get(sound_file) {
                    // ===== DURATION VALIDATION =====
                    let mut validated_start = start;
                    let mut validated_duration = duration;
                    println!(
                        "🔍 Validating timing for key '{}': start={:.3}ms, duration={:.3}ms, total={:.3}ms",
                        key_name,
                        start,
                        duration,
                        total_duration
                    );

                    // Check if start exceeds total duration
                    if start >= total_duration {
                        println!(
                            "⚠️ Key '{}': start time {:.3}ms exceeds total duration {:.3}ms, using default timing",
                            key_name,
                            start,
                            total_duration
                        );
                        validated_start = 0.0;
                        validated_duration = 100.0;
                    } else if start + duration > total_duration {
                        // Adjust duration to fit within audio bounds
                        validated_duration = total_duration - start;
                        println!(
                            "✂️ Key '{}': duration adjusted from {:.3}ms to {:.3}ms to fit within audio bounds (total: {:.3}ms)",
                            key_name,
                            duration,
                            validated_duration,
                            total_duration
                        );
                    } else {
                        println!("✅ Key '{}': timing is valid within audio bounds", key_name);
                    }
                    // ===== END VALIDATION =====

                    segments.insert(key_name.clone(), (validated_start, validated_duration));
                } else {
                    segments.insert(key_name.clone(), (0.0, 100.0));
                }
            }
        }
        Err(_) => {
            // Fallback to old method
            if
                let Err(_) = create_concatenated_audio_file(
                    soundpack_dir,
                    &unique_files,
                    &file_durations,
                    output_audio_path
                )
            {
                // Final fallback: copy first file and use individual timing
                if let Some(first_file) = unique_files.iter().next() {
                    let first_file_path = format!("{}/{}", soundpack_dir, first_file);
                    if Path::new(&first_file_path).exists() {
                        let _ = std::fs::copy(&first_file_path, output_audio_path);
                    }
                }

                // Use individual file approach as final fallback
                for (key_name, sound_file) in sound_files {
                    if let Some(&duration) = file_durations.get(sound_file) {
                        segments.insert(key_name.clone(), (0.0, duration));
                    }
                }
            } else {
                // Use metadata-based timing as fallback
                let mut current_position = 0.0;
                let mut total_duration = 0.0;

                for sound_file in &unique_files {
                    if let Some(&duration) = file_durations.get(sound_file) {
                        file_segments.insert(sound_file.clone(), (current_position, duration));
                        current_position += duration;
                        total_duration = current_position; // Track total concatenated duration
                    }
                }

                println!(
                    "📏 Total concatenated audio duration (fallback): {:.3}ms",
                    total_duration
                ); // Map keys to segments with validation
                for (key_name, sound_file) in sound_files {
                    if let Some(&(start, duration)) = file_segments.get(sound_file) {
                        // ===== DURATION VALIDATION =====
                        let mut validated_start = start;
                        let mut validated_duration = duration;
                        println!(
                            "🔍 Validating timing for key '{}' (fallback): start={:.3}ms, duration={:.3}ms, total={:.3}ms",
                            key_name,
                            start,
                            duration,
                            total_duration
                        );

                        // Check if start exceeds total duration
                        if start >= total_duration {
                            println!(
                                "⚠️ Key '{}': start time {:.3}ms exceeds total duration {:.3}ms, using default timing",
                                key_name,
                                start,
                                total_duration
                            );
                            validated_start = 0.0;
                            validated_duration = 100.0;
                        } else if start + duration > total_duration {
                            // Adjust duration to fit within audio bounds
                            validated_duration = total_duration - start;
                            println!(
                                "✂️ Key '{}': duration adjusted from {:.3}ms to {:.3}ms to fit within audio bounds (total: {:.3}ms)",
                                key_name,
                                duration,
                                validated_duration,
                                total_duration
                            );
                        } else {
                            println!("✅ Key '{}': timing is valid within audio bounds", key_name);
                        }
                        // ===== END VALIDATION =====

                        segments.insert(key_name.clone(), (validated_start, validated_duration));
                    } else {
                        segments.insert(key_name.clone(), (0.0, 100.0));
                    }
                }
            }
        }
    }

    // Round timing data to ensure gaps between segments are acceptable and timing is within bounds
    round_timing_data(&mut segments, 1000.0);

    Ok(segments)
}

/// Create a concatenated audio file from multiple source files using WAV format
fn create_concatenated_audio_file(
    soundpack_dir: &str,
    unique_files: &std::collections::BTreeSet<String>,
    _file_durations: &HashMap<String, f64>,
    output_path: &str
) -> Result<(), Box<dyn std::error::Error>> {
    use symphonia::core::audio::{ AudioBufferRef, Signal };
    use symphonia::core::codecs::{ DecoderOptions, CODEC_TYPE_NULL };
    use symphonia::core::formats::FormatOptions;
    use symphonia::core::io::MediaSourceStream;
    use symphonia::core::meta::MetadataOptions;
    use symphonia::core::probe::Hint;
    use std::fs::File;

    // Collect all audio samples from all files
    let mut all_samples: Vec<f32> = Vec::new();
    let mut sample_rate = 44100u32;
    let mut channels = 2u16;

    // Process each unique file
    for (index, sound_file) in unique_files.iter().enumerate() {
        let file_path = format!("{}/{}", soundpack_dir, sound_file);

        if !Path::new(&file_path).exists() {
            continue;
        }

        match File::open(&file_path) {
            Ok(audio_file) => {
                let mss = MediaSourceStream::new(Box::new(audio_file), Default::default());

                let mut hint = Hint::new();
                if let Some(extension) = Path::new(&file_path).extension() {
                    if let Some(ext_str) = extension.to_str() {
                        hint.with_extension(ext_str);
                    }
                }

                let meta_opts: MetadataOptions = Default::default();
                let fmt_opts: FormatOptions = Default::default();

                match symphonia::default::get_probe().format(&hint, mss, &fmt_opts, &meta_opts) {
                    Ok(probed) => {
                        let mut format = probed.format;
                        let track = format
                            .tracks()
                            .iter()
                            .find(|t| t.codec_params.codec != CODEC_TYPE_NULL);

                        if let Some(track) = track {
                            let dec_opts: DecoderOptions = Default::default();
                            match
                                symphonia::default
                                    ::get_codecs()
                                    .make(&track.codec_params, &dec_opts)
                            {
                                Ok(mut decoder) => {
                                    let track_id = track.id;

                                    // Set sample rate and channels from first file
                                    if index == 0 {
                                        sample_rate =
                                            track.codec_params.sample_rate.unwrap_or(44100);
                                        channels = track.codec_params.channels
                                            .map(|ch| ch.count() as u16)
                                            .unwrap_or(2);
                                    }

                                    // Decode audio packets
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
                                                // Convert to f32 samples - simplified version
                                                match decoded {
                                                    AudioBufferRef::F32(buf) => {
                                                        for &sample in buf.chan(0) {
                                                            all_samples.push(sample);
                                                        }
                                                        if
                                                            channels > 1 &&
                                                            buf.spec().channels.count() > 1
                                                        {
                                                            for &sample in buf.chan(1) {
                                                                all_samples.push(sample);
                                                            }
                                                        }
                                                    }
                                                    AudioBufferRef::S16(buf) => {
                                                        for &sample in buf.chan(0) {
                                                            all_samples.push(
                                                                (sample as f32) / (i16::MAX as f32)
                                                            );
                                                        }
                                                        if
                                                            channels > 1 &&
                                                            buf.spec().channels.count() > 1
                                                        {
                                                            for &sample in buf.chan(1) {
                                                                all_samples.push(
                                                                    (sample as f32) /
                                                                        (i16::MAX as f32)
                                                                );
                                                            }
                                                        }
                                                    }
                                                    _ => {
                                                        // Skip other formats for simplicity in concatenation
                                                        continue;
                                                    }
                                                }
                                            }
                                            Err(e) => {
                                                return Err(
                                                    format!(
                                                        "Failed to decode {}: {}",
                                                        sound_file,
                                                        e
                                                    ).into()
                                                );
                                            }
                                        }
                                    }
                                }
                                Err(e) => {
                                    return Err(
                                        format!(
                                            "Failed to create decoder for {}: {}",
                                            sound_file,
                                            e
                                        ).into()
                                    );
                                }
                            }
                        }
                    }
                    Err(e) => {
                        return Err(format!("Failed to probe {}: {}", sound_file, e).into());
                    }
                }
            }
            Err(e) => {
                return Err(format!("Failed to open {}: {}", file_path, e).into());
            }
        }
    }

    if all_samples.is_empty() {
        return Err("No audio samples were collected".into());
    }

    // Write concatenated samples to WAV file using hound
    let spec = hound::WavSpec {
        channels,
        sample_rate,
        bits_per_sample: 32,
        sample_format: hound::SampleFormat::Float,
    };

    let mut writer = hound::WavWriter
        ::create(output_path, spec)
        .map_err(|e| format!("Failed to create WAV writer: {}", e))?;

    // Write samples
    for sample in all_samples.iter() {
        writer.write_sample(*sample).map_err(|e| format!("Failed to write sample: {}", e))?;
    }

    // Finalize the WAV file
    writer.finalize().map_err(|e| format!("Failed to finalize WAV file: {}", e))?;

    Ok(())
}

/// Convert soundpack config from version 1 to version 2
/// Uses comprehensive IOHook keycode mapping (supports all platforms)
pub fn convert_v1_to_v2(
    v1_config_path: &str,
    output_path: &str,
    soundpack_dir: Option<&str>
) -> Result<(), Box<dyn std::error::Error>> {
    println!("🔄 Starting V1 to V2 conversion...");
    println!("📁 Input config: {}", v1_config_path);
    println!("📁 Output path: {}", output_path);

    // Determine soundpack directory - use provided or infer from config path
    let soundpack_dir = if let Some(dir) = soundpack_dir {
        println!("📁 Using provided soundpack directory: {}", dir);
        dir
    } else {
        let inferred_dir = Path::new(v1_config_path)
            .parent()
            .and_then(|p| p.to_str())
            .ok_or("Could not determine soundpack directory")?;
        println!("📁 Inferred soundpack directory: {}", inferred_dir);
        inferred_dir
    };

    // Read the V1 config
    println!("📖 Reading V1 config file...");
    let content = path
        ::read_file_contents(v1_config_path)
        .map_err(|e| format!("Failed to read V1 config: {}", e))?;

    let config: Value = serde_json::from_str(&content)?;
    println!("✅ Successfully parsed V1 config");

    // Log basic config info
    if let Some(name) = config.get("name") {
        println!("🎵 Soundpack name: {}", name.as_str().unwrap_or("Unknown"));
    }
    if let Some(defines) = config.get("defines").and_then(|d| d.as_object()) {
        println!("🎹 Found {} key definitions", defines.len());
    }

    let mut converted_config = Map::new();

    // Copy basic fields
    if let Some(id) = config.get("id") {
        converted_config.insert("id".to_string(), id.clone());
    }

    if let Some(name) = config.get("name") {
        converted_config.insert("name".to_string(), name.clone());
    }

    // Add author field (required in V2) - use from config or default to "N/A"
    if let Some(author) = config.get("author") {
        converted_config.insert("author".to_string(), author.clone());
    } else {
        converted_config.insert("author".to_string(), Value::String("N/A".to_string()));
    }

    // Optional fields
    if let Some(description) = config.get("description") {
        converted_config.insert("description".to_string(), description.clone());
    }

    if let Some(version) = config.get("version") {
        converted_config.insert("version".to_string(), version.clone());
    }

    if let Some(tags) = config.get("tags") {
        converted_config.insert("tags".to_string(), tags.clone());
    }

    if let Some(keycap) = config.get("keycap") {
        converted_config.insert("keycap".to_string(), keycap.clone());
    }

    if let Some(icon) = config.get("icon") {
        converted_config.insert("icon".to_string(), icon.clone());
    } // Add method field - determine from key_define_type or default to "single"
    let method = if let Some(key_define_type) = config.get("key_define_type") {
        if key_define_type.as_str() == Some("multi") {
            converted_config.insert("method".to_string(), Value::String("multi".to_string()));
            println!("🎵 Method: MULTI (detected from key_define_type)");
            "multi"
        } else {
            converted_config.insert("method".to_string(), Value::String("single".to_string()));
            println!("🎵 Method: SINGLE (from key_define_type: {:?})", key_define_type.as_str());
            "single"
        }
    } else {
        converted_config.insert("method".to_string(), Value::String("single".to_string()));
        println!("🎵 Method: SINGLE (default - no key_define_type found)");
        "single"
    };

    // Copy includes_numpad if present
    if let Some(includes_numpad) = config.get("includes_numpad") {
        converted_config.insert("includes_numpad".to_string(), includes_numpad.clone());
    }

    // Add mouse field (defaults to false for keyboard soundpacks)
    converted_config.insert("mouse".to_string(), Value::Bool(false));

    // Add config_version field set to 2
    converted_config.insert(
        "config_version".to_string(),
        Value::Number(serde_json::Number::from(2))
    );

    // Handle source field and multi-method audio processing
    let output_audio_filename = if method == "multi" {
        "combined_audio.wav".to_string()
    } else {
        // For single method, preserve the original sound field
        if let Some(sound) = config.get("sound") {
            converted_config.insert("source".to_string(), sound.clone());
        }
        String::new() // Not used for single method
    }; // Convert "defines" to "defs" with proper timing format based on method
    let mut defs = Map::new();
    if let Some(defines) = config.get("defines").and_then(|d| d.as_object()) {
        // Key mapping from IOHook scan codes to Web API key names
        // Uses comprehensive mapping that supports all platforms
        let key_mappings = create_iohook_to_web_key_mapping();
        println!("🔧 Using comprehensive IOHook keycode mapping (all platforms supported)");
        if method == "multi" {
            println!("🔄 Processing MULTI method - collecting sound files...");
            // Multi method: collect all sound files and create segments
            let mut sound_files = HashMap::new(); // First pass: collect all unique sound files mapped to their keys
            for (iohook_code, value) in defines {
                if let Ok(iohook_num) = iohook_code.parse::<u32>() {
                    if let Some(key_name) = key_mappings.get(&iohook_num) {
                        // Skip null values and only process actual string sound file names
                        if !value.is_null() {
                            if let Some(sound_file_str) = value.as_str() {
                                if !sound_file_str.is_empty() && sound_file_str != "null" {
                                    println!(
                                        "🎵 IOHook {}: {} -> {}",
                                        iohook_code,
                                        key_name,
                                        sound_file_str
                                    );
                                    sound_files.insert(
                                        key_name.clone(),
                                        sound_file_str.to_string()
                                    );
                                } else {
                                    println!(
                                        "⚠️ Skipping empty/null sound file for IOHook {}: {}",
                                        iohook_code,
                                        key_name
                                    );
                                }
                            } else {
                                println!(
                                    "⚠️ Invalid sound file type for IOHook {}: {} (not a string)",
                                    iohook_code,
                                    key_name
                                );
                            }
                        } else {
                            println!(
                                "⚠️ Skipping null value for IOHook {}: {}",
                                iohook_code,
                                key_name
                            );
                        }
                    } else {
                        println!("⚠️ Unknown IOHook code: {} -> no mapping found", iohook_code);
                    }
                } else {
                    println!("⚠️ Invalid IOHook code format: {}", iohook_code);
                }
            }

            println!(
                "📊 Collected {} unique sound files from {} keys",
                sound_files.len(),
                defines.len()
            ); // Create concatenated audio and get segment mappings
            let output_audio_path = format!("{}/{}", soundpack_dir, output_audio_filename);
            println!("🎚️ Creating concatenated audio: {}", output_audio_path);

            match
                create_concatenated_audio_and_segments(
                    soundpack_dir,
                    &sound_files,
                    &output_audio_path
                )
            {
                Ok(segments) => {
                    println!(
                        "✅ Successfully created concatenated audio with {} segments",
                        segments.len()
                    );

                    // ===== FINAL ROUNDING PHASE =====
                    // This is where we round all timing data to ensure proper alignment
                    let mut final_segments = segments;
                    let total_concatenated_duration = final_segments
                        .values()
                        .map(|(start, duration)| start + duration)
                        .fold(0.0, f64::max);
                    round_timing_data(&mut final_segments, total_concatenated_duration);
                    // ===== END FINAL ROUNDING =====

                    // Set the source to the audio file
                    converted_config.insert(
                        "source".to_string(),
                        Value::String(output_audio_filename)
                    );

                    // Create timing definitions using rounded segments
                    for (key_name, _) in &sound_files {
                        if let Some((start, duration)) = final_segments.get(key_name) {
                            println!(
                                "⏱️ Key {}: final timing start={:.0}ms, duration={:.0}ms",
                                key_name,
                                start,
                                duration
                            );
                            let timing = vec![
                                Value::Array(vec![Value::from(*start), Value::from(*duration)])
                            ];
                            defs.insert(key_name.clone(), Value::Array(timing));
                        } else {
                            println!("⚠️ No segment found for key: {}", key_name);
                        }
                    }
                }
                Err(e) => {
                    println!("❌ Failed to create concatenated audio: {}", e);
                    // Set source to the preserved filename even if processing failed
                    converted_config.insert(
                        "source".to_string(),
                        Value::String(output_audio_filename)
                    );

                    // Fallback: use default timing for multi method
                    for (key_name, _) in &sound_files {
                        let timing = vec![Value::Array(vec![Value::from(0.0), Value::from(100.0)])];
                        defs.insert(key_name.clone(), Value::Array(timing));
                    }
                }
            }
        } else {
            println!("🔄 Processing SINGLE method - validating timing data...");

            // For single method, we need to get the audio file duration to validate timing
            let audio_file_duration = if let Some(sound) = config.get("sound") {
                if let Some(sound_file) = sound.as_str() {
                    let audio_path = format!("{}/{}", soundpack_dir, sound_file);
                    match get_audio_duration_ms(&audio_path) {
                        Ok(duration) => {
                            println!("📏 Audio file '{}' duration: {:.3}ms", sound_file, duration);
                            Some(duration)
                        }
                        Err(e) => {
                            println!("⚠️ Failed to get audio duration for '{}': {}", sound_file, e);
                            None
                        }
                    }
                } else {
                    println!("⚠️ Sound field is not a string");
                    None
                }
            } else {
                println!("⚠️ No sound field found in config");
                None
            }; // Single method: preserve existing logic with null filtering
            for (iohook_code, value) in defines {
                // Convert IOHook code to key name
                if let Ok(iohook_num) = iohook_code.parse::<u32>() {
                    if let Some(key_name) = key_mappings.get(&iohook_num) {
                        // Skip null values in single method too
                        if value.is_null() {
                            continue;
                        }
                        let timing = if let Some(timing_array) = value.as_array() {
                            if timing_array.len() >= 2 {
                                // Keep [start_ms, duration_ms] format: [[start_ms, duration_ms]]
                                if
                                    let (Some(mut start), Some(mut duration)) = (
                                        timing_array[0].as_f64(),
                                        timing_array[1].as_f64(),
                                    )
                                {
                                    // ===== DURATION VALIDATION =====
                                    // If we have the audio file duration, validate timing
                                    if let Some(total_duration) = audio_file_duration {
                                        println!(
                                            "🔍 Validating timing for key '{}': start={:.3}ms, duration={:.3}ms, total={:.3}ms",
                                            key_name,
                                            start,
                                            duration,
                                            total_duration
                                        );

                                        // Check if start exceeds total duration
                                        if start >= total_duration {
                                            println!(
                                                "⚠️ Key '{}': start time {:.3}ms exceeds audio duration {:.3}ms, using default timing",
                                                key_name,
                                                start,
                                                total_duration
                                            );
                                            start = 0.0;
                                            duration = 100.0; // Use default duration, don't expand
                                        } else if start + duration > total_duration {
                                            // Only adjust if timing actually exceeds audio bounds
                                            let old_duration = duration;
                                            duration = total_duration - start;
                                            println!(
                                                "✂️ Key '{}': duration adjusted from {:.3}ms to {:.0}ms to fit within audio bounds (total: {:.0}ms)",
                                                key_name,
                                                old_duration,
                                                duration,
                                                total_duration
                                            );
                                        } else {
                                            println!("✅ Key '{}': timing is valid within audio bounds", key_name);
                                        }
                                    } else {
                                        println!("⚠️ Key '{}': no audio duration available, skipping validation", key_name);
                                    }
                                    // ===== END VALIDATION =====

                                    vec![
                                        Value::Array(
                                            vec![Value::from(start), Value::from(duration)]
                                        )
                                    ]
                                } else {
                                    // Fallback to default timing if conversion fails
                                    vec![Value::Array(vec![Value::from(0.0), Value::from(100.0)])]
                                }
                            } else {
                                // Array too short, use default timing
                                vec![Value::Array(vec![Value::from(0.0), Value::from(100.0)])]
                            }
                        } else if let Some(sound_file_str) = value.as_str() {
                            // Fallback: if it's a string (like in multi), use default timing
                            if !sound_file_str.is_empty() && sound_file_str != "null" {
                                vec![Value::Array(vec![Value::from(0.0), Value::from(100.0)])]
                            } else {
                                continue; // Skip empty entries
                            }
                        } else {
                            continue; // Skip unknown types
                        };
                        defs.insert(key_name.clone(), Value::Array(timing));
                    }
                }
            }
        }
    }
    converted_config.insert("defs".to_string(), Value::Object(defs));

    // Write the converted config
    println!("💾 Writing converted config to: {}", output_path);
    let converted_json = serde_json::to_string_pretty(&converted_config)?;
    path
        ::write_file_contents(output_path, &converted_json)
        .map_err(|e| format!("Failed to write converted config: {}", e))?;

    println!("✅ Successfully converted V1 to V2 config!");
    println!("📊 Final stats:");
    println!("  - Method: {}", method);
    if let Some(defs_obj) = converted_config.get("defs").and_then(|d| d.as_object()) {
        println!("  - Keys defined: {}", defs_obj.len());
    }

    Ok(())
}

// Windows VK mapping removed - we now use IOHook scan codes as the default standard

/// Create comprehensive mapping from IOHook scan codes to Web API key names
/// Combines all platform-specific codes to support any IOHook keycode regardless of platform
fn create_iohook_to_web_key_mapping() -> HashMap<u32, String> {
    let mut mapping = HashMap::new();

    // Function keys
    mapping.insert(59, "F1".to_string());
    mapping.insert(60, "F2".to_string());
    mapping.insert(61, "F3".to_string());
    mapping.insert(62, "F4".to_string());
    mapping.insert(63, "F5".to_string());
    mapping.insert(64, "F6".to_string());
    mapping.insert(65, "F7".to_string());
    mapping.insert(66, "F8".to_string());
    mapping.insert(67, "F9".to_string());
    mapping.insert(68, "F10".to_string());
    mapping.insert(87, "F11".to_string());
    mapping.insert(88, "F12".to_string());
    mapping.insert(91, "F13".to_string());
    mapping.insert(92, "F14".to_string());
    mapping.insert(93, "F15".to_string());

    // Number row
    mapping.insert(11, "Digit0".to_string());
    mapping.insert(2, "Digit1".to_string());
    mapping.insert(3, "Digit2".to_string());
    mapping.insert(4, "Digit3".to_string());
    mapping.insert(5, "Digit4".to_string());
    mapping.insert(6, "Digit5".to_string());
    mapping.insert(7, "Digit6".to_string());
    mapping.insert(8, "Digit7".to_string());
    mapping.insert(9, "Digit8".to_string());
    mapping.insert(10, "Digit9".to_string());

    // Letter keys A-Z (IOHook scan codes)
    mapping.insert(30, "KeyA".to_string());
    mapping.insert(48, "KeyB".to_string());
    mapping.insert(46, "KeyC".to_string());
    mapping.insert(32, "KeyD".to_string());
    mapping.insert(18, "KeyE".to_string());
    mapping.insert(33, "KeyF".to_string());
    mapping.insert(34, "KeyG".to_string());
    mapping.insert(35, "KeyH".to_string());
    mapping.insert(23, "KeyI".to_string());
    mapping.insert(36, "KeyJ".to_string());
    mapping.insert(37, "KeyK".to_string());
    mapping.insert(38, "KeyL".to_string());
    mapping.insert(50, "KeyM".to_string());
    mapping.insert(49, "KeyN".to_string());
    mapping.insert(24, "KeyO".to_string());
    mapping.insert(25, "KeyP".to_string());
    mapping.insert(16, "KeyQ".to_string());
    mapping.insert(19, "KeyR".to_string());
    mapping.insert(31, "KeyS".to_string());
    mapping.insert(20, "KeyT".to_string());
    mapping.insert(22, "KeyU".to_string());
    mapping.insert(47, "KeyV".to_string());
    mapping.insert(17, "KeyW".to_string());
    mapping.insert(45, "KeyX".to_string());
    mapping.insert(21, "KeyY".to_string());
    mapping.insert(44, "KeyZ".to_string());

    // Special keys
    mapping.insert(1, "Escape".to_string());
    mapping.insert(15, "Tab".to_string());
    mapping.insert(58, "CapsLock".to_string());
    mapping.insert(42, "ShiftLeft".to_string());
    mapping.insert(54, "ShiftRight".to_string());
    mapping.insert(29, "ControlLeft".to_string());
    mapping.insert(3613, "ControlRight".to_string());
    mapping.insert(56, "AltLeft".to_string());
    mapping.insert(3640, "AltRight".to_string());
    mapping.insert(3675, "MetaLeft".to_string()); // Windows/Command key
    mapping.insert(3676, "MetaRight".to_string()); // Windows/Command key
    mapping.insert(3677, "ContextMenu".to_string());
    mapping.insert(57, "Space".to_string());
    mapping.insert(28, "Enter".to_string());
    mapping.insert(14, "Backspace".to_string());

    // Punctuation keys
    mapping.insert(12, "Minus".to_string());
    mapping.insert(13, "Equal".to_string());
    mapping.insert(26, "BracketLeft".to_string());
    mapping.insert(27, "BracketRight".to_string());
    mapping.insert(43, "Backslash".to_string());
    mapping.insert(39, "Semicolon".to_string());
    mapping.insert(40, "Quote".to_string());
    mapping.insert(41, "Backquote".to_string());
    mapping.insert(51, "Comma".to_string());
    mapping.insert(52, "Period".to_string());
    mapping.insert(53, "Slash".to_string());

    // Arrow keys (standard IOHook codes)
    mapping.insert(57416, "ArrowUp".to_string());
    mapping.insert(57419, "ArrowLeft".to_string());
    mapping.insert(57421, "ArrowRight".to_string());
    mapping.insert(57424, "ArrowDown".to_string());

    // Arrow keys (Windows extended codes)
    mapping.insert(61000, "ArrowUp".to_string());
    mapping.insert(61003, "ArrowLeft".to_string());
    mapping.insert(61005, "ArrowRight".to_string());
    mapping.insert(61008, "ArrowDown".to_string());

    // Insert/Delete cluster (standard IOHook codes)
    mapping.insert(3666, "Insert".to_string());
    mapping.insert(3667, "Delete".to_string());
    mapping.insert(3655, "Home".to_string());
    mapping.insert(3663, "End".to_string());
    mapping.insert(3657, "PageUp".to_string());
    mapping.insert(3665, "PageDown".to_string());

    // Insert/Delete cluster (Windows extended codes)
    mapping.insert(61010, "Insert".to_string());
    mapping.insert(61011, "Delete".to_string());
    mapping.insert(60999, "Home".to_string());
    mapping.insert(61007, "End".to_string());
    mapping.insert(61001, "PageUp".to_string());
    mapping.insert(61009, "PageDown".to_string());

    // Print Screen, Scroll Lock, Pause
    mapping.insert(3639, "PrintScreen".to_string());
    mapping.insert(70, "ScrollLock".to_string());
    mapping.insert(3653, "Pause".to_string());

    // Numpad
    mapping.insert(82, "Numpad0".to_string());
    mapping.insert(79, "Numpad1".to_string());
    mapping.insert(80, "Numpad2".to_string());
    mapping.insert(81, "Numpad3".to_string());
    mapping.insert(75, "Numpad4".to_string());
    mapping.insert(76, "Numpad5".to_string());
    mapping.insert(77, "Numpad6".to_string());
    mapping.insert(71, "Numpad7".to_string());
    mapping.insert(72, "Numpad8".to_string());
    mapping.insert(73, "Numpad9".to_string());
    mapping.insert(3637, "NumpadDivide".to_string());
    mapping.insert(55, "NumpadMultiply".to_string());
    mapping.insert(74, "NumpadSubtract".to_string());
    mapping.insert(78, "NumpadAdd".to_string());
    mapping.insert(3612, "NumpadEnter".to_string());
    mapping.insert(83, "NumpadDecimal".to_string());
    mapping.insert(69, "NumLock".to_string());
    mapping.insert(3597, "NumpadEqual".to_string());

    // Special function key (macOS Fn key)
    mapping.insert(3666, "Fn".to_string());

    mapping
}
