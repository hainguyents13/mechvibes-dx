use crate::state::paths;
use dioxus::prelude::*;
use std::env;

#[component]
pub fn AppInfoDisplay() -> Element {
    // Get current executable path
    let exe_path = env::current_exe()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|_| "Unknown".to_string());

    // Get current working directory
    let current_dir = env::current_dir()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|_| "Unknown".to_string());    // Get absolute paths for directories and files
    let data_dir_absolute = paths::utils::get_data_dir_absolute();
    let config_file_absolute = paths::utils::get_config_file_absolute();
    let soundpacks_dir_absolute = paths::utils::get_soundpacks_dir_absolute();

    // Check file/directory existence
    let data_dir_exists = paths::utils::data_dir_exists();
    let config_file_exists = paths::utils::config_file_exists();
    let soundpacks_dir_exists = paths::utils::soundpacks_dir_exists();

    // Count soundpacks
    let soundpack_count = paths::utils::count_soundpacks();
    let soundpack_mouse_count = paths::utils::count_mouse_soundpacks();
    let soundpack_keyboard_count = paths::utils::count_keyboard_soundpacks();

    // Get OS info
    let os = env::consts::OS;
    let arch = env::consts::ARCH;

    rsx! {
      div { class: "mb-3",
        h3 { class: "text-base-content font-bold mb-2", "📍 Application Paths" }
        div { class: "mb-1",
          span { class: "text-base-content/70", "Executable: " }
          span { class: "text-base-content break-all", "{exe_path}" }
        }
        div {
          span { class: "text-base-content/70", "Working Dir: " }
          span { class: "text-base-content break-all", "{current_dir}" }
        }
      }
      div { class: "mb-3",
        h3 { class: "text-base-content font-bold mb-2", "📁 File System Status" }
        div { class: "space-y-1",
          div {
            span { class: if data_dir_exists { "text-base-content" } else { "text-error" },
              if data_dir_exists {
                "✅"
              } else {
                "❌"
              }
            }
            span { class: "ml-2 text-base-content/70 break-all", "{data_dir_absolute}" }
          }
          div {
            span { class: if config_file_exists { "text-base-content" } else { "text-error" },
              if config_file_exists {
                "✅"
              } else {
                "❌"
              }
            }
            span { class: "ml-2 text-base-content/70 break-all", "{config_file_absolute}" }
          }
          div {
            span { class: if soundpacks_dir_exists { "text-base-content" } else { "text-error" },
              if soundpacks_dir_exists {
                "✅"
              } else {
                "❌"
              }
            }
            span { class: "ml-2 text-base-content/70 break-all", "{soundpacks_dir_absolute}" }
          }
          div {
            span { class: "text-base-content", "🎵" }
            span { class: "ml-2 text-base-content/70",
              "Found {soundpack_count} soundpack(s) (Keyboard: {soundpack_keyboard_count}, Mouse: {soundpack_mouse_count})"
            }
          }
        }
      }
      div {
        h3 { class: "text-base-content font-bold mb-2", "💻 System Info" }
        div { class: "space-y-1",
          div {
            span { class: "text-base-content/70", "OS: " }
            span { class: "text-base-content", "{os}" }
          }
          div {
            span { class: "text-base-content/70", "Arch: " }
            span { class: "text-base-content", "{arch}" }
          }
        }
      }
    }
}
