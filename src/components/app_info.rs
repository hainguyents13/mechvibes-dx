use dioxus::prelude::*;
use std::env;
use std::path::Path;

#[component]
pub fn AppInfoDisplay() -> Element {
    use crate::libs::theme::use_effective_theme;
    let effective_theme = use_effective_theme();

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

    rsx! {        div { class: "fixed top-4 left-4 z-50",
            // Toggle button
            button {
                class: format!("{} {} px-3 py-2 rounded-lg text-sm {} transition-colors",
                        effective_theme.bg_secondary(),
                        effective_theme.text_primary(),
                        effective_theme.bg_hover()),
                onclick: move |_| show_info.set(!show_info()),
                if show_info() { "Hide Info" } else { "Show App Info" }
            }            // Info panel
            if show_info() {
                div {
                    class: format!("mt-2 {} {} p-4 rounded-lg text-xs font-mono max-w-lg {}",
                            effective_theme.bg_secondary(),
                            effective_theme.text_primary(),
                            effective_theme.border()),                    div { class: "mb-3",
                        h3 { class: format!("{} font-bold mb-2", effective_theme.text_primary()), "📍 Application Paths" }
                        div { class: "mb-1",
                            span { class: effective_theme.text_secondary(), "Executable: " }
                            span { class: format!("{} break-all", effective_theme.text_primary()), "{exe_path}" }
                        }
                        div {
                            span { class: effective_theme.text_secondary(), "Working Dir: " }
                            span { class: format!("{} break-all", effective_theme.text_primary()), "{current_dir}" }
                        }
                    }                    div { class: "mb-3",
                        h3 { class: format!("{} font-bold mb-2", effective_theme.text_primary()), "📁 File System Status" }
                        div { class: "space-y-1",
                            div {
                                span { class: if data_dir_exists { format!("{}", effective_theme.text_primary()) } else { "text-red-400" },
                                    if data_dir_exists { "✅" } else { "❌" }
                                }
                                span { class: format!("ml-2 {}", effective_theme.text_secondary()), "./data directory" }
                            }
                            div {
                                span { class: if config_file_exists { format!("{}", effective_theme.text_primary()) } else { "text-red-400" },
                                    if config_file_exists { "✅" } else { "❌" }
                                }
                                span { class: format!("ml-2 {}", effective_theme.text_secondary()), "./data/config.json" }
                            }
                            div {
                                span { class: if soundpacks_dir_exists { format!("{}", effective_theme.text_primary()) } else { "text-red-400" },
                                    if soundpacks_dir_exists { "✅" } else { "❌" }
                                }
                                span { class: format!("ml-2 {}", effective_theme.text_secondary()), "./soundpacks directory" }
                            }
                            div {
                                span { class: effective_theme.text_primary(), "🎵" }
                                span { class: format!("ml-2 {}", effective_theme.text_secondary()), "Found {soundpack_count} soundpack(s)" }
                            }
                        }
                    }                    div {
                        h3 { class: format!("{} font-bold mb-2", effective_theme.text_primary()), "💻 System Info" }
                        div { class: "space-y-1",
                            div {
                                span { class: effective_theme.text_secondary(), "OS: " }
                                span { class: effective_theme.text_primary(), "{os}" }
                            }
                            div {
                                span { class: effective_theme.text_secondary(), "Arch: " }
                                span { class: effective_theme.text_primary(), "{arch}" }
                            }
                        }
                    }
                }
            }
        }
    }
}
