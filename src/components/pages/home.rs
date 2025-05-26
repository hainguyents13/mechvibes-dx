use crate::components::logo::Logo;
use crate::components::soundpack_selector::SoundpackSelector;
use crate::components::volume_slider::VolumeSlider;
use crate::libs::AudioContext;
use dioxus::prelude::*;
use std::sync::Arc;

#[component]
pub fn HomePage(audio_ctx: Arc<AudioContext>) -> Element {
    // volume state
    let volume = use_signal(|| 1.0f32);

    // Update audio system volume when the volume control changes
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
        div { class: "divider" }
        // Volume control slider
        VolumeSlider { volume, on_change: None }
        // divider
        div { class: "divider" }
        // Version
        div { class: "text-sm text-gray-800 font-bold", "Mechvibes v3.0.6" }
        // Footer with credits
        div { class: "text-sm text-gray-500 mt-4",
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
