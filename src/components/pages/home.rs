use crate::components::logo::Logo;
use crate::components::soundpack_selector::SoundpackSelector;
use crate::components::volume_slider::VolumeSlider;
use crate::libs::AudioContext;
use crate::state::config_utils::use_config;
use dioxus::prelude::*;
use std::sync::Arc;

#[component]
pub fn HomePage(audio_ctx: Arc<AudioContext>) -> Element {    // Use shared config hook
    let (config, update_config) = use_config();
    // Volume state from config
    let mut volume = use_signal(|| config().volume);// Update audio system volume when the volume control changes (enable_sound is handled by sound_manager)
    let ctx = audio_ctx.clone();
    use_effect(move || {
        ctx.set_volume(volume());
    });

    rsx! {
      div { class: "container mx-auto p-16 text-center flex flex-col gap-2",
        div { class: "mb-12",
          // Mechvibes logo with animated press effect
          Logo {}
        } // Main content for home page

        // Soundpack selector
        SoundpackSelector { audio_ctx: audio_ctx.clone() }
        // divider
        div { class: "divider" } // Volume control slider
        VolumeSlider {
          volume,
          on_change: move |new_volume: f32| {
              volume.set(new_volume);
              update_config(
                  Box::new(move |config| {
                      config.volume = new_volume;
                  }),
              );
          },
        }
        // divider
        div { class: "divider" } // Version
        div { class: "text-sm text-base-content font-bold", "MechvibesDX (Beta)" }
        // Footer with credits
        div { class: "text-sm text-base-content/70 mt-4",
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
