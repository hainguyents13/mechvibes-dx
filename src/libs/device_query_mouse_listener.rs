use device_query::{ DeviceQuery, DeviceState };
use std::sync::mpsc::Sender;
use std::thread;
use std::time::Duration;

/// Start a mouse button poller using device_query.
///
/// Polls `device_state.get_mouse()` every 10ms and detects button
/// press/release edges. Sends standardized button codes to `mouse_tx`.
///
/// On macOS this is the cleanest approach: no CGEventTap at all, so
/// `device_query` keyboard polling is unaffected.
pub fn start_device_query_mouse_listener(mouse_tx: Sender<String>) {
    thread::spawn(move || {
        println!("🎮 Starting mouse poller (device_query, always on)...");

        let device_state_result = std::panic::catch_unwind(|| DeviceState::new());
        let device_state = match device_state_result {
            Ok(ds) => ds,
            Err(_) => {
                eprintln!("⚠️ WARNING: Failed to initialize device_query DeviceState for mouse polling.");
                return;
            }
        };

        let mut prev_buttons: Vec<bool> = vec![false, false, false, false, false];
        let button_names = ["", "MouseLeft", "MouseRight", "MouseMiddle", "", ""];

        loop {
            let mouse = device_state.get_mouse();

            for (idx, (prev, curr)) in prev_buttons.iter().zip(mouse.button_pressed.iter()).enumerate() {
                let name = match button_names.get(idx) {
                    Some(n) if !n.is_empty() => *n,
                    _ => continue,
                };

                if *curr && !prev {
                    crate::libs::input_manager::add_last_event(format!("🖱️ MouseDown: {}", name));
                    let _ = mouse_tx.send(name.to_string());
                } else if !*curr && *prev {
                    crate::libs::input_manager::add_last_event(format!("🖱️ MouseUp: {}", name));
                    let _ = mouse_tx.send(format!("UP:{}", name));
                }
            }

            prev_buttons = mouse.button_pressed.clone();
            thread::sleep(Duration::from_millis(10));
        }
    });
}
