#![windows_subsystem = "windows"]
#![allow(non_snake_case)]

mod components;
mod libs;
mod state;
pub use crate::components::header::Header;
use dioxus::desktop::{Config, LogicalPosition, LogicalSize, WindowBuilder};
use dioxus::prelude::*;
use libs::protocol;
use libs::ui;

fn main() {
    env_logger::init(); // Initialize app manifest first
    println!("🚀 Initializing MechvibesDX...");

    // Check for command line arguments (protocol handling)
    let args: Vec<String> = std::env::args().collect();
    if args.len() > 1 {
        // Handle protocol URL if passed as argument
        let url = &args[1];
        if url.starts_with("mechvibes://") {
            if let Err(e) = protocol::handle_protocol_url(url) {
                eprintln!("Failed to handle protocol URL {}: {}", url, e);
            }
            return; // Exit after handling protocol
        }
    }

    // Register protocol on first run
    if let Err(e) = protocol::register_protocol() {
        eprintln!("Warning: Failed to register mechvibes:// protocol: {}", e);
    }

    let _manifest = state::AppManifest::load();

    // Initialize global app state before rendering
    state::app::init_app_state();

    // Create a WindowBuilder with custom appearance
    let window_builder = WindowBuilder::default()
        .with_title("MechvibesDX")
        .with_transparent(false) // Disable transparency for better performance
        .with_always_on_top(true)
        .with_position(LogicalPosition::new(1700.0, 300.0))
        .with_inner_size(LogicalSize::new(500.0, 850.0))
        .with_fullscreen(None)
        .with_decorations(false) // Use custom title bar
        .with_resizable(false); // Disable window resizing

    // Create config with our window settings
    let config = Config::new().with_window(window_builder).with_menu(None);

    // Launch the app with our config
    dioxus::LaunchBuilder::desktop()
        .with_cfg(config)
        .launch(app_with_stylesheets)
}

fn app_with_stylesheets() -> Element {
    rsx! {
      // Use the UI root component directly - Header component is already included in ui::app
      ui::app {}
    }
}
