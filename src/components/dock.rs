use dioxus::prelude::*;
use dioxus_router::prelude::{navigator, use_route};
use lucide_dioxus::{House, Palette, Settings, Sparkles};

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
          Sparkles { class: "w-5 h-5" }
          span { class: "dock-label mt-1", "Effects" }
        }
        // Button Soundpacks
        button {
          class: if matches!(route, crate::libs::routes::Route::Soundpacks {}) { "dock-active" } else { "" },
          onclick: move |_| {
              nav.push("/soundpacks");
          },
          Palette { class: "w-5 h-5" }
          span { class: "dock-label mt-1", "Themes" }
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
