use dioxus::prelude::*;
use lucide_dioxus::{ House, Music, Palette, Settings, CloudSunRain };

#[allow(non_snake_case)]
#[component]
pub fn Dock() -> Element {
    let nav = navigator();
    let route = use_route::<crate::libs::routes::Route>();
    rsx! {
      div { class: "dock dock-xl bg-base-300/30 backdrop-blur-lg ",
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
          span { class: "dock-label mt-1", "Sound packs" }
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
        // Button Mood
        button {
          class: if matches!(route, crate::libs::routes::Route::Mood {}) { "dock-active" } else { "" },
          onclick: move |_| {
              nav.push("/mood");
          },
          CloudSunRain { class: "w-5 h-5" }
          span { class: "dock-label mt-1", "Mood" }
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
