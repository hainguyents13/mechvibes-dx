// #![windows_subsystem = "windows"]
#![allow(non_snake_case)]

mod components;
mod libs;
mod state;
mod utils;

pub use crate::components::header::Header;
use dioxus::desktop::{Config, LogicalPosition, LogicalSize, WindowBuilder};
use dioxus::prelude::*;
use libs::protocol;
use libs::ui;

fn main() {
    env_logger::init();

    println!("🚀 Initializing MechvibesDX...");

    // Initialize app manifest first
    let _manifest = state::manifest::AppManifest::load();

    // Check for command line arguments (protocol handling)
    let args: Vec<String> = std::env::args().collect();
    println!("🔍 Command line args: {:?}", args);

    if args.len() > 1 {
        // Handle protocol URL if passed as argument
        let url = &args[1];
        println!("🔗 Processing argument: {}", url);
        if url.starts_with("mechvibes://") {
            println!("✅ Detected protocol URL: {}", url);
            if let Err(e) = protocol::handle_protocol_url(url) {
                eprintln!("❌ Failed to handle protocol URL {}: {}", url, e);
            } else {
                println!("✅ Protocol URL handled successfully");
            }
            return; // Exit after handling protocol
        } else {
            println!("ℹ️ Argument is not a protocol URL: {}", url);
        }
    } else {
        println!("ℹ️ No command line arguments provided");
    }

    // Register protocol on first run
    // if let Err(e) = protocol::register_protocol() {
    //     eprintln!("Warning: Failed to register mechvibes:// protocol: {}", e);
    // }

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
      ui::app {}
    }
}
