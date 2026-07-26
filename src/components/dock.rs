use dioxus::prelude::*;
use lucide_dioxus::{ House, Music, Palette, Settings, CloudSunRain };

#[allow(non_snake_case)]
#[component]
pub fn Dock() -> Element {
    let nav = navigator();
    let route = use_route::<crate::libs::routes::Route>();
    rsx! {
      nav {
        class: "dock dock-xl",
        style: "background: rgba(var(--glass-bg, 30 30 30), 0.6); backdrop-filter: blur(24px) saturate(180%); -webkit-backdrop-filter: blur(24px) saturate(180%); border-top: 1px solid rgba(255,255,255,0.08);",
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
