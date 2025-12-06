use dioxus::prelude::*;

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
    println!("🎨 Layout rendering with theme: {:?} -> DaisyUI: {}", theme(), daisy_theme);

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

    rsx! {
      div {
        class: "h-screen flex flex-col",
        "data-theme": "{daisy_theme}",
        style: "{background_style()}",
        // Custom title bar for window controls
        crate::components::titlebar::TitleBar {}

        // Main content area with padding to account for title bar
        div { class: "flex-1 overflow-auto {crate::utils::spacing::CONTENT_PADDING}",
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
