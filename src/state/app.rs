// Standard library imports
use std::sync::{Arc, Mutex, RwLock};

// External crate imports
use dioxus::prelude::*;
use once_cell::sync::OnceCell;

// Internal crate imports
use crate::state::config::AppConfig;
use crate::state::soundpack_cache::SoundpackCache;

// Global app state for sharing between components
#[derive(Clone, Debug)]
pub struct AppState {
    pub config: Arc<AppConfig>,
    pub optimized_cache: Arc<SoundpackCache>,
}

impl AppState {
    pub fn new() -> Self {
        println!("🌍 Initializing global AppState...");

        Self {
            config: Arc::new(AppConfig::load()),
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

// Reload the current soundpack from configuration
pub fn reload_current_soundpack(audio_ctx: &crate::libs::audio::AudioContext) {
    let config = crate::state::config::AppConfig::load();
    let current_id = &config.current_soundpack;

    match crate::libs::audio::load_soundpack_optimized(audio_ctx, current_id) {
        Ok(_) => println!(
            "✅ Current soundpack '{}' reloaded successfully (optimized)",
            current_id
        ),
        Err(e) => eprintln!(
            "❌ Failed to reload current soundpack '{}': {}",
            current_id, e
        ),
    }
}

// Utility function to reload soundpacks from anywhere
pub fn reload_soundpacks() {
    println!("🔄 Reloading global soundpack cache...");

    // Update mutex state
    if let Some(mutex) = APP_STATE.get() {
        if let Ok(mut app_state) = mutex.lock() {
            let config = app_state.config.clone();
            let optimized_cache = Arc::new(SoundpackCache::load());
            *app_state = AppState {
                config,
                optimized_cache,
            };
        }
    }

    // Update signal state
    if let Some(rwlock) = APP_STATE_SIGNAL.get() {
        if let Ok(mut signal_state) = rwlock.write() {
            let config = signal_state.config.clone();
            let optimized_cache = Arc::new(SoundpackCache::load());
            *signal_state = AppState {
                config,
                optimized_cache,
            };
        }
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
