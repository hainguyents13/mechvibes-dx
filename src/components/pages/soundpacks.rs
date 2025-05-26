use crate::components::soundpack_selector::SoundpackSelector;
use crate::libs::AudioContext;
use dioxus::prelude::*;
use std::sync::Arc;

#[component]
pub fn SoundpacksPage(audio_ctx: Arc<AudioContext>) -> Element {
    rsx! {
        div { class: "container mx-auto p-16 text-center flex flex-col gap-6",
            // Page header
            div { class: "mb-8",
                h1 { class: "text-3xl font-bold text-white mb-4", "🎵 Soundpacks" }
                p { class: "text-gray-300 text-lg",
                    "Choose your favorite mechanical keyboard sound profile."
                }
            }

            // Soundpack selector - main component for this page
            div { class: "flex-1",
                SoundpackSelector { audio_ctx: audio_ctx.clone() }
            }

            // Additional info section
            div { class: "mt-8 bg-gray-800 p-6 rounded-lg",
                h3 { class: "text-xl font-semibold text-white mb-3", "About Soundpacks" }
                div { class: "text-left space-y-2 text-gray-300",
                    p { "Each soundpack contains unique audio samples recorded from real mechanical keyboards." }
                    p { "You can switch between soundpacks in real-time to find the perfect sound for your typing experience." }
                    p { class: "text-blue-400", "💡 Tip: Try different soundpacks while typing to hear the difference!" }
                }
            }
        }
    }
}
