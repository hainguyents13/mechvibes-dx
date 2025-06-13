use crate::state::themes::ThemesConfig;
use dioxus::prelude::*;
use once_cell::sync::Lazy;
use std::rc::Rc;
use std::sync::{ Arc, Mutex };

static THEMES_CONFIG: Lazy<Arc<Mutex<ThemesConfig>>> = Lazy::new(||
    Arc::new(Mutex::new(ThemesConfig::load()))
);

/// Global signal to trigger refresh of all theme components
static REFRESH_TRIGGER: GlobalSignal<u32> = Signal::global(|| 0);

/// Hook for accessing and updating themes configuration
pub fn use_themes() -> (Signal<ThemesConfig>, Rc<dyn Fn(Box<dyn FnOnce(&mut ThemesConfig)>)>) {
    // Load initial themes config
    let mut themes = use_signal(|| THEMES_CONFIG.lock().unwrap().clone());

    // Watch for refresh trigger changes
    use_effect(move || {
        let _trigger_value = REFRESH_TRIGGER();
        // When trigger changes, reload themes from global config
        themes.set(THEMES_CONFIG.lock().unwrap().clone());
    });

    let update_themes = Rc::new(|updater: Box<dyn FnOnce(&mut ThemesConfig)>| {
        // Update the global static config
        {
            let mut config_guard = THEMES_CONFIG.lock().unwrap();
            updater(&mut *config_guard);

            if let Err(e) = config_guard.save() {
                eprintln!("❌ Failed to save themes: {}", e);
                return;
            }
        } // Trigger refresh of all components using themes
        let current = REFRESH_TRIGGER();
        *REFRESH_TRIGGER.write() = current + 1;
    });

    (themes, update_themes)
}

/// Get a reference to the global themes config (read-only)
pub fn get_themes_config() -> ThemesConfig {
    THEMES_CONFIG.lock().unwrap().clone()
}
