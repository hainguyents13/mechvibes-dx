use crate::components::logo::Logo;
use crate::components::soundpack_selector::{ KeyboardSoundpackSelector, MouseSoundpackSelector };
use crate::components::volume_slider::{ KeyboardVolumeSlider, MouseVolumeSlider };
use crate::libs::AudioContext;
use crate::utils::config::use_config;
use crate::utils::constants::APP_NAME_DISPLAY;
use dioxus::prelude::*;
use futures_timer::Delay;
use lucide_dioxus::Heart;
use std::sync::atomic::{ AtomicU64, Ordering };
use std::sync::Arc;
use std::time::Duration;

#[component]
pub fn HomePage(audio_ctx: Arc<AudioContext>) -> Element {
    // Use shared config hook
    let (config, update_config) = use_config();

    // Get current version for display
    let current_version = crate::utils::constants::APP_VERSION;

    // Volume states from config
    let mut volume = use_signal(|| config().volume);
    let mut mouse_volume = use_signal(|| config().mouse_volume);

    // Use atomic counters to track save tasks and cancel old ones
    let save_counter = use_signal(|| Arc::new(AtomicU64::new(0)));
    let mouse_save_counter = use_signal(|| Arc::new(AtomicU64::new(0)));

    // Update audio system volume when the volume control changes (enable_sound is handled by sound_manager)
    let ctx = audio_ctx.clone();
    use_effect(move || {
        ctx.set_volume(volume());
    });

    // Update audio system mouse volume when the mouse volume control changes
    let ctx = audio_ctx.clone();
    use_effect(move || {
        ctx.set_mouse_volume(mouse_volume());
    });

    // Debounce effect for saving keyboard volume config changes
    {
        let update_config = update_config.clone();
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
                    update_config(
                        Box::new(move |config| {
                            config.volume = current_volume;
                        })
                    );
                }
                // If not the latest, this task was "cancelled" by a newer one
            });
        });
    }

    // Debounce effect for saving mouse volume config changes
    {
        let update_config = update_config.clone();
        use_effect(move || {
            let current_mouse_volume = mouse_volume();

            // Increment counter to invalidate previous save tasks
            let current_task_id = mouse_save_counter().fetch_add(1, Ordering::SeqCst) + 1;

            let update_config = update_config.clone();
            let mouse_save_counter_clone = mouse_save_counter();

            spawn(async move {
                // Wait for 500ms
                Delay::new(Duration::from_millis(500)).await;

                // Check if this task is still the latest one
                if mouse_save_counter_clone.load(Ordering::SeqCst) == current_task_id {
                    // This is still the latest task, save the config
                    update_config(
                        Box::new(move |config| {
                            config.mouse_volume = current_mouse_volume;
                        })
                    );
                }
                // If not the latest, this task was "cancelled" by a newer one
            });
        });
    }

    rsx! {
      div { class: "flex flex-col gap-10 px-3 pb-0",
        div { class: "mb-2 mt-4",
          // Mechvibes logo with animated press effect
          Logo {}
        }
        // Main content for home page
        div { class: "flex flex-col {crate::utils::spacing::GAP_SPACING}",
          div { class: "{crate::utils::spacing::SECTION_SPACING}",
            KeyboardSoundpackSelector {}
            KeyboardVolumeSlider {
              volume,
              on_change: move |new_volume: f32| {
                  volume.set(new_volume);
              },
            }
          }
          div { class: "divider m-0" }
          div { class: "{crate::utils::spacing::SECTION_SPACING}",
            // Mouse soundpack selector and volume control
            MouseSoundpackSelector {}
            MouseVolumeSlider {
              volume: mouse_volume,
              on_change: move |new_mouse_volume: f32| {
                  mouse_volume.set(new_mouse_volume);
              },
            }
          }
          // div { class: "divider m-0" }
          div { class: "text-center space-y-2 mt-8",
            // Version
            div { class: "text-sm text-base-content/70 font-bold",
              "{APP_NAME_DISPLAY} (v{current_version})"
            }
            // Footer with credits
            div { class: "text-xs text-base-content/50",
              span { "Made with " }
              Heart { class: "inline w-3.5 h-3.5 -mt-1 text-primary/70 fill-primary/30" }
              span { " by " }
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
}
