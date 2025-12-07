use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

#[cfg(target_os = "linux")]
use std::sync::mpsc::Sender;

#[cfg(target_os = "linux")]
pub fn start_evdev_keyboard_listener(
    keyboard_tx: Sender<String>,
    hotkey_tx: Sender<String>,
    _is_focused: Arc<Mutex<bool>>,
) {
    thread::spawn(move || {
        use evdev::{Device, EventType};

        println!("🔍 [evdev] Starting Linux keyboard listener (Wayland/X11 compatible)");

        // Track modifier keys for hotkey detection
        let mut ctrl_pressed = false;
        let mut alt_pressed = false;

        // Find all keyboard devices
        let mut keyboards = Vec::new();

        let devices: Vec<_> = evdev::enumerate().collect();
        if !devices.is_empty() {
            for (path, mut device) in devices {
                    // Check if device has keyboard capabilities
                    if device.supported_keys().is_some() {
                        println!("🔍 [evdev] Found keyboard device: {:?} - {}", path.display(), device.name().unwrap_or("Unknown"));

                        // Set device to non-blocking mode to prevent blocking on idle devices
                        if let Err(e) = device.set_nonblocking(true) {
                            eprintln!("⚠️ [evdev] Failed to set non-blocking mode for {:?}: {}", path.display(), e);
                        }

                        keyboards.push(device);
                    }
                }
            } else {
                eprintln!("❌ [evdev] No devices found");
                eprintln!("💡 [evdev] Make sure you're in the 'input' group: sudo usermod -a -G input $USER");
                return;
            }
        
        if keyboards.is_empty() {
            eprintln!("❌ [evdev] No keyboard devices found!");
            eprintln!("💡 [evdev] Make sure you have permission to access /dev/input/event*");
            return;
        }
        
        println!("🔍 [evdev] Monitoring {} keyboard device(s)", keyboards.len());
        
        // Monitor all keyboards in a loop
        loop {
            for device in &mut keyboards {
                // Fetch events (non-blocking)
                match device.fetch_events() {
                    Ok(events) => {
                        for event in events {
                            if event.event_type() == EventType::KEY {
                                let key_value = event.value();

                                // Convert event code to Key enum
                                let key = evdev::Key(event.code());
                                {
                                    let key_code = map_evdev_keycode(key);
                                    if !key_code.is_empty() {
                                        // Handle key press (value == 1)
                                        if key_value == 1 {
                                            // Track modifier keys for hotkey detection
                                            match key_code {
                                                "ControlLeft" | "ControlRight" => {
                                                    ctrl_pressed = true;
                                                }
                                                "AltLeft" | "AltRight" => {
                                                    alt_pressed = true;
                                                }
                                                "KeyM" => {
                                                    // Check for Ctrl+Alt+M hotkey combination
                                                    if ctrl_pressed && alt_pressed {
                                                        println!("🔥 [evdev] Hotkey detected: Ctrl+Alt+M - Toggling global sound");
                                                        let _ = hotkey_tx.send("TOGGLE_SOUND".to_string());
                                                        continue; // Don't process this as a regular key event
                                                    }
                                                }
                                                _ => {}
                                            }

                                            // Send key press event
                                            let _ = keyboard_tx.send(key_code.to_string());
                                        }
                                        // Handle key release (value == 0)
                                        else if key_value == 0 {
                                            // Track modifier key releases for hotkey detection
                                            match key_code {
                                                "ControlLeft" | "ControlRight" => {
                                                    ctrl_pressed = false;
                                                }
                                                "AltLeft" | "AltRight" => {
                                                    alt_pressed = false;
                                                }
                                                _ => {}
                                            }

                                            // Send key release event
                                            let _ = keyboard_tx.send(format!("UP:{}", key_code));
                                        }
                                        // Ignore key repeat (value == 2)
                                    }
                                }
                            }
                        }
                    }
                    Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                        // No events available, this is normal
                    }
                    Err(e) => {
                        eprintln!("⚠️ [evdev] Error fetching events: {}", e);
                    }
                }
            }
            
            // Small sleep to prevent busy-waiting
            thread::sleep(Duration::from_millis(10));
        }
    });
}

#[cfg(target_os = "linux")]
fn map_evdev_keycode(key: evdev::Key) -> &'static str {
    use evdev::Key;
    
    match key {
        // Letters
        Key::KEY_A => "KeyA", Key::KEY_B => "KeyB", Key::KEY_C => "KeyC", Key::KEY_D => "KeyD",
        Key::KEY_E => "KeyE", Key::KEY_F => "KeyF", Key::KEY_G => "KeyG", Key::KEY_H => "KeyH",
        Key::KEY_I => "KeyI", Key::KEY_J => "KeyJ", Key::KEY_K => "KeyK", Key::KEY_L => "KeyL",
        Key::KEY_M => "KeyM", Key::KEY_N => "KeyN", Key::KEY_O => "KeyO", Key::KEY_P => "KeyP",
        Key::KEY_Q => "KeyQ", Key::KEY_R => "KeyR", Key::KEY_S => "KeyS", Key::KEY_T => "KeyT",
        Key::KEY_U => "KeyU", Key::KEY_V => "KeyV", Key::KEY_W => "KeyW", Key::KEY_X => "KeyX",
        Key::KEY_Y => "KeyY", Key::KEY_Z => "KeyZ",

        // Numbers
        Key::KEY_1 => "Digit1", Key::KEY_2 => "Digit2", Key::KEY_3 => "Digit3", Key::KEY_4 => "Digit4",
        Key::KEY_5 => "Digit5", Key::KEY_6 => "Digit6", Key::KEY_7 => "Digit7", Key::KEY_8 => "Digit8",
        Key::KEY_9 => "Digit9", Key::KEY_0 => "Digit0",

        // Function keys
        Key::KEY_F1 => "F1", Key::KEY_F2 => "F2", Key::KEY_F3 => "F3", Key::KEY_F4 => "F4",
        Key::KEY_F5 => "F5", Key::KEY_F6 => "F6", Key::KEY_F7 => "F7", Key::KEY_F8 => "F8",
        Key::KEY_F9 => "F9", Key::KEY_F10 => "F10", Key::KEY_F11 => "F11", Key::KEY_F12 => "F12",

        // Special keys
        Key::KEY_SPACE => "Space",
        Key::KEY_ENTER => "Enter",
        Key::KEY_BACKSPACE => "Backspace",
        Key::KEY_TAB => "Tab",
        Key::KEY_ESC => "Escape",
        Key::KEY_CAPSLOCK => "CapsLock",
        Key::KEY_LEFTSHIFT => "ShiftLeft",
        Key::KEY_RIGHTSHIFT => "ShiftRight",
        Key::KEY_LEFTCTRL => "ControlLeft",
        Key::KEY_RIGHTCTRL => "ControlRight",
        Key::KEY_LEFTALT => "AltLeft",
        Key::KEY_RIGHTALT => "AltRight",
        Key::KEY_LEFTMETA => "MetaLeft",
        Key::KEY_RIGHTMETA => "MetaRight",

        // Arrow keys
        Key::KEY_UP => "ArrowUp",
        Key::KEY_DOWN => "ArrowDown",
        Key::KEY_LEFT => "ArrowLeft",
        Key::KEY_RIGHT => "ArrowRight",

        // Editing keys
        Key::KEY_INSERT => "Insert",
        Key::KEY_DELETE => "Delete",
        Key::KEY_HOME => "Home",
        Key::KEY_END => "End",
        Key::KEY_PAGEUP => "PageUp",
        Key::KEY_PAGEDOWN => "PageDown",

        // Punctuation
        Key::KEY_MINUS => "Minus",
        Key::KEY_EQUAL => "Equal",
        Key::KEY_LEFTBRACE => "BracketLeft",
        Key::KEY_RIGHTBRACE => "BracketRight",
        Key::KEY_BACKSLASH => "Backslash",
        Key::KEY_SEMICOLON => "Semicolon",
        Key::KEY_APOSTROPHE => "Quote",
        Key::KEY_GRAVE => "Backquote",
        Key::KEY_COMMA => "Comma",
        Key::KEY_DOT => "Period",
        Key::KEY_SLASH => "Slash",
        
        _ => "",
    }
}

