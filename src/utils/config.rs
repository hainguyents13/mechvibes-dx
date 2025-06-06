use crate::state::config::AppConfig;
use crate::utils::delay;
use dioxus::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

/// Hook that always loads fresh config from file
/// This ensures components get updated config even when changed externally (like hotkeys)
pub fn use_fresh_config() -> Signal<AppConfig> {
    let mut config = use_signal(|| AppConfig::load());

    // Refresh config from file periodically to catch external changes
    use_effect(move || {
        spawn(async move {
            loop {
                delay::Delay::ms(100).await;
                let fresh_config = AppConfig::load();
                if fresh_config.last_updated != config().last_updated {
                    config.set(fresh_config);
                }
            }
        });
    });

    config
}

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
    // Use fresh config that automatically reloads from file
    let config = use_fresh_config();
    let update_config = create_config_updater(config);
    (config, update_config)
}
