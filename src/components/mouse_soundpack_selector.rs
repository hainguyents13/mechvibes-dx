use crate::libs::audio::AudioContext;
use crate::state::config_utils::use_config;
use crate::state::paths;
use dioxus::prelude::*;
use futures_timer::Delay;
use lucide_dioxus::{Check, ChevronDown, Mouse, Music, Search};
use std::sync::Arc;
use std::time::Duration;

#[allow(non_snake_case)]
pub fn MouseSoundpackSelector() -> Element {
    rsx! {
      div { class: "space-y-2",

        div { class: "flex items-center gap-2 text-sm font-bold text-base-content/80",
          Mouse { class: "w-4 h-4" }
          "Mouse"
        }
        MouseSelector {}
      }
    }
}

#[allow(non_snake_case)]
fn MouseSelector() -> Element {
    // Use audio context from the layout provider
    let audio_ctx: Arc<AudioContext> = use_context();
    use crate::state::app::use_app_state;

    // Get global app state and shared config
    let app_state = use_app_state();
    let (config, update_config) = use_config();

    // UI state
    let error = use_signal(String::new);
    let mut is_open = use_signal(|| false);
    let mut search_query = use_signal(String::new);
    let is_loading = use_signal(|| false);

    // Use global app state for soundpacks
    let soundpacks = use_memo(move || app_state.with(|state| state.get_soundpacks()));

    // Get current mouse soundpack
    let current = use_memo(move || {
        let config = config();
        config.mouse_soundpack.clone()
    }); // Filter soundpacks based on search query and mouse type
    let filtered_soundpacks = use_memo(move || {
        let query = search_query().to_lowercase();
        let all_packs = soundpacks();

        // First filter to only mouse soundpacks (mouse: true)
        let mouse_packs: Vec<_> = all_packs
            .into_iter()
            .filter(|pack| pack.mouse) // Only include mouse soundpacks
            .collect();

        // Then filter by search query
        if query.is_empty() {
            mouse_packs
        } else {
            mouse_packs
                .into_iter()
                .filter(|pack| {
                    pack.name.to_lowercase().contains(&query)
                        || pack.id.to_lowercase().contains(&query)
                        || pack
                            .tags
                            .iter()
                            .any(|tag| tag.to_lowercase().contains(&query))
                })
                .collect()
        }
    });

    // Find current soundpack details
    let current_soundpack =
        use_memo(move || soundpacks().into_iter().find(|pack| pack.id == current()));

    rsx! {
      div { class: "space-y-2",
        div { class: "relative w-full",
          // DaisyUI dropdown button
          button {
            class: format!(
                "w-full btn btn-soft justify-start gap-3 h-16 rounded-lg {}",
                if is_open() { "btn-active" } else { "" },
            ),
            disabled: is_loading(),
            onclick: move |_| is_open.set(!is_open()),
            div { class: "flex items-center gap-3 flex-1",
              if let Some(pack) = current_soundpack() {
                div { class: "flex-shrink-0 overflow-hidden bg-blend-multiply w-10 h-10 bg-base-200 rounded-lg flex items-center justify-center",
                  if is_loading() {
                    span { class: "loading loading-spinner loading-sm" }
                  } else {
                    if let Some(icon) = &pack.icon {
                      img {
                        class: "w-full h-full object-cover",
                        src: paths::soundpacks::icon_path(&pack.id, icon),
                      }
                    } else {
                      Music { class: "w-5 h-5 text-base-content/50" }
                    }
                  }
                }
                div { class: "flex-1 min-w-0 text-left",
                  div { class: "font-medium truncate text-base-content text-sm",
                    "{pack.name}"
                  }
                  div { class: "text-xs truncate text-base-content/60",
                    if let Some(author) = &pack.author {
                      "v{pack.version} by {author}"
                    } else {
                      "v{pack.version}"
                    }
                  }
                }
              } else {
                div { class: "text-base-content/50 text-sm",
                  "Select a mouse soundpack..."
                }
              }
            }
            ChevronDown {
              class: format!(
                  "w-4 h-4 transition-transform {}",
                  if is_open() { "rotate-180" } else { "" },
              ),
            }
          }

          // Dropdown panel
          if is_open() {
            div { class: "absolute top-full left-0 right-0 mt-1 bg-base-100 border border-base-300 rounded-lg shadow-lg z-50 max-h-80 overflow-hidden",
              // Search input
              div { class: "p-3 border-b border-base-200",
                div { class: "relative",
                  Search { class: "absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-base-content/50" }
                  input {
                    class: "input input-bordered w-full px-4 py-2 text-base-content placeholder:text-base-content/40",
                    placeholder: "Search mouse soundpacks...",
                    value: "{search_query}",
                    oninput: move |evt| search_query.set(evt.value()),
                    autofocus: true,
                  }
                }
              }

              // Soundpack list
              div { class: "overflow-y-auto max-h-60",
                if filtered_soundpacks.read().is_empty() {
                  div { class: "p-4 text-center text-base-content/50",
                    "No mouse soundpacks found"
                  }
                } else {
                  for pack in filtered_soundpacks.read().iter() {
                    button {
                      key: "{pack.id}",
                      class: format!(
                          "w-full px-4 py-2 text-left btn btn-ghost btn-lg justify-start gap-4 border-b border-base-200 last:border-b-0 h-auto {}",
                          if pack.id == current() { " btn-disabled" } else { "" },
                      ),
                      onclick: {
                          let pack_id = pack.id.clone();
                          let mut error = error.clone();
                          let soundpacks = soundpacks.clone();
                          let mut is_open = is_open.clone();
                          let mut search_query = search_query.clone();
                          let is_loading = is_loading.clone();
                          let audio_ctx = audio_ctx.clone();
                          let update_config = update_config.clone();
                          move |_| {
                              is_open.set(false);
                              search_query.set(String::new());
                              error.set(String::new());
                              if let Some(pack_item) = soundpacks().iter().find(|p| p.id == pack_id) {
                                  println!("📦 Found mouse soundpack in cache: {}", pack_item.name);
                                  let pack_id_clone = pack_id.clone();
                                  update_config(
                                      Box::new(move |config| {
                                          println!(
                                              "💾 Updating mouse soundpack: {} -> {}",
                                              config.mouse_soundpack,
                                              pack_id_clone,
                                          );
                                          config.mouse_soundpack = pack_id_clone;
                                      }),
                                  );
                                  let pack_id_async = pack_id.clone();
                                  let pack_name = pack_item.name.clone();
                                  let audio_ctx_async = audio_ctx.clone();
                                  let mut error_async = error.clone();
                                  let mut is_loading_async = is_loading.clone();
                                  spawn(async move {
                                      is_loading_async.set(true);
                                      Delay::new(Duration::from_millis(300)).await;
                                      let result = crate::libs::audio::load_mouse_soundpack(
                                          &audio_ctx_async,
                                          &pack_id_async,
                                      );
                                      match result {
                                          Ok(_) => {
                                              println!(
                                                  "✅ Mouse soundpack changed to: {} (background loading)",
                                                  pack_name,
                                              );
                                          }
                                          Err(e) => {
                                              error_async
                                                  .set(format!("Failed to load mouse soundpack: {}", e));
                                              println!(
                                                  "❌ Failed to load mouse soundpack {}: {}",
                                                  pack_id_async,
                                                  e,
                                              );
                                          }
                                      }
                                      is_loading_async.set(false);
                                  });
                              } else {
                                  println!("❌ Mouse soundpack {} not found in cache", pack_id);
                              }
                          }
                      },
                      div { class: "flex items-center justify-between gap-3",
                        div {
                          class: format!(
                              "flex-shrink-0 w-8 h-8 bg-base-300 rounded-lg flex items-center justify-center overflow-hidden bg-blend-multiply {}",
                              if pack.id == current() { "bg-black" } else { "" },
                          ),
                          if pack.id != current() {
                            if let Some(icon) = &pack.icon {
                              img {
                                class: "w-full h-full object-cover",
                                src: paths::soundpacks::icon_path(&pack.id, icon),
                              }
                            } else {
                              Music { class: "w-4 h-4 text-base-content/50" }
                            }
                          } else {
                            Check { class: "w-4 h-4 text-white" }
                          }
                        }
                        div { class: "flex-1 min-w-0",
                          div { class: "text-xs font-medium truncate text-base-content",
                            "{pack.name}"
                          }
                          div { class: "text-xs truncate text-base-content/60",
                            if let Some(author) = &pack.author {
                              "v{pack.version} by {author}"
                            } else {
                              "v{pack.version}"
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
        }

        // Click outside to close
        if is_open() {
          div {
            class: "fixed inset-0 z-40",
            onclick: move |_| {
                is_open.set(false);
                search_query.set(String::new());
            },
          }
        }

        // Error display
        if !error().is_empty() {
          div { class: "text-xs text-error mt-1", "{error}" }
        }
      }
    }
}
