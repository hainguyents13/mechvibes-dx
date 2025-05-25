#![allow(non_snake_case)]

mod components;
mod libs;
mod state;
pub use crate::components::header::app as TailwindStylesheet;
use crate::state::keyboard::KeyboardState;
use dioxus::prelude::*;
use dioxus_radio::prelude::*;
use libs::ui;

// Định nghĩa kênh radio để các component có thể lắng nghe sự kiện
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum KeyboardChannel {
    Main,
}

// Triển khai RadioChannel cho KeyboardState
impl RadioChannel<KeyboardState> for KeyboardChannel {}

fn main() {
    env_logger::init();

    // Create a WindowBuilder with fixed size of 600x800
    let window_builder = dioxus_desktop::WindowBuilder::default()
        .with_title("Mechvibes DX")
        .with_transparent(true)
        .with_always_on_top(true)
        .with_inner_size(dioxus_desktop::LogicalSize::new(500.0, 800.0))
        .with_resizable(false);

    // Create config with our window settings
    let config = dioxus_desktop::Config::new()
        .with_window(window_builder)
        .with_menu(None);

    // Launch the app with our config
    dioxus::LaunchBuilder::desktop()
        .with_cfg(config)
        .launch(app_with_stylesheets)
}

fn app_with_stylesheets() -> Element {
    rsx! {
      TailwindStylesheet {}
      ui::app {}
    }
}
