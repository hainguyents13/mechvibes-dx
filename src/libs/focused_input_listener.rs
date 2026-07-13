use device_query::{ DeviceQuery, DeviceState, Keycode };
use std::collections::HashSet;
use std::sync::{ mpsc::Sender, Arc, Mutex };
use std::thread;
use std::time::Duration;

/// Maps device_query Keycode to our standardized key code format (same as rdev)
fn map_device_query_keycode(key: Keycode) -> &'static str {
    match key {
        // Letters
        Keycode::A => "KeyA",
        Keycode::B => "KeyB",
        Keycode::C => "KeyC",
        Keycode::D => "KeyD",
        Keycode::E => "KeyE",
        Keycode::F => "KeyF",
        Keycode::G => "KeyG",
        Keycode::H => "KeyH",
        Keycode::I => "KeyI",
        Keycode::J => "KeyJ",
        Keycode::K => "KeyK",
        Keycode::L => "KeyL",
        Keycode::M => "KeyM",
        Keycode::N => "KeyN",
        Keycode::O => "KeyO",
        Keycode::P => "KeyP",
        Keycode::Q => "KeyQ",
        Keycode::R => "KeyR",
        Keycode::S => "KeyS",
        Keycode::T => "KeyT",
        Keycode::U => "KeyU",
        Keycode::V => "KeyV",
        Keycode::W => "KeyW",
        Keycode::X => "KeyX",
        Keycode::Y => "KeyY",
        Keycode::Z => "KeyZ",

        // Numbers
        Keycode::Key0 => "Digit0",
        Keycode::Key1 => "Digit1",
        Keycode::Key2 => "Digit2",
        Keycode::Key3 => "Digit3",
        Keycode::Key4 => "Digit4",
        Keycode::Key5 => "Digit5",
        Keycode::Key6 => "Digit6",
        Keycode::Key7 => "Digit7",
        Keycode::Key8 => "Digit8",
        Keycode::Key9 => "Digit9",

        // Special keys
        Keycode::Space => "Space",
        Keycode::Backspace => "Backspace",
        Keycode::Enter => "Enter",
        Keycode::Tab => "Tab",
        Keycode::Escape => "Escape",
        Keycode::Delete => "Delete",
        Keycode::Insert => "Insert",

        // Modifiers
        Keycode::LShift => "ShiftLeft",
        Keycode::RShift => "ShiftRight",
        Keycode::LControl => "ControlLeft",
        Keycode::RControl => "ControlRight",
        Keycode::LAlt => "AltLeft",
        Keycode::RAlt => "AltRight",
        Keycode::LMeta => "MetaLeft",
        Keycode::RMeta => "MetaRight",

        // Arrow keys
        Keycode::Up => "ArrowUp",
        Keycode::Down => "ArrowDown",
        Keycode::Left => "ArrowLeft",
        Keycode::Right => "ArrowRight",

        // Navigation
        Keycode::Home => "Home",
        Keycode::End => "End",
        Keycode::PageUp => "PageUp",
        Keycode::PageDown => "PageDown",

        // Function keys
        Keycode::F1 => "F1",
        Keycode::F2 => "F2",
        Keycode::F3 => "F3",
        Keycode::F4 => "F4",
        Keycode::F5 => "F5",
        Keycode::F6 => "F6",
        Keycode::F7 => "F7",
        Keycode::F8 => "F8",
        Keycode::F9 => "F9",
        Keycode::F10 => "F10",
        Keycode::F11 => "F11",
        Keycode::F12 => "F12",

        // Punctuation
        Keycode::Minus => "Minus",
        Keycode::Equal => "Equal",
        Keycode::LeftBracket => "BracketLeft",
        Keycode::RightBracket => "BracketRight",
        Keycode::BackSlash => "Backslash",
        Keycode::Semicolon => "Semicolon",
        Keycode::Apostrophe => "Quote",
        Keycode::Grave => "Backquote",
        Keycode::Comma => "Comma",
        Keycode::Dot => "Period",
        Keycode::Slash => "Slash",

        // Numpad
        Keycode::Numpad0 => "Numpad0",
        Keycode::Numpad1 => "Numpad1",
        Keycode::Numpad2 => "Numpad2",
        Keycode::Numpad3 => "Numpad3",
        Keycode::Numpad4 => "Numpad4",
        Keycode::Numpad5 => "Numpad5",
        Keycode::Numpad6 => "Numpad6",
        Keycode::Numpad7 => "Numpad7",
        Keycode::Numpad8 => "Numpad8",
        Keycode::Numpad9 => "Numpad9",

        _ => "",
    }
}

fn is_ctrl(key: &str) -> bool {
    key == "ControlLeft" || key == "ControlRight"
}

fn is_alt(key: &str) -> bool {
    key == "AltLeft" || key == "AltRight"
}

/// Start a keyboard poller using device_query.
///
/// - When `active_when` is `Some(state)`, polling runs only while `*state == true`
///   (used on Windows/Linux X11 so rdev can own the unfocused path without duplicates).
/// - When `active_when` is `None`, polling always runs (used on macOS where rdev often
///   misses keyboard events while the app is unfocused/minimized).
///
/// If `hotkey_tx` is provided, Ctrl+Alt+M is detected here (needed when rdev keyboard
/// is suppressed).
pub fn start_focused_keyboard_listener(
    keyboard_tx: Sender<String>,
    active_when: Option<Arc<Mutex<bool>>>,
    hotkey_tx: Option<Sender<String>>,
) {
    thread::spawn(move || {
        let mode = if active_when.is_some() {
            "gated by focus"
        } else {
            "always on"
        };
        println!("🎮 Starting keyboard poller (device_query, {mode})...");

        let device_state_result = std::panic::catch_unwind(|| DeviceState::new());

        let device_state = match device_state_result {
            Ok(ds) => ds,
            Err(_) => {
                eprintln!(
                    "⚠️ WARNING: Failed to initialize device_query DeviceState. \
                     This is likely because the app lacks macOS Accessibility Permissions."
                );
                crate::libs::input_manager::set_accessibility_permissions(false);
                return;
            }
        };

        let mut prev_keys: HashSet<Keycode> = HashSet::new();
        let mut last_status_log = std::time::Instant::now();
        let mut ctrl_pressed = false;
        let mut alt_pressed = false;

        loop {
            let active = match &active_when {
                Some(state) => *state.lock().unwrap(),
                None => true,
            };

            if last_status_log.elapsed().as_secs() >= 5 {
                println!(
                    "🔍 [device_query] polling active: {} ({})",
                    active,
                    mode
                );
                last_status_log = std::time::Instant::now();
            }

            if !active {
                // Clear edge-detect state so re-focus doesn't replay held keys as presses
                prev_keys.clear();
                ctrl_pressed = false;
                alt_pressed = false;
                thread::sleep(Duration::from_millis(100));
                continue;
            }

            let keys = device_state.get_keys();
            let current_keys: HashSet<Keycode> = keys.into_iter().collect();

            // Newly pressed keys
            for key in current_keys.difference(&prev_keys) {
                let key_code = map_device_query_keycode(*key);
                if key_code.is_empty() {
                    continue;
                }

                if is_ctrl(key_code) {
                    ctrl_pressed = true;
                } else if is_alt(key_code) {
                    alt_pressed = true;
                } else if key_code == "KeyM" {
                    if let Some(ref hotkey_tx) = hotkey_tx {
                        if ctrl_pressed && alt_pressed {
                            println!("🔥 Hotkey detected (device_query): Ctrl+Alt+M - Toggling global sound");
                            let _ = hotkey_tx.send("TOGGLE_SOUND".to_string());
                            crate::libs::input_manager::add_last_event(
                                "🔥 Hotkey: Ctrl+Alt+M".to_string(),
                            );
                            // Don't play a normal key sound for the toggle combo
                            continue;
                        }
                    }
                }

                crate::libs::input_manager::add_last_event(format!("⌨️ KeyDown: {}", key_code));
                let _ = keyboard_tx.send(key_code.to_string());
            }

            // Released keys
            for key in prev_keys.difference(&current_keys) {
                let key_code = map_device_query_keycode(*key);
                if key_code.is_empty() {
                    continue;
                }

                if is_ctrl(key_code) {
                    ctrl_pressed = false;
                } else if is_alt(key_code) {
                    alt_pressed = false;
                }

                crate::libs::input_manager::add_last_event(format!("⌨️ KeyUp: {}", key_code));
                let _ = keyboard_tx.send(format!("UP:{}", key_code));
            }

            prev_keys = current_keys;
            thread::sleep(Duration::from_millis(10));
        }
    });
}
