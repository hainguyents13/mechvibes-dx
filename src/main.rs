// #![windows_subsystem = "windows"]
#![allow(non_snake_case)]

mod components;
mod libs;
mod state;
mod utils;

use dioxus::desktop::{ Config, LogicalPosition, LogicalSize, WindowBuilder };
use dioxus::prelude::*;
use libs::protocol;
use libs::ui;
use libs::window_manager::{ WindowAction, WINDOW_MANAGER };
use std::sync::mpsc;

// Function to conditionally set windows subsystem based on config
fn should_show_console() -> bool {
    // Try to load config to check debug console setting
    match std::panic::catch_unwind(|| { state::config::AppConfig::load().show_debug_console }) {
        Ok(show_debug) => show_debug,
        Err(_) => false, // Default to false if config loading fails
    }
}

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

    // Hide console window if debug console is disabled in config
    if !should_show_console() {
        #[cfg(windows)]
        {
            unsafe {
                let console = winapi::um::wincon::GetConsoleWindow();
                if !console.is_null() {
                    winapi::um::winuser::ShowWindow(console, winapi::um::winuser::SW_HIDE);
                }
            }
        }
    }

    env_logger::init();

    debug_print!("🚀 Initializing MechvibesDX...");

    // Initialize app manifest first
    let _manifest = state::manifest::AppManifest::load(); // Check for command line arguments (protocol handling)
    let args: Vec<String> = std::env::args().collect();
    debug_print!("🔍 Command line args: {:?}", args);

    if args.len() > 1 {
        // Handle protocol URL if passed as argument
        let url = &args[1];
        debug_print!("🔗 Processing argument: {}", url);
        if url.starts_with("mechvibes://") {
            debug_print!("✅ Detected protocol URL: {}", url);
            if let Err(e) = protocol::handle_protocol_url(url) {
                always_eprint!("❌ Failed to handle protocol URL {}: {}", url, e);
            } else {
                debug_print!("✅ Protocol URL handled successfully");
            }
            return; // Exit after handling protocol
        } else {
            debug_print!("ℹ️ Argument is not a protocol URL: {}", url);
        }
    } else {
        debug_print!("ℹ️ No command line arguments provided");
    }

    // Register protocol on first run
    // if let Err(e) = protocol::register_protocol() {
    //     eprintln!("Warning: Failed to register mechvibes:// protocol: {}", e);
    // }    // Initialize global app state before rendering
    state::app::init_app_state();

    // Create window action channel
    let (window_tx, _window_rx) = mpsc::channel::<WindowAction>();
    WINDOW_MANAGER.set_action_sender(window_tx); // Create a WindowBuilder with custom appearance
    let window_builder = WindowBuilder::default()
        .with_title("MechvibesDX")
        .with_transparent(false) // Disable transparency for better performance
        .with_always_on_top(false) // Allow normal window behavior for taskbar
        .with_position(LogicalPosition::new(1700.0, 300.0))
        .with_inner_size(LogicalSize::new(500.0, 850.0))
        .with_fullscreen(None)
        .with_decorations(false) // Use custom title bar
        .with_resizable(false) // Disable window resizing
        .with_window_icon(load_icon()); // Set window icon for taskbar

    // Create config with our window settings
    let config = Config::new().with_window(window_builder).with_menu(None);

    // Launch the app with our config
    dioxus::LaunchBuilder::desktop().with_cfg(config).launch(app_with_stylesheets)
}

fn app_with_stylesheets() -> Element {
    rsx! {
        ui::app {}
    }
}
