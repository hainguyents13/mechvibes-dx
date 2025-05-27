use dioxus::prelude::*;
use std::env;
use std::path::Path;

#[component]
pub fn AppInfoDisplay() -> Element {
    // Get current executable path
    let exe_path = env::current_exe()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|_| "Unknown".to_string());

    // Get current working directory
    let current_dir = env::current_dir()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|_| "Unknown".to_string());

    // Check file/directory existence
    let data_dir_exists = Path::new("./data").exists();
    let config_file_exists = Path::new("./data/config.json").exists();
    let soundpacks_dir_exists = Path::new("./soundpacks").exists();

    // Count soundpacks
    let soundpack_count = if soundpacks_dir_exists {
        std::fs::read_dir("./soundpacks")
            .map(|entries| {
                entries
                    .filter_map(|e| e.ok())
                    .filter(|e| e.path().is_dir())
                    .count()
            })
            .unwrap_or(0)
    } else {
        0
    };

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
            span { class: "ml-2 text-base-content/70", "./data directory" }
          }
          div {
            span { class: if config_file_exists { "text-base-content" } else { "text-error" },
              if config_file_exists {
                "✅"
              } else {
                "❌"
              }
            }
            span { class: "ml-2 text-base-content/70", "./data/config.json" }
          }
          div {
            span { class: if soundpacks_dir_exists { "text-base-content" } else { "text-error" },
              if soundpacks_dir_exists {
                "✅"
              } else {
                "❌"
              }
            }
            span { class: "ml-2 text-base-content/70", "./soundpacks directory" }
          }
          div {
            span { class: "text-base-content", "🎵" }
            span { class: "ml-2 text-base-content/70",
              "Found {soundpack_count} soundpack(s)"
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
