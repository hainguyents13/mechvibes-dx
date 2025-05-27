use dioxus::prelude::*;

use crate::{libs::theme::use_theme, state::config_utils::use_config};

#[derive(Clone, Routable, Debug, PartialEq)]
pub enum Route {
    #[layout(Layout)]
    #[route("/")]
    Home {},
    #[route("/themes")]
    Themes {},
    #[route("/soundpacks")]
    Soundpacks {},
    #[route("/settings")]
    Settings {},
}

#[component]
pub fn Layout() -> Element {
    let (config_signal, _set_config) = use_config();

    // Theme state - use theme context and initialize from config
    let mut theme = use_theme(); // Initialize theme from config on first load
    use_effect(move || {
        theme.set(config_signal.read().theme.clone());
    }); // Convert theme to DaisyUI theme name
    let daisy_theme = theme().to_daisy_theme();
    rsx! {
      div { class: "h-screen flex flex-col", "data-theme": daisy_theme,
        // Custom title bar for window controls
        crate::components::titlebar::TitleBar {}

        // Main content area with padding to account for title bar
        div { class: "flex-1 overflow-auto",
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

    let audio_context = use_hook(|| Arc::new(AudioContext::new()));
    rsx! {
      crate::components::pages::HomePage { audio_ctx: audio_context }
    }
}

#[component]
pub fn Soundpacks() -> Element {
    rsx! {
      crate::components::pages::SoundpacksPage {}
    }
}

#[component]
pub fn Themes() -> Element {
    rsx! {
      crate::components::pages::ThemesPage {}
    }
}

#[component]
pub fn Settings() -> Element {
    rsx! {
      crate::components::pages::SettingsPage {}
    }
}
