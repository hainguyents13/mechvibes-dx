use crate::{
    components::ui::{ImportStep, ProgressStep},
    state::app::{use_app_state, use_state_trigger},
    utils::soundpack_installer::{
        check_soundpack_id_conflict, extract_and_install_soundpack, get_soundpack_id_from_zip,
    },
    utils::soundpack_validator::{validate_soundpack_structure, validate_zip_file},
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
    let error_step = use_signal(|| ImportStep::Idle);
    let error_message = use_signal(|| String::new());
    let success_step = use_signal(|| ImportStep::Idle);
    let success_message = use_signal(|| String::new());

    // Success messages for each step
    let file_selected_message = use_signal(|| String::new());
    let validation_success_message = use_signal(|| String::new());
    let conflict_check_message = use_signal(|| String::new());
    let installation_success_message = use_signal(|| String::new());
    let finalization_success_message = use_signal(|| String::new());

    // Get app state outside the handler
    let app_state = use_app_state();
    let state_trigger = use_state_trigger();

    // File import handler
    let handle_import_click = {
        let audio_ctx = audio_ctx.clone();
        let app_state = app_state.clone();
        let state_trigger = state_trigger.clone();
        Callback::new(move |_| {
            let mut error_step = error_step.clone();
            let mut error_message = error_message.clone();
            let mut success_step = success_step.clone();
            let mut success_message = success_message.clone();
            let mut current_step = current_step.clone();
            let mut file_selected_message = file_selected_message.clone();
            let mut validation_success_message = validation_success_message.clone();
            let mut conflict_check_message = conflict_check_message.clone();
            let mut installation_success_message = installation_success_message.clone();
            let mut finalization_success_message = finalization_success_message.clone();
            let audio_ctx = audio_ctx.clone();
            let app_state = app_state.clone();
            let on_import_success = on_import_success.clone();
            let state_trigger = state_trigger.clone();

            spawn(async move {
                error_step.set(ImportStep::Idle);
                error_message.set(String::new());
                success_step.set(ImportStep::Idle);
                success_message.set(String::new());

                // Clear all step success messages
                file_selected_message.set(String::new());
                validation_success_message.set(String::new());
                conflict_check_message.set(String::new());
                installation_success_message.set(String::new());
                finalization_success_message.set(String::new());

                // Step 1: Open file dialog and select file
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

                // File selected successfully
                current_step.set(ImportStep::FileSelected);
                let file_name = file_handle.file_name();
                file_selected_message.set(format!("Selected: {}", file_name));
                futures_timer::Delay::new(std::time::Duration::from_millis(500)).await;
                let file_path = file_handle.path().to_string_lossy().to_string();

                // Step 2: Validating file
                current_step.set(ImportStep::Validating);
                futures_timer::Delay::new(std::time::Duration::from_millis(400)).await;

                // First validate ZIP file structure
                if let Err(e) = validate_zip_file(&file_path).await {
                    error_step.set(ImportStep::Validating);
                    error_message.set(format!("Invalid ZIP file: {}", e));
                    return;
                }

                // Then validate soundpack structure and configuration
                match validate_soundpack_structure(&file_path).await {
                    Ok((soundpack_name, _config_content)) => {
                        validation_success_message
                            .set(format!("Valid soundpack: {}", soundpack_name));

                        // Now get the soundpack ID for conflict checking
                        match get_soundpack_id_from_zip(&file_path) {
                            Ok(soundpack_id) => {
                                // Step 3: Checking for conflicts
                                current_step.set(ImportStep::CheckingConflicts);
                                futures_timer::Delay::new(std::time::Duration::from_millis(600))
                                    .await;

                                // Get current soundpacks from app state
                                let soundpacks = app_state.get_soundpacks();
                                if check_soundpack_id_conflict(&soundpack_id, &soundpacks) {
                                    error_step.set(ImportStep::CheckingConflicts);
                                    error_message.set(format!(
                                        "A soundpack with ID '{}' already exists.\nPlease remove the existing soundpack first",
                                        soundpack_id
                                    ));
                                    return;
                                }
                                conflict_check_message
                                    .set(format!("No conflicts found for ID: {}", soundpack_id));

                                // Step 4: Installing soundpack
                                current_step.set(ImportStep::Installing);
                                futures_timer::Delay::new(std::time::Duration::from_millis(1000))
                                    .await;

                                match extract_and_install_soundpack(&file_path) {
                                    Ok(soundpack_info) => {
                                        installation_success_message
                                            .set(format!("Installed: {}", soundpack_info.name));

                                        // Step 5: Finalizing installation
                                        current_step.set(ImportStep::Finalizing);
                                        futures_timer::Delay::new(
                                            std::time::Duration::from_millis(700),
                                        )
                                        .await;

                                        // Reload soundpacks in audio context
                                        crate::state::app::reload_current_soundpacks(&audio_ctx);

                                        // Refresh the soundpack cache to show the new soundpack in the UI
                                        println!(
                                            "🔄 Triggering soundpack cache refresh after import..."
                                        );
                                        state_trigger.call(());

                                        finalization_success_message
                                            .set("Ready to use!".to_string()); // Notify parent component (this will trigger UI update)
                                        on_import_success.call(soundpack_id.clone());

                                        // Step 6: Completed
                                        current_step.set(ImportStep::Completed);
                                        success_step.set(ImportStep::Completed);
                                        success_message.set(format!(
                                            "Successfully installed: {}",
                                            soundpack_info.name
                                        ));

                                        // Reset after showing success for a while
                                        futures_timer::Delay::new(
                                            std::time::Duration::from_millis(2000),
                                        )
                                        .await;
                                        current_step.set(ImportStep::Idle);
                                        success_step.set(ImportStep::Idle);
                                        success_message.set(String::new());
                                    }
                                    Err(e) => {
                                        error_step.set(ImportStep::Installing);
                                        error_message
                                            .set(format!("Failed to install soundpack: {}", e));
                                    }
                                }
                            }
                            Err(e) => {
                                error_step.set(ImportStep::Validating);
                                error_message.set(format!("Failed to read soundpack ID: {}", e));
                            }
                        }
                    }
                    Err(e) => {
                        error_step.set(ImportStep::Validating);
                        error_message.set(format!("Invalid soundpack file: {}", e));
                    }
                }
            });
        })
    };
    rsx! {
      dialog { class: "modal", id: "{modal_id}",
        div { class: "modal-box max-w-2xl",
          form { method: "dialog",
            button { class: "btn btn-sm btn-circle btn-ghost absolute right-2 top-2",
              "✕"
            }
          }
          h3 { class: "font-bold text-lg mb-4",
            HardDriveUpload { class: "w-5 h-5 mr-2" }
            "Import Soundpack"
          }

          div { class: "space-y-4",
            // Progress Steps
            div { class: "space-y-2",
              ProgressStep {
                step_number: 1,
                title: "Select File".to_string(),
                current_step: *current_step.read(),
                error_message: if *error_step.read() == ImportStep::FileSelected { error_message.read().clone() } else { String::new() },
                success_message: file_selected_message.read().clone(),
              }

              ProgressStep {
                step_number: 2,
                title: "Validate Structure".to_string(),
                current_step: *current_step.read(),
                error_message: if *error_step.read() == ImportStep::Validating { error_message.read().clone() } else { String::new() },
                success_message: validation_success_message.read().clone(),
              }

              ProgressStep {
                step_number: 3,
                title: "Check Conflicts".to_string(),
                current_step: *current_step.read(),
                error_message: if *error_step.read() == ImportStep::CheckingConflicts { error_message.read().clone() } else { String::new() },
                success_message: conflict_check_message.read().clone(),
              }

              ProgressStep {
                step_number: 4,
                title: "Install Files".to_string(),
                current_step: *current_step.read(),
                error_message: if *error_step.read() == ImportStep::Installing { error_message.read().clone() } else { String::new() },
                success_message: installation_success_message.read().clone(),
              }

              ProgressStep {
                step_number: 5,
                title: "Finalize".to_string(),
                current_step: *current_step.read(),
                error_message: if *error_step.read() == ImportStep::Finalizing { error_message.read().clone() } else { String::new() },
                success_message: finalization_success_message.read().clone(),
              }
            }
          

          // // Success message display
          // if !success_message.read().is_empty() {
          //   div { class: "alert alert-success",
          //     strong { "Success: " }
          //     "{success_message.read()}"
          //   }
          // }
          }

          // Modal Actions
          div { class: "modal-action mt-6",
            form { method: "dialog",
              button { class: "btn btn-ghost", "Close" }
            }
            button {
              class: "btn btn-primary",
              disabled: *current_step.read() != ImportStep::Idle
                  && *current_step.read() != ImportStep::Completed,
              onclick: handle_import_click,
              if *current_step.read() == ImportStep::Idle
                  || *current_step.read() == ImportStep::Completed
              {
                HardDriveUpload { class: "w-4 h-4 mr-2" }
                "Select file"
              } else {
                span { class: "loading loading-spinner loading-sm mr-2" }
                "Importing..."
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
