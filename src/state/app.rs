// Standard library imports
use std::sync::{Arc, Mutex, RwLock};

// External crate imports
use dioxus::prelude::*;
use once_cell::sync::OnceCell;

// Internal crate imports
use crate::state::soundpack_cache::SoundpackCache;

// Global app state for sharing between components
#[derive(Clone, Debug)]
pub struct AppState {
    pub optimized_cache: Arc<SoundpackCache>,
}

impl AppState {
    pub fn new() -> Self {
        println!("🌍 Initializing global AppState...");

        Self {
            optimized_cache: Arc::new(SoundpackCache::load()),
        }
    }

    // Load soundpack list from optimized cache
    pub fn get_soundpacks(&self) -> Vec<crate::state::soundpack_cache::SoundpackMetadata> {
        self.optimized_cache.soundpacks.values().cloned().collect()
    }
}

static APP_STATE_SIGNAL: OnceCell<RwLock<AppState>> = OnceCell::new();

// Hook for easy access to state from components
pub fn use_app_state() -> Signal<AppState> {
    // Initialize the global signal if needed
    if APP_STATE_SIGNAL.get().is_none() {
        if let Some(mutex) = APP_STATE.get() {
            if let Ok(state) = mutex.lock() {
                let _ = APP_STATE_SIGNAL.set(RwLock::new(state.clone()));
            }
        }
    }

    use_signal(move || {
        APP_STATE_SIGNAL
            .get()
            .and_then(|lock| lock.read().ok().map(|guard| guard.clone()))
            .unwrap_or_else(AppState::new)
    })
}

// Reload the current soundpacks from configuration
pub fn reload_current_soundpacks(audio_ctx: &crate::libs::audio::AudioContext) {
    let config = crate::state::config::AppConfig::load();

    // Load keyboard soundpack
    match crate::libs::audio::load_keyboard_soundpack(audio_ctx, &config.keyboard_soundpack) {
        Ok(_) => println!(
            "✅ Keyboard soundpack '{}' reloaded successfully",
            config.keyboard_soundpack
        ),
        Err(e) => eprintln!(
            "❌ Failed to reload keyboard soundpack '{}': {}",
            config.keyboard_soundpack, e
        ),
    }

    // Load mouse soundpack
    match crate::libs::audio::load_mouse_soundpack(audio_ctx, &config.mouse_soundpack) {
        Ok(_) => println!(
            "✅ Mouse soundpack '{}' reloaded successfully",
            config.mouse_soundpack
        ),
        Err(e) => eprintln!(
            "❌ Failed to reload mouse soundpack '{}': {}",
            config.mouse_soundpack, e
        ),
    }
}

pub static APP_STATE: OnceCell<Mutex<AppState>> = OnceCell::new();

// Call this function once at the start of your application to initialize APP_STATE
pub fn init_app_state() {
    // Initialize the mutex-based state if not already done
    if APP_STATE.get().is_none() {
        println!("📝 Initializing global app state (mutex)...");
        APP_STATE
            .set(Mutex::new(AppState::new()))
            .expect("Failed to initialize global app state mutex");
    }

    // Initialize the signal-based state if not already done
    if APP_STATE_SIGNAL.get().is_none() {
        println!("📝 Initializing global app state (signal)...");
        let app_state = match APP_STATE.get().and_then(|mutex| mutex.lock().ok()) {
            Some(state) => state.clone(),
            None => AppState::new(),
        };

        APP_STATE_SIGNAL
            .set(RwLock::new(app_state))
            .expect("Failed to initialize global app state signal");
    }
}
