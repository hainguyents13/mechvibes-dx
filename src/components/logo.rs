use crate::state::keyboard::KeyboardState;
use crate::KeyboardChannel;
use dioxus::prelude::*;
use dioxus_radio::prelude::*;

#[component]
pub fn Logo() -> Element {
    // Subscribe to the global keyboard state through the radio system
    let radio = use_radio::<KeyboardState, KeyboardChannel>(KeyboardChannel::Main);

    // Get the current key press state
    let key_pressed = radio.read().key_pressed;

    // Apply dynamic styling based on whether a key is pressed
    let base = "border-gray-900 border-4 font-black inline-block py-6 px-8 pt-7 text-5xl rounded-2xl transition-all duration-150 ease-in-out";
    let class = if key_pressed {
        format!("{} logo-pressed", base)
    } else {
        format!("{} logo", base)
    };

    rsx! {
      div { class, "Mechvibes 3" }
    }
}
