use crate::{
    components::ui::PageHeader,
    state::{app::use_app_state, paths},
};
use dioxus::prelude::*;
use lucide_dioxus::{
    ExternalLink, FolderOpen, Keyboard, Mouse, Music, Plus, RefreshCcw, Settings2, Trash,
};
use std::sync::Arc;

use std::path::Path;
use std::io;

#[cfg(target_os = "windows")]
pub fn open_directory<P: AsRef<Path>>(path: P) -> io::Result<()> {
    std::process::Command::new("explorer")
        .arg(path.as_ref())
        .status()
        .map(|_| ())
}

#[cfg(target_os = "macos")]
pub fn open_directory<P: AsRef<Path>>(path: P) -> io::Result<()> {
    std::process::Command::new("open")
        .arg(path.as_ref())
        .status()
        .map(|_| ())
}

#[cfg(target_os = "linux")]
pub fn open_directory<P: AsRef<Path>>(path: P) -> io::Result<()> {
    std::process::Command::new("xdg-open")
        .arg(path.as_ref())
        .status()
        .map(|_| ())
}

#[component]
pub fn SoundpacksPage() -> Element {
    // Get global app state for soundpacks
    let app_state = use_app_state();
    let soundpacks = use_memo(move || app_state.with(|state| state.get_soundpacks()));

    // Filter soundpacks by type
    let keyboard_soundpacks = use_memo(move || {
        soundpacks()
            .into_iter()
            .filter(|pack| !pack.mouse)
            .collect::<Vec<_>>()
    });

    let mouse_soundpacks = use_memo(move || {
        soundpacks()
            .into_iter()
            .filter(|pack| pack.mouse)
            .collect::<Vec<_>>()
    });

    // UI state for notification and loading
    // Get access to audio context for reloading soundpacks
    let audio_ctx: Arc<crate::libs::audio::AudioContext> = use_context();
    // Get access to app state for soundpack operations
    let app_state = use_app_state();
    let mut refreshing_soundpacks = use_signal(|| false);
    let mut refresh_soundpacks_cache = move || {
        println!("🔄 Refreshing soundpack cache from settings page...");

        // Set loading state to true - directly update the signal
        refreshing_soundpacks.set(true);
        println!("🌻 Loading state set to: {}", refreshing_soundpacks());

        // Clone necessary variables for the async task
        let mut refreshing_signal = refreshing_soundpacks.clone();
        let audio_ctx_clone = audio_ctx.clone();
        let mut app_state_clone = app_state.clone();

        // Perform the refresh operation in a separate task to not block the UI
        spawn(async move {
            // Use async sleep instead of std::thread::sleep
            use futures_timer::Delay;
            use std::time::Duration;

            // Add a small artificial delay to make loading state more visible
            Delay::new(Duration::from_millis(800)).await;

            println!("Starting soundpack refresh operation...");

            // This will create a new SoundpackCache instance that will refresh from directory
            let mut fresh_cache = crate::state::soundpack_cache::SoundpackCache::load();
            fresh_cache.refresh_from_directory();
            fresh_cache.save();

            // Update the app state with new cache
            app_state_clone.write().optimized_cache = Arc::new(fresh_cache);

            // Reload current soundpacks to apply any changes
            crate::state::app::reload_current_soundpacks(&audio_ctx_clone);

            // Add another small delay before changing the loading state back
            Delay::new(Duration::from_millis(200)).await;

            // Reset loading state
            refreshing_signal.set(false);
            println!("🌻 Loading state reset to: {}", refreshing_signal());

            println!("✅ Soundpack refresh complete");
        });
    };

    // Count soundpacks
    let (soundpack_count_keyboard, soundpack_count_mouse) =
        paths::utils::count_soundpacks_by_type();
    let soundpacks_dir_absolute = paths::utils::get_soundpacks_dir_absolute();

    rsx! {
      div { class: "p-12 pb-32",
        // Page header
        PageHeader {
          title: "Soundpacks".to_string(),
          subtitle: "Manage your soundpacks".to_string(),
          icon: Some(rsx! {
            Music { class: "w-8 h-8 mx-auto" }
          }),
        }

        // Tabs for soundpack types
        div { class: "tabs tabs-lift",
          // Keyboard tab
          label { class: "tab [--tab-border-color:var(--color-base-300)] [--tab-bg:var(--color-base-200)]",
            input {
              r#type: "radio",
              name: "soundpack-tab",
              checked: true,
            }
            Keyboard { class: "w-5 h-5 mr-2" }
            "Keyboard ({soundpack_count_keyboard})"
          }
          div { class: "tab-content  overflow-hidden bg-base-200 border-base-300 py-4 px-0",
            SoundpackTable {
              soundpacks: keyboard_soundpacks(),
              soundpack_type: "Keyboard",
            }
          }
          // Mouse tab
          label { class: "tab [--tab-border-color:var(--color-base-300)] [--tab-bg:var(--color-base-200)]",
            input { r#type: "radio", name: "soundpack-tab" }
            Mouse { class: "w-5 h-5 mr-2" }
            "Mouse ({soundpack_count_mouse})"
          }
          div { class: "tab-content overflow-hidden bg-base-200 border-base-300 py-4 px-0",
            SoundpackTable {
              soundpacks: mouse_soundpacks(),
              soundpack_type: "Mouse",
            }
          }
          // Manage tab
          label { class: "tab [--tab-border-color:var(--color-base-300)] [--tab-bg:var(--color-base-200)]",
            input { r#type: "radio", name: "soundpack-tab" }
            Settings2 { class: "w-5 h-5 mr-2" }
            "Manage"
          }
          div { class: "tab-content overflow-hidden bg-base-200 border-base-300 p-4",
            div { class: "space-y-4",
              div { class: "text-base-content",
                div {
                  div { class: "font-medium text-sm pb-1",
                    "Found {soundpack_count_keyboard + soundpack_count_mouse} soundpack(s)"
                  }
                  ul { class: "list-disc pl-6",
                    li { class: "text-sm text-base-content/70",
                      "Keyboard: {soundpack_count_keyboard}"
                    }
                    li { class: "text-sm text-base-content/70",
                      "Mouse: {soundpack_count_mouse}"
                    }
                  }
                }
              }
              div { class: "space-y-2",
                div { class: "text-base-content/70 text-sm",
                  "Refresh soundpack list to detect newly added or removed soundpacks."
                }
                div { class: "flex items-center gap-4",
                  button {
                    class: "btn btn-soft btn-sm",
                    onclick: move |_| {
                        refresh_soundpacks_cache();
                    },
                    disabled: refreshing_soundpacks(),
                    if refreshing_soundpacks() {
                      span { class: "loading loading-spinner loading-xs mr-2" }
                      "Refreshing..."
                    } else {
                      RefreshCcw { class: "w-4 h-4 mr-1" }
                      "Refresh"
                    }
                  }
                  // Last scan info
                  if app_state.read().optimized_cache.last_scan > 0 {
                    div { class: "text-xs text-base-content/60",
                      "Last scan "
                      {
                          let last_scan = app_state.read().optimized_cache.last_scan;
                          let now = std::time::SystemTime::now()
                              .duration_since(std::time::SystemTime::UNIX_EPOCH)
                              .unwrap_or_default()
                              .as_secs();
                          let diff = now.saturating_sub(last_scan);
                          if diff < 60 {
                              ": just now".to_string()
                          } else if diff < 3600 {
                              format!("{} min ago", diff / 60)
                          } else if diff < 86400 {
                              format!("{} hr ago", diff / 3600)
                          } else {
                              format!("{} days ago", diff / 86400)
                          }
                      }
                    }
                  }
                }
              }
              div { class: "divider" }
              div { class: "space-y-2",
                div { class: "text-base-content font-medium text-sm",
                  "Soundpack folder path"
                }
                input {
                  value: "{soundpacks_dir_absolute}",
                  class: "input input-sm w-full",
                  readonly: true,
                }
                button {
                  class: "btn btn-soft btn-sm",
                  onclick: move |_| {
                    // Open the soundpack directory in the file explorer
                    if let Err(e) = open_directory(&soundpacks_dir_absolute) {
                        eprintln!("Failed to open soundpack directory: {}", e);
                    }
                  },
                  FolderOpen { class: "w-4 h-4 mr-1" }
                  "Open soundpack folder"
                }
              }
              div { class: "divider" }
              div { class: "space-y-2",
                div { class: "text-base-content font-medium text-sm",
                  "Need more soundpacks?"
                }
                a {
                  class: "btn btn-soft btn-sm",
                  href: "https://mechvibes.com/soundpacks",
                  target: "_blank",
                  "Browse soundpacks"
                  ExternalLink { class: "w-4 h-4 ml-1" }
                }
              }
            }
          }
        }
      }
    }
}

#[component]
fn SoundpackTable(
    soundpacks: Vec<crate::state::soundpack_cache::SoundpackMetadata>,
    soundpack_type: &'static str,
) -> Element {
    // Search state
    let mut search_query = use_signal(String::new);

    // Clone soundpacks for use in memo
    let soundpacks_clone = soundpacks.clone();

    // Filter soundpacks based on search query
    let filtered_soundpacks = use_memo(move || {
        let query = search_query().to_lowercase();
        if query.is_empty() {
            soundpacks_clone.clone()
        } else {
            soundpacks_clone
                .iter()
                .filter(|pack| {
                    pack.name.to_lowercase().contains(&query)
                        || pack.id.to_lowercase().contains(&query)
                        || pack
                            .author
                            .as_ref()
                            .map_or(false, |author| author.to_lowercase().contains(&query))
                        || pack
                            .tags
                            .iter()
                            .any(|tag| tag.to_lowercase().contains(&query))
                })
                .cloned()
                .collect()
        }
    });

    if soundpacks.is_empty() {
        rsx! {
          div { class: "p-4 text-center text-base-content/70",
            "No {soundpack_type} soundpack found!"
          }
        }
    } else {
        rsx! {
          div { class: "space-y-4",
            // Search field
            div { class: "flex items-center px-3 gap-2",
              input {
                class: "input w-full",
                placeholder: "Search {soundpack_type.to_lowercase()} soundpacks...",
                value: "{search_query}",
                oninput: move |evt| search_query.set(evt.value()),
              }
              button { class: "btn btn-primary",
                Plus { class: "w-4 h-4 mr-2" }
                "Add"
              }
            }

            // Table
            div { class: "overflow-x-auto max-h-[calc(100vh-500px)]",
              if filtered_soundpacks().is_empty() {
                div { class: "p-4 text-center text-base-content/70",
                  "No result match your search!"
                }
              } else {
                table { class: "table table-sm w-full",
                  tbody {
                    for pack in filtered_soundpacks() {
                      SoundpackTableRow { soundpack: pack }
                    }
                  }
                }
              }
            }
          }
        }
    }
}

#[component]
fn SoundpackTableRow(soundpack: crate::state::soundpack_cache::SoundpackMetadata) -> Element {
    rsx! {
      tr { class: "hover:bg-base-100",
        td { class: "flex items-center gap-4",
          // Icon
          div { class: "flex items-center justify-center",
            if let Some(icon) = &soundpack.icon {
              if !icon.is_empty() {
                div { class: "w-8 h-8 rounded-lg overflow-hidden",
                  img {
                    class: "w-full h-full  object-cover",
                    src: "{icon}",
                    alt: "{soundpack.name}",
                  }
                }
              } else {
                div { class: "w-8 h-8 rounded bg-base-300 flex items-center justify-center",
                  Music { class: "w-4 h-4 text-base-content/40" }
                }
              }
            } else {
              div { class: "w-8 h-8 rounded bg-base-300 flex items-center justify-center",
                Music { class: "w-4 h-4 text-base-content/40" }
              }
            }
          }
          // Name
          div {
            div { class: "font-medium text-sm text-base-content line-clamp-1",
              "{soundpack.name}"
            }
            if let Some(author) = &soundpack.author {
              div { class: "text-xs text-base-content/50", "by {author}" }
            }
          }
        }
        // Actions
        td {
          div { class: "flex items-center justify-end gap-2",
            button {
              class: "btn btn-soft btn-xs",
              title: "Open soundpack folder",
              FolderOpen { class: "w-4 h-4" }
            }
            button {
              class: "btn btn-error btn-xs",
              title: "Delete this soundpack",
              Trash { class: "w-4 h-4" }
            }
          }
        }
      }
    }
}
