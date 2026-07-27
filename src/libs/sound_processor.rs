use std::thread;
use std::time::Duration;

use crate::libs::audio::get_global_audio_context;
use crate::libs::input_manager::get_input_channels;
use crate::libs::tray_service::request_tray_update;

/// Spawn a dedicated thread that consumes keyboard, mouse, and hotkey events
/// from the mpsc channels and plays sounds via AudioContext.
///
/// This thread runs independently of the Dioxus event loop, so sounds play
/// even when the main-thread run loop is throttled (macOS App Nap, window
/// unfocused, etc.).
pub fn start_sound_processor_thread() {
    thread::Builder::new()
        .name("sound-processor".into())
        .spawn(move || {
            println!("🔊 Sound processor thread started (independent of Dioxus event loop)");

            let channels = get_input_channels();
            let keyboard_rx = channels.keyboard_rx.clone();
            let mouse_rx = channels.mouse_rx.clone();
            let hotkey_rx = channels.hotkey_rx.clone();

            let ctx = get_global_audio_context();

            loop {
                // Process ALL pending keyboard events (drain the channel)
                if let Ok(kbd) = keyboard_rx.try_lock() {
                    while let Ok(keycode) = kbd.try_recv() {
                        if keycode.starts_with("UP:") {
                            let key = &keycode[3..];
                            ctx.play_key_event_sound(key, false);
                        } else if !keycode.is_empty() {
                            ctx.play_key_event_sound(&keycode, true);
                        }
                    }
                }

                // Process ALL pending mouse events
                if let Ok(ms) = mouse_rx.try_lock() {
                    while let Ok(button_code) = ms.try_recv() {
                        if button_code.starts_with("UP:") {
                            let button = &button_code[3..];
                            ctx.play_mouse_event_sound(button, false);
                        } else if !button_code.is_empty() {
                            ctx.play_mouse_event_sound(&button_code, true);
                        }
                    }
                }

                // Process ALL pending hotkey events
                if let Ok(hk) = hotkey_rx.try_lock() {
                    while let Ok(hotkey_command) = hk.try_recv() {
                        if hotkey_command == "TOGGLE_SOUND" {
                            let mut config = crate::state::config::AppConfig::load();
                            config.enable_sound = !config.enable_sound;
                            config.last_updated = chrono::Utc::now();
                            match config.save() {
                                Ok(_) => {
                                    request_tray_update();
                                    println!("🔄 Sound toggled: {}", config.enable_sound);
                                }
                                Err(e) => {
                                    eprintln!("❌ Failed to save config after sound toggle: {}", e);
                                }
                            }
                        }
                    }
                }

                thread::sleep(Duration::from_millis(5));
            }
        })
        .expect("Failed to spawn sound processor thread");
}
