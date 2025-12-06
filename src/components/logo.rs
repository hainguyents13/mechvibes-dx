use crate::state::keyboard::KeyboardState;
use crate::utils::config::use_config;
use dioxus::prelude::*;

#[component]
pub fn Logo() -> Element {
    // Get the global keyboard state from context
    let keyboard_state = use_context::<Signal<KeyboardState>>();
    let (config, _) = use_config();

    // Use computed signals that always reflect current config state
    let enable_sound = use_memo(move || config().enable_sound);
    let enable_logo_customization = use_memo(move || config().enable_logo_customization);
    let logo_customization = use_memo(move || config().logo_customization.clone());

    // Get the current key press state
    let key_pressed = keyboard_state.read().key_pressed;

    // Apply dynamic styling based on whether a key is pressed
    let base =
        "logo select-none border-4 font-black py-6 px-8 pt-7 text-5xl rounded-box transition-all duration-150 ease-in-out flex justify-center items-center ";

    // Create dynamic styles - only apply custom colors if logo customization is enabled
    let dynamic_style = if enable_logo_customization() {
        let logo_colors = logo_customization();

        // Determine background style based on current state (normal vs muted)
        let background_style = if enable_sound() {
            // Normal state - use background image if enabled, otherwise use color
            if logo_colors.use_background_image {
                if let Some(ref img) = logo_colors.background_image {
                    format!("background-image: url('{}'); background-size: cover; background-position: center", img)
                } else {
                    format!("background: {}", logo_colors.background_color)
                }
            } else {
                format!("background: {}", logo_colors.background_color)
            }
        } else {
            // Muted state - use muted background image if enabled, otherwise use muted color
            if logo_colors.use_muted_background_image {
                if let Some(ref img) = logo_colors.muted_background_image {
                    format!("background-image: url('{}'); background-size: cover; background-position: center", img)
                } else {
                    format!("background: {}", logo_colors.muted_background)
                }
            } else {
                format!("background: {}", logo_colors.muted_background)
            }
        };

        format!(
            "border-color: {}; color: {}; {}; {}",
            logo_colors.border_color,
            logo_colors.text_color,
            background_style,
            if !key_pressed && enable_sound() {
                format!("box-shadow: 0 5px 0 {}", logo_colors.shadow_color)
            } else {
                String::new()
            }
        )
    } else {
        // Default style - let CSS handle the default colors
        if !key_pressed && enable_sound() {
            "box-shadow: 0 5px 0 var(--color-primary); background: oklch(from var(--color-primary) l c h / 0.05)".to_string()
        } else {
            "background: oklch(from var(--color-primary) l c h / 0.05)".to_string()
        }
    };

    // Determine the class based on key press state
    let class = if key_pressed || !enable_sound() {
        format!("{} logo-pressed", base)
    } else {
        base.to_string()
    }; // Add default logo styling classes when customization is disabled
    let mut final_class = if enable_logo_customization() {
        class
    } else {
        format!("{} border-primary text-primary bg-transparent", class)
    };

    // Logo muted - add opacity when not using custom logo and sound is disabled
    final_class = if !enable_sound() {
        if enable_logo_customization() {
            let dimmed_class = if logo_customization().dimmed_when_muted {
                " opacity-50"
            } else {
                ""
            };
            format!("{} logo-muted{}", final_class, dimmed_class)
        } else {
            format!("{} logo-muted opacity-50", final_class)
        }
    } else {
        final_class
    };

    rsx! {
      div { class: "{final_class}", style: "{dynamic_style}", "Mechvibes" }
    }
}
