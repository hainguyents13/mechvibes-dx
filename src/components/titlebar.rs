use crate::libs::window_manager::WINDOW_MANAGER;
use crate::utils::constants::APP_NAME;
use dioxus::desktop::use_window;
use dioxus::prelude::*;
use lucide_dioxus::{ Minus, X, EyeClosed };

/// A custom title bar component that allows for window dragging and includes minimize/close buttons
#[component]
pub fn TitleBar() -> Element {
    let window = use_window();
    let window_drag = window.clone();
    let start_drag = move |_| {
        window_drag.drag();
    };

    // Function to minimize window to tray
    let minimize_to_tray = move |_| {
        // Hide the window to system tray
        WINDOW_MANAGER.hide();
        println!("🔽 Window minimized to system tray");
    };

    // Function to close application
    let window_close = window.clone();
    let close = move |_| {
        window_close.close();
    };

    // Function to minimize application
    let window_minimize = window.clone();
    let minimize = move |_| {
        window_minimize.set_minimized(true);
    };
    rsx! {
      div { class: "fixed inset-0 h-10 z-999 flex justify-between items-center select-none bg-gradient-to-b from-base-300 to-transparent transition-all ",
        // Left side - app title and draggable area
        div {
          class: "flex items-center flex-1 cursor-move  px-3 py-2",
          onmousedown: start_drag, // App title
          span { class: "text-sm font-semibold text-base-content", "{APP_NAME}" }
          // Optional version badge
          span { class: "ml-2 text-xs bg-base-300 text-base-content/50 px-1.5 py-0.5 rounded",
            "Beta"
          }
        }
        // Right side - window controls
        div { class: "flex items-center space-x-2 px-3 py-2",
          // Minimize to taskbar button
          div {
            class: "tooltip tooltip-bottom",
            "data-tip": "Minimize",
            button {
              class: "p-1.5 rounded-box hover:bg-neutral/70 text-base-content/70 hover:text-neutral-content transition-colors",
              onclick: minimize,
              Minus { class: "w-4 h-4" }
            }
          }
          // Minimize to tray button
          div {
            class: "tooltip tooltip-bottom",
            "data-tip": "Hide to tray",
            button {
              class: "p-1.5 rounded-box hover:bg-neutral/70 text-base-content/70 hover:text-neutral-content transition-colors",
              onclick: minimize_to_tray,
              EyeClosed { class: "w-4 h-4" }
            }
          }
          // Close button
          div { class: "tooltip tooltip-bottom", "data-tip": "Quit",
            button {
              class: "p-1.5 rounded-box hover:bg-error text-base-content/70 hover:text-error-content transition-colors",
              title: "Close",
              onclick: close,
              X { class: "w-4 h-4" }
            }
          }
        }
      }
    }
}
