use crate::state::config::AppConfig;
use dioxus::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

/// Hook that gets the global config from context
/// Config is loaded once at app startup and shared across all pages
/// This prevents config reset on page navigation
pub fn use_fresh_config() -> Signal<AppConfig> {
    // Get global config from context (provided in ui.rs app())
    use_context::<Signal<AppConfig>>()
}

/// Creates a config updater function that loads fresh config, applies changes, and saves
pub fn create_config_updater(
    config_signal: Signal<AppConfig>,
) -> Rc<dyn Fn(Box<dyn FnOnce(&mut AppConfig)>)> {
    let signal_ref = Rc::new(RefCell::new(config_signal));
    Rc::new(move |updater: Box<dyn FnOnce(&mut AppConfig)>| {
        let old_config = AppConfig::load(); // Load current config
        let mut new_config = old_config.clone(); // Clone for modification

        updater(&mut new_config);

        // Only update timestamp and save if config data actually changed (excluding metadata)
        if !new_config.data_equals(&old_config) {
            new_config.last_updated = chrono::Utc::now();
            match new_config.save() {
                Ok(_) => {
                    println!("✅ [config_utils] Config saved successfully");
                    println!("   keyboard_soundpack: {}", new_config.keyboard_soundpack);
                    println!("   mouse_soundpack: {}", new_config.mouse_soundpack);
                }
                Err(e) => {
                    eprintln!("❌ [config_utils] Failed to save config: {}", e);
                }
            }

            // Update the signal through RefCell
            signal_ref.borrow_mut().set(new_config);
        } else {
            println!("[config_utils] Config unchanged, skipping save");
        }
    })
}

/// Hook for managing configuration state with automatic updates
///
/// Returns a tuple of (config_signal, update_config_fn)
/// The update function can be used to make atomic config updates
pub fn use_config() -> (
    Signal<AppConfig>,
    Rc<dyn Fn(Box<dyn FnOnce(&mut AppConfig)>)>,
) {
    // Use fresh config that automatically reloads from file
    let config = use_fresh_config();
    let update_config = create_config_updater(config);
    (config, update_config)
}
