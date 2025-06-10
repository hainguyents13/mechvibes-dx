// #![windows_subsystem = "windows"]
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
use libs::input_manager::init_input_channels;
use std::sync::mpsc;

#[cfg(windows)]
use winapi::um::winuser::{ GetSystemMetrics, SM_CXSCREEN, SM_CYSCREEN };

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

#[cfg(windows)]
fn get_screen_size() -> (i32, i32) {
    unsafe {
        let width = GetSystemMetrics(SM_CXSCREEN);
        let height = GetSystemMetrics(SM_CYSCREEN);
        (width, height)
    }
}

#[cfg(not(windows))]
fn get_screen_size() -> (i32, i32) {
    // Default fallback for non-Windows platforms
    (1920, 1080)
}

fn calculate_window_size() -> LogicalSize<f64> {
    let (screen_width, screen_height) = get_screen_size();
    debug_print!("🖥️ Detected screen size: {}x{}", screen_width, screen_height);

    // Default app dimensions
    let default_width = 500.0;
    let default_height = 850.0;

    // Calculate appropriate size based on screen dimensions
    let (scale_w, scale_h) = if screen_height <= 720 {
        (1.0, 0.7) // Small screens (720p or smaller) - make app smaller
    } else if screen_height <= 1080 {
        (1.0, 0.8) // Medium screens (1080p) - slightly smaller
    } else {
        (1.0, 1.0) // Large screens (1440p and above) - default size
    };

    LogicalSize::new(default_width * scale_w, default_height * scale_h)
}

fn main() {
    // Initialize debug logging first
    utils::logger::init_debug_logging();

    env_logger::init();

    debug_print!("🚀 Initializing {}...", APP_NAME);

    // Initialize app manifest first
    let _manifest = state::manifest::AppManifest::load();

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
    // }

    // Initialize global app state before rendering
    state::app::init_app_state();

    // Create input event channels for communication between input listener and UI
    let (keyboard_tx, keyboard_rx) = mpsc::channel::<String>();
    let (mouse_tx, mouse_rx) = mpsc::channel::<String>();
    let (hotkey_tx, hotkey_rx) = mpsc::channel::<String>();

    // Initialize global input channels for UI to access
    init_input_channels(keyboard_rx, mouse_rx, hotkey_rx);

    // Start the unified input listener early in main
    debug_print!("🎮 Starting unified input listener from main...");
    start_unified_input_listener(keyboard_tx, mouse_tx, hotkey_tx);

    // Create window action channel
    let (window_tx, _window_rx) = mpsc::channel::<WindowAction>();
    WINDOW_MANAGER.set_action_sender(window_tx);

    // Create a WindowBuilder with custom appearance and responsive sizing
    let window_size = calculate_window_size();
    let window_builder = WindowBuilder::default()
        .with_title(APP_NAME)
        .with_transparent(false) // Disable transparency for better performance
        .with_always_on_top(false) // Allow normal window behavior for taskbar
        .with_inner_size(window_size)
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
