use dioxus::prelude::*;
use lucide_dioxus::{
    CloudSunRain,
    Play,
    Pause,
    SkipForward,
    Volume2,
    VolumeOff,
    TreePine,
    CloudRain,
    Waves,
    Zap,
    Flame,
    Wind,
    Moon,
    Coffee,
    Radio,
};

use crate::components::ui::{ PageHeader };
use crate::state::music::{
    MusicPlayerState,
    initialize_global_music_player_state,
    update_global_music_player_state,
    get_global_music_player_state_copy,
};
use crate::state::ambiance::{
    AmbiancePlayerState,
    initialize_global_ambiance_player_state,
    update_global_ambiance_player_state,
    get_global_ambiance_player_state_copy,
    play_ambiance_sound,
    stop_ambiance_sound,
    pause_all_ambiance_sounds,
    resume_all_ambiance_sounds,
    set_ambiance_sound_volume,
    set_global_ambiance_volume,
    set_global_ambiance_mute,
};

// Music Player Panel Component
#[component]
fn MusicPlayerPanel(
    music_player: Signal<MusicPlayerState>,
    refresh_trigger: Signal<i32>,
    is_loading: Signal<bool>
) -> Element {
    // Get current track info
    let (current_track, current_artist, _, _) = music_player().get_current_track_info();
    let current_track_image = music_player().get_current_track_image();
    // Local loading state for next track
    let is_next_track_loading = use_signal(|| false);

    rsx! {
      div { class: "bg-base-200 border border-base-300 rounded-box p-4 space-y-4 relative overflow-hidden",
        if is_loading() {
          div { class: "text-center py-4",
            span { class: "loading loading-spinner loading-md" }
            p { class: "text-sm text-base-content/70 mt-2", "Loading music..." }
          }
        } else {
          div {
            class: format!(
                "absolute right-[-99px] top-[-120px] h-50 w-50 rounded-full ease-linear opacity-80 {}",
                if music_player().is_playing { "animate-spin " } else { "" },
            ),
            style: "background-image: url('{current_track_image}'); background-size: cover; background-position: center; animation-duration: 20s;",
          }
          // Track Info
          div { class: "space-y-1 relative z-10",
            div { class: "text-sm font-semibold text-base-content", "{current_track}" }
            div { class: "text-xs text-base-content/90", "{current_artist}" }
          } // Control Buttons
          div { class: "flex items-center gap-2 relative z-10",
            button {
              class: "btn btn-primary btn-square rounded-box shadow-lg",
              onclick: move |_| {
                  update_global_music_player_state(|player| {
                      let _ = player.play_pause();
                  });
                  refresh_trigger.set(refresh_trigger() + 1);
              },
              if music_player().is_playing {
                Pause { class: "w-5 h-5" }
              } else {
                Play { class: "w-5 h-5" }
              }
            }
            button {
              class: format!(
                  "btn btn-ghost btn-square rounded-box {}",
                  if is_next_track_loading() { "loading" } else { "" },
              ),
              disabled: is_next_track_loading(),
              onclick: move |_| {
                  let mut refresh_trigger = refresh_trigger.clone();
                  let mut is_next_track_loading = is_next_track_loading.clone();
                  spawn(async move {
                      is_next_track_loading.set(true);
                      tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                      update_global_music_player_state(|player| {
                          if let Some(track_title) = player.next_track() {
                              println!("Next track: {}", track_title);
                          }
                      });
                      refresh_trigger.set(refresh_trigger() + 1);
                      is_next_track_loading.set(false);
                  });
              },
              if is_next_track_loading() {
                span { class: "loading loading-spinner loading-sm" }
              } else {
                SkipForward { class: "w-5 h-5" }
              }
            }
            // Volume Control
            div { class: "flex items-center grow gap-3 relative z-10",
              button {
                class: "btn btn-ghost btn-sm btn-square rounded-box",
                onclick: move |_| {
                    update_global_music_player_state(|player| {
                        player.toggle_mute();
                    });
                    refresh_trigger.set(refresh_trigger() + 1);
                },
                if music_player().is_muted {
                  VolumeOff { class: "w-4 h-4" }
                } else {
                  Volume2 { class: "w-4 h-4" }
                }
              }
              input {
                r#type: "range",
                class: "range range-xs ",
                min: "0",
                max: "100",
                value: "{music_player().volume}",
                disabled: music_player().is_muted,
                oninput: move |evt| {
                    if let Ok(val) = evt.value().parse::<f32>() {
                        update_global_music_player_state(|player| {
                            player.set_volume(val);
                        });
                        refresh_trigger.set(refresh_trigger() + 1);
                    }
                },
              }
              span { class: "text-xs text-base-content font-bold text-right w-8 shrink-0",
                "{music_player().volume as i32}%"
              }
            }
          }
        }
      }
    }
}

// Sound Selection Panel Component
#[component]
fn SoundSelectionPanel(
    ambiance_player: Signal<AmbiancePlayerState>,
    refresh_trigger: Signal<i32>
) -> Element {
    rsx! {
      div { class: "bg-base-200 border border-base-300 rounded-box p-6 space-y-4",
        // Header
        div { class: "space-y-1",
          h3 { class: "text-lg font-semibold", "Ambiance sounds" }
          p { class: "text-sm text-base-content/70",
            if ambiance_player().active_sounds.is_empty() {
              "Select at least one sound to play"
            } else {
              {
                  format!(
                      "{} active sound{}",
                      ambiance_player().active_sounds.len(),
                      if ambiance_player().active_sounds.len() == 1 { "" } else { "s" },
                  )
              }
            }
          }
        }
        // Global Controls
        div { class: "flex items-center gap-4 w-full justify-between",
          // Play/Pause Button
          button {
            class: "btn btn-primary btn-square",
            onclick: move |_| {
                update_global_ambiance_player_state(|player| {
                    player.toggle_play_pause();
                });
                if let Some(current_state) = get_global_ambiance_player_state_copy() {
                    if current_state.is_playing {
                        let _ = resume_all_ambiance_sounds();
                    } else {
                        let _ = pause_all_ambiance_sounds();
                    }
                }
                refresh_trigger.set(refresh_trigger() + 1);
            },
            if ambiance_player().is_playing {
              Pause { class: "w-4 h-4" }
            } else {
              Play { class: "w-4 h-4" }
            }
          }
          // Mute Button
          button {
            class: "btn btn-ghost btn-sm btn-square",
            onclick: move |_| {
                update_global_ambiance_player_state(|player| {
                    player.toggle_mute();
                    let _ = player.save_config();
                });
                if let Some(current_state) = get_global_ambiance_player_state_copy() {
                    let _ = set_global_ambiance_mute(current_state.is_muted);
                }
                refresh_trigger.set(refresh_trigger() + 1);
            },
            if ambiance_player().is_muted {
              VolumeOff { class: "w-4 h-4" }
            } else {
              Volume2 { class: "w-4 h-4" }
            }
          }
          input {
            r#type: "range",
            class: "range range-xs",
            min: "0",
            max: "100",
            value: "{(ambiance_player().global_volume * 100.0) as i32}",
            disabled: ambiance_player().is_muted || !ambiance_player().is_playing,
            oninput: move |evt| {
                if let Ok(val) = evt.value().parse::<f32>() {
                    update_global_ambiance_player_state(|player| {
                        player.set_global_volume(val / 100.0);
                        let _ = player.save_config();
                    });
                    let _ = set_global_ambiance_volume(val / 100.0);
                    refresh_trigger.set(refresh_trigger() + 1);
                }
            },
          }
          div { class: "text-xs text-base-content/70 whitespace-nowrap",
            "{(ambiance_player().global_volume * 100.0) as i32}%"
          }
        }
        // Sound Selection List
        div { class: "{crate::utils::spacing::SECTION_SPACING}",
          for sound in ambiance_player().sounds.iter() {
            div {
              class: format!(
                  " rounded-box border {} {} {}",
                  crate::utils::spacing::CARD_PADDING,
                  crate::utils::spacing::SECTION_SPACING,
                  if ambiance_player().is_sound_active(&sound.id) {
                      "bg-base-100 border-base-100"
                  } else {
                      "bg-base-300 border-base-300"
                  },
              ),
              // First line: Icon, Name, Description (left) + Toggle (right)
              div { class: "flex items-center justify-between",
                // Left side: Icon + Name + Description
                div { class: "flex items-center gap-3 flex-1",
                  // Sound Icon
                  div { class: "flex-shrink-0",
                    match sound.icon.as_str() {
                        "cloud-rain" => rsx! {
                          CloudRain { class: "w-5 h-5 text-base-content/50" }
                        },
                        "tree-pine" => rsx! {
                          TreePine { class: "w-5 h-5 text-base-content/50" }
                        },
                        "waves" => rsx! {
                          Waves { class: "w-5 h-5 text-base-content/50" }
                        },
                        "zap" => rsx! {
                          Zap { class: "w-5 h-5 text-base-content/50" }
                        },
                        "flame" => rsx! {
                          Flame { class: "w-5 h-5 text-base-content/50" }
                        },
                        "wind" => rsx! {
                          Wind { class: "w-5 h-5 text-base-content/50" }
                        },
                        "moon" => rsx! {
                          Moon { class: "w-5 h-5 text-base-content/50" }
                        },
                        "coffee" => rsx! {
                          Coffee { class: "w-5 h-5 text-base-content/50" }
                        },
                        "radio" => rsx! {
                          Radio { class: "w-5 h-5 text-base-content/50" }
                        },
                        _ => rsx! {
                          TreePine { class: "w-5 h-5 text-base-content/50" }
                        },
                    }
                  }
                  // Sound Name and Description
                  div { class: "flex-1 min-w-0",
                    div { class: "text-sm font-medium text-base-content truncate",
                      "{sound.name}"
                    }
                    div { class: "text-xs text-base-content/60 truncate",
                      "{sound.description}"
                    }
                  }
                }
                // Right side: Toggle Switch
                div { class: "flex-shrink-0",
                  input {
                    r#type: "checkbox",
                    class: "toggle toggle-xs toggle-primary",
                    checked: ambiance_player().is_sound_active(&sound.id),
                    onchange: {
                        let sound_id = sound.id.clone();
                        let sound_url = sound.audio_url.clone();
                        move |_| {
                            let is_currently_active = if let Some(current_state) = get_global_ambiance_player_state_copy() {
                                current_state.is_sound_active(&sound_id)
                            } else {
                                false
                            };
                            update_global_ambiance_player_state(|player| {
                                player.toggle_sound(sound_id.clone());
                                let _ = player.save_config();
                            });
                            if let Some(current_state) = get_global_ambiance_player_state_copy() {
                                if current_state.is_sound_active(&sound_id) && !is_currently_active {
                                    if current_state.is_playing {
                                        let volume = current_state.get_sound_volume(&sound_id);
                                        let global_volume = if current_state.is_muted {
                                            0.0
                                        } else {
                                            current_state.global_volume
                                        };
                                        let _ = play_ambiance_sound(
                                            sound_id.clone(),
                                            sound_url.clone(),
                                            volume * global_volume,
                                        );
                                    }
                                } else if !current_state.is_sound_active(&sound_id)
                                    && is_currently_active
                                {
                                    let _ = stop_ambiance_sound(&sound_id);
                                }
                            }
                            refresh_trigger.set(refresh_trigger() + 1);
                        }
                    },
                  }
                }
              }
              // Second line: Volume Control (only show if sound is active)
              if ambiance_player().is_sound_active(&sound.id) {
                div { class: "flex items-center gap-3",
                  input {
                    r#type: "range",
                    class: "range range-xs ml-8",
                    min: "0",
                    max: "100",
                    value: "{(ambiance_player().get_sound_volume(&sound.id) * 100.0) as i32}",
                    oninput: {
                        let sound_id = sound.id.clone();
                        move |evt: dioxus::prelude::Event<dioxus::html::FormData>| {
                            if let Ok(val) = evt.value().parse::<f32>() {
                                let volume = val / 100.0;
                                update_global_ambiance_player_state(|player| {
                                    player.set_sound_volume(sound_id.clone(), volume);
                                    let _ = player.save_config();
                                });
                                let _ = set_ambiance_sound_volume(&sound_id, volume);
                                refresh_trigger.set(refresh_trigger() + 1);
                            }
                        }
                    },
                  }
                  span { class: "text-xs text-base-content/70 w-10 text-right",
                    "{(ambiance_player().get_sound_volume(&sound.id) * 100.0) as i32}%"
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
pub fn MoodPage() -> Element {
    // Use global state instead of local signal
    let mut music_player = use_signal(|| MusicPlayerState::new());
    let mut ambiance_player = use_signal(|| AmbiancePlayerState::new());
    let mut is_loading = use_signal(|| false);

    // Force re-render when global state changes
    let refresh_trigger = use_signal(|| 0);

    // Initialize global players on component mount
    use_effect(move || {
        spawn(async move {
            is_loading.set(true);

            // Initialize global music player state if not already done
            if let Err(e) = initialize_global_music_player_state().await {
                eprintln!("Failed to initialize global music player: {}", e);
            }

            // Initialize global ambiance player state
            initialize_global_ambiance_player_state();

            // Get current global states
            if let Some(global_music_state) = get_global_music_player_state_copy() {
                music_player.set(global_music_state);
            }

            if let Some(global_ambiance_state) = get_global_ambiance_player_state_copy() {
                ambiance_player.set(global_ambiance_state);
            }

            is_loading.set(false);
        });
    });

    // Update local state when refresh trigger changes
    use_effect(move || {
        let _trigger = refresh_trigger();
        if let Some(global_music_state) = get_global_music_player_state_copy() {
            music_player.set(global_music_state);
        }
        if let Some(global_ambiance_state) = get_global_ambiance_player_state_copy() {
            ambiance_player.set(global_ambiance_state);
        }
    });

    rsx! {
      div { class: "{crate::utils::spacing::SECTION_SPACING_LG}",
        PageHeader {
          title: "Mood".to_string(),
          subtitle: "Music and ambient sounds to set the perfect atmosphere".to_string(),
          icon: Some(rsx! {
            CloudSunRain { class: "w-8 h-8 mx-auto" }
          }),
        }

        // Music Player Panel
        // MusicPlayerPanel { music_player, refresh_trigger, is_loading }
        // Sound Selection Panel
        SoundSelectionPanel { ambiance_player, refresh_trigger }
      }
    }
}
