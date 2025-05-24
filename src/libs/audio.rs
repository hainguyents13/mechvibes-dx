use rand::prelude::*;
use rodio::{Decoder, OutputStream, OutputStreamHandle, Sink};
use serde_json;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use std::sync::Mutex;
use std::vec;

// AudioContext owns the OutputStream and OutputStreamHandle
#[derive(Clone)]
pub struct AudioContext {
    _stream: std::sync::Arc<OutputStream>, // Keep the stream alive
    stream_handle: OutputStreamHandle,
    cached_file: std::sync::Arc<Mutex<Option<Vec<u8>>>>, // cache for single mode
}

impl AudioContext {
    pub fn new() -> Self {
        let (stream, stream_handle) =
            rodio::OutputStream::try_default().expect("Failed to create audio output stream");
        Self {
            _stream: std::sync::Arc::new(stream),
            stream_handle,
            cached_file: std::sync::Arc::new(Mutex::new(None)),
        }
    }

    pub fn play_random_sound(&self) {
        println!("🎵 Starting to play random sound...");
        let define_type = "single"; // or "single"
        let sounds = vec!["sounds/ping2.mp3"];
        let file = "sounds/oreo.ogg";
        let config_path = Path::new("d:/mechvibes-dx/sounds/config.json");
        let mut time_periods: Vec<(f32, f32)> = vec![];
        if let Ok(file) = File::open(config_path) {
            if let Ok(json) = serde_json::from_reader::<_, serde_json::Value>(file) {
                // Map config: each key maps to [start_ms, duration_ms]
                for (_key, value) in json.as_object().unwrap().iter() {
                    if let Some(arr) = value.as_array() {
                        if arr.len() == 2 {
                            let start = arr[0].as_f64().unwrap_or(0.0) as f32 / 1000.0;
                            let duration = arr[1].as_f64().unwrap_or(0.0) as f32 / 1000.0;
                            time_periods.push((start, duration));
                        }
                    }
                }
            }
        }
        let mut rng = rand::rng();
        match define_type {
            "multiple" => {
                if let Some(file_path) = sounds.choose(&mut rng) {
                    let file_path = file_path.to_string();
                    let path_options = vec![
                        Path::new("d:/mechvibes-dx").join(&file_path),
                        Path::new(&file_path).to_path_buf(),
                    ];
                    for path in path_options {
                        println!("🔍 Trying path: {}", path.display());
                        if let Ok(file) = File::open(&path) {
                            println!("✅ File opened successfully: {}", path.display());
                            if let Ok(source) = Decoder::new(BufReader::new(file)) {
                                println!("✅ Decoder created successfully");
                                if let Ok(sink) = Sink::try_new(&self.stream_handle) {
                                    sink.set_volume(1.0);
                                    sink.append(source);
                                    sink.detach();
                                    println!("🎶 Sound playing now!");
                                } else {
                                    eprintln!("❌ Failed to create Sink");
                                }
                                return;
                            }
                        }
                    }
                    eprintln!(
                        "❌ Failed to play sound '{}' . All path attempts failed.",
                        file_path
                    );
                }
            }
            "single" => {
                if time_periods.is_empty() {
                    eprintln!("❌ No time periods defined in config.json");
                    return;
                }
                let (start, duration) = *time_periods.choose(&mut rng).unwrap();
                // --- Caching logic ---
                let buf_opt = {
                    let mut cache = self.cached_file.lock().unwrap();
                    if cache.is_none() {
                        if let Ok(mut f) = File::open(file) {
                            let mut buf = Vec::new();
                            use std::io::Read;
                            if f.read_to_end(&mut buf).is_ok() {
                                *cache = Some(buf);
                            }
                        }
                    }
                    cache.clone()
                };
                if let Some(buf) = buf_opt {
                    use std::io::Cursor;
                    let cursor = Cursor::new(buf);
                    if let Ok(source) = Decoder::new(BufReader::new(cursor)) {
                        use rodio::Source;
                        let segment = source
                            .skip_duration(std::time::Duration::from_secs_f32(start))
                            .take_duration(std::time::Duration::from_secs_f32(duration));
                        if let Ok(sink) = Sink::try_new(&self.stream_handle) {
                            sink.set_volume(1.0);
                            sink.append(segment);
                            sink.detach();
                            println!(
                                "🎶 Playing segment: {}s ({}s duration, cached)",
                                start, duration
                            );
                        } else {
                            eprintln!("❌ Failed to create Sink");
                        }
                        return;
                    }
                }
                // fallback: try reading from disk if cache failed
                let path_options = vec![
                    Path::new("d:/mechvibes-dx").join(file),
                    Path::new(file).to_path_buf(),
                ];
                for path in path_options {
                    println!("🔍 Trying path: {}", path.display());
                    if let Ok(file) = File::open(&path) {
                        println!("✅ File opened successfully: {}", path.display());
                        if let Ok(source) = Decoder::new(BufReader::new(file)) {
                            println!("✅ Decoder created successfully");
                            use rodio::Source;
                            let segment = source
                                .skip_duration(std::time::Duration::from_secs_f32(start))
                                .take_duration(std::time::Duration::from_secs_f32(duration));
                            if let Ok(sink) = Sink::try_new(&self.stream_handle) {
                                sink.set_volume(1.0);
                                sink.append(segment);
                                sink.detach();
                                println!("🎶 Playing segment: {}s ({}s duration)", start, duration);
                            } else {
                                eprintln!("❌ Failed to create Sink");
                            }
                            return;
                        }
                    }
                }
                eprintln!(
                    "❌ Failed to play file '{}' for segment. All path attempts failed.",
                    file
                );
            }
            _ => {
                eprintln!("❌ Unknown define_type: {}", define_type);
            }
        }
    }
}
