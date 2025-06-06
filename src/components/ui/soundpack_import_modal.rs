use crate::{
    components::ui::{ImportStep, ProgressStep},
    state::app::{use_app_state, use_state_trigger},
    utils::soundpack_installer::{
        check_soundpack_id_conflict, extract_and_install_soundpack, get_soundpack_id_from_zip,
    },
};
use dioxus::prelude::*;
use lucide_dioxus::HardDriveUpload;
use std::sync::Arc;

#[component]
pub fn SoundpackImportModal(
    modal_id: String,
    audio_ctx: Arc<crate::libs::audio::AudioContext>,
    on_import_success: EventHandler<String>,
) -> Element {
    // Import step tracking
    let current_step = use_signal(|| ImportStep::Idle);
    let error = use_signal(|| String::new());
    let success = use_signal(|| String::new());
    // Get app state outside the handler
    let app_state = use_app_state();
    let state_trigger = use_state_trigger(); // File import handler
    let handle_import_click = {
        let audio_ctx = audio_ctx.clone();
        let app_state = app_state.clone();
        let state_trigger = state_trigger.clone();
        Callback::new(move |_| {
            let mut error = error.clone();
            let mut success = success.clone();
            let mut current_step = current_step.clone();
            let audio_ctx = audio_ctx.clone();
            let app_state = app_state.clone();
            let on_import_success = on_import_success.clone();
            let state_trigger = state_trigger.clone();

            spawn(async move {
                error.set(String::new());
                success.set(String::new());

                // Step 1: Opening file dialog
                current_step.set(ImportStep::OpeningDialog);
                futures_timer::Delay::new(std::time::Duration::from_millis(300)).await;

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
                        current_step.set(ImportStep::Idle);
                        return;
                    }
                };

                // Step 2: File selected
                current_step.set(ImportStep::FileSelected);
                futures_timer::Delay::new(std::time::Duration::from_millis(500)).await;

                let file_path = file_handle.path().to_string_lossy().to_string();

                // Step 3: Validating file
                current_step.set(ImportStep::Validating);
                futures_timer::Delay::new(std::time::Duration::from_millis(800)).await;

                // First, check if the soundpack ID already exists
                match get_soundpack_id_from_zip(&file_path) {
                    Ok(soundpack_id) => {
                        // Step 4: Checking for conflicts
                        current_step.set(ImportStep::CheckingConflicts);
                        futures_timer::Delay::new(std::time::Duration::from_millis(600)).await;

                        // Get current soundpacks from app state
                        let soundpacks = app_state.get_soundpacks();

                        if check_soundpack_id_conflict(&soundpack_id, &soundpacks) {
                            error.set(format!(
                                "A soundpack with ID '{}' already exists.\nPlease remove the existing soundpack first",
                                soundpack_id
                            ));
                            current_step.set(ImportStep::Idle);
                            return;
                        }

                        // Step 5: Installing soundpack
                        current_step.set(ImportStep::Installing);
                        futures_timer::Delay::new(std::time::Duration::from_millis(1000)).await;

                        match extract_and_install_soundpack(&file_path) {
                            Ok(soundpack_info) => {
                                // Step 6: Finalizing installation
                                current_step.set(ImportStep::Finalizing);
                                futures_timer::Delay::new(std::time::Duration::from_millis(700))
                                    .await;

                                // Reload soundpacks in audio context
                                crate::state::app::reload_current_soundpacks(&audio_ctx);

                                // Refresh the soundpack cache to show the new soundpack in the UI
                                println!("🔄 Triggering soundpack cache refresh after import...");
                                state_trigger.call(());

                                // Notify parent component (this will trigger UI update)
                                on_import_success.call(soundpack_id.clone());

                                // Step 7: Completed
                                current_step.set(ImportStep::Completed);
                                success.set(format!(
                                    "Successfully installed: {}",
                                    soundpack_info.name
                                ));

                                // Reset after showing success for a while
                                futures_timer::Delay::new(std::time::Duration::from_millis(2000))
                                    .await;
                                current_step.set(ImportStep::Idle);
                            }
                            Err(e) => {
                                error.set(format!("Failed to install soundpack: {}", e));
                                current_step.set(ImportStep::Idle);
                            }
                        }
                    }
                    Err(e) => {
                        error.set(format!("Invalid soundpack file: {}", e));
                        current_step.set(ImportStep::Idle);
                    }
                }
            });
        })
    };
    rsx! {
      dialog { class: "modal", id: "{modal_id}",
        div { class: "modal-box",
          form { method: "dialog",
            button {
              class: "btn btn-sm btn-circle btn-ghost absolute right-2 top-2",
              disabled: current_step() != ImportStep::Idle,
              "✕"
            }
          }
          h3 { class: "text-lg font-bold", "Import soundpack" }

          // Content
          div { class: "space-y-6 mt-4",
            // Instructions
            if current_step() == ImportStep::Idle && error().is_empty() && success().is_empty() {
              div { class: "text-sm text-base-content/70",
                "Select a ZIP file containing a soundpack to install it."
                br {}
                "Supports both v1 and v2 soundpack formats."
              }
            } // Progress steps
            if current_step() != ImportStep::Idle {
              div { class: "space-y-3",
                h4 { class: "font-medium text-sm text-base-content/80 mb-3",
                  "Import progress:"
                }
                ProgressStep {
                  step_number: 1,
                  title: "Opening file dialog".to_string(),
                  is_active: current_step() == ImportStep::OpeningDialog,
                  is_completed: current_step() > ImportStep::OpeningDialog,
                }

                ProgressStep {
                  step_number: 2,
                  title: "File selected".to_string(),
                  is_active: current_step() == ImportStep::FileSelected,
                  is_completed: current_step() > ImportStep::FileSelected,
                }

                ProgressStep {
                  step_number: 3,
                  title: "Validating soundpack file".to_string(),
                  is_active: current_step() == ImportStep::Validating,
                  is_completed: current_step() > ImportStep::Validating,
                }

                ProgressStep {
                  step_number: 4,
                  title: "Checking for conflicts".to_string(),
                  is_active: current_step() == ImportStep::CheckingConflicts,
                  is_completed: current_step() > ImportStep::CheckingConflicts,
                }

                ProgressStep {
                  step_number: 5,
                  title: "Installing soundpack".to_string(),
                  is_active: current_step() == ImportStep::Installing,
                  is_completed: current_step() > ImportStep::Installing,
                }

                ProgressStep {
                  step_number: 6,
                  title: "Finalizing installation".to_string(),
                  is_active: current_step() == ImportStep::Finalizing,
                  is_completed: current_step() > ImportStep::Finalizing,
                }

                ProgressStep {
                  step_number: 7,
                  title: "Import completed".to_string(),
                  is_active: current_step() == ImportStep::Completed,
                  is_completed: current_step() == ImportStep::Completed,
                }
              }
            }

            // Error
            if !error().is_empty() {
              div { class: "alert alert-error alert-soft",
                div { class: "text-sm", "{error()}" }
              }
            }

            // Success
            if !success().is_empty() {
              div { class: "alert alert-success alert-outline",
                div { class: "text-sm", "{success()}" }
              }
            }

            // Import button
            div { class: "flex justify-end gap-2",
              form { method: "dialog",
                button {
                  class: "btn btn-ghost",
                  disabled: current_step() != ImportStep::Idle,
                  "Cancel"
                }
              }
              button {
                class: "btn btn-neutral",
                onclick: handle_import_click,
                disabled: current_step() != ImportStep::Idle,
                HardDriveUpload { class: "w-4 h-4 mr-1" }
                "Select file"
              }
            }
          }
        }
        form { method: "dialog", class: "modal-backdrop",
          button { "close" }
        }
      }
    }
}
