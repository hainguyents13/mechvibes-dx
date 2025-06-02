use super::themes::ThemesConfig;
use dioxus::prelude::*;
use once_cell::sync::Lazy;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

static THEMES_CONFIG: Lazy<Arc<Mutex<ThemesConfig>>> =
    Lazy::new(|| Arc::new(Mutex::new(ThemesConfig::load())));

/// Creates a themes updater function that loads fresh themes, applies changes, and saves
pub fn create_themes_updater(
    themes_signal: Signal<ThemesConfig>,
) -> Rc<dyn Fn(Box<dyn FnOnce(&mut ThemesConfig)>)> {
    let signal_ref = Rc::new(RefCell::new(themes_signal));
    Rc::new(move |updater: Box<dyn FnOnce(&mut ThemesConfig)>| {
        let mut config_guard = THEMES_CONFIG.lock().unwrap();
        updater(&mut *config_guard);

        if let Err(e) = config_guard.save() {
            eprintln!("❌ Failed to save themes: {}", e);
        }

        // Update the signal through RefCell
        signal_ref.borrow_mut().set(config_guard.clone());
        println!("[theme_utils] Themes updated");
    })
}

/// Hook for accessing and updating themes configuration
pub fn use_themes() -> (
    Signal<ThemesConfig>,
    Rc<dyn Fn(Box<dyn FnOnce(&mut ThemesConfig)>)>,
) {
    let themes = use_signal(|| THEMES_CONFIG.lock().unwrap().clone());

    let update_themes = create_themes_updater(themes);

    (themes, update_themes)
}

/// Get a reference to the global themes config (read-only)
pub fn get_themes_config() -> ThemesConfig {
    THEMES_CONFIG.lock().unwrap().clone()
}
