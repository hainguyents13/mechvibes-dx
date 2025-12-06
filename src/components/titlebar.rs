use crate::libs::window_manager::WINDOW_MANAGER;
use crate::utils::constants::APP_NAME;
use crate::state::app::use_update_info;
use dioxus::desktop::use_window;
use dioxus::prelude::*;
use lucide_dioxus::{ Minus, X, EyeClosed, Download };

/// A custom title bar component that allows for window dragging and includes minimize/close buttons
#[component]
pub fn TitleBar() -> Element {
    let window = use_window();
    let window_drag = window.clone();
    let update_info = use_update_info();

    // Get current version for display
    let current_version = crate::utils::constants::APP_VERSION;

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
      div { class: "fixed inset-0 h-10 z-999 flex justify-between items-center select-none gap-0 bg-gradient-to-b from-base-300/70 to-transparent backdrop-blur-sm transition-all ",
      // Left side - app title and draggable area
        div {
          class: "flex items-center grow cursor-move px-3 py-2",
          onmousedown: start_drag,
          // App title
          span { class: "text-sm font-semibold text-base-content", "{APP_NAME}" }
          // Optional version badge
          span { class: "ml-2 text-xs bg-base-300 text-base-content/50 px-1.5 py-0.5 rounded",
            "v{current_version}"
          }
        }


        // Right side - window controls
        div { class: "flex items-center space-x-2 px-3 py-2 pl-0",
          // Update notification (separate from draggable area)
          if let Some(update) = update_info.clone() {
            if update.update_available {
              div { 
                class: "tooltip tooltip-bottom ml-2",
                "data-tip": "New version {update.latest_version} available!",
                if let Some(url) = &update.download_url {
                  button { 
                    class: "btn btn-success btn-xs",
                    onclick: {
                      let url = url.clone();
                      move |_| {
                        println!("🔗 Opening update URL: {}", url);
                        // Open URL in default browser
                        if let Err(e) = open::that(&url) {
                          eprintln!("Failed to open URL: {}", e);
                        } else {
                          println!("✅ Successfully opened URL: {}", url);
                        }
                      }
                    },
                    Download { class: "w-3 h-3" }
                    "{update.latest_version}"
                  }
                }
              }
            }
          }
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
