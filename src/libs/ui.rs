use crate::libs::keyboard::start_keyboard_listener;
use crate::libs::routes::Route;
use crate::libs::AudioContext;
use crate::state::keyboard::KeyboardState;
use dioxus::prelude::*;
use std::sync::mpsc;
use std::sync::Arc;

pub fn app() -> Element {
    // Get global app state from the global signal
    let app_state = crate::state::app::use_app_state();
    use_context_provider(|| app_state);

    // Create global keyboard state using signals
    let keyboard_state = use_signal(|| KeyboardState::new());

    // Provide the keyboard state context to all child components
    use_context_provider(|| keyboard_state);

    // Initialize the audio system for mechvibes sounds
    let audio_context = use_hook(|| Arc::new(AudioContext::new()));

    // Create a channel for real-time keyboard event communication
    let (tx, rx) = mpsc::channel::<String>();
    let rx = Arc::new(rx);

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

    // Process keyboard events and update both audio and UI state
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
                        println!("🔍 UI received key event: '{}'", keycode);
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
    rsx! {
      // Main application Router
      Router::<Route> {}
    }
}
