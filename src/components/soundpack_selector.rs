use crate::libs::audio::AudioContext;
use dioxus::prelude::*;
use futures_timer::Delay;
use lucide_dioxus::{ChevronDown, Music, Search};
use std::sync::Arc;
use std::time::Duration;

#[derive(Clone, Props)]
pub struct SoundpackSelectorProps {
    pub audio_ctx: Arc<AudioContext>,
}

impl PartialEq for SoundpackSelectorProps {
    fn eq(&self, _other: &Self) -> bool {
        true // Arc comparison not needed for this component
    }
}

#[allow(non_snake_case)]
pub fn SoundpackSelector(props: SoundpackSelectorProps) -> Element {
    use crate::libs::theme::use_effective_theme;
    use crate::state::app::use_app_state;

    // Get global app state
    let app_state = use_app_state();

    // UI state
    let effective_theme = use_effective_theme();
    let mut error = use_signal(String::new);
    let mut is_open = use_signal(|| false);
    let mut search_query = use_signal(String::new);
    let mut is_refreshing = use_signal(|| false); // Use global app state for soundpacks and current soundpack
    let soundpacks =
        use_memo(move || app_state.with(|state| state.soundpack_cache.get_soundpacks().clone()));
    let current = use_signal(|| app_state.with(|state| state.config.current_soundpack.clone()));

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
      div { class: "relative w-full", // Combobox trigger button
        button {
          class: format!(
              "w-full {} {} rounded-lg px-4 py-3 text-left {} transition-colors flex items-center justify-between",
              effective_theme.bg_secondary(),
              effective_theme.border(),
              effective_theme.bg_hover_secondary(),
          ),
          onclick: move |_| is_open.set(!is_open()),
          // Current selection display
          div { class: "flex items-center gap-3 flex-1",
            if let Some(pack) = current_soundpack() {
              // Soundpack icon
              div {
                class: format!(
                    "flex-shrink-0 w-8 h-8 {} rounded-lg flex items-center justify-center",
                    effective_theme.bg_tertiary(),
                ),
                if pack.icon.is_some() {
                  img {
                    class: "w-6 h-6 rounded",
                    src: format!("./soundpacks/{}/{}", pack.name, pack.icon.as_ref().unwrap()),
                    alt: "icon",
                  }
                } else {
                  Music { class: format!("w-4 h-4 {}", effective_theme.text_tertiary()) }
                }
              }
              // Soundpack info
              div { class: "flex-1 min-w-0",
                div { class: format!("{} font-medium truncate", effective_theme.text_primary()),
                  "{pack.name}"
                }
                div { class: format!("{} text-sm truncate", effective_theme.text_tertiary()),
                  "by {pack.author}"
                }
              }
            } else {
              div { class: effective_theme.text_tertiary(), "Select a soundpack..." }
            }
          }
          // Dropdown arrow
          ChevronDown {
            class: format!(
                "w-4 h-4 {} transition-transform {}",
                effective_theme.text_tertiary(),
                if is_open() { "transform rotate-180" } else { "" },
            ),
          }
        }
        // Dropdown panel
        if is_open() {
          div {
            class: format!(
                "absolute top-full left-0 right-0 mt-1 {} {} rounded-lg shadow-lg z-50 max-h-80 overflow-hidden",
                effective_theme.bg_secondary(),
                effective_theme.border(),
            ),
            // Search input
            div { class: format!("p-3 border-b {}", effective_theme.border()),
              div { class: "relative",
                Search {
                  class: format!(
                      "absolute left-3 top-1/2 transform -translate-y-1/2 w-4 h-4 {}",
                      effective_theme.text_tertiary(),
                  ),
                }
                input {
                  class: format!(
                      "w-full {} {} rounded-lg pl-10 pr-4 py-2 {} {} focus:outline-none focus:ring-2 focus:ring-blue-500",
                      effective_theme.bg_tertiary(),
                      effective_theme.border(),
                      effective_theme.text_primary(),
                      effective_theme.placeholder(),
                  ),
                  placeholder: "Search soundpacks...",
                  value: "{search_query}",
                  oninput: move |evt| search_query.set(evt.value()),
                  // Auto focus when dropdown opens
                  autofocus: true,
                }
              }
            }
            // Soundpack list
            div { class: "overflow-y-auto max-h-64",
              if filtered_soundpacks.read().is_empty() {
                div { class: format!("p-4 text-center {}", effective_theme.text_tertiary()),
                  "No soundpacks found"
                }
              } else {
                for pack in filtered_soundpacks.read().iter() {
                  button {
                    key: "{pack.soundpack.id}",
                    class: format!(
                        "w-full p-3 text-left {} transition-colors border-b {} last:border-b-0 {}",
                        effective_theme.bg_hover_secondary(),
                        effective_theme.border(),
                        if pack.soundpack.id == current() { effective_theme.bg_tertiary() } else { "" },
                    ),
                    onclick: {
                        let pack_id = pack.soundpack.id.clone();
                        let mut error = error.clone();
                        let soundpacks = soundpacks.clone();
                        let mut current = current.clone();
                        let mut is_open = is_open.clone();
                        let mut search_query = search_query.clone();
                        let audio_ctx = props.audio_ctx.clone();
                        move |_| {
                            error.set(String::new());
                            if let Some(pack_item) = soundpacks()
                                .iter()
                                .find(|p| p.soundpack.id == pack_id)
                            {
                                // Lấy config từ global state thay vì tải lại từ file
                                let mut config = app_state.with(|state| (*state.config).clone());
                                config.current_soundpack = pack_id.clone();
                                if let Err(e) = crate::state::app::save_config(config) {
                                    error.set(format!("Failed to save config: {}", e));
                                    return;
                                }
                                match crate::libs::audio::load_soundpack(&audio_ctx) {
                                    Ok(_) => {
                                        current.set(pack_id.clone());
                                        is_open.set(false);
                                        search_query.set(String::new());
                                        println!(
                                            "✅ Soundpack changed to: {} (loaded from cache)",
                                            pack_item.soundpack.name,
                                        );
                                    }
                                    Err(e) => {
                                        error.set(format!("Failed to load soundpack: {}", e));
                                    }
                                }
                            }
                        }
                    },
                    div { class: "flex items-center gap-3",
                      // Soundpack icon
                      div {
                        class: format!(
                            "flex-shrink-0 w-10 h-10 {} rounded-lg flex items-center justify-center",
                            effective_theme.bg_tertiary(),
                        ),
                        if pack.soundpack.icon.is_some() {
                          // TODO: Render actual icon when available
                          span { class: "text-xl", "🎵" }
                        } else {
                          Music { class: format!("w-5 h-5 {}", effective_theme.text_tertiary()) }
                        }
                      }
                      // Soundpack info
                      div { class: "flex-1 min-w-0",
                        div { class: format!("{} font-medium truncate", effective_theme.text_primary()),
                          "{pack.soundpack.name}"
                        }
                        div { class: format!("{} text-sm truncate", effective_theme.text_tertiary()),
                          "by {pack.soundpack.author}"
                        }
                        if let Some(description) = &pack.soundpack.description {
                          div { class: format!("{} text-xs truncate mt-1", effective_theme.text_tertiary()),
                            "{description}"
                          }
                        }
                      }
                      // Selected indicator
                      if pack.soundpack.id == current() {
                        div { class: format!("flex-shrink-0 w-2 h-2 {} rounded-full", effective_theme.bg_secondary()) }
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
        div { class: "text-sm text-red-400 mt-2", "{error}" }
      }
      // Refresh button
      div { class: "",
        button {
          class: format!("btn btn-primary {}", if is_refreshing() { "select-none" } else { "" }),
          title: "Refresh soundpack list from disk",
          onclick: move |_| {
              if !is_refreshing() {
                  is_refreshing.set(true);
                  error.set(String::new());
                  let mut is_refreshing = is_refreshing.clone();
                  spawn(async move {
                      Delay::new(Duration::from_millis(1000)).await;
                      crate::state::app::reload_soundpacks();
                      is_refreshing.set(false);
                  });
              }
          },
          if is_refreshing() {
            span { class: "loading loading-spinner" }
          } else {
            ""
          }
          "Refresh"
        }
      }
    }
}
