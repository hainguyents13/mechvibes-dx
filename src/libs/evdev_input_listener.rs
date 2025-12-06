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
        use evdev::{Device, EventType, Key};

        println!("🔍 [evdev] Starting Linux keyboard listener (Wayland/X11 compatible)");

        // Track modifier keys for hotkey detection
        let mut ctrl_pressed = false;
        let mut alt_pressed = false;

        // Find all keyboard devices
        let mut keyboards = Vec::new();
        
        match evdev::enumerate().map(|t| t.collect::<Vec<_>>()) {
            Ok(devices) => {
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
            }
            Err(e) => {
                eprintln!("❌ [evdev] Failed to enumerate devices: {}", e);
                eprintln!("💡 [evdev] Make sure you're in the 'input' group: sudo usermod -a -G input $USER");
                return;
            }
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

                                if let Ok(key) = Key::new(event.code()) {
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
    use evdev::Key::*;
    
    match key {
        // Letters
        KEY_A => "KeyA", KEY_B => "KeyB", KEY_C => "KeyC", KEY_D => "KeyD",
        KEY_E => "KeyE", KEY_F => "KeyF", KEY_G => "KeyG", KEY_H => "KeyH",
        KEY_I => "KeyI", KEY_J => "KeyJ", KEY_K => "KeyK", KEY_L => "KeyL",
        KEY_M => "KeyM", KEY_N => "KeyN", KEY_O => "KeyO", KEY_P => "KeyP",
        KEY_Q => "KeyQ", KEY_R => "KeyR", KEY_S => "KeyS", KEY_T => "KeyT",
        KEY_U => "KeyU", KEY_V => "KeyV", KEY_W => "KeyW", KEY_X => "KeyX",
        KEY_Y => "KeyY", KEY_Z => "KeyZ",
        
        // Numbers
        KEY_1 => "Digit1", KEY_2 => "Digit2", KEY_3 => "Digit3", KEY_4 => "Digit4",
        KEY_5 => "Digit5", KEY_6 => "Digit6", KEY_7 => "Digit7", KEY_8 => "Digit8",
        KEY_9 => "Digit9", KEY_0 => "Digit0",
        
        // Function keys
        KEY_F1 => "F1", KEY_F2 => "F2", KEY_F3 => "F3", KEY_F4 => "F4",
        KEY_F5 => "F5", KEY_F6 => "F6", KEY_F7 => "F7", KEY_F8 => "F8",
        KEY_F9 => "F9", KEY_F10 => "F10", KEY_F11 => "F11", KEY_F12 => "F12",
        
        // Special keys
        KEY_SPACE => "Space",
        KEY_ENTER => "Enter",
        KEY_BACKSPACE => "Backspace",
        KEY_TAB => "Tab",
        KEY_ESC => "Escape",
        KEY_CAPSLOCK => "CapsLock",
        KEY_LEFTSHIFT => "ShiftLeft",
        KEY_RIGHTSHIFT => "ShiftRight",
        KEY_LEFTCTRL => "ControlLeft",
        KEY_RIGHTCTRL => "ControlRight",
        KEY_LEFTALT => "AltLeft",
        KEY_RIGHTALT => "AltRight",
        KEY_LEFTMETA => "MetaLeft",
        KEY_RIGHTMETA => "MetaRight",
        
        // Arrow keys
        KEY_UP => "ArrowUp",
        KEY_DOWN => "ArrowDown",
        KEY_LEFT => "ArrowLeft",
        KEY_RIGHT => "ArrowRight",
        
        // Editing keys
        KEY_INSERT => "Insert",
        KEY_DELETE => "Delete",
        KEY_HOME => "Home",
        KEY_END => "End",
        KEY_PAGEUP => "PageUp",
        KEY_PAGEDOWN => "PageDown",
        
        // Punctuation
        KEY_MINUS => "Minus",
        KEY_EQUAL => "Equal",
        KEY_LEFTBRACE => "BracketLeft",
        KEY_RIGHTBRACE => "BracketRight",
        KEY_BACKSLASH => "Backslash",
        KEY_SEMICOLON => "Semicolon",
        KEY_APOSTROPHE => "Quote",
        KEY_GRAVE => "Backquote",
        KEY_COMMA => "Comma",
        KEY_DOT => "Period",
        KEY_SLASH => "Slash",
        
        _ => "",
    }
}

