use dioxus::desktop::use_window;
use dioxus::prelude::*;
use lucide_dioxus::{Minus, X};

/// A custom title bar component that allows for window dragging and includes minimize/close buttons
#[component]
pub fn TitleBar() -> Element {
    let window = use_window();

    // Function to make window draggable
    let window_drag = window.clone();
    let start_drag = move |_| {
        window_drag.drag();
    };

    // Function to minimize window
    let window_min = window.clone();
    let minimize = move |_| {
        window_min.set_minimized(true);
    };

    // Function to close application
    let window_close = window.clone();
    let close = move |_| {
        window_close.close();
    };
    rsx! {
      div { class: "flex justify-between items-center select-none bg-transparent hover:bg-gradient-to-b from-base-300 to-transparent transition-all focus:bg-base-200",
        // Left side - app title and draggable area
        div {
          class: "flex items-center flex-1 cursor-move  px-3 py-2",
          onmousedown: start_drag,
          // App title
          span { class: "text-sm font-semibold text-base-content", "Mechvibes DX" }
          // Optional version badge
          span { class: "ml-2 text-xs bg-base-300 text-base-content/50 px-1.5 py-0.5 rounded",
            "Beta"
          }
        }
        // Right side - window controls
        div { class: "flex items-center space-x-2 px-3 py-2", // Minimize button
          button {
            class: "p-1.5 rounded-md hover:bg-neutral/70 text-base-content/70 hover:text-neutral-content transition-colors",
            title: "Minimize",
            onclick: minimize,
            Minus { class: "w-4 h-4" }
          }
          // Close button
          button {
            class: "p-1.5 rounded-md hover:bg-error text-base-content/70 hover:text-error-content transition-colors",
            title: "Close",
            onclick: close,
            X { class: "w-4 h-4" }
          }
        }
      }
    }
}
