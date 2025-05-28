use crate::libs::audio::AudioContext;
use crate::state::config_utils::use_config;
use dioxus::prelude::*;
use futures_timer::Delay;
use lucide_dioxus::{Check, ChevronDown, Music, Search};
use std::sync::Arc;
use std::time::Duration;

#[allow(non_snake_case)]
pub fn SoundpackSelector() -> Element {
    // Use audio context from the layout provider
    let audio_ctx: Arc<AudioContext> = use_context();
    use crate::state::app::use_app_state;

    // Get global app state and shared config
    let app_state = use_app_state();
    let (config, update_config) = use_config();

    // UI state
    let mut error = use_signal(String::new);
    let mut is_open = use_signal(|| false);
    let mut search_query = use_signal(String::new);
    let mut is_refreshing = use_signal(|| false);

    // Use global app state for soundpacks and get current soundpack from config
    let soundpacks =
        use_memo(move || app_state.with(|state| state.soundpack_cache.get_soundpacks().clone()));
    let current = use_memo(move || config().current_soundpack.clone());

    // Filter soundpacks based on search query
    let filtered_soundpacks = use_memo(move || {
        let query = search_query().to_lowercase();
        if query.is_empty() {
            soundpacks()
        } else {
            soundpacks()
                .into_iter()
                .filter(|pack| {
                    pack.soundpack.name.to_lowercase().contains(&query)
                        || pack.soundpack.author.to_lowercase().contains(&query)
                        || pack.soundpack.id.to_lowercase().contains(&query)
                })
                .collect()
        }
    });

    // Find current soundpack info
    let current_soundpack = use_memo(move || {
        soundpacks()
            .iter()
            .find(|pack| pack.soundpack.id == current())
            .map(|item| &item.soundpack)
            .cloned()
    });

    rsx! {
      div { class: "space-y-2",
        div { class: "relative w-full",
          // DaisyUI dropdown button
          button {
            class: format!(
                "w-full btn btn-soft justify-start gap-3 h-18 rounded-lg {}",
                if is_open() { "btn-active" } else { "" },
            ),
            onclick: move |_| is_open.set(!is_open()),
            div { class: "flex items-center gap-3 flex-1",
              if let Some(pack) = current_soundpack() {
                div { class: "flex-shrink-0 w-12 h-12 bg-base-200 rounded-lg flex items-center justify-center",
                  // if let Some(icon) = &pack.icon {
                  //   img {
                  //     class: "w-6 h-6 rounded",
                  //     src: format!("./soundpacks/{}/{}", pack.name, icon),
                  //     alt: "icon",
                  //   }
                  // } else {
                  //   Music { class: "w-4 h-4 text-base-content/50" }
                  // }
                  Music { class: "w-6 h-6 text-base-content/50" }
                }
                div { class: "flex-1 min-w-0 text-left",
                  div { class: "font-medium truncate text-base-content",
                    "{pack.name}"
                  }
                  div { class: "text-sm truncate text-base-content/60",
                    "by {pack.author}"
                  }
                }
              } else {
                div { class: "text-base-content/50", "Select a soundpack..." }
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
                    class: "input input-bordered w-full pr-4 py-2 text-base-content placeholder:text-base-content/40",
                    placeholder: "Search soundpacks...",
                    value: "{search_query}",
                    oninput: move |evt| search_query.set(evt.value()),
                    autofocus: true,
                  }
                }
              }
              // Soundpack list
              div { class: "overflow-y-auto max-h-300",
                if filtered_soundpacks.read().is_empty() {
                  div { class: "p-4 text-center text-base-content/50",
                    "No soundpacks found"
                  }
                } else {
                  for pack in filtered_soundpacks.read().iter() {
                    button {
                      key: "{pack.soundpack.id}",
                      class: format!(
                          "w-full p-3 text-left btn btn-ghost btn-lg justify-start gap-5 border-b border-base-200 last:border-b-0 h-16 {}",
                          if pack.soundpack.id == current() { " btn-disabled" } else { "" },
                      ),
                      onclick: {                          let pack_id = pack.soundpack.id.clone();                          let mut error = error.clone();
                          let soundpacks = soundpacks.clone();
                          let mut is_open = is_open.clone();
                          let mut search_query = search_query.clone();
                          let audio_ctx = audio_ctx.clone();
                          let update_config = update_config.clone();
                          move |_| {
                              println!("🎯 User selected soundpack: {}", pack_id);
                              is_open.set(false);
                              error.set(String::new());
                              if let Some(pack_item) = soundpacks()
                                  .iter()
                                  .find(|p| p.soundpack.id == pack_id)
                              {
                                  println!("📦 Found soundpack in cache: {}", pack_item.soundpack.name);
                                  let pack_id_clone = pack_id.clone();
                                  update_config(
                                      Box::new(move |config| {
                                          println!("💾 Updating config: {} -> {}", config.current_soundpack, pack_id_clone);
                                          config.current_soundpack = pack_id_clone;
                                      }),
                                  );
                                  match crate::libs::audio::load_soundpack_by_id(&audio_ctx, &pack_id) {
                                      Ok(_) => {
                                          search_query.set(String::new());
                                          println!(
                                              "✅ Soundpack changed to: {} (loaded from cache)",
                                              pack_item.soundpack.name,
                                          );
                                      }
                                      Err(e) => {
                                          error.set(format!("Failed to load soundpack: {}", e));
                                          println!("❌ Failed to load soundpack {}: {}", pack_id, e);
                                      }
                                  }
                              } else {
                                  println!("❌ Soundpack {} not found in cache", pack_id);
                              }
                          }
                      },
                      div { class: "flex items-center justify-between gap-3",
                        div {
                          class: format!(
                              "flex-shrink-0 w-10 h-10 bg-base-300 rounded-lg flex items-center justify-center {}",
                              if pack.soundpack.id == current() { "bg-black" } else { "" },
                          ),
                          if pack.soundpack.id != current() {
                            if let Some(icon) = &pack.full_icon_path {
                              img {
                                class: "w-6 h-6 rounded",
                                src: icon.to_string(),
                                alt: "icon",
                              }
                            } else {
                              Music { class: "w-5 h-5 text-base-content/50" }
                            }
                          } else {
                            Check { class: "w-5 h-5 text-white" }
                          }
                        }
                        div { class: "flex-1 min-w-0",
                          div { class: "text-sm font-medium truncate text-base-content",
                            "{pack.soundpack.name}"
                          }
                          div { class: "text-xs truncate text-base-content/60",
                            if let Some(version) = &pack.soundpack.version {
                              "v{version} "
                            }
                            "by {pack.soundpack.author}"
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
          div { class: "text-sm text-error mt-2", "{error}" }
        }
        // Refresh button
        div { class: "flex justify-between items-center",
          div { class: "text-sm text-base-content/60 ", "{soundpacks().len()} soundpacks" }
          div { class: "flex items-center gap-2",
            a {
              href: "https://mechvibes.com/soundpacks",
              target: "_blank",
              class: "btn btn-white btn-sm",
              title: "Get more soundpacks from Mechvibes",
              rel: "noopener noreferrer",
              "More"
            }
            button {
              class: "btn btn-white btn-sm ",
              title: "Refresh soundpack list from disk and reload current soundpack",              onclick: move |_| {
                  if !is_refreshing() {
                      is_refreshing.set(true);
                      error.set(String::new());                      let mut is_refreshing = is_refreshing.clone();
                      let audio_ctx = audio_ctx.clone();
                      spawn(async move {
                          Delay::new(Duration::from_millis(1000)).await;
                          crate::state::app::reload_soundpacks();
                          // Also reload the current soundpack in memory
                          crate::state::app::reload_current_soundpack(&audio_ctx);
                          is_refreshing.set(false);
                      });
                  }
              },
              if is_refreshing() {
                span { class: "loading loading-spinner loading-xs" }
              }
              "Refresh"
            }
          }
        }
      }
    }
}
