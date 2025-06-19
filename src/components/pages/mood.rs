use dioxus::prelude::*;
use lucide_dioxus::{ CloudSunRain, Play, Pause, SkipForward, Volume2, VolumeOff };

use crate::components::ui::{ PageHeader };
use crate::state::music::{
    MusicPlayerState,
    initialize_global_music_player_state,
    update_global_music_player_state,
    get_global_music_player_state_copy,
};

#[component]
pub fn MoodPage() -> Element {
    // Use global state instead of local signal
    let mut music_player = use_signal(|| MusicPlayerState::new());
    let mut is_loading = use_signal(|| false);

    // Force re-render when global state changes
    let mut refresh_trigger = use_signal(|| 0);

    // Initialize global music player state on component mount
    use_effect(move || {
        spawn(async move {
            is_loading.set(true);

            // Initialize global state if not already done
            if let Err(e) = initialize_global_music_player_state().await {
                eprintln!("Failed to initialize global music player: {}", e);
            }

            // Get current global state
            if let Some(global_state) = get_global_music_player_state_copy() {
                music_player.set(global_state);
            }

            is_loading.set(false);
        });
    });

    // Update local state when refresh trigger changes
    use_effect(move || {
        let _trigger = refresh_trigger();
        if let Some(global_state) = get_global_music_player_state_copy() {
            music_player.set(global_state);
        }
    });

    // Get current track info
    let (current_track, current_artist, _, _) = music_player().get_current_track_info();
    let current_track_image = music_player().get_current_track_image();

    rsx! {
      div { class: "space-y-6",
        PageHeader {
          title: "Mood".to_string(),
          subtitle: "".to_string(),
          icon: Some(rsx! {
            CloudSunRain { class: "w-8 h-8 mx-auto" }
          }),
        }
        // Music Player Component
        div { class: "bg-base-200 border border-base-300 rounded-box p-6 space-y-4 relative overflow-hidden",
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
              div { class: "text-sm font-semibold text-base-content drop-shadow-lg",
                "{current_track}"
              }
              div { class: "text-xs text-base-content/90 drop-shadow-md",
                "{current_artist}"
              }
            }
            // Control Buttons
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
                class: "btn btn-ghost btn-square rounded-box ",
                onclick: move |_| {
                    update_global_music_player_state(|player| {
                        if let Some(track_title) = player.next_track() {
                            println!("Next track: {}", track_title);
                        }
                    });
                    refresh_trigger.set(refresh_trigger() + 1);
                },
                SkipForward { class: "w-5 h-5" }
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
}
