use dioxus::prelude::*;
use dioxus_router::prelude::{navigator, use_route};
use lucide_dioxus::{House, Music, Palette, Settings, Sparkles};

#[allow(non_snake_case)]
#[component]
pub fn Dock() -> Element {
    let nav = navigator();
    let route = use_route::<crate::libs::routes::Route>();
    rsx! {
      div { class: "dock dock-xl bg-base-200",
        // Button Home
        button {
          class: if matches!(route, crate::libs::routes::Route::Home {}) { "dock-active" } else { "" },
          onclick: move |_| {
              nav.push("/");
          },
          House { class: "w-5 h-5" }
          span { class: "dock-label mt-1", "Home" }
        }
        // Button Soundpacks
        button {
          class: if matches!(route, crate::libs::routes::Route::Soundpacks {}) { "dock-active" } else { "" },
          onclick: move |_| {
              nav.push("/soundpacks");
          },
          Music { class: "w-5 h-5" }
          span { class: "dock-label mt-1", "Soundpacks" }
        }
        // Button Effects
        button {
          class: if matches!(route, crate::libs::routes::Route::Effects {}) { "dock-active" } else { "" },
          onclick: move |_| {
              nav.push("/effects");
          },
          Sparkles { class: "w-5 h-5" }
          span { class: "dock-label mt-1", "Effects" }
        }
        // Button Customize
        button {
          class: if matches!(route, crate::libs::routes::Route::Customize {}) { "dock-active" } else { "" },
          onclick: move |_| {
              nav.push("/customize");
          },
          Palette { class: "w-5 h-5" }
          span { class: "dock-label mt-1", "Customize" }
        }
        // Button Settings
        button {
          class: if matches!(route, crate::libs::routes::Route::Settings {}) { "dock-active" } else { "" },
          onclick: move |_| {
              nav.push("/settings");
          },
          Settings { class: "w-5 h-5" }
          span { class: "dock-label mt-1", "Settings" }
        }
      }
    }
}
