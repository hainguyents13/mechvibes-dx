use dioxus::prelude::*;
use lucide_dioxus::{House, Inbox, Settings};

#[allow(non_snake_case)]
#[component]
pub fn Dock() -> Element {
    rsx! {
      div { class: "dock",
        // Button Home
        button { class: "dock-active",
          House { class: "fill-accent" }
          span { class: "dock-label", "Home" }
        }
        // Button Inbox (active)
        button { class: "",
          Inbox { class: "" }
          span { class: "dock-label", "Inbox" }
        }
        // Button Settings
        button {
          Settings { class: "" }
          span { class: "dock-label", "Settings" }
        }
      }
    }
}
