use crate::components::logo::Logo;
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
            }

            // Main content for home page
            div { class: "space-y-6",
                h1 { class: "text-3xl font-bold text-white mb-4", "Welcome to Mechvibes DX" }
                p { class: "text-gray-300 text-lg mb-6",
                    "Experience the satisfying sound of mechanical keyboards with every keystroke."
                }

                // Quick stats or features
                div { class: "grid grid-cols-1 md:grid-cols-3 gap-4 mt-8",
                    div { class: "bg-gray-800 p-4 rounded-lg",
                        div { class: "text-2xl mb-2", "🎹" }
                        h3 { class: "text-white font-semibold", "Realistic Sounds" }
                        p { class: "text-gray-400 text-sm", "High-quality keyboard sounds" }
                    }
                    div { class: "bg-gray-800 p-4 rounded-lg",
                        div { class: "text-2xl mb-2", "🎵" }
                        h3 { class: "text-white font-semibold", "Multiple Soundpacks" }
                        p { class: "text-gray-400 text-sm", "Choose your favorite keyboard" }
                    }
                    div { class: "bg-gray-800 p-4 rounded-lg",
                        div { class: "text-2xl mb-2", "⚡" }
                        h3 { class: "text-white font-semibold", "Low Latency" }
                        p { class: "text-gray-400 text-sm", "Instant response to keystrokes" }
                    }
                }
            }

            // Volume control slider for sound effects
            VolumeSlider { volume }
        }
    }
}
