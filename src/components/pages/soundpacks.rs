use crate::{
    components::ui::PageHeader,
    state::{app::use_app_state, paths},
};
use dioxus::prelude::*;
use lucide_dioxus::{
    ExternalLink, FolderOpen, Keyboard, Mouse, Music, Plus, RefreshCcw, Settings2, Trash, Upload, X,
};
use std::io::Read;
use std::sync::Arc;
use zip::ZipArchive;

// Structure to hold soundpack information after extraction
#[derive(Debug, Clone)]
struct SoundpackInfo {
    name: String,
    id: String,
}

// Helper function to extract and install soundpack from ZIP file
fn extract_and_install_soundpack(file_path: &str) -> Result<SoundpackInfo, String> {
    use std::fs::{create_dir_all, File};
    use std::path::Path;

    // Open ZIP file
    let file = File::open(file_path).map_err(|e| format!("Failed to open ZIP file: {}", e))?;
    let mut archive =
        ZipArchive::new(file).map_err(|e| format!("Failed to read ZIP archive: {}", e))?;

    // Find config.json to determine soundpack info
    let mut config_content = String::new();
    let mut soundpack_id = String::new();
    let mut found_config = false;
    // First pass: find and read config.json
    for i in 0..archive.len() {
        let mut file = archive
            .by_index(i)
            .map_err(|e| format!("Failed to read archive entry: {}", e))?;
        let file_path = file.name().to_string(); // Convert to owned String

        // Look for config.json in any directory level
        if file_path.ends_with("config.json") {
            file.read_to_string(&mut config_content)
                .map_err(|e| format!("Failed to read config.json: {}", e))?;

            // Extract directory name as soundpack ID
            let path_parts: Vec<&str> = file_path.split('/').collect();
            if path_parts.len() > 1 {
                // config.json is in a subdirectory, use that as the ID
                soundpack_id = path_parts[0].to_string();
            } else {
                // config.json is in root, we'll need to determine ID from config
                soundpack_id = "imported_soundpack".to_string();
            }
            found_config = true;
            break;
        }
    }

    if !found_config {
        return Err("No config.json found in ZIP file".to_string());
    }

    // Parse config to get soundpack info
    let config: serde_json::Value = serde_json::from_str(&config_content)
        .map_err(|e| format!("Failed to parse config.json: {}", e))?;

    let soundpack_name = config
        .get("name")
        .and_then(|v| v.as_str())
        .unwrap_or("Unknown Soundpack")
        .to_string();

    // If we couldn't determine ID from path, try to use a clean version of the name
    if soundpack_id == "imported_soundpack" {
        soundpack_id = soundpack_name
            .to_lowercase()
            .chars()
            .filter(|c| c.is_alphanumeric() || *c == '_' || *c == '-')
            .collect::<String>()
            .replace(' ', "_");
    }

    // Handle V1 to V2 conversion if needed
    let validation_result = crate::utils::soundpack_validator::validate_soundpack_config(&format!(
        "temp_config_{}.json",
        soundpack_id
    ));
    let mut final_config_content = config_content.clone();

    if validation_result.status
        == crate::utils::soundpack_validator::SoundpackValidationStatus::VersionOneNeedsConversion
    {
        // Convert V1 to V2 format
        let temp_input = format!("temp_v1_{}.json", soundpack_id);
        let temp_output = format!("temp_v2_{}.json", soundpack_id);

        std::fs::write(&temp_input, &config_content)
            .map_err(|e| format!("Failed to write temp config: {}", e))?;

        match crate::utils::config_converter::convert_v1_to_v2(&temp_input, &temp_output) {
            Ok(()) => {
                final_config_content = std::fs::read_to_string(&temp_output)
                    .map_err(|e| format!("Failed to read converted config: {}", e))?;

                // Clean up temp files
                let _ = std::fs::remove_file(&temp_input);
                let _ = std::fs::remove_file(&temp_output);
            }
            Err(e) => {
                // Clean up temp files on error
                let _ = std::fs::remove_file(&temp_input);
                let _ = std::fs::remove_file(&temp_output);
                return Err(format!("Failed to convert V1 soundpack: {}", e));
            }
        }
    }

    // Determine installation directory using soundpack ID
    let soundpacks_dir = crate::state::paths::utils::get_soundpacks_dir_absolute();
    let install_dir = Path::new(&soundpacks_dir).join(&soundpack_id);

    // Create installation directory
    create_dir_all(&install_dir)
        .map_err(|e| format!("Failed to create soundpack directory: {}", e))?;

    // Second pass: extract all files
    let mut archive =
        ZipArchive::new(File::open(file_path).map_err(|e| format!("Failed to reopen ZIP: {}", e))?)
            .map_err(|e| format!("Failed to reread ZIP archive: {}", e))?;
    for i in 0..archive.len() {
        let mut file = archive
            .by_index(i)
            .map_err(|e| format!("Failed to read archive entry: {}", e))?;
        let file_path = file.name().to_string(); // Convert to owned String

        // Skip directories
        if file_path.ends_with('/') {
            continue;
        }

        // Determine output path - strip the first directory level if it exists
        let output_path = if file_path.contains('/') {
            let parts: Vec<&str> = file_path.split('/').collect();
            if parts.len() > 1 {
                // Skip the first part and join the rest
                install_dir.join(parts[1..].join("/"))
            } else {
                install_dir.join(&file_path)
            }
        } else {
            install_dir.join(&file_path)
        };

        // Create parent directories if needed
        if let Some(parent) = output_path.parent() {
            create_dir_all(parent).map_err(|e| format!("Failed to create directory: {}", e))?;
        }

        // Extract file
        let mut output_file =
            File::create(&output_path).map_err(|e| format!("Failed to create file: {}", e))?;
        std::io::copy(&mut file, &mut output_file)
            .map_err(|e| format!("Failed to extract file: {}", e))?;

        // If this is config.json, write the potentially converted version
        if file_path.ends_with("config.json") {
            std::fs::write(&output_path, &final_config_content)
                .map_err(|e| format!("Failed to write converted config: {}", e))?;
        }
    }
    Ok(SoundpackInfo {
        name: soundpack_name,
        id: soundpack_id,
    })
}

// Upload modal component
#[component]
fn SoundpackUploadModal(
    show: Signal<bool>,
    progress: Signal<String>,
    error: Signal<String>,
    success: Signal<String>,
    on_upload: EventHandler<MouseEvent>,
) -> Element {
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
        div { class: "relative bg-base-100 rounded-lg shadow-xl p-6 w-full max-w-md mx-4",
          // Header
          div { class: "flex items-center justify-between mb-4",
            h3 { class: "text-lg font-semibold text-base-content", "Upload Soundpack" }
            if progress().is_empty() {
              button {
                class: "btn btn-ghost btn-sm btn-circle",
                onclick: move |_| show.set(false),
                X { class: "w-4 h-4" }
              }
            }
          }

          // Content
          div { class: "space-y-4",
            // Instructions
            if progress().is_empty() && error().is_empty() && success().is_empty() {
              div { class: "text-sm text-base-content/70",
                "Select a ZIP file containing a soundpack to install it."
                br {}
                "Supports both V1 and V2 soundpack formats."
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

            // Upload button
            if progress().is_empty() && success().is_empty() {
              div { class: "flex justify-end gap-2",
                button {
                  class: "btn btn-ghost",
                  onclick: move |_| show.set(false),
                  "Cancel"
                }
                button {
                  class: "btn btn-primary",
                  onclick: move |evt| on_upload.call(evt),
                  Upload { class: "w-4 h-4 mr-2" }
                  "Select File"
                }
              }
            }
          }
        }
      }
    }
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

    // Get access to audio context for reloading soundpacks
    let audio_ctx: Arc<crate::libs::audio::AudioContext> = use_context();
    // Upload modal state
    let mut show_upload_modal = use_signal(|| false);
    let mut upload_progress = use_signal(|| String::new());
    let mut upload_error = use_signal(|| String::new());
    let mut upload_success = use_signal(|| String::new());

    // UI state for notification and loading
    let mut refreshing_soundpacks = use_signal(|| false);
    // File upload handler
    let handle_upload_click = {
        let audio_ctx_upload = audio_ctx.clone();
        move |_| {
            upload_error.set(String::new());
            upload_success.set(String::new());
            upload_progress.set("Selecting file...".to_string());

            // Clone necessary variables for the async task
            let mut progress_signal = upload_progress.clone();
            let mut error_signal = upload_error.clone();
            let mut success_signal = upload_success.clone();
            let mut modal_signal = show_upload_modal.clone();
            let mut app_state_clone = app_state.clone();
            let audio_ctx_clone = audio_ctx_upload.clone();

            // Try test files so the demo works regardless of which is present
            let test_files = [
                "test_soundpack.zip",
                "test_soundpack_v1.zip",
                "test_nested_soundpack.zip",
                "test_v1_proper.zip",
            ];

            spawn(async move {
                // Try each test file until one works
                let mut success = false;
                for test_file in test_files {
                    progress_signal.set(format!("Processing {}...", test_file));

                    match extract_and_install_soundpack(test_file) {
                        Ok(soundpack_info) => {
                            success_signal
                                .set(format!("Successfully installed: {}", soundpack_info.name));
                            progress_signal.set(String::new());

                            // Refresh soundpack cache
                            let mut fresh_cache =
                                crate::state::soundpack_cache::SoundpackCache::load();
                            fresh_cache.refresh_from_directory();
                            fresh_cache.save();
                            app_state_clone.write().optimized_cache = Arc::new(fresh_cache);

                            // Reload current soundpacks
                            crate::state::app::reload_current_soundpacks(&audio_ctx_clone);

                            success = true;
                            break;
                        }
                        Err(_) => {
                            // Continue to next test file if this one fails
                            continue;
                        }
                    }
                }

                if !success {
                    error_signal.set("Could not find any test soundpack files to install. In a real implementation, we would use a native file dialog here.".to_string());
                    progress_signal.set(String::new());
                } else {
                    // Close modal after delay
                    use futures_timer::Delay;
                    use std::time::Duration;
                    Delay::new(Duration::from_millis(2000)).await;
                    modal_signal.set(false);
                    success_signal.set(String::new());
                }
            });
        }
    };

    let mut refresh_soundpacks_cache = {
        let audio_ctx_refresh = audio_ctx.clone();
        move || {
            println!("🔄 Refreshing soundpack cache from settings page...");

            // Set loading state to true - directly update the signal
            refreshing_soundpacks.set(true);
            println!("🌻 Loading state set to: {}", refreshing_soundpacks());

            // Clone necessary variables for the async task
            let mut refreshing_signal = refreshing_soundpacks.clone();
            let audio_ctx_clone = audio_ctx_refresh.clone();
            let mut app_state_clone = app_state.clone();

            // Perform the refresh operation in a separate task to not block the UI
            spawn(async move {
                // Use async sleep instead of std::thread::sleep
                use futures_timer::Delay;
                use std::time::Duration;

                Delay::new(Duration::from_millis(100)).await;

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
                Delay::new(Duration::from_millis(100)).await;

                // Reset loading state
                refreshing_signal.set(false);
                println!("🌻 Loading state reset to: {}", refreshing_signal());

                println!("✅ Soundpack refresh complete");
            });
        }
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
            "Keyboard ({keyboard_soundpacks().len()})"
          }
          div { class: "tab-content  overflow-hidden bg-base-200 border-base-300 py-4 px-0",
            SoundpackTable {
              soundpacks: keyboard_soundpacks(),
              soundpack_type: "Keyboard",
              on_add_click: Some(EventHandler::new(move |_| show_upload_modal.set(true))),
            }
          }
          // Mouse tab
          label { class: "tab [--tab-border-color:var(--color-base-300)] [--tab-bg:var(--color-base-200)]",
            input { r#type: "radio", name: "soundpack-tab" }
            Mouse { class: "w-5 h-5 mr-2" }
            "Mouse ({mouse_soundpacks().len()})"
          }
          div { class: "tab-content overflow-hidden bg-base-200 border-base-300 py-4 px-0",
            SoundpackTable {
              soundpacks: mouse_soundpacks(),
              soundpack_type: "Mouse",
              on_add_click: Some(EventHandler::new(move |_| show_upload_modal.set(true))),
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
                button { class: "btn btn-soft btn-sm",
                  FolderOpen { class: "w-4 h-4 mr-1" }
                  "Open soundpack folder"
                }
              }
              div { class: "divider" }
              div { class: "space-y-2",
                div { class: "text-base-content font-medium text-sm",
                  "Install new soundpack"
                }
                div { class: "text-base-content/70 text-sm",
                  "Upload a ZIP file containing a soundpack to install it."
                }
                button {
                  class: "btn btn-primary btn-sm",
                  onclick: move |_| show_upload_modal.set(true),
                  Upload { class: "w-4 h-4 mr-2" }
                  "Upload Soundpack"
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

        // Upload modal
        SoundpackUploadModal {
          show: show_upload_modal,
          progress: upload_progress,
          error: upload_error,
          success: upload_success,
          on_upload: handle_upload_click,
        }
      }
    }
}

#[component]
fn SoundpackTable(
    soundpacks: Vec<crate::state::soundpack_cache::SoundpackMetadata>,
    soundpack_type: &'static str,
    on_add_click: Option<EventHandler<MouseEvent>>,
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
          div { class: "space-y-4", // Search field
            div { class: "flex items-center px-3 gap-2",
              input {
                class: "input w-full",
                placeholder: "Search {soundpack_type.to_lowercase()} soundpacks...",
                value: "{search_query}",
                oninput: move |evt| search_query.set(evt.value()),
              }
              if let Some(add_handler) = on_add_click {
                button {
                  class: "btn btn-primary",
                  onclick: move |evt| add_handler.call(evt),
                  Plus { class: "w-4 h-4 mr-2" }
                  "Add"
                }
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
