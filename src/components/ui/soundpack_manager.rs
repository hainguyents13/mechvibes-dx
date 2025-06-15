use crate::state::{ app::use_app_state };
use dioxus::prelude::*;
use lucide_dioxus::{ ExternalLink, FolderOpen, RefreshCcw };
use std::sync::Arc;

#[component]
pub fn SoundpackManager(on_import_click: EventHandler<MouseEvent>) -> Element {
    let app_state = use_app_state();
    let audio_ctx: Arc<crate::libs::audio::AudioContext> = use_context();
    let state_trigger = crate::state::app::use_state_trigger();

    // UI state for notification and loading
    let refreshing_soundpacks = use_signal(|| false);
    let refresh_soundpacks_cache = {
        let audio_ctx_refresh = audio_ctx.clone();
        let mut refreshing_soundpacks = refreshing_soundpacks.clone();
        let state_trigger_clone = state_trigger.clone();
        Callback::new(move |_| {
            println!("🔄 Refresh button clicked!");
            // Set loading state to true
            refreshing_soundpacks.set(true);
            // Clone necessary variables for the async task
            let mut refreshing_signal = refreshing_soundpacks.clone();
            let audio_ctx_clone = audio_ctx_refresh.clone();
            let trigger = state_trigger_clone.clone();

            // Perform the refresh operation in a separate task to not block the UI
            spawn(async move {
                // Use async sleep instead of std::thread::sleep
                use futures_timer::Delay;
                use std::time::Duration;

                Delay::new(Duration::from_millis(100)).await;
                println!("🔄 Starting soundpack refresh operation...");

                // Use the state trigger to refresh cache and update UI
                // This will automatically update the count as well
                println!("🔄 Calling state trigger...");
                trigger.call(());
                println!("🔄 State trigger called successfully");

                // Reload current soundpacks to apply any changes
                println!("🔄 Reloading current soundpacks...");
                crate::state::app::reload_current_soundpacks(&audio_ctx_clone);

                // Add another small delay before changing the loading state back
                Delay::new(Duration::from_millis(100)).await;
                // Reset loading state
                refreshing_signal.set(false);
                println!("✅ Soundpack refresh complete");
            });
        })
    }; // Get soundpacks directory path
    let soundpacks_dir_absolute = crate::utils::path::get_soundpacks_dir_absolute();

    // Get current counts from cache
    let soundpack_count_keyboard = app_state.optimized_cache.count.keyboard;
    let soundpack_count_mouse = app_state.optimized_cache.count.mouse;

    rsx! {
      div { class: "space-y-4",        
        div { class: "text-base-content",
          div {
            div { class: "font-medium text-sm pb-1",
              if soundpack_count_keyboard + soundpack_count_mouse == 0 {
                "Click refresh to scan for soundpacks"
              } else {
                "Found {soundpack_count_keyboard + soundpack_count_mouse} soundpack(s)"
              }
            }
            if soundpack_count_keyboard + soundpack_count_mouse > 0 {
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
        }
        div { class: "space-y-2",
          div { class: "text-base-content/70 text-sm",
            "Refresh soundpack list to detect newly added or removed soundpacks."
          }
          div { class: "flex items-center gap-4",
            button {
              class: "btn  btn-soft btn-sm",
              onclick: refresh_soundpacks_cache,
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
            if app_state.optimized_cache.last_scan > 0 {
              div { class: "text-xs text-base-content/60",
                "Last scan "
                {
                    let last_scan = app_state.optimized_cache.last_scan;
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
          div { class: "text-base-content font-medium text-sm", "Soundpack folder path" }
          div { class: "text-sm text-base-content/70",
            "This is the absolute path to the soundpack directory where Mechvibes looks for soundpacks."
          }
          input {
            value: "{soundpacks_dir_absolute}",
            class: "input input-sm w-full",
            readonly: true,
          }
          button {
            class: "btn btn-soft btn-sm",            onclick: move |_| {
                let _ = crate::utils::path::open_path(&soundpacks_dir_absolute.clone());
            },
            FolderOpen { class: "w-4 h-4 mr-1" }
            "Open"
          }
        }
        div { class: "divider" }
        div { class: "space-y-3",
          div { class: "text-base-content font-medium text-sm", "Need more soundpacks?" }
          div { class: "text-sm text-base-content/70",
            "Check out the Mechvibes website to find more soundpacks. You can also create your own soundpacks using the Soundpack Editor."
          }
          div { class: "flex items-center gap-2",
            a {
              class: "btn btn-soft btn-sm",
              href: "https://mechvibes.com/soundpacks?utm_source=mechvibes&utm_medium=app&utm_campaign=soundpack_manager",
              target: "_blank",
              "Browse soundpacks"
              ExternalLink { class: "w-4 h-4 ml-1" }
            }
            a {
              class: "btn btn-soft btn-sm",
              href: "https://mechvibes.com/editor?utm_source=mechvibes&utm_medium=app&utm_campaign=soundpack_manager",
              target: "_blank",
              "Open Editor"
              ExternalLink { class: "w-4 h-4 ml-1" }
            }
          }
        }
      }
    }
}
