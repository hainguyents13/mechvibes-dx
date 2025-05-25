use crate::state::config::AppConfig;
use crate::state::soundpack::SoundPack;
use rodio::Source;
use rodio::{Decoder, OutputStream, OutputStreamHandle, Sink};
use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use std::sync::Mutex;

static AUDIO_VOLUME: std::sync::OnceLock<Mutex<f32>> = std::sync::OnceLock::new();

// AudioContext owns the OutputStream and OutputStreamHandle
#[derive(Clone)]
pub struct AudioContext {
    _stream: std::sync::Arc<OutputStream>,
    stream_handle: OutputStreamHandle,
    cached_samples: std::sync::Arc<Mutex<Option<(Vec<f32>, u16, u32)>>>, // PCM, channels, sample_rate
    key_map: std::sync::Arc<Mutex<HashMap<String, Vec<[f32; 2]>>>>,
    max_voices: usize,
    // Thêm quản lý trạng thái phím và sink từng phím
    key_pressed: std::sync::Arc<Mutex<HashMap<String, bool>>>,
    key_sinks: std::sync::Arc<Mutex<HashMap<String, Sink>>>,
}

impl AudioContext {
    pub fn new() -> Self {
        let (stream, stream_handle) =
            rodio::OutputStream::try_default().expect("Failed to create audio output stream");

        // Khởi tạo với giá trị mặc định
        let context = Self {
            _stream: std::sync::Arc::new(stream),
            stream_handle,
            cached_samples: std::sync::Arc::new(Mutex::new(None)),
            key_map: std::sync::Arc::new(Mutex::new(HashMap::new())),
            max_voices: 5,
            key_pressed: std::sync::Arc::new(Mutex::new(HashMap::new())),
            key_sinks: std::sync::Arc::new(Mutex::new(HashMap::new())),
        };

        // Khởi tạo volume từ config
        let config = AppConfig::load();
        AUDIO_VOLUME.get_or_init(|| Mutex::new(config.volume)); // Load soundpack from config
        println!("🔍 Loading initial soundpack...");
        match context.try_reload_soundpack() {
            Ok(_) => println!("✅ Initial soundpack loaded successfully"),
            Err(e) => eprintln!("❌ Failed to load initial soundpack: {}", e),
        }

        context
    }

    /// Phát âm thanh cho phím theo event (keydown hoặc keyup)
    pub fn play_key_event_sound(&self, key: &str, is_keydown: bool) {
        println!(
            "🔍 Handling key event: '{}' ({})",
            key,
            if is_keydown { "down" } else { "up" }
        );

        let mut pressed = self.key_pressed.lock().unwrap();
        if is_keydown {
            if *pressed.get(key).unwrap_or(&false) {
                // Đã nhấn giữ, bỏ qua
                return;
            }
            pressed.insert(key.to_string(), true);
        } else {
            // Keyup: chỉ phát nếu phím đang nhấn
            if !*pressed.get(key).unwrap_or(&false) {
                return;
            }
            pressed.insert(key.to_string(), false);
        }
        drop(pressed); // Lấy start, duration theo event
        let key_map = self.key_map.lock().unwrap();
        let (start, duration) = match key_map.get(key) {
            Some(arr) if arr.len() == 2 => {
                println!("✅ Found mapping for key '{}'", key);
                let idx = if is_keydown { 0 } else { 1 };
                let arr = arr[idx];
                (arr[0] / 1000.0, arr[1] / 1000.0)
            }
            _ => {
                println!(
                    "❌ Available key mappings: {:?}",
                    key_map.keys().collect::<Vec<_>>()
                );
                eprintln!("❌ No mapping for key '{}' in config.json", key);
                return;
            }
        };
        // Phát từ PCM buffer đã decode sẵn
        let pcm_opt = self.cached_samples.lock().unwrap().clone();
        if let Some((samples, channels, sample_rate)) = pcm_opt {
            let start_sample = (start * sample_rate as f32 * channels as f32) as usize;
            let end_sample = ((start + duration) * sample_rate as f32 * channels as f32) as usize;
            let end_sample = end_sample.min(samples.len());
            if start_sample >= end_sample || start_sample >= samples.len() {
                eprintln!(
                    "❌ Invalid sample range for key '{}': {}..{} (max {})",
                    key,
                    start_sample,
                    end_sample,
                    samples.len()
                );
                return;
            }
            let segment_samples = samples[start_sample..end_sample].to_vec();
            let segment = rodio::buffer::SamplesBuffer::new(channels, sample_rate, segment_samples);
            if let Ok(sink) = Sink::try_new(&self.stream_handle) {
                // Lấy volume toàn cục
                let mut volume = 1.0;
                if let Some(global) = AUDIO_VOLUME.get() {
                    volume = *global.lock().unwrap();
                }
                sink.set_volume(volume * 5.0); // Tăng volume lên 5 lần
                sink.append(segment);
                let mut key_sinks = self.key_sinks.lock().unwrap();
                if key_sinks.len() >= self.max_voices {
                    if let Some((old_key, old_sink)) =
                        key_sinks.iter().next().map(|(k, s)| (k.clone(), s))
                    {
                        old_sink.stop();
                        key_sinks.remove(&old_key);
                        let mut pressed = self.key_pressed.lock().unwrap();
                        pressed.insert(old_key, false);
                    }
                }
                key_sinks.insert(
                    format!("{}-{}", key, if is_keydown { "down" } else { "up" }),
                    sink,
                );
            } else {
                eprintln!("❌ Failed to create Sink");
            }
            return;
        }
        eprintln!("❌ No cached PCM buffer available");
    } // Cập nhật và lưu âm lượng vào cấu hình
    pub fn set_volume(&self, volume: f32) {
        // Cập nhật âm lượng cho các phím hiện tại
        let key_sinks = self.key_sinks.lock().unwrap();
        for sink in key_sinks.values() {
            sink.set_volume(volume);
        }

        // Cập nhật biến global
        if let Some(global) = AUDIO_VOLUME.get() {
            let mut g = global.lock().unwrap();
            *g = volume;
        }

        // Lưu vào file cấu hình
        let mut config = AppConfig::load();
        config.volume = volume;
        let _ = config.save();
    }

    pub fn try_reload_soundpack(&self) -> Result<(), String> {
        // Kiểm tra danh sách soundpack có sẵn
        let soundpacks = std::fs::read_dir("./sounds")
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
            return Err("No soundpacks found in ./sounds directory".to_string());
        }

        // Load current config và danh sách soundpack với ID
        let mut config = AppConfig::load();
        let current_id = &config.current_soundpack;

        // Tìm và load tất cả các soundpack để lấy ID
        let mut available_packs = Vec::new();
        for path in &soundpacks {
            if let Ok(content) = std::fs::read_to_string(format!("{}/config.json", path)) {
                if let Ok(pack) = serde_json::from_str::<SoundPack>(&content) {
                    available_packs.push((path.clone(), pack));
                }
            }
        }

        if available_packs.is_empty() {
            return Err("No valid soundpacks found".to_string());
        }

        // Tìm soundpack hiện tại hoặc dùng cái đầu tiên
        let (soundpack_path, soundpack) = match available_packs
            .iter()
            .find(|(_, pack)| pack.id == *current_id)
        {
            Some((path, pack)) => (path.clone(), pack.clone()),
            None => {
                let (path, pack) = available_packs[0].clone();
                println!(
                    "⚠️ Soundpack '{}' not found, using '{}' instead",
                    current_id, pack.name
                );
                (path, pack)
            }
        };

        // Load sound file từ source field
        let sound_file_path = soundpack
            .source
            .as_ref()
            .map(|src| format!("{}/{}", soundpack_path, src.trim_start_matches("./")))
            .ok_or_else(|| "No source field in soundpack config".to_string())?;

        if !std::path::Path::new(&sound_file_path).exists() {
            return Err(format!("Sound file not found: {}", sound_file_path));
        }

        // Read and decode the audio file
        let file = File::open(&sound_file_path)
            .map_err(|e| format!("Failed to open sound file: {}", e))?;
        let mut buf = Vec::new();
        use std::io::Read;
        file.take(10_000_000) // Limit to 10MB to prevent memory issues
            .read_to_end(&mut buf)
            .map_err(|e| format!("Failed to read sound file: {}", e))?;

        let cursor = std::io::Cursor::new(buf);
        let decoder = Decoder::new(BufReader::new(cursor))
            .map_err(|e| format!("Failed to decode audio: {}", e))?;
        let sample_rate = decoder.sample_rate();
        let channels = decoder.channels();
        let samples: Vec<f32> = decoder.convert_samples().collect();
        let cached_samples = Some((samples, channels, sample_rate));

        // Luôn cập nhật config với ID hiện tại
        config.current_soundpack = soundpack.id.clone();
        config
            .save()
            .map_err(|e| format!("Failed to save config: {}", e))?;

        // Update the context
        if let Ok(mut samples) = self.cached_samples.lock() {
            *samples = cached_samples;
        }

        // Update the key mappings through the Mutex
        let keys: Vec<_> = soundpack.def.keys().collect();
        println!(
            "🔍 Loaded soundpack '{}' with {} key mappings: {:?}",
            soundpack.name,
            keys.len(),
            keys
        );

        if let Ok(mut key_map) = self.key_map.lock() {
            key_map.clear();
            key_map.extend(soundpack.def);
        } else {
            return Err("Failed to acquire lock on key_map".to_string());
        }

        // Clear any playing sounds
        if let Ok(mut sinks) = self.key_sinks.lock() {
            sinks.clear();
        }
        if let Ok(mut pressed) = self.key_pressed.lock() {
            pressed.clear();
        }

        println!(
            "✅ Loaded soundpack: {} by {}",
            soundpack.name, soundpack.author
        );
        Ok(())
    }
}
