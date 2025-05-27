use dioxus::prelude::*;
use dioxus_router::prelude::{navigator, use_route};
use lucide_dioxus::{House, Music, Settings};

#[allow(non_snake_case)]
#[component]
pub fn Dock() -> Element {
    let nav = navigator();
    let route = use_route::<crate::libs::routes::Route>();
    rsx! {
      div { class: "dock dock-lg",
        // Button Home
        button {
          class: if matches!(route, crate::libs::routes::Route::Home {}) { "dock-active" } else { "" },
          onclick: move |_| {
              nav.push("/");
          },
          House { class: "w-5 h-5" }
          span { class: "dock-label", "Home" }
        }
        // Button Soundpacks
        button {
          class: if matches!(route, crate::libs::routes::Route::Soundpacks {}) { "dock-active" } else { "" },
          onclick: move |_| {
              nav.push("/soundpacks");
          },
          Music { class: "w-5 h-5" }
          span { class: "dock-label", "Soundpacks" }
        }
        // Button Settings
        button {
          class: if matches!(route, crate::libs::routes::Route::Settings {}) { "dock-active" } else { "" },
          onclick: move |_| {
              nav.push("/settings");
          },
          Settings { class: "w-5 h-5" }
          span { class: "dock-label", "Settings" }
        }
      }
    }
}
