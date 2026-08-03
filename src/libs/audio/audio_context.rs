use crate::state::config::AppConfig;
use std::sync::Mutex;

use super::engine::{ engine_handle, AudioCommand, AudioEngineHandle };

static AUDIO_VOLUME: std::sync::OnceLock<Mutex<f32>> = std::sync::OnceLock::new();
static MOUSE_AUDIO_VOLUME: std::sync::OnceLock<Mutex<f32>> = std::sync::OnceLock::new();

// Cached config flags to avoid loading config on every keypress
static ENABLE_SOUND: std::sync::OnceLock<Mutex<bool>> = std::sync::OnceLock::new();
static ENABLE_KEYBOARD_SOUND: std::sync::OnceLock<Mutex<bool>> = std::sync::OnceLock::new();
static ENABLE_MOUSE_SOUND: std::sync::OnceLock<Mutex<bool>> = std::sync::OnceLock::new();

/// Submits a field-level mutation to the single config writer.
///
/// Both invariants this used to hold by hand now come from
/// `config_writer::apply` itself: the mutation runs against the authoritative
/// state (so it can never revert a field another subsystem just changed), and a
/// mutation that changes nothing is not written (UI mount effects re-assert
/// their current value on every render, and unconditional saves turned that
/// into a continuous rewrite loop that raced the mute flag).
fn persist(mutate: impl FnOnce(&mut AppConfig)) {
    crate::state::config_writer::apply(mutate);
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
        let config = crate::state::config_writer::current();
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
        // Always tell the engine (cheap, and makes the call idempotent). The
        // writer only touches the file when the value actually moved, which
        // matters because mount effects re-assert the current volume on every
        // navigation.
        persist(|config| {
            config.volume = volume;
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
        persist(|config| {
            config.mouse_volume = volume;
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
        persist(|config| {
            config.enable_sound = enabled;
        });
        self.send(AudioCommand::SetSoundEnabled(enabled));
    }

    pub fn set_keyboard_sound_enabled(&self, enabled: bool) {
        if let Some(global) = ENABLE_KEYBOARD_SOUND.get() {
            *global.lock().unwrap() = enabled;
        }
        persist(|config| {
            config.enable_keyboard_sound = enabled;
        });
        self.send(AudioCommand::SetKeyboardSoundEnabled(enabled));
    }

    pub fn set_mouse_sound_enabled(&self, enabled: bool) {
        if let Some(global) = ENABLE_MOUSE_SOUND.get() {
            *global.lock().unwrap() = enabled;
        }
        persist(|config| {
            config.enable_mouse_sound = enabled;
        });
        self.send(AudioCommand::SetMouseSoundEnabled(enabled));
    }
}
