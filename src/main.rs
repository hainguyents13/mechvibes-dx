#![allow(non_snake_case)]

mod libs;
use dioxus::prelude::*;
use libs::ui;

fn main() {
    env_logger::init(); // Debug log

    launch(ui::app);
}
