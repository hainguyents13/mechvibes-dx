use crate::state::config::AppConfig;
use rodio::{OutputStream, OutputStreamHandle, Sink};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

static AUDIO_VOLUME: std::sync::OnceLock<Mutex<f32>> = std::sync::OnceLock::new();
static MOUSE_AUDIO_VOLUME: std::sync::OnceLock<Mutex<f32>> = std::sync::OnceLock::new();

#[derive(Clone)]
pub struct AudioContext {
    _stream: Arc<OutputStream>,
    pub(crate) stream_handle: OutputStreamHandle,
    pub(crate) keyboard_samples: Arc<Mutex<Option<(Vec<f32>, u16, u32)>>>,
    pub(crate) mouse_samples: Arc<Mutex<Option<(Vec<f32>, u16, u32)>>>,
    pub(crate) key_map: Arc<Mutex<HashMap<String, Vec<[f32; 2]>>>>,
    pub(crate) mouse_map: Arc<Mutex<HashMap<String, Vec<[f32; 2]>>>>,
    pub(crate) max_voices: usize,
    pub(crate) key_pressed: Arc<Mutex<HashMap<String, bool>>>,
    pub(crate) mouse_pressed: Arc<Mutex<HashMap<String, bool>>>,
    pub(crate) key_sinks: Arc<Mutex<HashMap<String, Sink>>>,
    pub(crate) mouse_sinks: Arc<Mutex<HashMap<String, Sink>>>,
}

// Manual PartialEq implementation for component compatibility
impl PartialEq for AudioContext {
    fn eq(&self, other: &Self) -> bool {
        // For component props, we consider AudioContext instances equal if they're the same Arc
        Arc::ptr_eq(&self._stream, &other._stream)
    }
}

impl AudioContext {
    pub fn new() -> Self {
        println!("🎵 Creating new AudioContext...");
        let (stream, stream_handle) =
            rodio::OutputStream::try_default().expect("Failed to create audio output stream");
        let context = Self {
            _stream: Arc::new(stream),
            stream_handle,
            keyboard_samples: Arc::new(Mutex::new(None)),
            mouse_samples: Arc::new(Mutex::new(None)),
            key_map: Arc::new(Mutex::new(HashMap::new())),
            mouse_map: Arc::new(Mutex::new(HashMap::new())),
            max_voices: 5,
            key_pressed: Arc::new(Mutex::new(HashMap::new())),
            mouse_pressed: Arc::new(Mutex::new(HashMap::new())),
            key_sinks: Arc::new(Mutex::new(HashMap::new())),
            mouse_sinks: Arc::new(Mutex::new(HashMap::new())),
        }; // Initialize volume from config
        let config = AppConfig::load();
        AUDIO_VOLUME.get_or_init(|| Mutex::new(config.volume));
        MOUSE_AUDIO_VOLUME.get_or_init(|| Mutex::new(config.mouse_volume));
        // Load soundpack from config with cache optimization
        println!("🔍 Loading initial soundpack from cache...");
        match super::soundpack_loader::load_soundpack(&context) {
            Ok(_) => println!("✅ Initial soundpack loaded successfully from cache"),
            Err(e) => eprintln!("❌ Failed to load initial soundpack: {}", e),
        }

        context
    }
    pub fn set_volume(&self, volume: f32) {
        // Update volume for current keys only
        let key_sinks = self.key_sinks.lock().unwrap();
        for sink in key_sinks.values() {
            sink.set_volume(volume);
        }

        // Update global variable
        if let Some(global) = AUDIO_VOLUME.get() {
            let mut g = global.lock().unwrap();
            *g = volume;
        }

        // Save to config file
        let mut config = AppConfig::load();
        config.volume = volume;
        let _ = config.save();
    }

    pub fn get_volume(&self) -> f32 {
        AUDIO_VOLUME
            .get()
            .and_then(|v| v.lock().ok())
            .map(|v| *v)
            .unwrap_or(1.0)
    }

    pub fn set_mouse_volume(&self, volume: f32) {
        // Update volume for current mouse events only
        let mouse_sinks = self.mouse_sinks.lock().unwrap();
        for sink in mouse_sinks.values() {
            sink.set_volume(volume);
        }

        // Update global variable
        if let Some(global) = MOUSE_AUDIO_VOLUME.get() {
            let mut g = global.lock().unwrap();
            *g = volume;
        }

        // Save to config file
        let mut config = AppConfig::load();
        config.mouse_volume = volume;
        let _ = config.save();
    }

    pub fn get_mouse_volume(&self) -> f32 {
        MOUSE_AUDIO_VOLUME
            .get()
            .and_then(|v| v.lock().ok())
            .map(|v| *v)
            .unwrap_or(1.0)
    }
}
