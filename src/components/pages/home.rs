use crate::components::logo::Logo;
use crate::components::soundpack_selector::SoundpackSelector;
use crate::components::volume_slider::VolumeSlider;
use crate::libs::AudioContext;
use crate::state::config_utils::use_config;
use dioxus::prelude::*;
use futures_timer::Delay;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;

#[component]
pub fn HomePage(audio_ctx: Arc<AudioContext>) -> Element {
    // Use shared config hook
    let (config, update_config) = use_config(); // Volume state from config
    let mut volume = use_signal(|| config().volume);
    // Use atomic counter to track save tasks and cancel old ones
    let save_counter = use_signal(|| Arc::new(AtomicU64::new(0)));

    // Update audio system volume when the volume control changes (enable_sound is handled by sound_manager)
    let ctx = audio_ctx.clone();
    use_effect(move || {
        ctx.set_volume(volume());
    });

    // Debounce effect for saving config changes
    use_effect(move || {
        let current_volume = volume();

        // Increment counter to invalidate previous save tasks
        let current_task_id = save_counter().fetch_add(1, Ordering::SeqCst) + 1;

        let update_config = update_config.clone();
        let save_counter_clone = save_counter();

        spawn(async move {
            // Wait for 500ms
            Delay::new(Duration::from_millis(500)).await;

            // Check if this task is still the latest one
            if save_counter_clone.load(Ordering::SeqCst) == current_task_id {
                // This is still the latest task, save the config
                update_config(Box::new(move |config| {
                    config.volume = current_volume;
                }));
            }
            // If not the latest, this task was "cancelled" by a newer one
        });
    });

    rsx! {
      div { class: "flex flex-col gap-10 p-16",
        div { class: "mb-2",
          // Mechvibes logo with animated press effect
          Logo {}
        }
        // Main content for home page
        div {
          // Soundpack selector          // div { class: "divider", "Keyboard" } // Version
          class: "flex flex-col gap-6",
          SoundpackSelector {}
          VolumeSlider {
            volume,
            on_change: move |new_volume: f32| {
                volume.set(new_volume);
            },
          }
        }
        div { class: "text-center",
          // divider
          div { class: "divider" }
          // Version
          div { class: "text-sm text-base-content/70 font-bold", "MechvibesDX (Beta)" }
          // Footer with credits
          div { class: "text-sm text-base-content/50 mt-4",
            "Made with ❤️ by "
            a {
              href: "https://github.com/hainguyents13/mechvibes-dx",
              target: "_blank",
              class: "link ",
              "hainguyents13"
            }
            br {}
            " and "
            a {
              href: "https://github.com/hainguyents13/mechvibes-dx/graphs/contributors",
              target: "_blank",
              class: "link ",
              "these awesome people"
            }
          }
        }
      }
    }
}
