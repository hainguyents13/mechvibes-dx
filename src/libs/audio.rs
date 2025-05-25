use rodio::Source;
use rodio::{Decoder, OutputStream, OutputStreamHandle, Sink};
use serde_json;
use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use std::sync::Mutex;

// AudioContext owns the OutputStream and OutputStreamHandle
#[derive(Clone)]
pub struct AudioContext {
    _stream: std::sync::Arc<OutputStream>,
    stream_handle: OutputStreamHandle,
    cached_samples: std::sync::Arc<Mutex<Option<(Vec<f32>, u16, u32)>>>, // PCM, channels, sample_rate
    key_map: std::sync::Arc<HashMap<String, Vec<[f32; 2]>>>,
    max_voices: usize,
    // Thêm quản lý trạng thái phím và sink từng phím
    key_pressed: std::sync::Arc<Mutex<HashMap<String, bool>>>,
    key_sinks: std::sync::Arc<Mutex<HashMap<String, Sink>>>,
}

impl AudioContext {
    pub fn new() -> Self {
        let (stream, stream_handle) =
            rodio::OutputStream::try_default().expect("Failed to create audio output stream");
        // Cache sound file
        let mut cached_samples = None;
        if let Ok(mut f) = File::open("sounds/oreo.ogg") {
            let mut buf = Vec::new();
            use std::io::Read;
            if f.read_to_end(&mut buf).is_ok() {
                let cursor = std::io::Cursor::new(buf);
                if let Ok(decoder) = Decoder::new(BufReader::new(cursor)) {
                    let sample_rate = decoder.sample_rate();
                    let channels = decoder.channels();
                    let samples: Vec<f32> = decoder.convert_samples().collect();
                    cached_samples = Some((samples, channels, sample_rate));
                }
            }
        }
        // Cache key map
        let mut key_map = HashMap::new();
        let config_path = Path::new("d:/mechvibes-dx/sounds/config.json");
        if let Ok(file_cfg) = File::open(config_path) {
            if let Ok(json) = serde_json::from_reader::<_, serde_json::Value>(file_cfg) {
                if let Some(obj) = json.as_object() {
                    for (key, value) in obj.iter() {
                        if let Some(arr) = value.as_array() {
                            let mut v = Vec::new();
                            for item in arr {
                                if let Some(pair) = item.as_array() {
                                    if pair.len() == 2 {
                                        let start = pair[0].as_f64().unwrap_or(0.0) as f32;
                                        let duration = pair[1].as_f64().unwrap_or(0.0) as f32;
                                        v.push([start, duration]);
                                    }
                                }
                            }
                            if !v.is_empty() {
                                key_map.insert(key.clone(), v);
                            }
                        }
                    }
                }
            }
        }
        AUDIO_VOLUME.get_or_init(|| Mutex::new(1.0));
        Self {
            _stream: std::sync::Arc::new(stream),
            stream_handle,
            cached_samples: std::sync::Arc::new(Mutex::new(cached_samples)),
            key_map: std::sync::Arc::new(key_map),
            max_voices: 5,
            key_pressed: std::sync::Arc::new(Mutex::new(HashMap::new())),
            key_sinks: std::sync::Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Phát âm thanh cho phím theo event (keydown hoặc keyup)
    pub fn play_key_event_sound(&self, key: &str, is_keydown: bool) {
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
        drop(pressed);
        // Lấy start, duration theo event
        let (start, duration) = match self.key_map.get(key) {
            Some(arr) if arr.len() == 2 => {
                let idx = if is_keydown { 0 } else { 1 };
                let arr = arr[idx];
                (arr[0] / 1000.0, arr[1] / 1000.0)
            }
            _ => {
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
    }

    // Thêm vào AudioContext:
    pub fn set_volume(&self, volume: f32) {
        let key_sinks = self.key_sinks.lock().unwrap();
        for sink in key_sinks.values() {
            sink.set_volume(volume);
        }
        if let Some(global) = AUDIO_VOLUME.get() {
            let mut g = global.lock().unwrap();
            *g = volume;
        }
    }
}

use std::sync::OnceLock;
static AUDIO_VOLUME: OnceLock<Mutex<f32>> = OnceLock::new();
