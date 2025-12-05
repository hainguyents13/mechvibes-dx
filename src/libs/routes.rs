use dioxus::prelude::*;
use crate::libs::AudioContext;
use std::sync::Arc;

use crate::{ libs::theme::use_theme, utils::config::use_config };

#[derive(Clone, Routable, Debug, PartialEq)]
pub enum Route {
    #[layout(Layout)] #[route("/")] Home {},
    #[route("/customize")] Customize {},
    #[route("/soundpacks")] Soundpacks {},
    #[route("/mood")] Mood {},
    #[route("/settings")] Settings {},
}

#[component]
pub fn Layout() -> Element {
    let (config_signal, _set_config) = use_config();

    // Theme state - use theme context and initialize from config
    let mut theme = use_theme();

    // Initialize theme from config on first load
    use_effect(move || {
        theme.set(config_signal.read().theme.clone());
    });

    // Convert theme to DaisyUI theme name
    let daisy_theme = theme().to_daisy_theme();

    // Get background customization settings (reactive to config changes)
    let background_style = use_memo(move || {
        let config = config_signal.read();
        if config.enable_background_customization {
            let bg_config = &config.background_customization;
            if bg_config.use_image && bg_config.background_image.is_some() {
                // Use background image
                format!(
                    "background: url({}) center center / cover no-repeat;",
                    bg_config.background_image.as_ref().unwrap()
                )
            } else {
                // Use background color
                format!("background: {};", bg_config.background_color)
            }
        } else {
            // Default background (let theme handle it)
            String::new()
        }
    });

    // ========== HYBRID APPROACH: Window Focus Detection ==========
    // Track window focus state - when focused, use Dioxus event handlers (no global hooks)
    // When unfocused, rely on rdev global hooks (necessary for background operation)
    let mut is_focused = use_signal(|| false);

    // Get audio context to play sounds when window is focused
    let audio_context: Arc<AudioContext> = use_context();

    // Keyboard event handler for when window is focused
    let ctx_keydown = audio_context.clone();
    let onkeydown = move |evt: KeyboardEvent| {
        if *is_focused.read() {
            let key_code = evt.code().to_string();
            println!("🪟 Window focused - Dioxus handler: Key pressed: {}", key_code);
            ctx_keydown.play_key_event_sound(&key_code, true);
        }
    };

    let ctx_keyup = audio_context.clone();
    let onkeyup = move |evt: KeyboardEvent| {
        if *is_focused.read() {
            let key_code = evt.code().to_string();
            println!("🪟 Window focused - Dioxus handler: Key released: {}", key_code);
            ctx_keyup.play_key_event_sound(&key_code, false);
        }
    };

    // Focus/blur handlers to track window focus state
    let onfocus = move |_| {
        is_focused.set(true);
        println!("🪟 Window FOCUSED - using Dioxus event handlers (no global hooks)");
    };

    let onblur = move |_| {
        is_focused.set(false);
        println!("🪟 Window UNFOCUSED - using rdev global hooks");
    };

    rsx! {
      div {
        class: "h-screen flex flex-col",
        "data-theme": "{daisy_theme}",
        style: "{background_style()}",
        tabindex: 0,  // Make div focusable to receive keyboard events
        onkeydown: onkeydown,
        onkeyup: onkeyup,
        onfocus: onfocus,
        onblur: onblur,
        // Custom title bar for window controls
        crate::components::titlebar::TitleBar {}

        // Main content area with padding to account for title bar
        div { class: "flex-1 overflow-auto pb-28 px-8 pt-20 py-12",
          // Outlet for nested routes
          Outlet::<Route> {}
        }
        // Dock at the bottom
        crate::components::dock::Dock {}
      }
    }
}

#[component]
pub fn Home() -> Element {
    use crate::libs::AudioContext;
    use std::sync::Arc;

    // Use audio context from the layout provider instead of creating new one
    let audio_context: Arc<AudioContext> = use_context();
    rsx! {
      crate::components::pages::HomePage { audio_ctx: audio_context }
    }
}

#[component]
pub fn Soundpacks() -> Element {
    rsx! {
      crate::components::pages::Soundpacks {}
    }
}

#[component]
pub fn Mood() -> Element {
    rsx! {
      crate::components::pages::MoodPage {}
    }
}

#[component]
pub fn Customize() -> Element {
    rsx! {
      crate::components::pages::CustomizePage {}
    }
}

#[component]
pub fn Settings() -> Element {
    rsx! {
      crate::components::pages::SettingsPage {}
    }
}
