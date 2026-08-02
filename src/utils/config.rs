use crate::debug_print;
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
        // Always re-read from disk immediately before mutating: `save()`
        // rewrites the entire struct, so applying an edit to a copy captured
        // earlier would silently revert whatever another path wrote in between.
        let old_config = AppConfig::load();
        let mut new_config = old_config.clone();

        updater(&mut new_config);

        let changed = !new_config.data_equals(&old_config);

        // Only write to disk when something actually changed - callers include
        // mount effects that re-assert the value they already hold, and saving
        // those unconditionally rewrote the config continuously.
        if changed {
            new_config.last_updated = chrono::Utc::now();
            match new_config.save() {
                Ok(_) => debug_print!("✅ [config_utils] Config saved successfully"),
                Err(e) => eprintln!("❌ [config_utils] Failed to save config: {}", e),
            }
        }

        // Publish to the signal regardless of whether *this* call changed
        // anything. The value on disk may already differ from the signal
        // because another path (e.g. `AudioContext::set_*`, which persists on
        // its own) just wrote it; skipping the update here would leave the UI
        // rendering a stale value - that is what made the mute button appear
        // to do nothing after its state had already been applied.
        // Compared with `data_equals` (ignores `last_updated`) so a timestamp
        // bump alone can't trigger a pointless re-render.
        let signal_is_stale = !signal_ref.borrow().peek().data_equals(&new_config);
        if signal_is_stale {
            signal_ref.borrow_mut().set(new_config);
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
