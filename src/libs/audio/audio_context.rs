use crate::state::config::AppConfig;
use std::sync::Mutex;

use super::engine::{ engine_handle, AudioCommand, AudioEngineHandle };

static AUDIO_VOLUME: std::sync::OnceLock<Mutex<f32>> = std::sync::OnceLock::new();
static MOUSE_AUDIO_VOLUME: std::sync::OnceLock<Mutex<f32>> = std::sync::OnceLock::new();

// Cached config flags to avoid loading config on every keypress
static ENABLE_SOUND: std::sync::OnceLock<Mutex<bool>> = std::sync::OnceLock::new();
static ENABLE_KEYBOARD_SOUND: std::sync::OnceLock<Mutex<bool>> = std::sync::OnceLock::new();
static ENABLE_MOUSE_SOUND: std::sync::OnceLock<Mutex<bool>> = std::sync::OnceLock::new();

/// Loads the newest config from disk, applies `mutate`, and saves only if
/// `mutate` reports an actual change.
///
/// Two invariants this exists to hold:
///
/// 1. **Never persist a stale struct.** The whole `AppConfig` is rewritten on
///    every save, so mutating a copy that was read earlier silently reverts any
///    field another path changed in between. Loading immediately before the
///    write keeps that window as small as possible.
/// 2. **Never save when nothing changed.** UI mount effects re-assert their
///    current value on render, and unconditional saves turned that into a
///    continuous rewrite loop that raced the mute flag.
fn persist_if_changed(mutate: impl FnOnce(&mut AppConfig) -> bool) {
    let mut config = AppConfig::load();
    if !mutate(&mut config) {
        return;
    }
    config.last_updated = chrono::Utc::now();
    if let Err(e) = config.save() {
        eprintln!("❌ [AudioContext] Failed to save config: {}", e);
    }
}

/// Updates the cached `enable_sound` flag without persisting or notifying the
/// engine.
///
/// Used by the engine's Ctrl+Alt+M handler, which already wrote the config and
/// moved its own state: without this the UI-side cache would keep the previous
/// value, and the tray toggle (which derives its next value from
/// `is_sound_enabled`) would need two clicks to take effect after a hotkey.
pub(super) fn sync_sound_enabled_cache(enabled: bool) {
    if let Some(global) = ENABLE_SOUND.get() {
        if let Ok(mut guard) = global.lock() {
            *guard = enabled;
        }
    }
}

/// Thin facade over the audio engine thread (see `engine.rs`). Playback,
/// device switching and soundpack loading all live in the engine's owned
/// state now - this struct only forwards `AudioCommand`s through the
/// engine handle and keeps the cached volume/enabled flags that hot paths
/// (e.g. `is_sound_enabled`) read without going through the channel.
///
/// Kept as a distinct type (rather than exposing `AudioEngineHandle`
/// directly) so the ~8 `use_context::<Arc<AudioContext>>()` call sites
/// across components don't need to change.
#[derive(Clone)]
pub struct AudioContext {
    handle: AudioEngineHandle,
}

// Component props require PartialEq; the engine itself is a singleton, so
// any two AudioContext instances are equivalent.
impl PartialEq for AudioContext {
    fn eq(&self, _other: &Self) -> bool {
        true
    }
}

impl AudioContext {
    /// Builds a facade around the already-running engine thread. Panics if
    /// `spawn_engine()` hasn't run yet (it must run in `main()` before the
    /// Dioxus UI is built).
    pub fn new() -> Self {
        let config = AppConfig::load();
        AUDIO_VOLUME.get_or_init(|| Mutex::new(config.volume));
        MOUSE_AUDIO_VOLUME.get_or_init(|| Mutex::new(config.mouse_volume));
        ENABLE_SOUND.get_or_init(|| Mutex::new(config.enable_sound));
        ENABLE_KEYBOARD_SOUND.get_or_init(|| Mutex::new(config.enable_keyboard_sound));
        ENABLE_MOUSE_SOUND.get_or_init(|| Mutex::new(config.enable_mouse_sound));

        Self { handle: engine_handle() }
    }

    pub(crate) fn send(&self, command: AudioCommand) {
        self.handle.send(command);
    }

    pub fn set_volume(&self, volume: f32) {
        if let Some(global) = AUDIO_VOLUME.get() {
            *global.lock().unwrap() = volume;
        }
        // Always tell the engine (cheap, and makes the call idempotent), but
        // only touch the config file when the value actually changed. Mount
        // effects re-assert the current volume on every navigation/render, and
        // an unconditional save turned that into a steady rewrite of the whole
        // config struct - which is what raced the mute flag.
        persist_if_changed(|config| {
            if config.volume == volume {
                return false;
            }
            config.volume = volume;
            true
        });
        self.send(AudioCommand::SetVolume(volume));
    }

    pub fn get_volume(&self) -> f32 {
        AUDIO_VOLUME.get()
            .and_then(|v| v.lock().ok())
            .map(|v| *v)
            .unwrap_or(1.0)
    }

    pub fn set_mouse_volume(&self, volume: f32) {
        if let Some(global) = MOUSE_AUDIO_VOLUME.get() {
            *global.lock().unwrap() = volume;
        }
        persist_if_changed(|config| {
            if config.mouse_volume == volume {
                return false;
            }
            config.mouse_volume = volume;
            true
        });
        self.send(AudioCommand::SetMouseVolume(volume));
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

    pub fn set_sound_enabled(&self, enabled: bool) {
        if let Some(global) = ENABLE_SOUND.get() {
            *global.lock().unwrap() = enabled;
        }
        persist_if_changed(|config| {
            if config.enable_sound == enabled {
                return false;
            }
            config.enable_sound = enabled;
            true
        });
        self.send(AudioCommand::SetSoundEnabled(enabled));
    }

    pub fn set_keyboard_sound_enabled(&self, enabled: bool) {
        if let Some(global) = ENABLE_KEYBOARD_SOUND.get() {
            *global.lock().unwrap() = enabled;
        }
        persist_if_changed(|config| {
            if config.enable_keyboard_sound == enabled {
                return false;
            }
            config.enable_keyboard_sound = enabled;
            true
        });
        self.send(AudioCommand::SetKeyboardSoundEnabled(enabled));
    }

    pub fn set_mouse_sound_enabled(&self, enabled: bool) {
        if let Some(global) = ENABLE_MOUSE_SOUND.get() {
            *global.lock().unwrap() = enabled;
        }
        persist_if_changed(|config| {
            if config.enable_mouse_sound == enabled {
                return false;
            }
            config.enable_mouse_sound = enabled;
            true
        });
        self.send(AudioCommand::SetMouseSoundEnabled(enabled));
    }
}
