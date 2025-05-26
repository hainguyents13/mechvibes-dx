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
      div { class: "container mx-auto p-16 text-center flex flex-col gap-6",
        div { class: "mb-12",
          // Mechvibes logo with animated press effect
          Logo {}
        } // Main content for home page

        // Soundpack selector
        SoundpackSelector { audio_ctx: audio_ctx.clone() }
        // Volume control slider for sound effects
        VolumeSlider { volume, on_change: None }
      }
    }
}
