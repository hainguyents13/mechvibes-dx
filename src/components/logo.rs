use crate::state::keyboard::KeyboardState;
use crate::utils::config_utils::use_config;
use dioxus::prelude::*;

#[component]
pub fn Logo() -> Element {
    // Get the global keyboard state from context
    let keyboard_state = use_context::<Signal<KeyboardState>>();
    let (config, _) = use_config();
    // Use computed signals that always reflect current config state
    let enable_sound = use_memo(move || config().enable_sound);
    // Get the current key press state
    let key_pressed = keyboard_state.read().key_pressed;

    // Apply dynamic styling based on whether a key is pressed
    let base = "select-none border-4 font-black block py-6 px-8 pt-7 text-5xl rounded-2xl transition-all duration-150 ease-in-out bg-base-300 flex justify-center items-center";

    // Determine the class based on key press state
    let class = if key_pressed || !enable_sound() {
        format!(
            "{} {} logo-pressed",
            base,
            if !enable_sound() { "opacity-50" } else { "" }
        )
    } else {
        format!("{} shadow-[0_5px_0_var(--color-base-content)]", base)
    };

    rsx! {
      div { class, "Mechvibes" }
    }
}
