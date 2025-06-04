use crate::libs::input_listener::start_unified_input_listener;
use crate::libs::routes::Route;
use crate::libs::AudioContext;
use crate::state::keyboard::KeyboardState;
use crate::Header;
use dioxus::prelude::*;
use notify_rust::Notification;
use std::sync::{mpsc, Arc};
use std::time::Duration;

pub fn app() -> Element {
    // Create update signal for event-driven state management
    let update_signal = use_signal(|| 0u32);
    use_context_provider(|| update_signal);

    // Create global keyboard state using signals
    let keyboard_state = use_signal(|| KeyboardState::new());

    // Provide the keyboard state context to all child components
    use_context_provider(|| keyboard_state);

    // Initialize the audio system for mechvibes sounds - moved here to be accessible by both keyboard processing and UI
    let audio_context = use_hook(|| Arc::new(AudioContext::new()));

    // Provide audio context to all child components (this will be used by Layout and other components)
    use_context_provider(|| audio_context.clone());
    {
        // Load current soundpacks on startup
        let ctx = audio_context.clone();
        use_effect(move || {
            println!("🎵 Loading current soundpacks on startup...");
            crate::state::app::reload_current_soundpacks(&ctx);
        });
    }

    // Create channels for real-time input event communication
    let (keyboard_tx, keyboard_rx) = mpsc::channel::<String>();
    let (mouse_tx, mouse_rx) = mpsc::channel::<String>();
    let (hotkey_tx, hotkey_rx) = mpsc::channel::<String>();
    let keyboard_rx = Arc::new(keyboard_rx);
    let mouse_rx = Arc::new(mouse_rx);
    let hotkey_rx = Arc::new(hotkey_rx);

    // Launch the unified input listener (handles both keyboard and mouse)
    {
        use_effect(move || {
            let keyboard_tx = keyboard_tx.clone();
            let mouse_tx = mouse_tx.clone();
            let hotkey_tx = hotkey_tx.clone();
            spawn(async move {
                start_unified_input_listener(keyboard_tx, mouse_tx, hotkey_tx);
            });
        });
    }

    // Process keyboard events and update both audio and UI state
    {
        let ctx = audio_context.clone();
        let keyboard_rx = keyboard_rx.clone();
        let mut keyboard_state = keyboard_state;

        use_future(move || {
            let ctx = ctx.clone();
            let keyboard_rx = keyboard_rx.clone();

            async move {
                loop {
                    if let Ok(keycode) = keyboard_rx.try_recv() {
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
                    futures_timer::Delay::new(std::time::Duration::from_millis(20)).await;
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
                    futures_timer::Delay::new(std::time::Duration::from_millis(20)).await;
                }
            }
        });
    } // Process hotkey Ctrl+Alt+M to toggle global sound
    {
        let hotkey_rx = hotkey_rx.clone();

        // Create signals for debounced notification system using atomic counter pattern
        let notification_counter = use_signal(|| Arc::new(std::sync::atomic::AtomicU64::new(0)));

        use_future(move || {
            let hotkey_rx = hotkey_rx.clone();
            async move {
                loop {
                    if let Ok(hotkey_command) = hotkey_rx.try_recv() {
                        if hotkey_command == "TOGGLE_SOUND" {
                            // Load current config, toggle enable_sound, and save
                            let mut config = crate::state::config::AppConfig::load();
                            let old_state = config.enable_sound;
                            config.enable_sound = !config.enable_sound;
                            config.last_updated = chrono::Utc::now();
                            match config.save() {
                                Ok(_) => {
                                    let status = if config.enable_sound {
                                        "ENABLED"
                                    } else {
                                        "DISABLED"
                                    };
                                    println!(
                                        "🔊 HOTKEY TRIGGERED: Global sound toggled from {} to {}",
                                        if old_state { "ENABLED" } else { "DISABLED" },
                                        status
                                    );

                                    // Handle debounced notifications
                                    if config.show_notifications {
                                        // Increment counter to invalidate previous notification tasks
                                        let current_task_id = notification_counter()
                                            .fetch_add(1, std::sync::atomic::Ordering::SeqCst)
                                            + 1;

                                        println!(
                                            "🔔 Scheduling debounced notification (task ID: {})",
                                            current_task_id
                                        );

                                        // Store the current sound state for the delayed notification
                                        let current_state = config.enable_sound;
                                        let notification_counter_clone = notification_counter();

                                        // Start a new delayed notification task
                                        spawn(async move {
                                            // Wait for 1s
                                            futures_timer::Delay::new(Duration::from_secs(1)).await;

                                            // Check if this task is still the latest one
                                            if notification_counter_clone
                                                .load(std::sync::atomic::Ordering::SeqCst)
                                                == current_task_id
                                            {
                                                // Show notification with the final state
                                                let message = if current_state {
                                                    "Global sound enabled"
                                                } else {
                                                    "Global sound disabled"
                                                };

                                                if let Err(e) = Notification::new()
                                                    .summary("MechvibesDX")
                                                    .body(message)
                                                    .timeout(3000) // 3 seconds
                                                    .show()
                                                {
                                                    eprintln!(
                                                        "❌ Failed to show notification: {}",
                                                        e
                                                    );
                                                } else {
                                                    println!(
                                                        "✅ Debounced notification shown: {}",
                                                        message
                                                    );
                                                }
                                            } else {
                                                println!("🚫 Notification task {} cancelled due to newer hotkey press", current_task_id);
                                            }
                                        });
                                    }
                                }
                                Err(e) => {
                                    eprintln!("❌ Failed to save config after sound toggle: {}", e);
                                }
                            }
                        }
                    }
                    futures_timer::Delay::new(std::time::Duration::from_millis(10)).await;
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
