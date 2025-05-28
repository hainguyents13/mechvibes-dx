use crate::state::config::AppConfig;
use dioxus::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

/// Creates a config updater function that loads fresh config, applies changes, and saves
pub fn create_config_updater(
    config_signal: Signal<AppConfig>,
) -> Rc<dyn Fn(Box<dyn FnOnce(&mut AppConfig)>)> {
    let signal_ref = Rc::new(RefCell::new(config_signal));
    Rc::new(move |updater: Box<dyn FnOnce(&mut AppConfig)>| {
        let mut new_config = AppConfig::load(); // Always load fresh from file
        updater(&mut new_config);
        new_config.last_updated = chrono::Utc::now();
        let _ = new_config.save();

        // Update the signal through RefCell
        signal_ref.borrow_mut().set(new_config);
        println!("[config_utils] Config updated");
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
    let config = use_signal(|| AppConfig::load());
    let update_config = create_config_updater(config);
    (config, update_config)
}
