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
    rsx! {
      div { class: "min-h-screen bg-base-200",
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
