use dioxus::prelude::*;

#[derive(Clone, Routable, Debug, PartialEq)]
pub enum Route {
    #[layout(Layout)]
    #[route("/")]
    Home {},
    #[route("/soundpacks")]
    Soundpacks {},
    #[route("/settings")]
    Settings {},
}

#[component]
pub fn Layout() -> Element {
    use crate::libs::theme::use_effective_theme;
    let effective_theme = use_effective_theme();

    rsx! {
      div { class: format!("min-h-screen {}", effective_theme.bg_primary()),
        // Main content area
        Outlet::<Route> {}

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
pub fn Settings() -> Element {
    use crate::libs::AudioContext;
    use std::sync::Arc;

    let audio_context = use_hook(|| Arc::new(AudioContext::new()));
    rsx! {
      crate::components::pages::SettingsPage { audio_ctx: audio_context }
    }
}
