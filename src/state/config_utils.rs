use crate::state::config::AppConfig;
use dioxus::prelude::*;
use std::rc::Rc;
use std::cell::RefCell;

/// Creates a config updater function that loads fresh config, applies changes, and saves
pub fn create_config_updater(config_signal: Signal<AppConfig>) -> Rc<dyn Fn(Box<dyn FnOnce(&mut AppConfig)>)> {
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
pub fn use_config() -> (Signal<AppConfig>, Rc<dyn Fn(Box<dyn FnOnce(&mut AppConfig)>)>) {
    let config = use_signal(|| AppConfig::load());
    let update_config = create_config_updater(config);
    (config, update_config)
}

/// Convenience functions for common config updates

/// Update volume setting
pub fn update_volume(update_config: &Rc<dyn Fn(Box<dyn FnOnce(&mut AppConfig)>)>, volume: f32) {
    update_config(Box::new(move |config| {
        config.volume = volume;
    }));
}

/// Update current soundpack
pub fn update_current_soundpack(update_config: &Rc<dyn Fn(Box<dyn FnOnce(&mut AppConfig)>)>, soundpack: String) {
    update_config(Box::new(move |config| {
        config.current_soundpack = soundpack;
    }));
}

/// Update enable sound setting
pub fn update_enable_sound(update_config: &Rc<dyn Fn(Box<dyn FnOnce(&mut AppConfig)>)>, enable: bool) {
    update_config(Box::new(move |config| {
        config.enable_sound = enable;
    }));
}

/// Update theme setting
pub fn update_theme(update_config: &Rc<dyn Fn(Box<dyn FnOnce(&mut AppConfig)>)>, theme: crate::libs::theme::Theme) {
    update_config(Box::new(move |config| {
        config.theme = theme;
    }));
}

/// Update auto start setting
pub fn update_auto_start(update_config: &Rc<dyn Fn(Box<dyn FnOnce(&mut AppConfig)>)>, auto_start: bool) {
    update_config(Box::new(move |config| {
        config.auto_start = auto_start;
    }));
}

/// Update show notifications setting
pub fn update_show_notifications(update_config: &Rc<dyn Fn(Box<dyn FnOnce(&mut AppConfig)>)>, show_notifications: bool) {
    update_config(Box::new(move |config| {
        config.show_notifications = show_notifications;
    }));
}

/// Reset all settings to defaults
pub fn reset_to_defaults(update_config: &Rc<dyn Fn(Box<dyn FnOnce(&mut AppConfig)>)>) {
    update_config(Box::new(|config| {
        let default_config = AppConfig::default();
        config.volume = default_config.volume;
        config.enable_sound = default_config.enable_sound;
        config.auto_start = default_config.auto_start;
        config.show_notifications = default_config.show_notifications;
        config.theme = default_config.theme;
    }));
}
