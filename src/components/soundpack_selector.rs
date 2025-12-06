use crate::libs::audio::AudioContext;
use crate::utils::config::use_config;
use dioxus::prelude::*;
use futures_timer::Delay;
use lucide_dioxus::{ Check, ChevronDown, Keyboard, Mouse, Music, Search };
use std::sync::Arc;
use std::time::Duration;

#[derive(Clone, PartialEq, Copy, Debug)]
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
          span { class: "text-primary", {props.icon} }
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
    let is_loading = use_signal(|| false);

    // Use global app state for soundpacks
    let soundpacks = use_memo(move || app_state.get_soundpacks());

    // Get current soundpack based on type
    let current = use_memo(move || {
        let config = config();
        match soundpack_type {
            SelectorType::Keyboard => config.keyboard_soundpack.clone(),
            SelectorType::Mouse => config.mouse_soundpack.clone(),
        }
    }); // Filter soundpacks based on search query and type, then sort by last_modified
    let filtered_soundpacks = use_memo(move || {
        let query = search_query().to_lowercase();
        let all_packs = soundpacks(); // Filter by type first
        let type_filtered_packs: Vec<_> = all_packs
            .into_iter()
            .filter(|pack| {
                match soundpack_type {
                    SelectorType::Keyboard =>
                        pack.soundpack_type == crate::state::soundpack::SoundpackType::Keyboard,
                    SelectorType::Mouse =>
                        pack.soundpack_type == crate::state::soundpack::SoundpackType::Mouse,
                }
            })
            .collect();

        // Then filter by search query
        let mut filtered_packs = if query.is_empty() {
            type_filtered_packs
        } else {
            type_filtered_packs
                .into_iter()
                .filter(|pack| {
                    pack.name.to_lowercase().contains(&query) ||
                        pack.id.to_lowercase().contains(&query) ||
                        pack.tags.iter().any(|tag| tag.to_lowercase().contains(&query))
                })
                .collect()
        }; // Sort by last_modified in descending order (most recent first)
        filtered_packs.sort_by(|a, b| b.last_modified.cmp(&a.last_modified));

        filtered_packs
    });
    let current_soundpack = use_memo(
        move ||
            soundpacks()
                .into_iter()
                .find(|pack| pack.folder_path == current()) // Use folder_path for comparison
    ); // Get appropriate placeholder and search text based on type
    let (placeholder_text, search_placeholder, not_found_text, no_soundpack_text) = match
        soundpack_type
    {
        SelectorType::Keyboard =>
            (
                "Select a keyboard sound pack...",
                "Search keyboard sound packs...",
                "No keyboard sound packs found",
                "No sound packs available",
            ),
        SelectorType::Mouse =>
            (
                "Select a mouse sound pack...",
                "Search mouse sound packs...",
                "No mouse sound packs found",
                "No sound packs available",
            ),
    };

    // Check if there are any soundpacks available for this type
    let has_soundpacks = use_memo(move || {
        soundpacks()
            .into_iter()
            .any(|pack| {
                match soundpack_type {
                    SelectorType::Keyboard =>
                        pack.soundpack_type == crate::state::soundpack::SoundpackType::Keyboard,
                    SelectorType::Mouse =>
                        pack.soundpack_type == crate::state::soundpack::SoundpackType::Mouse,
                }
            })
    });

    rsx! {
      div { class: "space-y-2",
        div { class: "relative w-full",
          // Dropdown toggle button
          button {
            id: format!("soundpack-btn-{:?}", soundpack_type),
            class: format!(
                "w-full btn btn-soft justify-start gap-3 h-17 rounded-box {}",
                if is_open() { "btn-active" } else { "" },
            ),
            style: format!("anchor-name: --soundpack-anchor-{:?};", soundpack_type),
            disabled: is_loading() || !has_soundpacks(),
            onclick: move |_| {
                if has_soundpacks() {
                    is_open.set(!is_open());
                }
            },
            div { class: "flex items-center gap-3 flex-1 ",
              if !has_soundpacks() {
                div { class: "text-base-content/50 text-sm", "{no_soundpack_text}" }
              } else if let Some(pack) = current_soundpack() {
                div { class: "flex-shrink-0 overflow-hidden  w-11 h-11 bg-base-200 rounded-box flex items-center justify-center",
                  if is_loading() {
                    span { class: "loading loading-spinner loading-sm" }
                  } else {
                    if let Some(icon) = &pack.icon {
                      if !icon.is_empty() {
                        img {
                          class: "w-full h-full bg-blend-multiply object-cover",
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
                  div { class: "font-medium line-clamp-1 text-base-content  text-sm",
                    "{pack.name}"
                  }
                  div { class: "text-xs font-normal truncate text-base-content/50",
                    if let Some(author) = &pack.author {
                      "by {author}"
                    } else {
                      "by N/A"
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
          if is_open() && has_soundpacks() {
            div {
              class: "bg-base-200 border border-base-300 rounded-box shadow-lg z-50 ",
              style: format!(
                  "position: absolute; position-anchor: --soundpack-anchor-{:?}; position-area: block-end; width: anchor-size(width); margin-top: 4px;",
                  soundpack_type,
              ),
              // Search input
              div { class: "p-3 border-b border-base-200",
                div { class: "relative",
                  Search { class: "absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-primary/50" }
                  input {
                    class: "input input-sm w-full px-4 py-2 text-base-content placeholder:text-base-content/40",
                    placeholder: "{search_placeholder}",
                    value: "{search_query}",
                    oninput: move |evt| search_query.set(evt.value()),
                    autofocus: true,
                  }
                }
              }

              // Soundpack list
              div { class: "overflow-y-auto max-h-50",
                if filtered_soundpacks.read().is_empty() {
                  div { class: "p-4 text-center text-base-content/50",
                    "{not_found_text}"
                  }
                } else {
                  for pack in filtered_soundpacks.read().iter() {
                    button {
                      key: "{pack.id}",
                      class: format!(
                          "w-full px-4 rounded-none py-2 text-left btn btn-lg justify-start gap-4 border-b border-base-300 last:border-b-0 h-auto {}",
                          if pack.folder_path == current() { "btn-disabled" } else { "btn-ghost" },
                      ),
                      disabled: pack.folder_path == current(),
                      // Use folder_path for comparison
                      onclick: {
                          let pack_id = pack.folder_path.clone();
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
                              if let Some(_) = soundpacks().iter().find(|p| p.folder_path == pack_id) {
                                  let pack_id_clone = pack_id.clone();
                                  let soundpack_type_clone = soundpack_type_click.clone();
                                  update_config(
                                      Box::new(move |config| {
                                          match soundpack_type_clone {
                                              SelectorType::Keyboard => {
                                                  config.keyboard_soundpack = pack_id_clone;
                                              }
                                              SelectorType::Mouse => {
                                                  config.mouse_soundpack = pack_id_clone;
                                              }
                                          }
                                      }),
                                  );
                                  let pack_id_async = pack_id.clone();
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
                                          Ok(_) => {}
                                          Err(e) => {
                                              let type_str = match soundpack_type_async {
                                                  SelectorType::Keyboard => "keyboard",
                                                  SelectorType::Mouse => "mouse",
                                              };
                                              error_async
                                                  .set(
                                                      format!("Failed to load {} soundpack: {}", type_str, e),
                                                  );
                                          }
                                      }
                                      is_loading_async.set(false);
                                  });
                              }
                          }
                      },
                      div { class: "flex items-center justify-between gap-3 ",
                        div { class: "flex-shrink-0 w-8 h-8 rounded-box flex items-center justify-center bg-base-100 overflow-hidden relative",
                          if let Some(icon) = &pack.icon {
                            if !icon.is_empty() {
                              img {
                                class: "w-full h-full object-cover bg-blend-multiply",
                                src: "{icon}",
                              }
                            } else {
                              Music { class: "w-4 h-4 text-primary/50 bg-base-100" }
                            }
                          } else {
                            Music { class: "w-4 h-4 text-primary/50 bg-base-100" }
                          }
                          if pack.folder_path == current() {
                            // Use folder_path for comparison
                            div { class: "absolute inset-0 bg-base-300/70 flex items-center justify-center ",
                              Check { class: "text-white w-6 h-6" }
                            }
                          }
                        }
                        div { class: "flex-1 min-w-0",
                          div { class: "text-xs font-medium line-clamp-1 text-base-content",
                            "{pack.name}"
                          }
                          div { class: "text-xs font-normal line-clamp-1 text-base-content/50",
                            if let Some(author) = &pack.author {
                              "by {author}"
                            } else {
                              "by N/A"
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
        if is_open() && has_soundpacks() {
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
