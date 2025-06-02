use rdev::{listen, Button, Event, EventType, Key};
use std::collections::HashSet;
use std::sync::{mpsc::Sender, Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

// Maps a keyboard key to its standardized code
fn map_key_to_code(key: Key) -> &'static str {
    match key {
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
        Key::MetaLeft => "MetaLeft",
        Key::MetaRight => "MetaRight",

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

        // Number keys 0-9
        Key::Num0 => "Digit0",
        Key::Num1 => "Digit1",
        Key::Num2 => "Digit2",
        Key::Num3 => "Digit3",
        Key::Num4 => "Digit4",
        Key::Num5 => "Digit5",
        Key::Num6 => "Digit6",
        Key::Num7 => "Digit7",
        Key::Num8 => "Digit8",
        Key::Num9 => "Digit9",

        // Punctuation and symbols
        Key::Minus => "Minus",         // -
        Key::Equal => "Equal",         // =
        Key::Comma => "Comma",         // ,
        Key::Dot => "Period",          // .
        Key::Quote => "Quote",         // '
        Key::BackQuote => "Backquote", // `
        Key::Slash => "Slash",         // /

        // Unknown or unmapped keys
        _ => "",
    }
}

// Maps a mouse button to its standardized code
fn map_button_to_code(button: Button) -> &'static str {
    match button {
        Button::Left => "MouseLeft",
        Button::Right => "MouseRight",
        Button::Middle => "MouseMiddle",
        Button::Unknown(code) => {
            // Handle additional mouse buttons (side buttons, etc.)
            match code {
                4 => "Mouse4", // Back/Previous
                5 => "Mouse5", // Forward/Next
                6 => "Mouse6", // Extra button 1
                7 => "Mouse7", // Extra button 2
                8 => "Mouse8", // Extra button 3
                _ => "MouseUnknown",
            }
        }
    }
}

/// Start a unified input listener that handles both keyboard and mouse events
/// This solves the issue where rdev can only have one global listener at a time
pub fn start_unified_input_listener(
    keyboard_tx: Sender<String>,
    mouse_tx: Sender<String>,
    hotkey_tx: Sender<String>,
) {
    println!("🎮 Starting unified input listener (keyboard + mouse + hotkeys)...");

    thread::spawn(move || {
        println!("🎮 Unified input listener thread started");

        // Separate state tracking for keyboard and mouse
        let keyboard_last_press = Arc::new(Mutex::new(Instant::now()));
        let mouse_last_press = Arc::new(Mutex::new(Instant::now()));
        let pressed_keys = Arc::new(Mutex::new(HashSet::<String>::new()));
        let pressed_buttons = Arc::new(Mutex::new(HashSet::<String>::new()));

        // Track pressed modifier keys for hotkey detection
        let mut ctrl_pressed = false;
        let mut alt_pressed = false;

        let result = listen(move |event: Event| {
            match event.event_type {
                // ===== KEYBOARD EVENTS =====
                EventType::KeyPress(key) => {
                    let key_code = map_key_to_code(key);
                    if !key_code.is_empty() {
                        // println!("⌨️ Key Pressed: {}", key_code);

                        // Track modifier keys for hotkey detection
                        match key_code {
                            "ControlLeft" | "ControlRight" => ctrl_pressed = true,
                            "AltLeft" | "AltRight" => alt_pressed = true,
                            "KeyM" => {
                                // Check for Ctrl+Alt+M hotkey combination
                                if ctrl_pressed && alt_pressed {
                                    println!(
                                        "🔥 Hotkey detected: Ctrl+Alt+M - Toggling global sound"
                                    );
                                    let _ = hotkey_tx.send("TOGGLE_SOUND".to_string());
                                    return; // Don't process this as a regular key event
                                }
                            }
                            _ => {}
                        }

                        // Check if key is already pressed
                        let mut pressed = pressed_keys.lock().unwrap();
                        if pressed.contains(&key_code.to_string()) {
                            return; // Key already pressed, ignore
                        }
                        pressed.insert(key_code.to_string());
                        drop(pressed);

                        // Apply debounce
                        let now = Instant::now();
                        let mut last = keyboard_last_press.lock().unwrap();
                        if now.duration_since(*last) > Duration::from_millis(1) {
                            *last = now;
                            let _ = keyboard_tx.send(key_code.to_string());
                        }
                    }
                }
                EventType::KeyRelease(key) => {
                    let key_code = map_key_to_code(key);
                    if !key_code.is_empty() {
                        // println!("⌨️ Key Released: {}", key_code);

                        // Track modifier key releases for hotkey detection
                        match key_code {
                            "ControlLeft" | "ControlRight" => ctrl_pressed = false,
                            "AltLeft" | "AltRight" => alt_pressed = false,
                            _ => {}
                        }

                        // Remove key from pressed set
                        let mut pressed = pressed_keys.lock().unwrap();
                        pressed.remove(&key_code.to_string());
                        drop(pressed);

                        let _ = keyboard_tx.send(format!("UP:{}", key_code));
                    }
                }

                // ===== MOUSE EVENTS =====
                EventType::ButtonPress(button) => {
                    let button_code = map_button_to_code(button);
                    if !button_code.is_empty() && button_code != "MouseUnknown" {
                        // println!("🖱️ Mouse Button Pressed: {}", button_code);

                        // Check if button is already pressed
                        let mut pressed = pressed_buttons.lock().unwrap();
                        if pressed.contains(&button_code.to_string()) {
                            return; // Button already pressed, ignore
                        }
                        pressed.insert(button_code.to_string());
                        drop(pressed);

                        // Apply debounce
                        let now = Instant::now();
                        let mut last = mouse_last_press.lock().unwrap();
                        if now.duration_since(*last) > Duration::from_millis(1) {
                            *last = now;
                            let _ = mouse_tx.send(button_code.to_string());
                        }
                    }
                }
                EventType::ButtonRelease(button) => {
                    let button_code = map_button_to_code(button);
                    if !button_code.is_empty() && button_code != "MouseUnknown" {
                        // println!("🖱️ Mouse Button Released: {}", button_code);

                        // Remove button from pressed set
                        let mut pressed = pressed_buttons.lock().unwrap();
                        pressed.remove(&button_code.to_string());
                        drop(pressed);

                        let _ = mouse_tx.send(format!("UP:{}", button_code));
                    }
                }
                EventType::Wheel {
                    delta_x: _,
                    delta_y,
                } => {
                    let wheel_event = if delta_y > 0 {
                        "MouseWheelUp"
                    } else if delta_y < 0 {
                        "MouseWheelDown"
                    } else {
                        return; // No vertical scroll, ignore
                    };

                    println!("🖱️ Mouse Wheel: {}", wheel_event);

                    // Apply longer debounce for wheel events
                    let now = Instant::now();
                    let mut last = mouse_last_press.lock().unwrap();
                    if now.duration_since(*last) > Duration::from_millis(50) {
                        *last = now;
                        let _ = mouse_tx.send(wheel_event.to_string());
                    }
                }
                EventType::MouseMove { x: _, y: _ } => {
                    // Mouse move events are too noisy, ignore them
                    // println!("🖱️ Mouse Move: ({}, {})", x, y);
                }
            }
        });

        if let Err(error) = result {
            eprintln!("❌ Unified input listener error: {:?}", error);
        }
    });
}
