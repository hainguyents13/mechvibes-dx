use rdev::{listen, Button, Event, EventType};
use std::collections::HashSet;
use std::sync::{mpsc::Sender, Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

// Maps a mouse button to its standardized code
// This mapping ensures consistent button codes for audio playback
// across different platforms and configurations.
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

// Start listening to mouse events and send button press events to audio system
pub fn start_mouse_listener(
    play_sound_tx: Sender<String>,
    pressed_buttons: Arc<Mutex<HashSet<String>>>,
) {
    println!("🖱️ Starting mouse listener...");

    thread::spawn(move || {
        let last_press: Arc<Mutex<Instant>> = Arc::new(Mutex::new(Instant::now()));
        let result = listen(move |event: Event| {
            match event.event_type {
                EventType::ButtonPress(button) => {
                    let button_code = map_button_to_code(button);
                    if !button_code.is_empty() && button_code != "MouseUnknown" {
                        println!("🖱️ Mouse Button Pressed: {}", button_code);

                        // Add button to pressed set to track state
                        let mut pressed = pressed_buttons.lock().unwrap();
                        if pressed.contains(&button_code.to_string()) {
                            // Prevent duplicate events for held buttons
                            return;
                        }
                        pressed.insert(button_code.to_string());
                        drop(pressed); // Release lock early

                        let now = Instant::now();
                        let mut last = last_press.lock().unwrap();
                        if now.duration_since(*last) > Duration::from_millis(1) {
                            *last = now;
                            let _ = play_sound_tx.send(button_code.to_string());
                        }
                    }
                }
                EventType::ButtonRelease(button) => {
                    let button_code = map_button_to_code(button);
                    if !button_code.is_empty() && button_code != "MouseUnknown" {
                        // Remove button from pressed set
                        let mut pressed = pressed_buttons.lock().unwrap();
                        pressed.remove(&button_code.to_string());
                        drop(pressed); // Release lock early

                        println!("🖱️ Mouse Button Released: {}", button_code);
                        let _ = play_sound_tx.send(format!("UP:{}", button_code));
                    }
                }
                EventType::Wheel {
                    delta_x: _,
                    delta_y,
                } => {
                    // Handle mouse wheel events
                    let wheel_event = if delta_y > 0 {
                        "MouseWheelUp"
                    } else if delta_y < 0 {
                        "MouseWheelDown"
                    } else {
                        return; // No vertical scroll, ignore
                    };

                    println!("🖱️ Mouse Wheel: {}", wheel_event);

                    let now = Instant::now();
                    let mut last = last_press.lock().unwrap();
                    if now.duration_since(*last) > Duration::from_millis(50) {
                        // Longer delay for wheel
                        *last = now;
                        let _ = play_sound_tx.send(wheel_event.to_string());
                    }
                }
                _ => {}
            }
        });

        if let Err(error) = result {
            eprintln!("❌ Mouse listener error: {:?}", error);
        }
    });
}
