use crate::components::window_controller::WindowController;
use crate::components::header::Header;
use crate::libs::routes::Route;
use crate::libs::tray_service::request_tray_update;
use crate::libs::input_manager::get_input_channels;
use crate::libs::AudioContext;
use crate::state::keyboard::KeyboardState;
use crate::utils::delay;
use crate::utils::path;
use crate::utils::constants::APP_NAME;
use crate::{ debug_print, always_eprint };

use dioxus::prelude::*;
use dioxus_router::prelude::Router;
use dioxus::desktop::{ use_asset_handler, wry::http::Response };
use notify_rust::Notification;
use std::sync::Arc;

pub fn app() -> Element {
    // Get input channels from global state (initialized in main)
    let input_channels = get_input_channels();

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
            debug_print!("🎵 Loading current soundpacks on startup...");
            crate::state::app::reload_current_soundpacks(&ctx);
        });
    }

    // Extract receivers from input channels
    let keyboard_rx = input_channels.keyboard_rx.clone();
    let mouse_rx = input_channels.mouse_rx.clone();
    let hotkey_rx = input_channels.hotkey_rx.clone(); // Process keyboard events and update both audio and UI state
    {
        let ctx = audio_context.clone();
        let keyboard_rx = keyboard_rx.clone();
        let mut keyboard_state = keyboard_state;

        use_future(move || {
            let ctx = ctx.clone();
            let keyboard_rx = keyboard_rx.clone();

            async move {
                loop {
                    if let Ok(receiver) = keyboard_rx.try_lock() {
                        if let Ok(keycode) = receiver.try_recv() {
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
                    }
                    delay::Delay::key_event().await;
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
                    if let Ok(receiver) = mouse_rx.try_lock() {
                        if let Ok(button_code) = receiver.try_recv() {
                            if button_code.starts_with("UP:") {
                                let button = &button_code[3..];
                                ctx.play_mouse_event_sound(button, false);
                            } else if !button_code.is_empty() {
                                ctx.play_mouse_event_sound(&button_code, true);
                            }
                        }
                    }
                    delay::Delay::key_event().await;
                }
            }
        });
    }

    // Process hotkey Ctrl+Alt+M to toggle global sound
    {
        let hotkey_rx = hotkey_rx.clone();

        // Create signals for debounced notification system using atomic counter pattern
        let notification_counter = use_signal(|| Arc::new(std::sync::atomic::AtomicU64::new(0)));

        use_future(move || {
            let hotkey_rx = hotkey_rx.clone();
            async move {
                loop {
                    if let Ok(receiver) = hotkey_rx.try_lock() {
                        if let Ok(hotkey_command) = receiver.try_recv() {
                            if hotkey_command == "TOGGLE_SOUND" {
                                // Load current config, toggle enable_sound, and save
                                let mut config = crate::state::config::AppConfig::load();
                                config.enable_sound = !config.enable_sound;
                                config.last_updated = chrono::Utc::now();
                                match config.save() {
                                    Ok(_) => {
                                        // Request tray menu update to reflect the new sound state
                                        request_tray_update();

                                        // Handle debounced notifications
                                        if config.show_notifications {
                                            // Increment counter to invalidate previous notification tasks
                                            let current_task_id =
                                                notification_counter().fetch_add(
                                                    1,
                                                    std::sync::atomic::Ordering::SeqCst
                                                ) + 1;

                                            // Store the current sound state for the delayed notification
                                            let current_state = config.enable_sound;
                                            let notification_counter_clone = notification_counter();

                                            // Start a new delayed notification task
                                            spawn(async move {
                                                // Wait for 1s
                                                delay::Delay::ms(1000).await;

                                                // Check if this task is still the latest one
                                                if
                                                    notification_counter_clone.load(
                                                        std::sync::atomic::Ordering::SeqCst
                                                    ) == current_task_id
                                                {
                                                    // Show notification with the final state
                                                    let message = if current_state {
                                                        "Global sound enabled"
                                                    } else {
                                                        "Global sound disabled"
                                                    };
                                                    let _ = Notification::new()
                                                        .summary(APP_NAME)
                                                        .body(message)
                                                        .timeout(3000) // 3 seconds
                                                        .show();
                                                } else {
                                                    debug_print!("🚫 Notification task {} cancelled due to newer hotkey press", current_task_id);
                                                }
                                            });
                                        }
                                    }
                                    Err(e) => {
                                        always_eprint!("❌ Failed to save config after sound toggle: {}", e);
                                    }
                                }
                            }
                        }
                    }
                    delay::Delay::key_event().await;
                }
            }
        });
    }

    // Set up asset handler for serving soundpack images
    use_asset_handler("soundpack-images", |request, response| {
        let request_path = request.uri().path();

        // Parse the path: /soundpack-images/{soundpack_id}/{filename}
        let path_parts: Vec<&str> = request_path.trim_start_matches('/').split('/').collect();

        if path_parts.len() == 3 && path_parts[0] == "soundpack-images" {
            let soundpack_id = path_parts[1];
            let filename = path_parts[2];

            // Get the soundpack directory path using the correct function
            let soundpacks_path = std::path::PathBuf::from(path::get_soundpacks_dir_absolute());
            let image_path = soundpacks_path.join(soundpack_id).join(filename);

            if image_path.exists() {
                // Read the file and determine content type
                match std::fs::read(&image_path) {
                    Ok(data) => {
                        let mut response_builder = Response::builder();

                        let content_type = match
                            image_path.extension().and_then(|ext| ext.to_str())
                        {
                            Some("png") => "image/png",
                            Some("jpg") | Some("jpeg") => "image/jpeg",
                            Some("gif") => "image/gif",
                            Some("svg") => "image/svg+xml",
                            Some("webp") => "image/webp",
                            Some("ico") => "image/x-icon",
                            _ => "application/octet-stream",
                        };

                        response_builder = response_builder
                            .header("Content-Type", content_type)
                            .header("Cache-Control", "public, max-age=3600");

                        if let Ok(http_response) = response_builder.body(data) {
                            response.respond(http_response);
                            return;
                        }
                    }
                    Err(e) => {
                        debug_print!(
                            "❌ Failed to read soundpack image file {:?}: {}",
                            image_path,
                            e
                        );
                    }
                }
            }
        }

        // Return 404 for invalid paths or missing files
        if
            let Ok(not_found_response) = Response::builder()
                .status(404)
                .header("Content-Type", "text/plain")
                .body(b"Not Found".to_vec())
        {
            response.respond(not_found_response);
        }
    });

    rsx! {
      // prettier-ignore
      WindowController {}
      // prettier-ignore
      Header {}
      Router::<Route> {}
    }
}
