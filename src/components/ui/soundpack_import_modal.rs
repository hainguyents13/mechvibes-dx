use crate::{
    state::app::use_app_state,
    utils::soundpack_installer::{
        check_soundpack_id_conflict, extract_and_install_soundpack, get_soundpack_id_from_zip,
    },
};
use dioxus::prelude::*;
use lucide_dioxus::{HardDriveUpload, X};
use std::sync::Arc;

#[component]
pub fn SoundpackImportModal(
    show: Signal<bool>,
    audio_ctx: Arc<crate::libs::audio::AudioContext>,
    on_import_success: EventHandler<String>,
) -> Element {
    let progress = use_signal(|| String::new());
    let error = use_signal(|| String::new());
    let success = use_signal(|| String::new()); // Get app state outside the handler
    let app_state = use_app_state();

    // File import handler
    let handle_import_click = {
        let audio_ctx = audio_ctx.clone();
        let app_state = app_state.clone();
        Callback::new(move |_| {
            let mut error = error.clone();
            let mut success = success.clone();
            let mut progress = progress.clone();
            let mut show = show.clone();
            let audio_ctx = audio_ctx.clone();
            let app_state = app_state.clone();
            let on_import_success = on_import_success.clone();

            spawn(async move {
                error.set(String::new());
                success.set(String::new());
                progress.set("Opening file dialog...".to_string());

                // Open file dialog to select ZIP file
                let file_dialog = rfd::AsyncFileDialog::new()
                    .add_filter("ZIP Files", &["zip"])
                    .set_title("Select Soundpack ZIP File")
                    .pick_file()
                    .await;

                let file_handle = match file_dialog {
                    Some(handle) => handle,
                    None => {
                        // User cancelled the dialog
                        progress.set(String::new());
                        return;
                    }
                };

                let file_path = file_handle.path().to_string_lossy().to_string();
                progress.set("Checking soundpack...".to_string());

                // First, check if the soundpack ID already exists
                match get_soundpack_id_from_zip(&file_path) {
                    Ok(soundpack_id) => {
                        // Get current soundpacks from app state
                        let soundpacks = app_state.get_soundpacks();

                        if check_soundpack_id_conflict(&soundpack_id, &soundpacks) {
                            error.set(format!(
                                "A soundpack with ID '{}' already exists.\nPlease remove the existing soundpack first",
                                soundpack_id
                            ));
                            progress.set(String::new());
                            return;
                        }

                        // No conflict, proceed with extraction
                        progress.set("Extracting soundpack...".to_string());

                        match extract_and_install_soundpack(&file_path) {
                            Ok(soundpack_info) => {
                                success.set(format!(
                                    "Successfully installed: {}",
                                    soundpack_info.name
                                ));
                                progress.set(String::new());

                                // Reload soundpacks in audio context
                                crate::state::app::reload_current_soundpacks(&audio_ctx);

                                // Notify parent component (this will trigger UI update)
                                on_import_success.call(soundpack_id);

                                // Close modal after delay
                                show.set(false);
                                success.set(String::new());
                            }
                            Err(e) => {
                                error.set(format!("Failed to install soundpack: {}", e));
                                progress.set(String::new());
                            }
                        }
                    }
                    Err(e) => {
                        error.set(format!("Invalid soundpack file: {}", e));
                        progress.set(String::new());
                    }
                }
            });
        })
    };

    if !show() {
        return rsx! {
          div {}
        };
    }

    rsx! {
      div { class: "fixed inset-0 z-50 flex items-center justify-center",
        // Backdrop
        div {
          class: "absolute inset-0 bg-black/50",
          onclick: move |_| {
              if progress().is_empty() {
                  show.set(false);
              }
          },
        }

        // Modal content
        div { class: "relative bg-base-100 rounded-box shadow-xl p-6 w-full max-w-md mx-4",
          // Header
          div { class: "flex items-center justify-between mb-4",
            h3 { class: "text-lg font-semibold text-base-content", "Import soundpack" }
            if progress().is_empty() {
              button {
                class: "btn btn-ghost btn-sm btn-circle",
                onclick: move |_| show.set(false),
                X { class: "w-4 h-4" }
              }
            }
          }

          // Content
          div { class: "space-y-4", // Instructions
            if progress().is_empty() && error().is_empty() && success().is_empty() {
              div { class: "text-sm text-base-content/70",
                "Select a ZIP file containing a soundpack to install it."
                br {}
                "Supports both v1 and v2 soundpack formats."
              }
            }

            // Progress
            if !progress().is_empty() {
              div { class: "flex items-center gap-3",
                span { class: "loading loading-spinner loading-sm" }
                span { class: "text-sm text-base-content", "{progress()}" }
              }
            }

            // Error
            if !error().is_empty() {
              div { class: "alert alert-error",
                div { class: "text-sm", "{error()}" }
              }
            }

            // Success
            if !success().is_empty() {
              div { class: "alert alert-success",
                div { class: "text-sm", "{success()}" }
              }
            }

            // Import button
            if progress().is_empty() && success().is_empty() {
              div { class: "flex justify-end gap-2",
                button {
                  class: "btn btn-ghost",
                  onclick: move |_| show.set(false),
                  "Cancel"
                }
                button {
                  class: "btn btn-neutral",
                  onclick: handle_import_click,
                  HardDriveUpload { class: "w-4 h-4 mr-2" }
                  "Select file"
                }
              }
            }
          }
        }
      }
    }
}
