use crate::state::config::AppConfig;
use crate::state::soundpack_cache::SoundpackCache;
use dioxus::prelude::*;
use std::sync::Arc;

// Global app state để chia sẻ giữa các component
#[derive(Clone, Debug)]
pub struct AppState {
    pub config: Arc<AppConfig>,
    pub soundpack_cache: Arc<SoundpackCache>,
}

impl AppState {
    pub fn new() -> Self {
        println!("🌍 Initializing global AppState...");
        Self {
            config: Arc::new(AppConfig::load()),
            soundpack_cache: Arc::new(SoundpackCache::load()),
        }
    }
}

use once_cell::sync::OnceCell;
use std::sync::Mutex;
use std::sync::RwLock;

static APP_STATE_SIGNAL: OnceCell<RwLock<AppState>> = OnceCell::new();

// Hook để dễ dàng truy cập state từ component
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

    match crate::libs::audio::load_soundpack_by_id(audio_ctx, current_id) {
        Ok(_) => println!(
            "✅ Current soundpack '{}' reloaded successfully",
            current_id
        ),
        Err(e) => eprintln!(
            "❌ Failed to reload current soundpack '{}': {}",
            current_id, e
        ),
    }
}

// Hàm tiện ích để reload soundpacks từ bất kỳ đâu
#[allow(dead_code)]
pub fn reload_soundpacks() {
    println!("🔄 Reloading global soundpack cache...");

    // Update mutex state
    if let Some(mutex) = APP_STATE.get() {
        if let Ok(mut app_state) = mutex.lock() {
            let config = app_state.config.clone();
            let soundpack_cache = Arc::new(SoundpackCache::rebuild());
            *app_state = AppState {
                config,
                soundpack_cache,
            };
        }
    }

    // Update signal state
    if let Some(rwlock) = APP_STATE_SIGNAL.get() {
        if let Ok(mut signal_state) = rwlock.write() {
            let config = signal_state.config.clone();
            let soundpack_cache = Arc::new(SoundpackCache::rebuild());
            *signal_state = AppState {
                config,
                soundpack_cache,
            };
        }
    }
}

pub static APP_STATE: OnceCell<Mutex<AppState>> = OnceCell::new();

pub fn save_config(config: AppConfig) -> Result<(), String> {
    // Đảm bảo update version và timestamp khi lưu
    let mut config = config;
    config.version = env!("CARGO_PKG_VERSION").to_string();
    config.last_updated = chrono::Utc::now();

    // Lưu config
    config.save().map_err(|e| e.to_string())?;

    // Cập nhật config trong bộ nhớ một cách hiệu quả
    let new_config = Arc::new(config);

    // Update mutex state
    if let Some(mutex) = APP_STATE.get() {
        if let Ok(mut app_state) = mutex.lock() {
            let soundpack_cache = app_state.soundpack_cache.clone();
            *app_state = AppState {
                config: new_config.clone(),
                soundpack_cache,
            };
        }
    }

    // Update signal state
    if let Some(rwlock) = APP_STATE_SIGNAL.get() {
        if let Ok(mut signal_state) = rwlock.write() {
            let soundpack_cache = signal_state.soundpack_cache.clone();
            *signal_state = AppState {
                config: new_config,
                soundpack_cache,
            };
        }
    }

    Ok(())
}

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
