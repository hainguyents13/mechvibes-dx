use rdev::{listen, Event, EventType, Key};
use std::sync::{mpsc::Sender, Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

/// Maps a keyboard key to its standardized code across different platforms
fn map_key_to_code(key: Key) -> &'static str {
    let code = match key {
        // Common keys across all platforms
        Key::Space => "Space",
        Key::Backspace => "Backspace",
        Key::CapsLock => "CapsLock",
        Key::Tab => "Tab",
        Key::Return => "Enter",
        Key::Escape => "Escape",
        Key::Delete => "Delete",

        // Modifier keys with left/right variants
        Key::Alt => "AltLeft",
        Key::AltGr => "AltRight",
        Key::ShiftLeft => "ShiftLeft",
        Key::ShiftRight => "ShiftRight",
        Key::ControlLeft => "ControlLeft",
        Key::ControlRight => "ControlRight",
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

        // Arrow keys
        Key::UpArrow => "ArrowUp",
        Key::DownArrow => "ArrowDown",
        Key::LeftArrow => "ArrowLeft",
        Key::RightArrow => "ArrowRight",

        // Navigation keys
        Key::Home => "Home",
        Key::End => "End",
        Key::PageUp => "PageUp",
        Key::PageDown => "PageDown",

        // Function keys
        Key::F1 => "F1",
        Key::F2 => "F2",
        Key::F3 => "F3",
        Key::F4 => "F4",
        Key::F5 => "F5",
        Key::F6 => "F6",
        Key::F7 => "F7",
        Key::F8 => "F8",
        Key::F9 => "F9",
        Key::F10 => "F10",
        Key::F11 => "F11",
        Key::F12 => "F12",

        // Alpha keys A-Z
        Key::KeyA => "KeyA",
        Key::KeyB => "KeyB",
        Key::KeyC => "KeyC",
        Key::KeyD => "KeyD",
        Key::KeyE => "KeyE",
        Key::KeyF => "KeyF",
        Key::KeyG => "KeyG",
        Key::KeyH => "KeyH",
        Key::KeyI => "KeyI",
        Key::KeyJ => "KeyJ",
        Key::KeyK => "KeyK",
        Key::KeyL => "KeyL",
        Key::KeyM => "KeyM",
        Key::KeyN => "KeyN",
        Key::KeyO => "KeyO",
        Key::KeyP => "KeyP",
        Key::KeyQ => "KeyQ",
        Key::KeyR => "KeyR",
        Key::KeyS => "KeyS",
        Key::KeyT => "KeyT",
        Key::KeyU => "KeyU",
        Key::KeyV => "KeyV",
        Key::KeyW => "KeyW",
        Key::KeyX => "KeyX",
        Key::KeyY => "KeyY",
        Key::KeyZ => "KeyZ",

        // Number keys
        Key::Num0 => "Digit0",
        Key::Num1 => "Digit1",
        Key::Num2 => "Digit2",
        Key::Num3 => "Digit3",
        Key::Num4 => "Digit4",
        Key::Num5 => "Digit5",
        Key::Num6 => "Digit6",
        Key::Num7 => "Digit7",
        Key::Num8 => "Digit8",
        Key::Num9 => "Digit9", // Special characters and punctuation
        // These can vary by keyboard layout
        Key::BackQuote => "Backquote", // `
        Key::Minus => "Minus",         // -
        Key::Equal => "Equal",         // =
        Key::Quote => "Quote",         // '
        Key::Comma => "Comma",         // ,
        Key::Slash => "Slash",         // /

        // Reserved for future rdev updates
        /*
        Key::BracketLeft => "BracketLeft", // [
        Key::BracketRight => "BracketRight", // ]
        Key::Backslash => "Backslash", // \
        Key::Semicolon => "Semicolon", // ;
        Key::Period => "Period", // .
        Key::MediaVolumeDown => "AudioVolumeDown",
        Key::MediaVolumeUp => "AudioVolumeUp",
        Key::MediaVolumeMute => "AudioVolumeMute",
        Key::MediaPlayPause => "MediaPlayPause",
        Key::MediaPreviousTrack => "MediaTrackPrevious",
        Key::MediaNextTrack => "MediaTrackNext",
        */
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
