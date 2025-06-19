use serde::{ Deserialize, Serialize };
use crate::state::config::AppConfig;
use std::collections::HashMap;
use std::sync::{ Arc, Mutex };
use std::thread;
use rodio::{ Decoder, Sink, Source };
use std::fs::File;
use std::io::BufReader;

// Simple global state for playing sounds
static GLOBAL_AMBIANCE_SINKS: std::sync::OnceLock<
    Arc<Mutex<HashMap<String, Sink>>>
> = std::sync::OnceLock::new();

// Global ambiance player state
static GLOBAL_AMBIANCE_PLAYER_STATE: std::sync::OnceLock<Arc<Mutex<AmbiancePlayerState>>> = std::sync::OnceLock::new();

// Initialize global ambiance player
pub fn initialize_global_ambiance_player() {
    let _sinks_ref = GLOBAL_AMBIANCE_SINKS.get_or_init(|| { Arc::new(Mutex::new(HashMap::new())) });

    // Initialize global state
    let _state_ref = GLOBAL_AMBIANCE_PLAYER_STATE.get_or_init(|| {
        Arc::new(Mutex::new(AmbiancePlayerState::initialize()))
    });

    println!("🎵 Global ambiance player initialized");
}

// Initialize global ambiance player state (call from main component)
pub fn initialize_global_ambiance_player_state() {
    let state_ref = GLOBAL_AMBIANCE_PLAYER_STATE.get_or_init(|| {
        Arc::new(Mutex::new(AmbiancePlayerState::initialize()))
    });

    // Force initialization
    let _state_lock = state_ref.lock().unwrap();
}

// Update global ambiance player state
pub fn update_global_ambiance_player_state<F>(f: F) where F: FnOnce(&mut AmbiancePlayerState) {
    if let Some(state_ref) = GLOBAL_AMBIANCE_PLAYER_STATE.get() {
        if let Ok(mut state_lock) = state_ref.lock() {
            f(&mut *state_lock);
        }
    }
}

// Get a copy of global ambiance player state
pub fn get_global_ambiance_player_state_copy() -> Option<AmbiancePlayerState> {
    GLOBAL_AMBIANCE_PLAYER_STATE.get().and_then(|state_ref| {
        state_ref
            .lock()
            .ok()
            .map(|state| state.clone())
    })
}

// Play a sound
pub fn play_ambiance_sound(sound_id: String, audio_url: String, volume: f32) -> Result<(), String> {
    let sinks_ref = GLOBAL_AMBIANCE_SINKS.get().ok_or("Ambiance player not initialized")?;
    let mut sinks_lock = sinks_ref.lock().unwrap();

    // Stop existing sound if playing
    if let Some(sink) = sinks_lock.remove(&sound_id) {
        sink.stop();
    }

    // Create new audio stream for this sound
    thread::spawn(move || {
        let result = (|| -> Result<(), String> {
            let (_stream, stream_handle) = rodio::OutputStream
                ::try_default()
                .map_err(|e| format!("Failed to create audio output stream: {}", e))?;

            let sink = Sink::try_new(&stream_handle).map_err(|e|
                format!("Failed to create audio sink: {}", e)
            )?;

            // Load audio file from local path
            let audio_path = audio_url.replace("assets/", "");
            let full_path = format!("assets/{}", audio_path);

            let file = File::open(&full_path).map_err(|e|
                format!("Failed to open audio file {}: {}", full_path, e)
            )?;
            let buf_reader = BufReader::new(file);

            let decoder = Decoder::new(buf_reader).map_err(|e|
                format!("Failed to decode audio: {}", e)
            )?;

            sink.set_volume(volume.clamp(0.0, 1.0));
            sink.append(decoder.repeat_infinite());

            // Store sink in global map
            if let Some(sinks_ref) = GLOBAL_AMBIANCE_SINKS.get() {
                let mut sinks_lock = sinks_ref.lock().unwrap();
                sinks_lock.insert(sound_id.clone(), sink);
            }

            println!("🎵 Started playing ambiance sound: {}", sound_id);

            // Keep the stream alive
            loop {
                thread::sleep(std::time::Duration::from_secs(1));
                // Check if sink is still in the map (not stopped)
                if let Some(sinks_ref) = GLOBAL_AMBIANCE_SINKS.get() {
                    let sinks_lock = sinks_ref.lock().unwrap();
                    if !sinks_lock.contains_key(&sound_id) {
                        break;
                    }
                } else {
                    break;
                }
            }

            Ok(())
        })();

        if let Err(e) = result {
            eprintln!("❌ Failed to play ambiance sound {}: {}", sound_id, e);
        }
    });

    Ok(())
}

// Stop a sound
pub fn stop_ambiance_sound(sound_id: &str) -> Result<(), String> {
    let sinks_ref = GLOBAL_AMBIANCE_SINKS.get().ok_or("Ambiance player not initialized")?;
    let mut sinks_lock = sinks_ref.lock().unwrap();

    if let Some(sink) = sinks_lock.remove(sound_id) {
        sink.stop();
        println!("🔇 Stopped ambiance sound: {}", sound_id);
    }

    Ok(())
}

// Pause all sounds
pub fn pause_all_ambiance_sounds() -> Result<(), String> {
    let sinks_ref = GLOBAL_AMBIANCE_SINKS.get().ok_or("Ambiance player not initialized")?;
    let sinks_lock = sinks_ref.lock().unwrap();

    for sink in sinks_lock.values() {
        sink.pause();
    }
    println!("⏸️ Paused all ambiance sounds");

    Ok(())
}

// Resume all sounds
pub fn resume_all_ambiance_sounds() -> Result<(), String> {
    let sinks_ref = GLOBAL_AMBIANCE_SINKS.get().ok_or("Ambiance player not initialized")?;
    let sinks_lock = sinks_ref.lock().unwrap();

    for sink in sinks_lock.values() {
        sink.play();
    }
    println!("▶️ Resumed all ambiance sounds");

    Ok(())
}

// Set sound volume
pub fn set_ambiance_sound_volume(sound_id: &str, volume: f32) -> Result<(), String> {
    let sinks_ref = GLOBAL_AMBIANCE_SINKS.get().ok_or("Ambiance player not initialized")?;
    let sinks_lock = sinks_ref.lock().unwrap();

    if let Some(sink) = sinks_lock.get(sound_id) {
        sink.set_volume(volume.clamp(0.0, 1.0));
    }

    Ok(())
}

// Set global volume for all sounds
pub fn set_global_ambiance_volume(volume: f32) -> Result<(), String> {
    let sinks_ref = GLOBAL_AMBIANCE_SINKS.get().ok_or("Ambiance player not initialized")?;
    let sinks_lock = sinks_ref.lock().unwrap();

    let clamped_volume = volume.clamp(0.0, 1.0);
    for sink in sinks_lock.values() {
        sink.set_volume(clamped_volume);
    }

    Ok(())
}

// Set global mute for all sounds
pub fn set_global_ambiance_mute(muted: bool) -> Result<(), String> {
    let sinks_ref = GLOBAL_AMBIANCE_SINKS.get().ok_or("Ambiance player not initialized")?;
    let sinks_lock = sinks_ref.lock().unwrap();

    for sink in sinks_lock.values() {
        if muted {
            sink.pause();
        } else {
            sink.play();
        }
    }

    Ok(())
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AmbianceSound {
    pub id: String,
    pub name: String,
    pub description: String,
    pub audio_url: String,
    pub icon: String, // Icon name for UI display
}

#[derive(Debug, Clone)]
pub struct AmbiancePlayerState {
    pub sounds: Vec<AmbianceSound>,
    pub active_sounds: HashMap<String, f32>, // sound_id -> individual volume (0.0 to 1.0)
    pub global_volume: f32, // 0.0 to 1.0 - global multiplier for all sounds
    pub is_muted: bool,
    pub is_playing: bool, // Global play/pause state for all ambiance sounds
}

impl Default for AmbiancePlayerState {
    fn default() -> Self {
        Self {
            sounds: Self::get_builtin_sounds(),
            active_sounds: HashMap::new(),
            global_volume: 0.5,
            is_muted: false,
            is_playing: false, // Default to paused, user needs to click play to start
        }
    }
}

impl AmbiancePlayerState {
    pub fn new() -> Self {
        Self::default()
    }

    /// Initialize from config
    pub fn initialize() -> Self {
        let config = AppConfig::load();

        Self {
            sounds: Self::get_builtin_sounds(),
            active_sounds: config.ambiance_active_sounds.clone(),
            global_volume: config.ambiance_global_volume,
            is_muted: config.ambiance_is_muted,
            is_playing: false, // Always start paused, don't persist play state
        }
    }

    /// Save current state to config
    pub fn save_config(&self) -> Result<(), String> {
        let mut config = AppConfig::load();
        config.ambiance_active_sounds = self.active_sounds.clone();
        config.ambiance_global_volume = self.global_volume;
        config.ambiance_is_muted = self.is_muted;
        // Don't save is_playing - always start paused
        config.save()
    }

    /// Get built-in ambiance sounds (using local assets)
    pub fn get_builtin_sounds() -> Vec<AmbianceSound> {
        vec![
            AmbianceSound {
                id: "rain".to_string(),
                name: "Rain".to_string(),
                description: "Gentle rainfall".to_string(),
                audio_url: "assets/sounds/rain.mp3".to_string(),
                icon: "cloud-rain".to_string(),
            },
            AmbianceSound {
                id: "forest".to_string(),
                name: "Forest".to_string(),
                description: "Birds chirping in the forest".to_string(),
                audio_url: "assets/sounds/forest.mp3".to_string(),
                icon: "tree-pine".to_string(),
            },
            AmbianceSound {
                id: "ocean".to_string(),
                name: "Ocean Waves".to_string(),
                description: "Calming ocean waves".to_string(),
                audio_url: "assets/sounds/ocean.mp3".to_string(),
                icon: "waves".to_string(),
            },
            AmbianceSound {
                id: "thunderstorm".to_string(),
                name: "Thunderstorm".to_string(),
                description: "Thunder and heavy rain".to_string(),
                audio_url: "assets/sounds/thunderstorm.mp3".to_string(),
                icon: "zap".to_string(),
            },
            AmbianceSound {
                id: "fire".to_string(),
                name: "Campfire".to_string(),
                description: "Crackling campfire".to_string(),
                audio_url: "assets/sounds/fire.mp3".to_string(),
                icon: "flame".to_string(),
            },
            AmbianceSound {
                id: "wind".to_string(),
                name: "Wind".to_string(),
                description: "Gentle wind through trees".to_string(),
                audio_url: "assets/sounds/wind.mp3".to_string(),
                icon: "wind".to_string(),
            },
            AmbianceSound {
                id: "stream".to_string(),
                name: "Stream".to_string(),
                description: "Babbling brook".to_string(),
                audio_url: "assets/sounds/stream.mp3".to_string(),
                icon: "waves".to_string(),
            },
            AmbianceSound {
                id: "river".to_string(),
                name: "River".to_string(),
                description: "Flowing river water".to_string(),
                audio_url: "assets/sounds/river.mp3".to_string(),
                icon: "waves".to_string(),
            },
            AmbianceSound {
                id: "cricket".to_string(),
                name: "Night Crickets".to_string(),
                description: "Crickets and night sounds".to_string(),
                audio_url: "assets/sounds/cricket.mp3".to_string(),
                icon: "moon".to_string(),
            },
            AmbianceSound {
                id: "chatter".to_string(),
                name: "Coffee Shop".to_string(),
                description: "Ambient coffee shop chatter".to_string(),
                audio_url: "assets/sounds/chatter.mp3".to_string(),
                icon: "coffee".to_string(),
            }
        ]
    }

    /// Get sound by ID
    pub fn get_sound_by_id(&self, sound_id: &str) -> Option<&AmbianceSound> {
        self.sounds.iter().find(|sound| sound.id == sound_id)
    }

    /// Check if a sound is currently active (playing)
    pub fn is_sound_active(&self, sound_id: &str) -> bool {
        self.active_sounds.contains_key(sound_id)
    }

    /// Get volume for a specific sound
    pub fn get_sound_volume(&self, sound_id: &str) -> f32 {
        self.active_sounds.get(sound_id).copied().unwrap_or(0.5)
    }

    /// Toggle a sound on/off
    pub fn toggle_sound(&mut self, sound_id: String) {
        if self.active_sounds.contains_key(&sound_id) {
            // Stop the sound
            self.active_sounds.remove(&sound_id);
            let _ = stop_ambiance_sound(&sound_id);
        } else {
            // Start the sound
            self.active_sounds.insert(sound_id.clone(), 0.5); // Default volume 50%
            if self.is_playing && !self.is_muted {
                if let Some(sound) = self.get_sound_by_id(&sound_id) {
                    let effective_volume = 0.5 * self.global_volume;
                    let _ = play_ambiance_sound(
                        sound_id.clone(),
                        sound.audio_url.clone(),
                        effective_volume
                    );
                }
            }
        }
        let _ = self.save_config();
    }

    /// Set volume for a specific sound
    pub fn set_sound_volume(&mut self, sound_id: String, volume: f32) {
        let clamped_volume = volume.clamp(0.0, 1.0);
        if self.active_sounds.contains_key(&sound_id) {
            self.active_sounds.insert(sound_id.clone(), clamped_volume);

            // Update audio player volume if sound is playing
            if self.is_playing && !self.is_muted {
                let effective_volume = clamped_volume * self.global_volume;
                let _ = set_ambiance_sound_volume(&sound_id, effective_volume);
            }

            let _ = self.save_config();
        }
    }
    /// Set global volume (0.0 to 1.0) - affects all active sounds
    pub fn set_global_volume(&mut self, volume: f32) {
        self.global_volume = volume.clamp(0.0, 1.0);

        // Update audio player global volume
        let _ = set_global_ambiance_volume(self.global_volume);

        let _ = self.save_config();
    }

    /// Toggle global mute
    pub fn toggle_mute(&mut self) {
        self.is_muted = !self.is_muted;

        // Update audio player mute state
        let _ = set_global_ambiance_mute(self.is_muted);

        let _ = self.save_config();
    }

    /// Toggle global play/pause state
    pub fn toggle_play_pause(&mut self) {
        self.is_playing = !self.is_playing;

        if self.is_playing {
            // Start playing all active sounds
            self.start_all_active_sounds();
            let _ = resume_all_ambiance_sounds();
        } else {
            // Pause all sounds
            let _ = pause_all_ambiance_sounds();
        }

        // Don't save to config - play state is not persistent
    }

    /// Start all active sounds when play is pressed
    fn start_all_active_sounds(&self) {
        if !self.is_playing || self.is_muted {
            return;
        }

        for (sound_id, &volume) in &self.active_sounds {
            if let Some(sound) = self.get_sound_by_id(sound_id) {
                let effective_volume = volume * self.global_volume;
                let _ = play_ambiance_sound(
                    sound_id.clone(),
                    sound.audio_url.clone(),
                    effective_volume
                );
            }
        }
    }
}
