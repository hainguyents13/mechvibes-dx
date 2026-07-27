use crate::state::paths;
use crate::state::soundpack::SoundpackMetadata;
use crate::state::{ app::use_state_trigger };
use crate::utils::config::use_config;
use crate::utils::path::{ open_path, directory_exists };
use dioxus::document::eval;
use dioxus::prelude::*;
use lucide_dioxus::{ Check, FolderOpen, Loader, Music, Plus, RefreshCw, Trash };
use std::sync::Arc;

use super::ConfirmDeleteModal;

/// Open a soundpack folder in the system file manager
fn open_soundpack_folder(soundpack_id: &str) -> Result<(), String> {
    use std::path::PathBuf;
    let soundpack_path = paths::soundpacks::soundpack_dir(soundpack_id);
    let normalized_path = PathBuf::from(&soundpack_path);
    let normalized_str = normalized_path.to_string_lossy().to_string();
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
    let (_config, _update_config) = use_config();

    // Per-pack loading state
    let loading_pack_id = use_signal(|| None::<String>);

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
                    SoundpackTableRow {
                      soundpack: pack,
                      soundpack_type: soundpack_type,
                      loading_pack_id: loading_pack_id.clone(),
                      audio_ctx: audio_ctx.clone(),
                      state_trigger: state_trigger.clone(),
                    }
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
pub fn SoundpackTableRow(
    soundpack: SoundpackMetadata,
    soundpack_type: &'static str,
    loading_pack_id: Signal<Option<String>>,
    audio_ctx: Arc<crate::libs::audio::AudioContext>,
    state_trigger: Callback<()>,
) -> Element {
    let state_trigger_row = use_state_trigger();
    let (config, update_config) = use_config();

    // Clone all fields upfront - soundpack is moved into closures below
    let sp_folder = soundpack.folder_path.clone();
    let sp_id = soundpack.id.clone();
    let sp_name = soundpack.name.clone();
    let sp_author = soundpack.author.clone();
    let sp_icon = soundpack.icon.clone();

    let is_loading = use_memo({
        let fp = sp_folder.clone();
        move || loading_pack_id() == Some(fp.clone())
    });

    let is_active = use_memo({
        let fp = sp_folder.clone();
        move || {
            let cfg = config();
            match soundpack_type {
                "Keyboard" => cfg.keyboard_soundpack == fp,
                "Mouse" => cfg.mouse_soundpack == fp,
                _ => false,
            }
        }
    });

    // Handlers for button clicks
    let on_open_folder = {
        let folder_path = sp_folder.clone();
        let soundpack_id = sp_id.clone();
        let soundpack_name = sp_name.clone();
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

    // Load handler
    let on_load = {
        let pack_id = sp_folder.clone();
        let pack_type = soundpack_type.to_string();
        let loading_pack_id = loading_pack_id.clone();
        let update_config = update_config.clone();
        let audio_ctx = audio_ctx.clone();
        let state_trigger_load = state_trigger.clone();
        move |_| {
            let pack_id = pack_id.clone();
            let pack_type = pack_type.clone();
            let mut loading_pack_id = loading_pack_id.clone();
            let update_config = update_config.clone();
            let audio_ctx = audio_ctx.clone();
            let state_trigger_load = state_trigger_load.clone();
            spawn(async move {
                loading_pack_id.set(Some(pack_id.clone()));
                // Update config
                let pack_id_clone = pack_id.clone();
                let pack_type_clone = pack_type.clone();
                update_config(Box::new(move |config| {
                    match pack_type_clone.as_str() {
                        "Keyboard" => { config.keyboard_soundpack = pack_id_clone; }
                        "Mouse" => { config.mouse_soundpack = pack_id_clone; }
                        _ => {}
                    }
                }));
                // Load audio
                let load_result = match pack_type.as_str() {
                    "Keyboard" => crate::libs::audio::load_keyboard_soundpack(&audio_ctx, &pack_id),
                    "Mouse" => crate::libs::audio::load_mouse_soundpack(&audio_ctx, &pack_id),
                    _ => Ok(()),
                };
                match load_result {
                    Ok(_) => println!("✅ Loaded soundpack: {}", pack_id),
                    Err(e) => eprintln!("❌ Failed to load soundpack {}: {}", pack_id, e),
                }
                loading_pack_id.set(None);
                state_trigger_load.call(());
            });
        }
    };

    // Handler for delete button click
    let on_confirm_delete = {
        let soundpack_id = sp_id.clone();
        let soundpack_folder = sp_folder.clone();
        let trigger = state_trigger_row.clone();
        let update_config = update_config.clone();
        move |_| {
            let soundpack_id = soundpack_id.clone();
            let soundpack_folder = soundpack_folder.clone();
            let trigger = trigger.clone();
            let update_config = update_config.clone();
            spawn(async move {
                // If the pack being deleted is currently active, clear the selection
                // to prevent a dangling config reference after deletion.
                let current_cfg = crate::state::config::AppConfig::load();
                let is_kb_active = current_cfg.keyboard_soundpack == soundpack_folder;
                let is_ms_active = current_cfg.mouse_soundpack == soundpack_folder;
                if is_kb_active || is_ms_active {
                    update_config(Box::new(move |config| {
                        if is_kb_active { config.keyboard_soundpack = String::new(); }
                        if is_ms_active { config.mouse_soundpack = String::new(); }
                    }));
                }

                match delete_soundpack(&soundpack_id) {
                    Ok(_) => {
                        // Trigger state refresh to update the UI
                        crate::state::app::refresh_global_cache();
                        trigger.call(());
                    }
                    Err(e) => {
                        eprintln!("❌ Failed to delete soundpack {}: {}", soundpack_id, e);
                    }
                }
            });
        }
    };
    rsx! {
      tr { class: "hover:bg-base-100 group",
        td { class: "flex items-center gap-4",
          // Icon
          div { class: "flex items-center justify-center",
            if let Some(icon) = &sp_icon {
              if !icon.is_empty() {
                div { class: "w-8 h-8 rounded-box overflow-hidden",
                  img {
                    class: "w-full h-full object-cover",
                    src: "{icon}",
                    alt: "{sp_name}",
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
              "{sp_name}"
            }
            if let Some(author) = &sp_author {
              div { class: "text-xs text-base-content/50", "by {author}" }
            }
          }
        }
        // Actions
        td {
          div { class: "flex items-center justify-end gap-1",
            // Load button (only when not currently active or loading)
            if !is_active() || is_loading() {
              button {
                class: "btn btn-soft btn-xs relative overflow-hidden",
                disabled: is_loading(),
                onclick: on_load,
                if is_loading() {
                  span { class: "loading loading-spinner loading-xs" }
                } else {
                  Loader { class: "w-4 h-4" }
                  "Load"
                }
              }
            } else {
              // Active indicator
              span { class: "inline-flex items-center gap-1 text-xs text-success font-medium",
                Check { class: "w-4 h-4 animate-check-pop" }
                "Active"
              }
            }
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
                          sp_id,
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
        modal_id: format!("confirm_delete_modal_{}", sp_id),
        soundpack_name: sp_name.clone(),
        on_confirm: on_confirm_delete,
      }
    }
}
