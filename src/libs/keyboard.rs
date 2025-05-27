use rdev::{listen, Event, EventType, Key};
use std::sync::{mpsc::Sender, Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};
use std::collections::HashMap;
use once_cell::sync::Lazy;

/// Make a static table of key codes for optimized lookups
static KEY_CODE_MAP: Lazy<HashMap<Key, &'static str>> = Lazy::new(|| {
    let mut map = HashMap::new();

    // Common keys across all platforms
    map.insert(Key::Space, "Space");
    map.insert(Key::Backspace, "Backspace");
    map.insert(Key::CapsLock, "CapsLock");
    map.insert(Key::Tab, "Tab");
    map.insert(Key::Return, "Enter");
    map.insert(Key::Escape, "Escape");
    map.insert(Key::Delete, "Delete");

    // Modifier keys with left/right variants
    map.insert(Key::Alt, "AltLeft");
    map.insert(Key::AltGr, "AltRight");
    map.insert(Key::ShiftLeft, "ShiftLeft");
    map.insert(Key::ShiftRight, "ShiftRight");
    map.insert(Key::ControlLeft, "ControlLeft");
    map.insert(Key::ControlRight, "ControlRight");

    // Platform-specific meta keys handled in function due to conditional logic

    // Arrow keys
    map.insert(Key::UpArrow, "ArrowUp");
    map.insert(Key::DownArrow, "ArrowDown");
    map.insert(Key::LeftArrow, "ArrowLeft");
    map.insert(Key::RightArrow, "ArrowRight");

    // Navigation keys
    map.insert(Key::Home, "Home");
    map.insert(Key::End, "End");
    map.insert(Key::PageUp, "PageUp");
    map.insert(Key::PageDown, "PageDown");

    // Function keys
    map.insert(Key::F1, "F1");
    map.insert(Key::F2, "F2");
    map.insert(Key::F3, "F3");
    map.insert(Key::F4, "F4");
    map.insert(Key::F5, "F5");
    map.insert(Key::F6, "F6");
    map.insert(Key::F7, "F7");
    map.insert(Key::F8, "F8");
    map.insert(Key::F9, "F9");
    map.insert(Key::F10, "F10");
    map.insert(Key::F11, "F11");
    map.insert(Key::F12, "F12");

    // Alpha keys A-Z
    map.insert(Key::KeyA, "KeyA");
    map.insert(Key::KeyB, "KeyB");
    map.insert(Key::KeyC, "KeyC");
    map.insert(Key::KeyD, "KeyD");
    map.insert(Key::KeyE, "KeyE");
    map.insert(Key::KeyF, "KeyF");
    map.insert(Key::KeyG, "KeyG");
    map.insert(Key::KeyH, "KeyH");
    map.insert(Key::KeyI, "KeyI");
    map.insert(Key::KeyJ, "KeyJ");
    map.insert(Key::KeyK, "KeyK");
    map.insert(Key::KeyL, "KeyL");
    map.insert(Key::KeyM, "KeyM");
    map.insert(Key::KeyN, "KeyN");
    map.insert(Key::KeyO, "KeyO");
    map.insert(Key::KeyP, "KeyP");
    map.insert(Key::KeyQ, "KeyQ");
    map.insert(Key::KeyR, "KeyR");
    map.insert(Key::KeyS, "KeyS");
    map.insert(Key::KeyT, "KeyT");
    map.insert(Key::KeyU, "KeyU");
    map.insert(Key::KeyV, "KeyV");
    map.insert(Key::KeyW, "KeyW");
    map.insert(Key::KeyX, "KeyX");
    map.insert(Key::KeyY, "KeyY");
    map.insert(Key::KeyZ, "KeyZ");

    // Number keys
    map.insert(Key::Num0, "Digit0");
    map.insert(Key::Num1, "Digit1");
    map.insert(Key::Num2, "Digit2");
    map.insert(Key::Num3, "Digit3");
    map.insert(Key::Num4, "Digit4");
    map.insert(Key::Num5, "Digit5");
    map.insert(Key::Num6, "Digit6");
    map.insert(Key::Num7, "Digit7");
    map.insert(Key::Num8, "Digit8");
    map.insert(Key::Num9, "Digit9");

    // Special characters and punctuation
    map.insert(Key::BackQuote, "Backquote");
    map.insert(Key::Minus, "Minus");
    map.insert(Key::Equal, "Equal");
    map.insert(Key::Quote, "Quote");
    map.insert(Key::Comma, "Comma");
    map.insert(Key::Slash, "Slash");

    // Reserved for future rdev updates
    /*
    map.insert(Key::BracketLeft, "BracketLeft");
    map.insert(Key::BracketRight, "BracketRight");
    map.insert(Key::Backslash, "Backslash");
    map.insert(Key::Semicolon, "Semicolon");
    map.insert(Key::Period, "Period");
    map.insert(Key::MediaVolumeDown, "AudioVolumeDown");
    map.insert(Key::MediaVolumeUp, "AudioVolumeUp");
    map.insert(Key::MediaVolumeMute, "AudioVolumeMute");
    map.insert(Key::MediaPlayPause, "MediaPlayPause");
    map.insert(Key::MediaPreviousTrack, "MediaTrackPrevious");
    map.insert(Key::MediaNextTrack, "MediaTrackNext");
    */

    map
});

/// Maps a keyboard key to its standardized code across different platforms
fn map_key_to_code(key: Key) -> &'static str {
    // Check static map first for most common keys
    if let Some(&code) = KEY_CODE_MAP.get(&key) {
        println!("🔍 Mapping key {:?} to code '{}'", key, code);
        return code;
    }

    // Handle platform-specific meta keys that need conditional logic
    let code = match key {
        Key::MetaLeft => {
            if cfg!(target_os = "macos") {
                "MetaLeft"
            } else {
                "OSLeft"
            }
        }
        Key::MetaRight => {
            if cfg!(target_os = "macos") {
                "MetaRight"
            } else {
                "OSRight"
            }
        }
        // Unknown or unmapped keys
        _ => "",
    };

    println!("🔍 Mapping key {:?} to code '{}'", key, code);
    code
}

pub fn start_keyboard_listener(play_sound_tx: Sender<String>) {
    let last_press = Arc::new(Mutex::new(Instant::now()));

    thread::spawn(move || {
        println!("🎹 Keyboard listener started...");

        let result = listen(move |event: Event| match event.event_type {
            EventType::KeyPress(key) => {
                let key_code = map_key_to_code(key);
                if !key_code.is_empty() {
                    println!("🛠 Key Pressed: {}", key_code);
                    let now = Instant::now();
                    let mut last = last_press.lock().unwrap();
                    if now.duration_since(*last) > Duration::from_millis(1) {
                        *last = now;
                        let _ = play_sound_tx.send(key_code.to_string());
                    }
                }
            }
            EventType::KeyRelease(key) => {
                let key_code = map_key_to_code(key);
                if !key_code.is_empty() {
                    println!("🛠 Key Released: {}", key_code);
                    let _ = play_sound_tx.send(format!("UP:{}", key_code));
                }
            }
            _ => {}
        });

        if let Err(error) = result {
            eprintln!("❌ Keyboard listener error: {:?}", error);
        }
    });
}
