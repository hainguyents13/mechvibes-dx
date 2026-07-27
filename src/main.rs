#![windows_subsystem = "windows"]
#![allow(non_snake_case)]

mod components;
mod libs;
mod state;
mod utils;

use dioxus::desktop::{ Config, LogicalSize, WindowBuilder };
use dioxus::prelude::*;
use utils::constants::{ APP_NAME };
use libs::ui;
use libs::window_manager::{ WindowAction, WINDOW_MANAGER };
use libs::focused_input_listener::start_focused_keyboard_listener;
#[allow(unused_imports)]
use libs::input_listener::start_unified_input_listener;
use libs::input_manager::{ init_input_channels, init_window_focus_state_with_value };
use std::sync::mpsc;

#[cfg(target_os = "linux")]
use libs::evdev_input_listener::start_evdev_keyboard_listener;

// Use .ico format for better Windows compatibility
const EMBEDDED_ICON: &[u8] = include_bytes!("../assets/icon.ico");

/// Keep the process active while backgrounded so input listeners are not App-Nap throttled.
#[cfg(target_os = "macos")]
#[allow(deprecated, unexpected_cfgs)]
fn disable_app_nap() {
    use cocoa::base::{id, nil};
    use objc::{class, msg_send, sel, sel_impl};

    unsafe {
        let process_info: id = msg_send![class!(NSProcessInfo), processInfo];
        // NSActivityUserInitiatedAllowingIdleSystemSleep = 0x00FFFFFFULL
        let options: u64 = 0x00FF_FFFF;
        let reason: id = msg_send![class!(NSString), stringWithUTF8String: b"MechvibesDX keyboard monitoring\0".as_ptr()];
        let _: id = msg_send![process_info, beginActivityWithOptions: options reason: reason];
        let _ = nil;
        debug_print!("🔋 Disabled App Nap for reliable background key capture");
    }
}

fn load_icon() -> Option<dioxus::desktop::tao::window::Icon> {
    // Try to create icon from embedded ICO data
    // Windows taskbar works best with 32x32 icons
    match image::load_from_memory_with_format(EMBEDDED_ICON, image::ImageFormat::Ico) {
        Ok(img) => {
            let rgba = img.to_rgba8();
            let (width, height) = rgba.dimensions();
            debug_print!("📐 Loaded icon from ICO: {}x{}", width, height);

            // Always resize to 32x32 for maximum Windows taskbar compatibility
            // This is the standard size Windows expects for taskbar icons
            let target_size = 32u32;

            let final_rgba = if width != target_size || height != target_size {
                debug_print!("🔄 Resizing icon from {}x{} to {}x{} for Windows taskbar", width, height, target_size, target_size);
                image::imageops::resize(&rgba, target_size, target_size, image::imageops::FilterType::Lanczos3)
            } else {
                debug_print!("✅ Icon already at optimal size ({}x{})", width, height);
                rgba
            };

            match dioxus::desktop::tao::window::Icon::from_rgba(final_rgba.into_raw(), target_size, target_size) {
                Ok(icon) => {
                    debug_print!("✅ Successfully created window icon ({}x{})", target_size, target_size);
                    Some(icon)
                }
                Err(e) => {
                    always_eprint!("❌ Failed to create window icon from RGBA data: {}", e);
                    None
                }
            }
        }
        Err(e) => {
            always_eprint!("❌ Failed to load embedded ICO data: {}", e);
            None
        }
    }
}

fn main() {
    // Initialize debug logging first
    utils::logger::init_debug_logging();

    env_logger::init();

    debug_print!("🚀 Initializing {}...", APP_NAME);

    // Initialize app manifest first
    let _manifest = state::manifest::AppManifest::load();

    // Ensure soundpack directories exist
    if let Err(e) = state::paths::soundpacks::ensure_soundpack_directories() {
        debug_eprint!("⚠️ Failed to create soundpack directories: {}", e);
    }

    // Check for command line arguments (protocol handling and startup options)
    let args: Vec<String> = std::env::args().collect();
    debug_print!("🔍 Command line args: {:?}", args);

    // Check if we should start minimized (from auto-startup)
    let should_start_minimized =
        args.contains(&"--minimized".to_string()) ||
        (state::config::AppConfig::load().auto_start &&
            state::config::AppConfig::load().start_minimized);

    // Register protocol on first run
    // if let Err(e) = protocol::register_protocol() {
    //     eprintln!("Warning: Failed to register mechvibes:// protocol: {}", e);
    // }    // Initialize global app state before rendering
    state::app::init_app_state();
    state::app::init_update_state();

    // Initialize music player
    if let Err(e) = state::music::initialize_music_player() {
        debug_eprint!("⚠️ Failed to initialize music player: {}", e);
    } else {
        debug_print!("🎵 Music player initialized successfully");
    }

    // Initialize ambiance player
    state::ambiance::initialize_global_ambiance_player();
    debug_print!("🎵 Ambiance player initialized");

    // Note: Update service will be initialized within the UI components
    // to ensure proper Dioxus runtime context

    // Create input event channels for communication between input listener and UI
    let (keyboard_tx, keyboard_rx) = mpsc::channel::<String>();
    let (mouse_tx, mouse_rx) = mpsc::channel::<String>();
    let (hotkey_tx, hotkey_rx) = mpsc::channel::<String>();

    // Clone senders for global access (for window-level keyboard events)
    let keyboard_tx_clone = keyboard_tx.clone();
    let mouse_tx_clone = mouse_tx.clone();
    let hotkey_tx_clone = hotkey_tx.clone();

    // Initialize global input channels for UI to access (including senders for window events)
    init_input_channels(keyboard_rx, mouse_rx, hotkey_rx, keyboard_tx_clone, mouse_tx_clone, hotkey_tx_clone);

    // Initialize window focus state
    // If window starts visible (not minimized), it will be focused
    let initial_focus_state = !should_start_minimized;
    init_window_focus_state_with_value(initial_focus_state);
    debug_print!("🔍 Initial window focus state: {}", if initial_focus_state { "FOCUSED" } else { "UNFOCUSED" });

    // macOS: request Accessibility + Input Monitoring before starting listeners.
    // Without Input Monitoring, the packaged .app often only sees special keys
    // (Backspace, CapsLock, …) from other apps — letters/numbers are filtered.
    // cargo run uses a different binary path, so its TCC grants do not apply to the .app.
    #[cfg(target_os = "macos")]
    {
        disable_app_nap();
        let ok = libs::input_manager::ensure_macos_input_permissions();
        debug_print!("🔒 Full keyboard capture permissions: {}", ok);
    }
    #[cfg(not(target_os = "macos"))]
    {
        libs::input_manager::set_accessibility_permissions(
            libs::input_manager::check_accessibility_permissions()
        );
    }

    // Detect display server on Linux
    #[cfg(target_os = "linux")]
    let display_server = std::env::var("XDG_SESSION_TYPE").unwrap_or_else(|_| "x11".to_string());

    #[cfg(target_os = "linux")]
    debug_print!("🔍 Detected display server: {}", display_server);

    // Start input listeners based on platform and display server
    #[cfg(target_os = "linux")]
    {
        use libs::input_manager::get_window_focus_state;
        use std::sync::{Arc, Mutex};

        if display_server == "wayland" {
            // On Wayland, use evdev for keyboard input (works both focused and unfocused)
            // evdev also handles hotkey detection (Ctrl+Alt+M)
            debug_print!("🎮 Starting evdev keyboard listener (Wayland mode)...");
            let focus_state = get_window_focus_state();
            start_evdev_keyboard_listener(keyboard_tx.clone(), hotkey_tx.clone(), focus_state);

            // Use rdev for mouse events only (no keyboard/hotkeys on Wayland)
            // Pass "always focused" state to prevent rdev from sending keyboard events
            debug_print!("🎮 Starting unified input listener for mouse events (Wayland mode)...");
            let always_focused = Arc::new(Mutex::new(true));
            start_unified_input_listener(keyboard_tx, mouse_tx, hotkey_tx, Some(always_focused));
        } else {
            // On X11, use the hybrid approach (rdev + device_query)
            // rdev handles keyboard when unfocused, device_query when focused
            let focus_state = get_window_focus_state();

            debug_print!("🎮 Starting unified input listener (X11 mode - unfocused)...");
            start_unified_input_listener(keyboard_tx.clone(), mouse_tx, hotkey_tx, Some(focus_state.clone()));

            debug_print!("🎮 Starting focused keyboard listener (X11 mode - focused)...");
            start_focused_keyboard_listener(keyboard_tx, Some(focus_state), None);
        }
    }

    // macOS: pure device_query polling for both keyboard and mouse.
    // Avoids rdev's HID-level tap which crashes due to background thread UI assertions.
    #[cfg(target_os = "macos")]
    {
        debug_print!("🎮 Starting keyboard poller (macOS - device_query)...");
        start_focused_keyboard_listener(keyboard_tx, None, Some(hotkey_tx));
        debug_print!("🎮 Starting mouse poller (macOS - device_query)...");
        crate::libs::device_query_mouse_listener::start_device_query_mouse_listener(mouse_tx);
    }

    // Windows: hybrid approach — rdev when unfocused, device_query when focused.
    #[cfg(target_os = "windows")]
    {
        use libs::input_manager::get_window_focus_state;
        let focus_state = get_window_focus_state();

        debug_print!("🎮 Starting unified input listener (Windows - unfocused keyboard + mouse)...");
        start_unified_input_listener(
            keyboard_tx.clone(),
            mouse_tx,
            hotkey_tx,
            Some(focus_state.clone()),
        );

        debug_print!("🎮 Starting focused keyboard listener (Windows - focused)...");
        start_focused_keyboard_listener(keyboard_tx, Some(focus_state), None);
    }

    // Create window action channel
    let (window_tx, _window_rx) = mpsc::channel::<WindowAction>();
    WINDOW_MANAGER.set_action_sender(window_tx);

    // Create AudioContext early and store globally so both the sound-processor
    // thread and the Dioxus UI share the same instance.
    let audio_ctx = std::sync::Arc::new(libs::AudioContext::new());
    libs::audio::init_global_audio_context(audio_ctx);

    // Spawn a dedicated sound-processor thread that is independent of the
    // Dioxus event loop.  On macOS the main-thread run loop can be throttled
    // (App Nap) when the window loses focus, which silences keyboard sounds
    // that are consumed inside a Dioxus `use_future`.
    libs::sound_processor::start_sound_processor_thread();

    // Window dimensions - allow vertical resizing
    let window_width = 470;
    let min_height = 600;   // Minimum height for compact mode
    let default_height = 820; // Default height
    let max_height = 820;  // Maximum height

    // Load icon before creating window
    let window_icon = load_icon();
    if window_icon.is_none() {
        always_eprint!("⚠️ Warning: Failed to load window icon - taskbar icon may not appear");
    }

    // Create a WindowBuilder with native decorations (macOS traffic lights) + transparency
    let window_builder = WindowBuilder::default()
        .with_title(APP_NAME)
        .with_transparent(true)
        .with_always_on_top(false)
        .with_inner_size(LogicalSize::new(window_width, default_height))
        .with_min_inner_size(LogicalSize::new(window_width, min_height))
        .with_max_inner_size(LogicalSize::new(window_width, max_height))
        .with_fullscreen(None)
        .with_decorations(true) // Native title bar with traffic lights
        .with_resizable(true)
        .with_visible(!should_start_minimized)
        .with_window_icon(window_icon);

    // Create config with our window settings and custom protocol handlers
    let config = Config::new()
        .with_window(window_builder)
        .with_close_behaviour(dioxus::desktop::WindowCloseBehaviour::WindowHides)
        .with_menu(None);

    // Launch the app with our config
    dioxus::LaunchBuilder::desktop().with_cfg(config).launch(app_with_stylesheets)
}

fn app_with_stylesheets() -> Element {
    rsx! {
        ui::app {}
    }
}
