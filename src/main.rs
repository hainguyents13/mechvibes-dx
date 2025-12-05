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
use libs::input_listener::start_unified_input_listener;
use libs::focused_input_listener::start_focused_keyboard_listener;
use libs::input_manager::{ init_input_channels, init_window_focus_state, get_window_focus_state };
use std::sync::mpsc;

#[cfg(target_os = "linux")]
use libs::evdev_input_listener::start_evdev_keyboard_listener;

// Use .ico format for better Windows compatibility
const EMBEDDED_ICON: &[u8] = include_bytes!("../assets/icon.ico");

fn load_icon() -> Option<dioxus::desktop::tao::window::Icon> {
    // Try to create icon from embedded ICO data
    match image::load_from_memory_with_format(EMBEDDED_ICON, image::ImageFormat::Ico) {
        Ok(img) => {
            let rgba = img.to_rgba8();
            let (width, height) = rgba.dimensions();
            match dioxus::desktop::tao::window::Icon::from_rgba(rgba.into_raw(), width, height) {
                Ok(icon) => {
                    debug_print!("✅ Loaded embedded ICO icon ({}x{})", width, height);
                    Some(icon)
                }
                Err(e) => {
                    always_eprint!("❌ Failed to create icon from embedded ICO data: {}", e);
                    None
                }
            }
        }
        Err(e) => {
            debug_eprint!("❌ Failed to load embedded ICO data: {}", e);
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
    init_window_focus_state();

    // Detect display server on Linux
    #[cfg(target_os = "linux")]
    let display_server = std::env::var("XDG_SESSION_TYPE").unwrap_or_else(|_| "x11".to_string());

    #[cfg(target_os = "linux")]
    debug_print!("🔍 Detected display server: {}", display_server);

    // Start input listeners based on platform and display server
    #[cfg(target_os = "linux")]
    {
        if display_server == "wayland" {
            // On Wayland, use evdev for keyboard input (works both focused and unfocused)
            debug_print!("🎮 Starting evdev keyboard listener (Wayland mode)...");
            let focus_state = get_window_focus_state();
            start_evdev_keyboard_listener(keyboard_tx.clone(), focus_state);

            // Use rdev for mouse events and hotkeys
            debug_print!("🎮 Starting unified input listener for mouse/hotkeys (Wayland mode)...");
            start_unified_input_listener(keyboard_tx, mouse_tx, hotkey_tx);
        } else {
            // On X11, use the hybrid approach (rdev + device_query)
            debug_print!("🎮 Starting unified input listener (X11 mode - unfocused)...");
            start_unified_input_listener(keyboard_tx.clone(), mouse_tx, hotkey_tx);

            debug_print!("🎮 Starting focused keyboard listener (X11 mode - focused)...");
            let focus_state = get_window_focus_state();
            start_focused_keyboard_listener(keyboard_tx, focus_state);
        }
    }

    // On Windows and macOS, use the hybrid approach (rdev + device_query)
    #[cfg(not(target_os = "linux"))]
    {
        debug_print!("🎮 Starting unified input listener (unfocused)...");
        start_unified_input_listener(keyboard_tx.clone(), mouse_tx, hotkey_tx);

        debug_print!("🎮 Starting focused keyboard listener (focused)...");
        let focus_state = get_window_focus_state();
        start_focused_keyboard_listener(keyboard_tx, focus_state);
    }

    // Create window action channel
    let (window_tx, _window_rx) = mpsc::channel::<WindowAction>();
    WINDOW_MANAGER.set_action_sender(window_tx);

    // Create a WindowBuilder with custom appearance and responsive sizing
    let window_builder = WindowBuilder::default()
        .with_title(APP_NAME)
        .with_transparent(true) // Disable transparency for better performance
        .with_always_on_top(false) // Allow normal window behavior for taskbar
        .with_inner_size(LogicalSize::new(470, 800))
        .with_fullscreen(None)
        .with_decorations(false) // Use custom title bar
        .with_resizable(false) // Enable window resizing for landscape mode
        .with_visible(!should_start_minimized) // Hide window if starting minimized
        .with_window_icon(load_icon()); // Set window icon for taskbar

    // Create config with our window settings and custom protocol handlers
    let config = Config::new().with_window(window_builder).with_menu(None);

    // Launch the app with our config
    dioxus::LaunchBuilder::desktop().with_cfg(config).launch(app_with_stylesheets)
}

fn app_with_stylesheets() -> Element {
    rsx! {
        ui::app {}
    }
}
