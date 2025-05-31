use crate::state::keyboard::KeyboardState;
use dioxus::prelude::*;

#[component]
pub fn Logo() -> Element {
    // Get the global keyboard state from context
    let keyboard_state = use_context::<Signal<KeyboardState>>();

    // Get the current key press state
    let key_pressed = keyboard_state.read().key_pressed;

    // Apply dynamic styling based on whether a key is pressed
    let base = "border-black select-none border-4 font-black block py-6 px-8 pt-7 text-5xl rounded-2xl transition-all duration-150 ease-in-out bg-base-300 ";

    // Determine the class based on key press state
    let class = if key_pressed {
        format!("{} logo-pressed", base)
    } else {
        format!("{} logo ", base)
    };

    rsx! {
      div { class, "MechvibesDX" }
    }
}
