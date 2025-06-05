use crate::libs::audio::AudioContext;
use crate::utils::config::use_config;
use dioxus::prelude::*;
use futures_timer::Delay;
use lucide_dioxus::{Check, ChevronDown, Keyboard, Mouse, Music, Search};
use std::sync::Arc;
use std::time::Duration;

#[derive(Clone, PartialEq, Copy)]
pub enum SelectorType {
    Keyboard,
    Mouse,
}

#[derive(Props, Clone, PartialEq)]
pub struct SoundpackSelectorProps {
    pub soundpack_type: SelectorType,
    pub icon: Element,
    pub label: String,
}

#[component]
pub fn SoundpackSelector(props: SoundpackSelectorProps) -> Element {
    rsx! {
      div { class: "space-y-2",
        div { class: "flex items-center gap-2 text-sm font-bold text-base-content/80",
          {props.icon}
          "{props.label}"
        }
        SoundpackDropdown { soundpack_type: props.soundpack_type }
      }
    }
}

#[component]
fn SoundpackDropdown(soundpack_type: SelectorType) -> Element {
    // Use audio context from the layout provider
    let audio_ctx: Arc<AudioContext> = use_context();

    // Use the new event-driven app state
    use crate::state::app::use_app_state;
    let app_state = use_app_state();
    let (config, update_config) = use_config();

    // UI state
    let error = use_signal(String::new);
    let mut is_open = use_signal(|| false);
    let mut search_query = use_signal(String::new);
    let is_loading = use_signal(|| false); // Use global app state for soundpacks
    let soundpacks = use_memo(move || app_state.get_soundpacks());

    // Get current soundpack based on type
    let current = use_memo(move || {
        let config = config();
        match soundpack_type {
            SelectorType::Keyboard => config.keyboard_soundpack.clone(),
            SelectorType::Mouse => config.mouse_soundpack.clone(),
        }
    });

    // Filter soundpacks based on search query and type
    let filtered_soundpacks = use_memo(move || {
        let query = search_query().to_lowercase();
        let all_packs = soundpacks();

        // Filter by type first
        let type_filtered_packs: Vec<_> = all_packs
            .into_iter()
            .filter(|pack| match soundpack_type {
                SelectorType::Keyboard => !pack.mouse, // Keyboard soundpacks have mouse: false
                SelectorType::Mouse => pack.mouse,     // Mouse soundpacks have mouse: true
            })
            .collect();

        // Then filter by search query
        if query.is_empty() {
            type_filtered_packs
        } else {
            type_filtered_packs
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

    // Get appropriate placeholder and search text based on type
    let (placeholder_text, search_placeholder, not_found_text) = match soundpack_type {
        SelectorType::Keyboard => (
            "Select a keyboard soundpack...",
            "Search keyboard soundpacks...",
            "No keyboard soundpacks found",
        ),
        SelectorType::Mouse => (
            "Select a mouse soundpack...",
            "Search mouse soundpacks...",
            "No mouse soundpacks found",
        ),
    };

    rsx! {
      div { class: "space-y-2",
        div { class: "relative w-full",
          // Dropdown toggle button
          button {
            class: format!(
                "w-full btn btn-soft justify-start gap-3 h-17 rounded-box {}",
                if is_open() { "btn-active" } else { "" },
            ),
            disabled: is_loading(),
            onclick: move |_| is_open.set(!is_open()),
            div { class: "flex items-center gap-3 flex-1",
              if let Some(pack) = current_soundpack() {
                div { class: "flex-shrink-0 overflow-hidden bg-blend-multiply w-11 h-11 bg-base-200 rounded-box flex items-center justify-center",
                  if is_loading() {
                    span { class: "loading loading-spinner loading-sm" }
                  } else {
                    if let Some(icon) = &pack.icon {
                      if !icon.is_empty() {
                        img {
                          class: "w-full h-full object-cover",
                          src: "{icon}",
                        }
                      } else {
                        Music { class: "w-5 h-5 text-base-content/50" }
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
                  div { class: "text-xs truncate text-base-content/50",
                    if let Some(author) = &pack.author {
                      "by {author}"
                    }
                  }
                }
              } else {
                div { class: "text-base-content/50 text-sm", "{placeholder_text}" }
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
            div { class: "absolute top-full left-0 right-0 mt-1 bg-base-100 border border-base-300 rounded-box shadow-lg z-50 max-h-80 overflow-hidden",
              // Search input
              div { class: "p-3 border-b border-base-200",
                div { class: "relative",
                  Search { class: "absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-base-content/50" }
                  input {
                    class: "input input-bordered w-full px-4 py-2 text-base-content placeholder:text-base-content/40",
                    placeholder: "{search_placeholder}",
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
                    "{not_found_text}"
                  }
                } else {
                  for pack in filtered_soundpacks.read().iter() {
                    button {
                      key: "{pack.id}",
                      class: format!(
                          "w-full px-4 rounded-none py-2 text-left btn btn-lg justify-start gap-4 border-b border-base-200 last:border-b-0 h-auto {}",
                          if pack.id == current() { "btn-disabled" } else { "btn-ghost" },
                      ),
                      disabled: pack.id == current(),
                      onclick: {
                          let pack_id = pack.id.clone();
                          let mut error = error.clone();
                          let soundpacks = soundpacks.clone();
                          let mut is_open = is_open.clone();
                          let mut search_query = search_query.clone();
                          let is_loading = is_loading.clone();
                          let audio_ctx = audio_ctx.clone();
                          let update_config = update_config.clone();
                          let soundpack_type_click = soundpack_type.clone();
                          move |_| {
                              is_open.set(false);
                              search_query.set(String::new());
                              error.set(String::new());
                              if let Some(pack_item) = soundpacks().iter().find(|p| p.id == pack_id) {
                                  let type_str = match soundpack_type_click {
                                      SelectorType::Keyboard => "keyboard",
                                      SelectorType::Mouse => "mouse",
                                  };
                                  println!(
                                      "📦 Found {} soundpack in cache: {}",
                                      type_str,
                                      pack_item.name,
                                  );
                                  let pack_id_clone = pack_id.clone();
                                  let soundpack_type_clone = soundpack_type_click.clone();
                                  update_config(
                                      Box::new(move |config| {
                                          match soundpack_type_clone {
                                              SelectorType::Keyboard => {
                                                  println!(
                                                      "💾 Updating keyboard soundpack: {} -> {}",
                                                      config.keyboard_soundpack,
                                                      pack_id_clone,
                                                  );
                                                  config.keyboard_soundpack = pack_id_clone;
                                              }
                                              SelectorType::Mouse => {
                                                  println!(
                                                      "💾 Updating mouse soundpack: {} -> {}",
                                                      config.mouse_soundpack,
                                                      pack_id_clone,
                                                  );
                                                  config.mouse_soundpack = pack_id_clone;
                                              }
                                          }
                                      }),
                                  );
                                  let pack_id_async = pack_id.clone();
                                  let pack_name = pack_item.name.clone();
                                  let audio_ctx_async = audio_ctx.clone();
                                  let mut error_async = error.clone();
                                  let mut is_loading_async = is_loading.clone();
                                  let soundpack_type_async = soundpack_type_click.clone();
                                  spawn(async move {
                                      is_loading_async.set(true);
                                      Delay::new(Duration::from_millis(1)).await;
                                      let result = match soundpack_type_async {
                                          SelectorType::Keyboard => {
                                              crate::libs::audio::load_keyboard_soundpack(
                                                  &audio_ctx_async,
                                                  &pack_id_async,
                                              )
                                          }
                                          SelectorType::Mouse => {
                                              crate::libs::audio::load_mouse_soundpack(
                                                  &audio_ctx_async,
                                                  &pack_id_async,
                                              )
                                          }
                                      };
                                      match result {
                                          Ok(_) => {
                                              let type_str = match soundpack_type_async {
                                                  SelectorType::Keyboard => "Keyboard",
                                                  SelectorType::Mouse => "Mouse",
                                              };
                                              println!(
                                                  "✅ {} soundpack changed to: {} (background loading)",
                                                  type_str,
                                                  pack_name,
                                              );
                                          }
                                          Err(e) => {
                                              let type_str = match soundpack_type_async {
                                                  SelectorType::Keyboard => "keyboard",
                                                  SelectorType::Mouse => "mouse",
                                              };
                                              error_async
                                                  .set(
                                                      format!("Failed to load {} soundpack: {}", type_str, e),
                                                  );
                                              println!(
                                                  "❌ Failed to load {} soundpack {}: {}",
                                                  type_str,
                                                  pack_id_async,
                                                  e,
                                              );
                                          }
                                      }
                                      is_loading_async.set(false);
                                  });
                              } else {
                                  let type_str = match soundpack_type_click {
                                      SelectorType::Keyboard => "Keyboard",
                                      SelectorType::Mouse => "Mouse",
                                  };
                                  println!("❌ {} soundpack {} not found in cache", type_str, pack_id);
                              }
                          }
                      },
                      div { class: "flex items-center justify-between gap-3",
                        div { class: "flex-shrink-0 w-10 h-10 rounded-box flex items-center justify-center bg-base-100 overflow-hidden bg-blend-multiply relative",
                          if let Some(icon) = &pack.icon {
                            if !icon.is_empty() {
                              img {
                                class: "w-full h-full object-cover",
                                src: "{icon}",
                              }
                            } else {
                              Music { class: "w-4 h-4 text-base-content/50 bg-base-100" }
                            }
                          } else {
                            Music { class: "w-4 h-4 text-base-content/50 bg-base-100" }
                          }
                          if pack.id == current() {
                            div { class: "absolute inset-0 bg-base-300/70 flex items-center justify-center ",
                              Check { class: "text-white w-6 h-6" }
                            }
                          }
                        }
                        div { class: "flex-1 min-w-0",
                          div { class: "text-xs font-medium truncate text-base-content",
                            "{pack.name}"
                          }
                          div { class: "text-xs truncate text-base-content/50",
                            if let Some(author) = &pack.author {
                              "by {author}"
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

// Wrapper components for keyboard and mouse soundpack selectors

#[component]
pub fn KeyboardSoundpackSelector() -> Element {
    rsx! {
      SoundpackSelector {
        soundpack_type: SelectorType::Keyboard,
        label: "Keyboard".to_string(),
        icon: rsx! {
          Keyboard { class: "w-4 h-4" }
        },
      }
    }
}

#[component]
pub fn MouseSoundpackSelector() -> Element {
    rsx! {
      SoundpackSelector {
        soundpack_type: SelectorType::Mouse,
        label: "Mouse".to_string(),
        icon: rsx! {
          Mouse { class: "w-4 h-4" }
        },
      }
    }
}
