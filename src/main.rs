#![allow(non_snake_case)]

mod components;
mod libs;
mod state;
pub use crate::components::header::Header;
use dioxus::desktop::{Config, LogicalPosition, LogicalSize, WindowBuilder};
use dioxus::prelude::*;
use libs::ui;

fn main() {
    env_logger::init(); // Initialize app manifest first
    println!("🚀 Initializing Mechvibes DX...");
    let _manifest = state::AppManifest::load();

    // Initialize global app state before rendering
    state::app::init_app_state();

    // Create a WindowBuilder with fixed size of 600x800
    let window_builder = WindowBuilder::default()
        .with_title("Mechvibes DX")
        .with_transparent(true)
        .with_always_on_top(true)
        .with_position(LogicalPosition::new(1700.0, 300.0))
        .with_inner_size(LogicalSize::new(500.0, 800.0))
        .with_resizable(false);

    // Create config with our window settings
    let config = Config::new().with_window(window_builder).with_menu(None);

    // Launch the app with our config
    dioxus::LaunchBuilder::desktop()
        .with_cfg(config)
        .launch(app_with_stylesheets)
}

fn app_with_stylesheets() -> Element {
    rsx! {
      Header {}
      ui::app {}
    }
}
