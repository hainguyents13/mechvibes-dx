use crate::state::config::AppConfig;
use crate::libs::device_manager::DeviceManager;
use rodio::{ OutputStream, OutputStreamHandle, Sink };
use std::collections::HashMap;
use std::sync::{ Arc, Mutex };
use std::time::Instant;

static AUDIO_VOLUME: std::sync::OnceLock<Mutex<f32>> = std::sync::OnceLock::new();
static MOUSE_AUDIO_VOLUME: std::sync::OnceLock<Mutex<f32>> = std::sync::OnceLock::new();

// Cached config flags to avoid loading config on every keypress
static ENABLE_SOUND: std::sync::OnceLock<Mutex<bool>> = std::sync::OnceLock::new();
static ENABLE_KEYBOARD_SOUND: std::sync::OnceLock<Mutex<bool>> = std::sync::OnceLock::new();
static ENABLE_MOUSE_SOUND: std::sync::OnceLock<Mutex<bool>> = std::sync::OnceLock::new();

// (samples, channels, sample_rate) behind an Arc so cloning the pointer for
// hot-path playback doesn't clone the underlying PCM buffer.
type PcmBuffer = Arc<Mutex<Option<(Arc<Vec<f32>>, u16, u32)>>>;

#[derive(Clone)]
#[allow(dead_code)]
pub struct AudioContext {
    _stream: Arc<OutputStream>,
    pub(crate) stream_handle: OutputStreamHandle,
    pub(crate) keyboard_samples: PcmBuffer,
    pub(crate) mouse_samples: PcmBuffer,
    // Original (pre-resample) decoded samples, kept so a future device switch
    // can re-resample to the new device rate without re-reading from disk.
    pub(crate) keyboard_samples_original: PcmBuffer,
    pub(crate) mouse_samples_original: PcmBuffer,
    pub(crate) key_map: Arc<Mutex<HashMap<String, Vec<[f32; 2]>>>>,
    pub(crate) mouse_map: Arc<Mutex<HashMap<String, Vec<[f32; 2]>>>>,
    pub(crate) max_voices: usize,
    pub(crate) key_pressed: Arc<Mutex<HashMap<String, bool>>>,
    pub(crate) mouse_pressed: Arc<Mutex<HashMap<String, bool>>>,
    pub(crate) key_sinks: Arc<Mutex<Vec<Sink>>>,
    pub(crate) mouse_sinks: Arc<Mutex<Vec<Sink>>>,
    pub(crate) device_manager: DeviceManager,
    // Output device sample rate, probed once when the stream opens (Phase 5
    // Fix 3) so soundpack loads don't re-probe the device every time - on
    // Linux/ALSA that enumeration can briefly interrupt other audio.
    // `None` means the probe failed; callers should skip resampling.
    pub(crate) cached_device_rate: Arc<Mutex<Option<u32>>>,
    // Timing tracking for rapid event detection
    pub(crate) last_keyboard_sound_time: Arc<Mutex<Option<Instant>>>,
    pub(crate) last_mouse_sound_time: Arc<Mutex<Option<Instant>>>,
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
        // Initialize device manager
        let device_manager = DeviceManager::new();
        let config = AppConfig::load();

        // Use selected device from config or fall back to default
        // Note: On Linux, this will enumerate devices and may briefly interrupt audio playback
        let (stream, stream_handle) = match &config.selected_audio_device {
            Some(device_id) => {
                println!("🔊 [AudioContext] Attempting to use selected device: {}", device_id);
                match device_manager.get_output_device_by_id(device_id) {
                    Ok(Some(device)) => {
                        match rodio::OutputStream::try_from_device(&device) {
                            Ok((stream, handle)) => {
                                println!("✅ [AudioContext] Using selected device: {}", device_id);
                                (stream, handle)
                            }
                            Err(e) => {
                                eprintln!(
                                    "❌ Failed to create stream from selected device {}: {}",
                                    device_id,
                                    e
                                );
                                eprintln!("🔄 Falling back to default device...");
                                rodio::OutputStream
                                    ::try_default()
                                    .expect("Failed to create default audio output stream")
                            }
                        }
                    }
                    Ok(None) => {
                        eprintln!("❌ Selected audio device {} not found, using default", device_id);
                        rodio::OutputStream
                            ::try_default()
                            .expect("Failed to create default audio output stream")
                    }
                    Err(e) => {
                        eprintln!("❌ Error accessing selected device {}: {}", device_id, e);
                        rodio::OutputStream
                            ::try_default()
                            .expect("Failed to create default audio output stream")
                    }
                }
            }
            None => {
                println!("🔊 [AudioContext] Using system default audio device");
                rodio::OutputStream
                    ::try_default()
                    .expect("Failed to create default audio output stream")
            }
        };

        let context = Self {
            _stream: Arc::new(stream),
            stream_handle,
            keyboard_samples: Arc::new(Mutex::new(None)),
            mouse_samples: Arc::new(Mutex::new(None)),
            keyboard_samples_original: Arc::new(Mutex::new(None)),
            mouse_samples_original: Arc::new(Mutex::new(None)),
            key_map: Arc::new(Mutex::new(HashMap::new())),
            mouse_map: Arc::new(Mutex::new(HashMap::new())),
            max_voices: 32, // Increased max voices to reduce audio interruptions
            key_pressed: Arc::new(Mutex::new(HashMap::new())),
            mouse_pressed: Arc::new(Mutex::new(HashMap::new())),
            key_sinks: Arc::new(Mutex::new(Vec::new())),
            mouse_sinks: Arc::new(Mutex::new(Vec::new())),
            device_manager: device_manager.clone(),
            cached_device_rate: Arc::new(Mutex::new(None)),
            last_keyboard_sound_time: Arc::new(Mutex::new(None)),
            last_mouse_sound_time: Arc::new(Mutex::new(None)),
        };

        // Probe the device rate once now (the stream we just opened already
        // touched the device, so one more query here is cheap/safe) and
        // cache it so soundpack loads never re-probe.
        let device_rate = device_manager.get_current_output_sample_rate();
        if let Ok(mut cached) = context.cached_device_rate.lock() {
            *cached = device_rate;
        }

        // Initialize volume and sound flags from config
        let config = AppConfig::load();
        AUDIO_VOLUME.get_or_init(|| Mutex::new(config.volume));
        MOUSE_AUDIO_VOLUME.get_or_init(|| Mutex::new(config.mouse_volume));
        ENABLE_SOUND.get_or_init(|| Mutex::new(config.enable_sound));
        ENABLE_KEYBOARD_SOUND.get_or_init(|| Mutex::new(config.enable_keyboard_sound));
        ENABLE_MOUSE_SOUND.get_or_init(|| Mutex::new(config.enable_mouse_sound));

        // Load soundpack from config
        match super::soundpack_loader::load_soundpack(&context) {
            Ok(_) => {}
            Err(e) => eprintln!("❌ Failed to load initial soundpack: {}", e),
        }

        context
    }
    pub fn set_volume(&self, volume: f32) {
        // Update volume for current keys only
        let key_sinks = self.key_sinks.lock().unwrap();
        for sink in key_sinks.iter() {
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
        AUDIO_VOLUME.get()
            .and_then(|v| v.lock().ok())
            .map(|v| *v)
            .unwrap_or(1.0)
    }

    pub fn set_mouse_volume(&self, volume: f32) {
        // Update volume for current mouse events only
        let mouse_sinks = self.mouse_sinks.lock().unwrap();
        for sink in mouse_sinks.iter() {
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
        MOUSE_AUDIO_VOLUME.get()
            .and_then(|v| v.lock().ok())
            .map(|v| *v)
            .unwrap_or(1.0)
    }

    // Cached config flag getters (no file I/O, safe to call in hot path)
    pub fn is_sound_enabled(&self) -> bool {
        ENABLE_SOUND.get()
            .and_then(|v| v.lock().ok())
            .map(|v| *v)
            .unwrap_or(true)
    }

    pub fn is_keyboard_sound_enabled(&self) -> bool {
        ENABLE_KEYBOARD_SOUND.get()
            .and_then(|v| v.lock().ok())
            .map(|v| *v)
            .unwrap_or(true)
    }

    pub fn is_mouse_sound_enabled(&self) -> bool {
        ENABLE_MOUSE_SOUND.get()
            .and_then(|v| v.lock().ok())
            .map(|v| *v)
            .unwrap_or(true)
    }

    // Cached config flag setters (update cache + save to file)
    pub fn set_sound_enabled(&self, enabled: bool) {
        if let Some(global) = ENABLE_SOUND.get() {
            let mut g = global.lock().unwrap();
            *g = enabled;
        }

        // Save to config file
        let mut config = AppConfig::load();
        config.enable_sound = enabled;
        let _ = config.save();
    }

    pub fn set_keyboard_sound_enabled(&self, enabled: bool) {
        if let Some(global) = ENABLE_KEYBOARD_SOUND.get() {
            let mut g = global.lock().unwrap();
            *g = enabled;
        }

        // Save to config file
        let mut config = AppConfig::load();
        config.enable_keyboard_sound = enabled;
        let _ = config.save();
    }

    pub fn set_mouse_sound_enabled(&self, enabled: bool) {
        if let Some(global) = ENABLE_MOUSE_SOUND.get() {
            let mut g = global.lock().unwrap();
            *g = enabled;
        }

        // Save to config file
        let mut config = AppConfig::load();
        config.enable_mouse_sound = enabled;
        let _ = config.save();
    }
}
