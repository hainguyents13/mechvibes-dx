use crate::state::paths;
use dioxus::prelude::*;
use lucide_dioxus::{Check, Folder, FolderCog, LaptopMinimalCheck};
use std::env;
use std::process::Command;

/// Open the application directory in the system file manager
fn open_app_directory() -> Result<(), String> {
    let app_root =
        std::env::current_dir().map_err(|e| format!("Failed to get current directory: {}", e))?;

    let result = if cfg!(target_os = "windows") {
        Command::new("explorer").arg(&app_root).spawn()
    } else if cfg!(target_os = "macos") {
        Command::new("open").arg(&app_root).spawn()
    } else {
        // Linux and other Unix-like systems
        Command::new("xdg-open").arg(&app_root).spawn()
    };

    match result {
        Ok(_) => Ok(()),
        Err(e) => Err(format!("Failed to open directory: {}", e)),
    }
}

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

    // Get absolute paths for directories and files
    let data_dir_absolute = paths::utils::get_data_dir_absolute();
    let config_file_absolute = paths::utils::get_config_file_absolute();
    let soundpacks_dir_absolute = paths::utils::get_soundpacks_dir_absolute();

    // Check file/directory existence
    let data_dir_exists = paths::utils::data_dir_exists();
    let config_file_exists = paths::utils::config_file_exists();
    let soundpacks_dir_exists = paths::utils::soundpacks_dir_exists();

    // Count soundpacks
    let (soundpack_count_keyboard, soundpack_count_mouse) =
        paths::utils::count_soundpacks_by_type();

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
            div { class: "ml-1 text-base-content/70 flex gap-2 items-center break-all",
              if soundpacks_dir_exists {
                Check { class: "w-4 h-4" }
              } else {
                "❌"
              }
              "{soundpacks_dir_absolute}"
            }
            div { class: "ml-1 text-base-content/70 flex gap-2 items-center break-all",
              Check { class: "w-4 h-4" }
              "Found {soundpack_count_keyboard + soundpack_count_mouse} soundpack(s) (Keyboard: {soundpack_count_keyboard}, Mouse: {soundpack_count_mouse})"
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
                        Ok(_) => println!("✅ Successfully opened app directory"),
                        Err(e) => eprintln!("❌ Failed to open app directory: {}", e),
                    }
                });
            },
            "Open app directory"
          }
        }
      }
    }
}
