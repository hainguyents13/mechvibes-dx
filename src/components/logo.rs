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
    let logo_customization = use_memo(move || config().logo_customization.clone());

    // Get the current key press state
    let key_pressed = keyboard_state.read().key_pressed; // Apply dynamic styling based on whether a key is pressed
    let base = "select-none border-4 font-black block py-6 px-8 pt-7 text-5xl rounded-box transition-all duration-150 ease-in-out flex justify-center items-center";

    // Create dynamic styles using the custom colors
    let logo_colors = logo_customization();
    let dynamic_style = format!(
        "border-color: {}; color: {}; background-color: {}; {}",
        logo_colors.border_color,
        logo_colors.text_color,
        logo_colors.background_color,
        if !key_pressed && enable_sound() {
            format!("box-shadow: 0 5px 0 {}", logo_colors.shadow_color)
        } else {
            String::new()
        }
    );

    // Determine the class based on key press state
    let class = if key_pressed || !enable_sound() {
        format!(
            "{} {} logo-pressed",
            base,
            if !enable_sound() { "opacity-50" } else { "" }
        )
    } else {
        base.to_string()
    };

    rsx! {
      div {
        class,
        style: "{dynamic_style}",
        "Mechvibes"
      }
    }
}
