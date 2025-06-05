use crate::state::paths;
use crate::state::soundpack::SoundpackMetadata;
use crate::state::{app::use_state_trigger, paths::utils::open_path};
use crate::utils::path;
use dioxus::document::eval;
use dioxus::prelude::*;
use lucide_dioxus::{FolderOpen, Music, Plus, Trash};

use super::ConfirmDeleteModal;

/// Open a soundpack folder in the system file manager
fn open_soundpack_folder(soundpack_id: &str) -> Result<(), String> {
    let soundpack_path = paths::soundpacks::soundpack_dir(soundpack_id);
    open_path(&soundpack_path).map_err(|e| format!("Failed to open soundpack folder: {}", e))
}

/// Delete a soundpack directory and all its contents
fn delete_soundpack(soundpack_id: &str) -> Result<(), String> {
    let soundpack_path = paths::soundpacks::soundpack_dir(soundpack_id); // Check if the directory exists
    if !path::directory_exists(&soundpack_path) {
        return Err(format!("Soundpack directory not found: {}", soundpack_path));
    }

    // Remove the entire directory
    std::fs::remove_dir_all(&soundpack_path)
        .map_err(|e| format!("Failed to delete soundpack directory: {}", e))?;

    println!("🗑️ Successfully deleted soundpack: {}", soundpack_id);
    Ok(())
}

#[component]
pub fn SoundpackTable(
    soundpacks: Vec<SoundpackMetadata>,
    soundpack_type: &'static str,
    on_add_click: Option<EventHandler<MouseEvent>>,
) -> Element {
    // Search state
    let mut search_query = use_signal(String::new);

    // Filter soundpacks based on search query - computed every render to be reactive to props changes
    let query = search_query().to_lowercase();
    let filtered_soundpacks: Vec<SoundpackMetadata> = if query.is_empty() {
        soundpacks.clone()
    } else {
        soundpacks
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
    };

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
              if let Some(add_handler) = on_add_click {
                button {
                  class: "btn btn-neutral",
                  onclick: move |evt| add_handler.call(evt),
                  Plus { class: "w-4 h-4 mr-2" }
                  "Add"
                }
              }
            } // Table
            div { class: "overflow-x-auto max-h-[calc(100vh-500px)]",
              if filtered_soundpacks.is_empty() {
                div { class: "p-4 text-center text-base-content/70",
                  "No result match your search!"
                }
              } else {
                table { class: "table table-sm w-full",
                  tbody {
                    for pack in filtered_soundpacks {
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
pub fn SoundpackTableRow(soundpack: SoundpackMetadata) -> Element {
    let state_trigger = use_state_trigger();

    // Handlers for button clicks
    let on_open_folder = {
        let soundpack_id = soundpack.id.clone();
        move |_| {
            let soundpack_id = soundpack_id.clone();
            spawn(async move {
                match open_soundpack_folder(&soundpack_id) {
                    Ok(_) => println!(
                        "✅ Successfully opened folder for soundpack: {}",
                        soundpack_id
                    ),
                    Err(e) => eprintln!(
                        "❌ Failed to open folder for soundpack {}: {}",
                        soundpack_id, e
                    ),
                }
            });
        }
    };

    // Handler for delete button click

    let on_confirm_delete = {
        let soundpack_id = soundpack.id.clone();
        let trigger = state_trigger.clone();
        move |_| {
            let soundpack_id = soundpack_id.clone();
            let trigger = trigger.clone();
            spawn(async move {
                match delete_soundpack(&soundpack_id) {
                    Ok(_) => {
                        println!("✅ Successfully deleted soundpack: {}", soundpack_id);
                        eval(&format!("confirm_delete_modal_{}.close()", soundpack_id));
                        // Trigger state refresh to update the UI
                        trigger.call(());
                    }
                    Err(e) => {
                        eprintln!("❌ Failed to delete soundpack {}: {}", soundpack_id, e);
                        // Could show an error modal here if needed
                    }
                }
            });
        }
    };
    rsx! {
      tr { class: "hover:bg-base-100",
        td { class: "flex items-center gap-4",
          // Icon
          div { class: "flex items-center justify-center",
            if let Some(icon) = &soundpack.icon {
              if !icon.is_empty() {
                div { class: "w-8 h-8 rounded-box overflow-hidden",
                  img {
                    class: "w-full h-full object-cover",
                    src: "{icon}",
                    alt: "{soundpack.name}",
                  }
                }
              } else {
                div { class: "w-8 h-8 rounded-box bg-base-300 flex items-center justify-center",
                  Music { class: "w-4 h-4 text-base-content/40" }
                }
              }
            } else {
              div { class: "w-8 h-8 rounded-box bg-base-300 flex items-center justify-center",
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
          div { class: "flex items-center justify-end gap-1",
            button {
              class: "btn btn-soft btn-xs",
              title: "Open soundpack folder",
              onclick: on_open_folder,
              FolderOpen { class: "w-4 h-4" }
            }
            button {
              class: "btn btn-soft btn-error btn-xs",
              title: "Delete this soundpack",
              onclick: move |_| {
                  eval(
                      &format!(
                          "document.getElementById(\"confirm_delete_modal_{}\").showModal()",
                          soundpack.id,
                      ),
                  );
              },
              Trash { class: "w-4 h-4" }
            }
          }
        }
      }
      // Delete confirmation modal
      ConfirmDeleteModal {
        modal_id: format!("confirm_delete_modal_{}", soundpack.id),
        soundpack_name: soundpack.name.clone(),
        on_confirm: on_confirm_delete,
      }
    }
}
