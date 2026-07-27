pub mod audio;
pub mod device_manager;
pub mod focused_input_listener;
pub mod input_device_manager;
pub mod input_listener;
pub mod input_manager;
pub mod protocol;
pub mod routes;
pub mod sound_processor;
pub mod theme;
pub mod tray;
pub mod tray_service;
pub mod ui;
pub mod window_manager;

#[cfg(target_os = "linux")]
pub mod evdev_input_listener;

#[cfg(target_os = "macos")]
pub mod device_query_mouse_listener;

#[cfg(target_os = "macos")]
pub mod macos_vibrancy;

// Re-export main audio types
pub use audio::AudioContext;
