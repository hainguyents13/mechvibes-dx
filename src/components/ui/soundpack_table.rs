use crate::state::paths;
use crate::state::soundpack::SoundpackMetadata;
use crate::state::{ app::use_state_trigger };
use crate::utils::path::{ open_path, directory_exists };
use dioxus::document::eval;
use dioxus::prelude::*;
use lucide_dioxus::{ FolderOpen, Music, Plus, RefreshCw, Trash };
use std::sync::Arc;

use super::ConfirmDeleteModal;

/// Open a soundpack folder in the system file manager
/// Opens the specific soundpack folder
fn open_soundpack_folder(soundpack_id: &str) -> Result<(), String> {
    use std::path::PathBuf;

    let soundpack_path = paths::soundpacks::soundpack_dir(soundpack_id);

    // Normalize path separators for Windows
    let normalized_path = PathBuf::from(&soundpack_path);
    let normalized_str = normalized_path.to_string_lossy().to_string();

    println!("🔍 Opening soundpack folder:");
    println!("   Soundpack ID: {}", soundpack_id);
    println!("   Resolved path: {}", soundpack_path);
    println!("   Normalized path: {}", normalized_str);

    // Check if path exists
    if !normalized_path.exists() {
        return Err(format!("Soundpack folder does not exist: {}", normalized_str));
    }

    open_path(&normalized_str).map_err(|e| format!("Failed to open soundpack folder: {}", e))
}

/// Delete a soundpack directory and all its contents
fn delete_soundpack(soundpack_id: &str) -> Result<(), String> {
    let soundpack_path = paths::soundpacks::soundpack_dir(soundpack_id);

    // Check if the directory exists
    if !directory_exists(&soundpack_path) {
        return Err(format!("Soundpack directory not found: {}", soundpack_path));
    }

    // Remove the entire directory
    std::fs
        ::remove_dir_all(&soundpack_path)
        .map_err(|e| format!("Failed to delete soundpack directory: {}", e))?;

    println!("🗑️ Successfully deleted soundpack: {}", soundpack_id);
    Ok(())
}

#[component]
pub fn SoundpackTable(
    soundpacks: Vec<SoundpackMetadata>,
    soundpack_type: &'static str,
    on_add_click: Option<EventHandler<MouseEvent>>
) -> Element {
    // Search state
    let mut search_query = use_signal(String::new);

    // Refresh state
    let refreshing_soundpacks = use_signal(|| false);
    let state_trigger = use_state_trigger();
    let audio_ctx: Arc<crate::libs::audio::AudioContext> = use_context();

    // Filter soundpacks based on search query - computed every render to be reactive to props changes
    let query = search_query().to_lowercase();
    let filtered_soundpacks: Vec<SoundpackMetadata> = if query.is_empty() {
        soundpacks.clone()
    } else {
        soundpacks
            .iter()
            .filter(|pack| {
                pack.name.to_lowercase().contains(&query) ||
                    pack.id.to_lowercase().contains(&query) ||
                    pack.author
                        .as_ref()
                        .map_or(false, |author| author.to_lowercase().contains(&query)) ||
                    pack.tags.iter().any(|tag| tag.to_lowercase().contains(&query))
            })
            .cloned()
            .collect()
    };

    // Refresh handler
    let refresh_soundpacks_cache = {
        let audio_ctx_refresh = audio_ctx.clone();
        let refreshing_soundpacks = refreshing_soundpacks.clone();
        let state_trigger_clone = state_trigger.clone();
        Callback::new(move |_| {
            // Prevent multiple concurrent refreshes
            if refreshing_soundpacks() {
                println!("🔄 Refresh already in progress, skipping...");
                return;
            }

            let audio_ctx = audio_ctx_refresh.clone();
            let mut refreshing_soundpacks = refreshing_soundpacks.clone();
            let state_trigger = state_trigger_clone.clone();

            spawn(async move {
                refreshing_soundpacks.set(true);
                println!("🔄 Refreshing soundpack cache...");

                // Reload soundpacks in audio context
                crate::state::app::reload_current_soundpacks(&audio_ctx);

                // Trigger state update to refresh UI
                state_trigger.call(());

                println!("✅ Soundpack cache refreshed");
                refreshing_soundpacks.set(false);
            });
        })
    };

    rsx! {
      div { class: "space-y-4",
        // Search field
        div { class: "flex items-center px-3 gap-2",
          input {
            class: "input input-sm w-full",
            placeholder: "Search {soundpack_type.to_lowercase()} sound packs...",
            value: "{search_query}",
            oninput: move |evt| search_query.set(evt.value()),
          }
          button {
            class: "btn btn-sm btn-ghost",
            disabled: refreshing_soundpacks(),
            onclick: refresh_soundpacks_cache,
            title: "Refresh sound pack list",
            if refreshing_soundpacks() {
              span { class: "loading loading-spinner loading-xs" }
            } else {
              RefreshCw { class: "w-4 h-4" }
            }
          }
          if let Some(add_handler) = on_add_click {
            button {
              class: "btn btn-sm btn-neutral",
              onclick: move |evt| add_handler.call(evt),
              Plus { class: "w-4 h-4 mr-2" }
              "Add"
            }
          }
        }
        if soundpacks.is_empty() {
          div { class: "p-4 text-center text-sm text-base-content/70",
            "No {soundpack_type} sound pack found. You can add new sound packs by clicking the 'Add' button above."
          }
        } else {
          // Table
          div { class: "overflow-x-auto overflow-y-auto max-h-[calc(100vh-400px)] -mb-1",
            if filtered_soundpacks.is_empty() {
              div { class: "p-4 text-center text-sm text-base-content/70",
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
        let folder_path = soundpack.folder_path.clone();
        let soundpack_id = soundpack.id.clone();
        let soundpack_name = soundpack.name.clone();
        move |_| {
            let folder_path = folder_path.clone();
            let soundpack_id = soundpack_id.clone();
            let soundpack_name = soundpack_name.clone();
            spawn(async move {
                println!("🔍 Soundpack info:");
                println!("   Name: {}", soundpack_name);
                println!("   ID: {}", soundpack_id);
                println!("   Folder path: {}", folder_path);

                // Use folder_path if not empty, otherwise fall back to id
                let path_to_use = if !folder_path.is_empty() {
                    folder_path
                } else {
                    soundpack_id.clone()
                };

                match open_soundpack_folder(&path_to_use) {
                    Ok(_) =>
                        println!("✅ Successfully opened folder for soundpack: {}", soundpack_name),
                    Err(e) =>
                        eprintln!("❌ Failed to open folder for soundpack {}: {}", soundpack_name, e),
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
                        // The modal will close automatically due to form method="dialog"
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
