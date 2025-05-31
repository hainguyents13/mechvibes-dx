use crate::libs::keyboard::start_keyboard_listener;
use crate::libs::mouse::start_mouse_listener;
use crate::libs::routes::Route;
use crate::libs::AudioContext;
use crate::state::keyboard::KeyboardState;
use crate::Header;
use dioxus::prelude::*;
use std::collections::HashSet;
use std::sync::{mpsc, Arc, Mutex};

pub fn app() -> Element {
    // Get global app state from the global signal
    let app_state = crate::state::app::use_app_state();
    use_context_provider(|| app_state);

    // Create global keyboard state using signals
    let keyboard_state = use_signal(|| KeyboardState::new()); // Provide the keyboard state context to all child components
    use_context_provider(|| keyboard_state); // Initialize the audio system for mechvibes sounds - moved here to be accessible by both keyboard processing and UI
    let audio_context = use_hook(|| Arc::new(AudioContext::new()));

    // Provide audio context to all child components (this will be used by Layout and other components)
    use_context_provider(|| audio_context.clone());

    // Load current soundpack on startup
    {
        let ctx = audio_context.clone();
        use_effect(move || {
            println!("🎵 Loading current soundpack on startup...");
            crate::state::app::reload_current_soundpack(&ctx);
        });
    } // Create a channel for real-time keyboard event communication
    let (tx, rx) = mpsc::channel::<String>();
    let rx = Arc::new(rx);

    // Create a channel for real-time mouse event communication
    let (mouse_tx, mouse_rx) = mpsc::channel::<String>();
    let mouse_rx = Arc::new(mouse_rx);

    // Launch the keyboard event listener in a background task
    {
        let tx = tx.clone();
        use_future(move || {
            let tx = tx.clone();
            async move {
                spawn(async move {
                    start_keyboard_listener(tx);
                });
            }
        });
    }

    // Launch the mouse event listener in a background task
    {
        let mouse_tx = mouse_tx.clone();
        use_future(move || {
            let mouse_tx = mouse_tx.clone();
            async move {
                spawn(async move {
                    let pressed_buttons = Arc::new(Mutex::new(HashSet::<String>::new()));
                    start_mouse_listener(mouse_tx, pressed_buttons);
                });
            }
        });
    } // Process keyboard events and update both audio and UI state
    {
        let ctx = audio_context.clone();
        let rx = rx.clone();
        let mut keyboard_state = keyboard_state;

        use_future(move || {
            let ctx = ctx.clone();
            let rx = rx.clone();

            async move {
                loop {
                    if let Ok(keycode) = rx.try_recv() {
                        if keycode.starts_with("UP:") {
                            let key = &keycode[3..];
                            ctx.play_key_event_sound(key, false);

                            // Update keyboard state - key released
                            keyboard_state.write().key_pressed = false;
                        } else if !keycode.is_empty() {
                            ctx.play_key_event_sound(&keycode, true);
                            // Update keyboard state - key pressed
                            let mut state = keyboard_state.write();
                            state.key_pressed = true;
                            state.last_key = keycode.clone();
                        }
                    }
                    futures_timer::Delay::new(std::time::Duration::from_millis(1)).await;
                }
            }
        });
    }

    // Process mouse events and play sounds
    {
        let ctx = audio_context.clone();
        let mouse_rx = mouse_rx.clone();

        use_future(move || {
            let ctx = ctx.clone();
            let mouse_rx = mouse_rx.clone();

            async move {
                loop {
                    if let Ok(button_code) = mouse_rx.try_recv() {
                        if button_code.starts_with("UP:") {
                            let button = &button_code[3..];
                            ctx.play_mouse_event_sound(button, false);
                        } else if !button_code.is_empty() {
                            ctx.play_mouse_event_sound(&button_code, true);
                        }
                    }
                    futures_timer::Delay::new(std::time::Duration::from_millis(1)).await;
                }
            }
        });
    }

    rsx! {
      Header {}
      // Main application Router
      Router::<Route> {}
    }
}
