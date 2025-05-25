use crate::libs::keyboard::start_keyboard_listener;
use crate::libs::AudioContext;
use crate::state::keyboard::KeyboardState;
use crate::KeyboardChannel;
use dioxus::prelude::*;
use dioxus::prelude::{use_effect, use_future, use_hook, use_signal};
use dioxus_radio::prelude::*;
use std::sync::mpsc;
use std::sync::Arc;

pub fn app() -> Element {
    // Set up the global keyboard state management
    use_init_radio_station::<KeyboardState, KeyboardChannel>(|| KeyboardState::new());
    let radio = use_radio::<KeyboardState, KeyboardChannel>(KeyboardChannel::Main);

    // Initialize the audio system for mechanical keyboard sounds
    let audio_context = use_hook(|| Arc::new(AudioContext::new()));

    // Create a channel for real-time keyboard event communication
    let (tx, rx) = mpsc::channel::<String>();
    let rx = Arc::new(rx); // Launch the keyboard event listener in a background task
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
        let radio = radio.clone();

        use_future(move || {
            let ctx = ctx.clone();
            let rx = rx.clone();
            let mut radio = radio;

            async move {
                loop {
                    if let Ok(keycode) = rx.try_recv() {
                        println!("🔍 UI received key event: '{}'", keycode);
                        if keycode.starts_with("UP:") {
                            let key = &keycode[3..];
                            ctx.play_key_event_sound(key, false);

                            // Update keyboard state - key released
                            radio.write().key_pressed = false;
                        } else if !keycode.is_empty() {
                            ctx.play_key_event_sound(&keycode, true); // Update keyboard state - key pressed
                            let mut state = radio.write();
                            state.key_pressed = true;
                            state.last_key = keycode.clone();
                        }
                    }
                    futures_timer::Delay::new(std::time::Duration::from_millis(1)).await;
                }
            }
        });
    }

    // volume state
    // Default volume is 1.0 (100%)
    let volume = use_signal(|| 1.0f32); // Update audio system volume when the volume control changes
    let ctx = audio_context.clone();
    use_effect(move || {
        ctx.set_volume(volume());
    });

    // Render the main application interface
    rsx! {
      div { class: "container mx-auto p-16 text-center",
        // Mechanical keyboard logo with animated press effect
        crate::components::logo::Logo {}
        // Volume control slider for sound effects
        crate::components::volume_slider::VolumeSlider { volume } // Soundpack selector for switching sound packs in real-time
        crate::components::soundpack_selector::SoundpackSelector { audio_ctx: audio_context.clone() }

        // Button to test the current sound configuration
        crate::components::test_sound_button::TestSoundButton {}
      }
    }
}
