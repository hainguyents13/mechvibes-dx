use crate::components::window_controller::WindowController;
use crate::components::header::Header;
use crate::libs::routes::Route;
use crate::libs::tray_service::request_tray_update;
use crate::libs::input_manager::{ get_input_channels, set_window_focus };
use crate::libs::AudioContext;
use crate::state::keyboard::KeyboardState;
use crate::state::paths;
use crate::utils::delay;
use crate::{ debug_print, always_eprint };

use dioxus::prelude::*;
use dioxus::desktop::{ use_asset_handler, use_wry_event_handler, wry::http::Response };
use dioxus::desktop::tao::event::{ Event as TaoEvent };
use std::sync::Arc;

pub fn app() -> Element {
    // Loading state to prevent FOUC
    let mut is_loading = use_signal(|| true);

    // Hide content until CSS loads
    use_effect(move || {
        spawn(async move {
            // Wait for CSS to load
            futures_timer::Delay::new(std::time::Duration::from_millis(100)).await;
            is_loading.set(false);
        });
    });

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

    // Check for updates on startup (from completely closed state)
    use_effect(move || {
        spawn(async move {
            if
                let Ok(update_info) =
                    crate::utils::auto_updater::check_for_updates_on_startup().await
            {
                debug_print!("🔄 Startup update check completed");
                if update_info.update_available {
                    debug_print!("🆕 Update available on startup: {}", update_info.latest_version);
                }
            }
        });
    });

    // Extract receivers from input channels
    let keyboard_rx = input_channels.keyboard_rx.clone();
    let mouse_rx = input_channels.mouse_rx.clone();
    let hotkey_rx = input_channels.hotkey_rx.clone();

    // ===== WINDOW FOCUS TRACKING =====
    // Track window focus state to switch between rdev (unfocused) and device_query (focused)
    // This is a hybrid approach to work around the rdev + Wry/Winit incompatibility on Windows
    {
        use dioxus::desktop::tao::event::WindowEvent;

        use_wry_event_handler(move |event, _target| {
            if let TaoEvent::WindowEvent { event: window_event, .. } = event {
                // Check for focus events using proper pattern matching
                if let WindowEvent::Focused(focused) = window_event {
                    // Update global focus state
                    set_window_focus(*focused);
                }
            }
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
    } // Process hotkey Ctrl+Alt+M to toggle global sound
    {
        let hotkey_rx = hotkey_rx.clone();

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
                                        debug_print!("🔄 Sound toggled: {}", config.enable_sound);
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
    } // Initialize update service for background update checking
    #[cfg(feature = "auto-update")]
    {
        use crate::utils::auto_updater::UpdateService;
        use std::sync::Arc;
        use tokio::sync::Mutex;
        use_future(move || async move {
            let config = Arc::new(Mutex::new(crate::state::config::AppConfig::load()));
            let update_service = UpdateService::new(config);

            // Start background update checking
            update_service.start().await;
        });
    } // Set up asset handler for serving soundpack images
    use_asset_handler("soundpack-images", |request, response| {
        let request_path = request.uri().path();

        // Parse the path: /soundpack-images/{device_type}/{soundpack_name}/{filename}
        let path_parts: Vec<&str> = request_path.trim_start_matches('/').split('/').collect();

        if path_parts.len() >= 4 && path_parts[0] == "soundpack-images" {
            let device_type = path_parts[1];
            let soundpack_name = path_parts[2];
            let filename = path_parts[3];

            // Security: Validate path segments to prevent directory traversal
            // Reject empty segments, path separators, and parent directory references
            if device_type.is_empty() || soundpack_name.is_empty() || filename.is_empty() {
                let error_response = Response::builder()
                    .status(400)
                    .body(Vec::new())
                    .unwrap();
                response.respond(error_response);
                return;
            }

            if device_type.contains("..") || device_type.contains('/') || device_type.contains('\\')
                || soundpack_name.contains("..") || soundpack_name.contains('/') || soundpack_name.contains('\\')
                || filename.contains("..") || filename.contains('/') || filename.contains('\\')
            {
                let error_response = Response::builder()
                    .status(400)
                    .body(Vec::new())
                    .unwrap();
                response.respond(error_response);
                return;
            }

            // Join device_type/soundpack_name to form the full soundpack_id
            let soundpack_id = format!("{}/{}", device_type, soundpack_name);

            // Get the soundpack directory path - supports both built-in and custom locations
            let soundpack_dir = paths::soundpacks::soundpack_dir(&soundpack_id);
            let image_path = std::path::PathBuf::from(&soundpack_dir).join(filename);

            if image_path.exists() {
                // Read the file and determine content type
                match std::fs::read(&image_path) {
                    Ok(data) => {
                        let mut response_builder = Response::builder();

                        // Get extension and convert to lowercase for case-insensitive matching
                        let extension = image_path
                            .extension()
                            .and_then(|ext| ext.to_str())
                            .map(|ext| ext.to_lowercase());

                        let content_type = match extension.as_deref() {
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

    // Set up asset handler for serving custom user images
    use_asset_handler("custom-images", |request, response| {
        let request_path = request.uri().path();

        // Parse the path: /custom-images/{filename}
        let path_parts: Vec<&str> = request_path.trim_start_matches('/').split('/').collect();

        if path_parts.len() >= 2 && path_parts[0] == "custom-images" {
            let filename = path_parts[1];

            // Security: Reject empty filenames (e.g., trailing slash /custom-images/)
            if filename.is_empty() {
                let error_response = Response::builder()
                    .status(400)
                    .body(Vec::new())
                    .unwrap();
                response.respond(error_response);
                return;
            }

            // Security: Validate filename to prevent directory traversal
            // Reject path separators and parent directory references
            if filename.contains("..") || filename.contains('/') || filename.contains('\\') {
                let error_response = Response::builder()
                    .status(400)
                    .body(Vec::new())
                    .unwrap();
                response.respond(error_response);
                return;
            }

            // Get the custom images directory path
            let custom_images_dir = crate::state::paths::data::custom_images_dir();
            let image_path = custom_images_dir.join(filename);

            // Security: Ensure the resolved path is still within custom_images_dir
            if let Ok(canonical_path) = image_path.canonicalize() {
                if let Ok(canonical_base) = custom_images_dir.canonicalize() {
                    if !canonical_path.starts_with(&canonical_base) {
                        let error_response = Response::builder()
                            .status(403)
                            .body(Vec::new())
                            .unwrap();
                        response.respond(error_response);
                        return;
                    }
                }
            }

            if image_path.exists() {
                // Read the file and determine content type
                match std::fs::read(&image_path) {
                    Ok(data) => {
                        let mut response_builder = Response::builder();

                        // Get extension and convert to lowercase for case-insensitive matching
                        let extension = image_path
                            .extension()
                            .and_then(|ext| ext.to_str())
                            .map(|ext| ext.to_lowercase());

                        let content_type = match extension.as_deref() {
                            Some("png") => "image/png",
                            Some("jpg") | Some("jpeg") => "image/jpeg",
                            Some("gif") => "image/gif",
                            Some("svg") => "image/svg+xml",
                            Some("webp") => "image/webp",
                            Some("bmp") => "image/bmp",
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
                            "❌ Failed to read custom image file {:?}: {}",
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
        // Loading overlay (shown while CSS loads)
        if is_loading() {
            div {
                style: "position: fixed; inset: 0; z-index: 99999; display: flex; align-items: center; justify-content: center; background: #1a1a1a;",
                div {
                    style: "display: flex; flex-direction: column; align-items: center; gap: 1rem;",
                    div {
                        style: "width: 3rem; height: 3rem; border: 4px solid rgba(255, 255, 255, 0.2); border-top-color: rgba(255, 255, 255, 0.9); border-radius: 50%; animation: spin 1s linear infinite;",
                    }
                    div {
                        style: "font-size: 0.875rem; font-weight: 500; color: rgba(255, 255, 255, 0.7);",
                        "Loading..."
                    }
                }
            }
            // Add keyframes for spinner animation
            style { "@keyframes spin {{ to {{ transform: rotate(360deg); }} }}" }
        }

        // Main app content
        // prettier-ignore
        WindowController {}
        // prettier-ignore
        Header {}

        Router::<Route> {}
    }
}
