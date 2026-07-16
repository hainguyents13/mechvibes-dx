use crate::libs::audio::AudioContext;
use crate::libs::input_listener::start_unified_input_listener;
use crate::libs::input_manager::{
    init_input_channels, init_window_focus_state_with_value, get_window_focus_state, get_input_channels,
};
use crate::utils::constants::APP_NAME;
use std::sync::mpsc;
use std::time::Duration;

#[cfg(target_os = "linux")]
use crate::libs::evdev_input_listener::start_evdev_keyboard_listener;
#[cfg(target_os = "linux")]
use std::sync::{Arc, Mutex};

pub fn run_cli_mode(soundpack_name: &str) {
    println!("🔊 Starting {} in CLI mode with soundpack: {}", APP_NAME, soundpack_name);

    // Initialize audio context (loads soundpack, creates audio stream)
    let audio_ctx = AudioContext::new();

    // Create input event channels
    let (keyboard_tx, keyboard_rx) = mpsc::channel::<String>();
    let (mouse_tx, mouse_rx) = mpsc::channel::<String>();
    let (hotkey_tx, hotkey_rx) = mpsc::channel::<String>();

    let keyboard_tx_clone = keyboard_tx.clone();
    let mouse_tx_clone = mouse_tx.clone();
    let hotkey_tx_clone = hotkey_tx.clone();

    init_input_channels(
        keyboard_rx, mouse_rx, hotkey_rx,
        keyboard_tx_clone, mouse_tx_clone, hotkey_tx_clone,
    );

    // CLI mode is always "focused"
    init_window_focus_state_with_value(true);

    // Start input listeners
    #[cfg(target_os = "linux")]
    {
        let display_server = std::env::var("XDG_SESSION_TYPE").unwrap_or_else(|_| "x11".to_string());
        if display_server == "wayland" {
            let focus_state = get_window_focus_state();
            start_evdev_keyboard_listener(keyboard_tx.clone(), hotkey_tx.clone(), focus_state);
            let always_focused = Arc::new(Mutex::new(true));
            start_unified_input_listener(keyboard_tx, mouse_tx, hotkey_tx, Some(always_focused));
        } else {
            let focus_state = get_window_focus_state();
            start_unified_input_listener(keyboard_tx.clone(), mouse_tx, hotkey_tx, Some(focus_state.clone()));
            crate::libs::focused_input_listener::start_focused_keyboard_listener(keyboard_tx, focus_state);
        }
    }

    #[cfg(not(target_os = "linux"))]
    {
        let focus_state = get_window_focus_state();
        start_unified_input_listener(keyboard_tx.clone(), mouse_tx, hotkey_tx, Some(focus_state.clone()));
        crate::libs::focused_input_listener::start_focused_keyboard_listener(keyboard_tx, focus_state);
    }

    // Get channel receivers from global state
    let channels = get_input_channels();

    println!("✅ Listening for keyboard/mouse events... Press Ctrl+C to quit.");

    // Sound toggle state
    let mut sound_enabled = true;

    // Main event loop
    loop {
        // Check for hotkey (toggle sound)
        if let Ok(msg) = channels.hotkey_rx.lock().unwrap().try_recv() {
            if msg == "TOGGLE_SOUND" {
                sound_enabled = !sound_enabled;
                println!("🔊 Sound {}", if sound_enabled { "enabled" } else { "disabled" });
            }
        }

        // Process keyboard events
        if let Ok(keycode) = channels.keyboard_rx.lock().unwrap().try_recv() {
            if keycode.starts_with("UP:") {
                let key = &keycode[3..];
                if sound_enabled {
                    audio_ctx.play_key_event_sound(key, false);
                }
            } else if !keycode.is_empty() {
                if sound_enabled {
                    audio_ctx.play_key_event_sound(&keycode, true);
                }
            }
        }

        // Process mouse events
        if let Ok(button_code) = channels.mouse_rx.lock().unwrap().try_recv() {
            if button_code.starts_with("UP:") {
                let button = &button_code[3..];
                if sound_enabled {
                    audio_ctx.play_mouse_event_sound(button, false);
                }
            } else if !button_code.is_empty() {
                if sound_enabled {
                    audio_ctx.play_mouse_event_sound(&button_code, true);
                }
            }
        }

        std::thread::sleep(Duration::from_millis(1));
    }
}
