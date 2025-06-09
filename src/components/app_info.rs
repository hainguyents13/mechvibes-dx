use crate::utils::path::{
    get_data_dir_absolute,
    get_config_file_absolute,
    data_dir_exists,
    config_file_exists,
};
use crate::{ debug_print, always_eprint };
use crate::utils::path;
use dioxus::prelude::*;
use lucide_dioxus::{ Check, Folder, FolderCog, LaptopMinimalCheck };
use std::env;

/// Open the application directory in the system file manager
fn open_app_directory() -> Result<(), String> {
    let app_root = std::env
        ::current_dir()
        .map_err(|e| format!("Failed to get current directory: {}", e))?;

    path::open_path(&app_root.to_string_lossy())
}

#[component]
pub fn AppInfoDisplay() -> Element {
    // Get current executable path
    let exe_path = env
        ::current_exe()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|_| "Unknown".to_string());

    // Get current working directory
    let current_dir = env
        ::current_dir()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|_| "Unknown".to_string());

    // Get absolute paths for directories and files
    let data_dir_absolute = get_data_dir_absolute();
    let config_file_absolute = get_config_file_absolute();

    // Check file/directory existence
    let data_dir_exists = data_dir_exists();
    let config_file_exists = config_file_exists();

    // Get OS info
    let os = env::consts::OS;
    let arch = env::consts::ARCH;

    rsx! {
      div { class: "space-y-4",
        // Application Paths
        div {
          h3 { class: "mb-2 flex items-center gap-2",
            Folder { class: "w-5 h-5" }
            "Application Paths"
          }
          div { class: "mb-1",
            span { class: "text-base-content/70", "Executable: " }
            span { class: "break-all", "{exe_path}" }
          }
          div {
            span { class: "text-base-content/70", "Working Dir: " }
            span { class: "break-all", "{current_dir}" }
          }
        }
        // File System Status
        div {
          h3 { class: "mb-2 flex items-center gap-2",
            FolderCog { class: "w-5 h-5" }
            "File System Status"
          }
          div { class: "space-y-1",
            div { class: "ml-1 text-base-content/70 flex gap-2 items-center break-all",
              if data_dir_exists {
                Check { class: "w-4 h-4" }
              } else {
                "❌"
              }
              "{data_dir_absolute}"
            }
            div { class: "ml-1 text-base-content/70 flex gap-2 items-center break-all",
              if config_file_exists {
                Check { class: "w-4 h-4" }
              } else {
                "❌"
              }
              "{config_file_absolute}"
            }
          }
        }
        // System Info
        div {
          h3 { class: "mb-2 flex items-center gap-2",
            LaptopMinimalCheck { class: "w-5 h-5" }
            "System info"
          }
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
        // Open App Directory Button
        div {
          button {
            class: "btn btn-soft btn-sm",
            onclick: move |_| {
                spawn(async move {
                    match open_app_directory() {
                        Ok(_) => debug_print!("✅ Successfully opened app directory"),
                        Err(e) => always_eprint!("❌ Failed to open app directory: {}", e),
                    }
                });
            },
            "Open app directory"
          }
        }
      }
    }
}
