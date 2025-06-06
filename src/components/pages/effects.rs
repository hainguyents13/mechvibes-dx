use dioxus::prelude::*;
use lucide_dioxus::Sparkles;

use crate::components::ui::{Collapse, PageHeader, Toggler};

#[component]
pub fn EffectsPage() -> Element {
    // Audio effects toggles
    let mut random_pitch_enabled = use_signal(|| false);
    let mut random_keys_enabled = use_signal(|| false);

    // Background Music states
    let mut lofi_music_enabled = use_signal(|| false);
    let mut lofi_auto_play = use_signal(|| false);

    // Ambient Sounds states
    let mut rain_sound_enabled = use_signal(|| false);
    let mut rain_auto_play = use_signal(|| false);
    let mut crow_noise_enabled = use_signal(|| false);
    let mut crow_auto_play = use_signal(|| false);

    rsx! {
      div { class: "p-12 pb-32",
        // Page header
        PageHeader {
          title: "Effects".to_string(),
          subtitle: "Sound effects, ambient sounds, and more".to_string(),
          icon: Some(rsx! {
            Sparkles { class: "w-8 h-8 mx-auto" }
          }),
        }
        // Effects Configuration
        div { class: "space-y-4",
          // Audio Effects Section
          Collapse {
            title: "Audio effects".to_string(),
            group_name: "effect-collapse".to_string(),
            default_open: true,
            children: rsx! {
              div { class: "space-y-6 ",
                // Random Pitch Toggle
                Toggler {
                  title: "Random pitch".to_string(),
                  description: Some(
                      "Add subtle pitch variations to make sounds feel more natural and less repetitive."
                          .to_string(),
                  ),
                  checked: random_pitch_enabled(),
                  on_change: move |new_value: bool| {
                      random_pitch_enabled.set(new_value);
                  },
                }
                Toggler {
                  title: "Random keys".to_string(),
                  description: Some(
                      "Randomize all keys in the selected soundpacks, except for the modifier keys and spacebar."
                          .to_string(),
                  ),
                  checked: random_keys_enabled(),
                  on_change: move |new_value: bool| {
                      random_keys_enabled.set(new_value);
                  },
                }
              }
            },
          }
          // Background Music Section
          Collapse {
            title: "Background music".to_string(),
            group_name: "effect-collapse".to_string(),
            children: rsx! {
              div { class: "space-y-6",
                // Lo-fi Music Toggle
                Toggler {
                  title: "Lo-fi music".to_string(),
                  description: Some(
                      "Play relaxing lo-fi hip hop beats in the background while typing.".to_string(),
                  ),
                  checked: lofi_music_enabled(),
                  on_change: move |new_value: bool| {
                      lofi_music_enabled.set(new_value);
                  },
                }
              
                // Controls for Lofi Music
                div { class: "ml-4 flex items-center gap-2",
                  button {
                    class: "btn btn-xs btn-outline",
                    onclick: move |_| {
                        println!("🎲 Randomizing lo-fi music...");
                    },
                    "🎲 Randomize"
                  }
                  Toggler {
                    title: "Auto-play with Mechvibes".to_string(),
                    description: None,
                    checked: lofi_auto_play(),
                    on_change: move |new_value: bool| {
                        lofi_auto_play.set(new_value);
                    },
                    size: "xs",
                  }
                }
              }
            },
          }
          // Ambient Sounds Section
          Collapse {
            title: "Ambient sounds".to_string(),
            group_name: "effect-collapse".to_string(),
            children: rsx! {
              div { class: "space-y-6",
                // Rain Sound Toggle
                Toggler {
                  title: "Rain sound".to_string(),
                  description: Some("Gentle rain sounds to create a cozy atmosphere while working.".to_string()),
                  checked: rain_sound_enabled(),
                  on_change: move |new_value: bool| {
                      rain_sound_enabled.set(new_value);
                  },
                }
              
                // Controls for Rain Sound
                div { class: "ml-4 flex items-center gap-2",
                  button {
                    class: "btn btn-xs btn-outline",
                    onclick: move |_| {
                        println!("🎲 Randomizing rain sound...");
                    },
                    "🎲 Randomize"
                  }
                  Toggler {
                    title: "Auto-play with Mechvibes".to_string(),
                    description: None,
                    checked: rain_auto_play(),
                    on_change: move |new_value: bool| {
                        rain_auto_play.set(new_value);
                    },
                    size: "xs",
                  }
                }
              
                div { class: "divider" }
                // Café background noise Toggle
                Toggler {
                  title: "Café background noise".to_string(),
                  description: Some("Add background coffee shop chatter to your workspace.".to_string()),
                  checked: crow_noise_enabled(),
                  on_change: move |new_value: bool| {
                      crow_noise_enabled.set(new_value);
                  },
                }
              
                // Controls for Crow Noise
                div { class: "ml-4 flex items-center gap-2",
                  button {
                    class: "btn btn-xs btn-outline",
                    onclick: move |_| {
                        println!("🎲 Randomizing crow noise...");
                    },
                    "🎲 Randomize"
                  }
                  Toggler {
                    title: "Auto-play with Mechvibes".to_string(),
                    description: None,
                    checked: crow_auto_play(),
                    on_change: move |new_value: bool| {
                        crow_auto_play.set(new_value);
                    },
                    size: "xs",
                  }
                }
              }
            },
          }
        }
      }
    }
}
