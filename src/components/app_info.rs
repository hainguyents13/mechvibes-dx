use dioxus::prelude::*;
use std::env;
use std::path::Path;

#[component]
pub fn AppInfoDisplay() -> Element {
    let mut show_info = use_signal(|| false);

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
        div { class: "fixed top-4 left-4 z-50",
            // Toggle button
            button {
                class: "bg-gray-800 text-white px-3 py-2 rounded-lg text-sm hover:bg-gray-700 transition-colors",
                onclick: move |_| show_info.set(!show_info()),
                if show_info() { "Hide Info" } else { "Show App Info" }
            }

            // Info panel
            if show_info() {
                div {
                    class: "mt-2 bg-black bg-opacity-90 text-green-400 p-4 rounded-lg text-xs font-mono max-w-lg border border-gray-600",

                    div { class: "mb-3",
                        h3 { class: "text-yellow-400 font-bold mb-2", "📍 Application Paths" }
                        div { class: "mb-1",
                            span { class: "text-gray-300", "Executable: " }
                            span { class: "text-green-300 break-all", "{exe_path}" }
                        }
                        div {
                            span { class: "text-gray-300", "Working Dir: " }
                            span { class: "text-green-300 break-all", "{current_dir}" }
                        }
                    }

                    div { class: "mb-3",
                        h3 { class: "text-yellow-400 font-bold mb-2", "📁 File System Status" }
                        div { class: "space-y-1",
                            div {
                                span { class: if data_dir_exists { "text-green-400" } else { "text-red-400" },
                                    if data_dir_exists { "✅" } else { "❌" }
                                }
                                span { class: "ml-2", "./data directory" }
                            }
                            div {
                                span { class: if config_file_exists { "text-green-400" } else { "text-red-400" },
                                    if config_file_exists { "✅" } else { "❌" }
                                }
                                span { class: "ml-2", "./data/config.json" }
                            }
                            div {
                                span { class: if soundpacks_dir_exists { "text-green-400" } else { "text-red-400" },
                                    if soundpacks_dir_exists { "✅" } else { "❌" }
                                }
                                span { class: "ml-2", "./soundpacks directory" }
                            }
                            div {
                                span { class: "text-blue-400", "🎵" }
                                span { class: "ml-2", "Found {soundpack_count} soundpack(s)" }
                            }
                        }
                    }

                    div {
                        h3 { class: "text-yellow-400 font-bold mb-2", "💻 System Info" }
                        div { class: "space-y-1",
                            div {
                                span { class: "text-gray-300", "OS: " }
                                span { class: "text-blue-300", "{os}" }
                            }
                            div {
                                span { class: "text-gray-300", "Arch: " }
                                span { class: "text-blue-300", "{arch}" }
                            }
                        }
                    }
                }
            }
        }
    }
}
