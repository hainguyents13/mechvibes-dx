use crate::state::app::AppState;
use crate::utils::soundpack_installer::{
    check_soundpack_id_conflict, extract_and_install_soundpack, get_soundpack_id_from_zip,
};
use dioxus::prelude::*;
use futures_timer::Delay;
use std::sync::Arc;
use std::time::Duration;

/// Handle the complete soundpack upload process
pub async fn handle_soundpack_upload(
    progress_signal: &mut Signal<String>,
    error_signal: &mut Signal<String>,
    success_signal: &mut Signal<String>,
    modal_signal: &mut Signal<bool>,
    app_state: &mut Signal<AppState>,
    audio_ctx: &Arc<crate::libs::audio::AudioContext>,
) {
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
            progress_signal.set(String::new());
            return;
        }
    };

    let file_path = file_handle.path().to_string_lossy().to_string();
    progress_signal.set("Checking soundpack...".to_string());

    // First, check if the soundpack ID already exists
    match get_soundpack_id_from_zip(&file_path) {
        Ok(soundpack_id) => {
            let soundpacks = app_state.with(|state| state.get_soundpacks());

            if check_soundpack_id_conflict(&soundpack_id, &soundpacks) {
                error_signal.set(format!(
                    "A soundpack with ID '{}' already exists. Please remove the existing soundpack first or rename the new one.",
                    soundpack_id
                ));
                progress_signal.set(String::new());
                return;
            }

            // No conflict, proceed with extraction
            progress_signal.set("Extracting soundpack...".to_string());

            match extract_and_install_soundpack(&file_path) {
                Ok(soundpack_info) => {
                    success_signal.set(format!("Successfully installed: {}", soundpack_info.name));
                    progress_signal.set(String::new());
                    // Refresh soundpack cache and reload
                    // TODO: Implement refresh_soundpacks_after_install functionality
                    // refresh_soundpacks_after_install(app_state, audio_ctx);

                    // Close modal after delay
                    Delay::new(Duration::from_millis(2000)).await;
                    modal_signal.set(false);
                    success_signal.set(String::new());
                }
                Err(e) => {
                    error_signal.set(format!("Failed to install soundpack: {}", e));
                    progress_signal.set(String::new());
                }
            }
        }
        Err(e) => {
            error_signal.set(format!("Invalid soundpack file: {}", e));
            progress_signal.set(String::new());
        }
    }
}

/// Create upload handler closure for use in components
pub fn create_upload_handler(
    upload_progress: Signal<String>,
    upload_error: Signal<String>,
    upload_success: Signal<String>,
    app_state: Signal<AppState>,
    audio_ctx: Arc<crate::libs::audio::AudioContext>,
) -> impl Fn(dioxus::prelude::MouseEvent) {
    move |_| {
        // Clone necessary variables for the async task
        let mut progress_signal = upload_progress.clone();
        let mut error_signal = upload_error.clone();
        let mut success_signal = upload_success.clone();
        let mut app_state_clone = app_state.clone();
        let audio_ctx_clone = audio_ctx.clone();

        // Reset state
        error_signal.set(String::new());
        success_signal.set(String::new());
        progress_signal.set("Opening file dialog...".to_string());

        spawn(async move {
            // Create a dummy modal signal since we're handling this externally
            let mut modal_signal = use_signal(|| false);

            handle_soundpack_upload(
                &mut progress_signal,
                &mut error_signal,
                &mut success_signal,
                &mut modal_signal,
                &mut app_state_clone,
                &audio_ctx_clone,
            )
            .await;
        });
    }
}
