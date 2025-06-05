use dioxus::prelude::*;
use lucide_dioxus::Sparkles;

use crate::components::ui::PageHeader;

#[component]
pub fn EffectsPage() -> Element {
    // Local state for toggles
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
        } // Effects Configuration
        div { class: "space-y-4",
          // Audio Effects Section
          div { class: "collapse collapse-arrow border-base-300 bg-base-200",
            input {
              r#type: "radio",
              name: "effect-collapse",
              class: "",
              checked: true,
            }
            div { class: "collapse-title font-semibold", "Audio effects" }
            div { class: "collapse-content",
              div { class: "space-y-6 ",
                // Random Pitch Toggle
                label { class: "label cursor-pointer",
                  div { class: "space-y-1",
                    h3 { class: "text-base-content text-sm", "Random pitch" }
                    p { class: "text-xs whitespace-break-spaces text-base-content/70",
                      "Add subtle pitch variations to make sounds feel more natural and less repetitive."
                    }
                  }
                  input {
                    r#type: "checkbox",
                    class: "toggle toggle-sm",
                    checked: random_pitch_enabled(),
                    onchange: move |evt| {
                        random_pitch_enabled.set(evt.value() == "true");
                    },
                  }
                }
                label { class: "label cursor-pointer",
                  div { class: "space-y-1",
                    h3 { class: "text-base-content text-sm", "Random keys" }
                    p { class: "text-xs whitespace-break-spaces text-base-content/70",
                      "Randomize all keys in the selected soundpacks, except for the modifier keys and spacebar."
                    }
                  }
                  input {
                    r#type: "checkbox",
                    class: "toggle toggle-sm",
                    checked: random_keys_enabled(),
                    onchange: move |evt| {
                        random_keys_enabled.set(evt.value() == "true");
                    },
                  }
                }
              }
            }
          }

          // Background Music Section
          div { class: "collapse collapse-arrow border-base-300 bg-base-200",
            input {
              r#type: "radio",
              name: "effect-collapse",
              class: "",
            }
            div { class: "collapse-title font-semibold", "Background music" }
            div { class: "collapse-content",
              div { class: "space-y-6", // Lofi Music Toggle
                label { class: "label cursor-pointer",
                  div { class: "",
                    h3 { class: "text-base-content text-sm mb-1",
                      "Lo-fi music"
                    }
                    p { class: "text-xs whitespace-break-spaces text-base-content/70",
                      "Play relaxing lo-fi hip hop beats in the background while typing."
                    }
                  }
                  input {
                    r#type: "checkbox",
                    class: "toggle toggle-sm",
                    checked: lofi_music_enabled(),
                    onchange: move |evt| {
                        lofi_music_enabled.set(evt.value() == "true");
                    },
                  }
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
                  label { class: "label cursor-pointer",
                    span { class: "label-text text-xs mr-2",
                      "Auto-play with Mechvibes"
                    }
                    input {
                      r#type: "checkbox",
                      class: "toggle toggle-xs",
                      checked: lofi_auto_play(),
                      onchange: move |evt| {
                          lofi_auto_play.set(evt.value() == "true");
                      },
                    }
                  }
                }
              }
            }
          }

          // Ambient Sounds Section
          div { class: "collapse collapse-arrow border-base-300 bg-base-200",
            input {
              r#type: "radio",
              name: "effect-collapse",
              class: "",
            }
            div { class: "collapse-title font-semibold", "Ambient sounds" }
            div { class: "collapse-content",
              div { class: "space-y-6",
                // Rain Sound Toggle
                label { class: "label cursor-pointer w-full",
                  div { class: "space-y-1",
                    h3 { class: "text-base-content text-sm ", "Rain sound" }
                    p { class: "text-xs whitespace-break-spaces text-base-content/70",
                      "Gentle rain sounds to create a cozy atmosphere while working."
                    }
                  }
                  input {
                    r#type: "checkbox",
                    class: "toggle toggle-sm ml-auto",
                    checked: rain_sound_enabled(),
                    onchange: move |evt| {
                        rain_sound_enabled.set(evt.value() == "true");
                    },
                  }
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
                  label { class: "label cursor-pointer",
                    span { class: "label-text text-xs mr-2",
                      "Auto-play with Mechvibes"
                    }
                    input {
                      r#type: "checkbox",
                      class: "toggle toggle-xs",
                      checked: rain_auto_play(),
                      onchange: move |evt| {
                          rain_auto_play.set(evt.value() == "true");
                      },
                    }
                  }
                }
                div { class: "divider" }
                // Café background noise Toggle
                label { class: "label w-full cursor-pointer",
                  div { class: "space-y-1",
                    h3 { class: "text-base-content text-sm",
                      "Café background noise"
                    }
                    p { class: "text-xs whitespace-break-spaces text-base-content/70",
                      "Add background coffee shop chatter to your workspace."
                    }
                  }
                  input {
                    r#type: "checkbox",
                    class: "toggle toggle-sm ml-auto",
                    checked: crow_noise_enabled(),
                    onchange: move |evt| {
                        crow_noise_enabled.set(evt.value() == "true");
                    },
                  }
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
                  label { class: "label cursor-pointer",
                    span { class: "label-text text-xs mr-2",
                      "Auto-play with Mechvibes"
                    }
                    input {
                      r#type: "checkbox",
                      class: "toggle toggle-xs",
                      checked: crow_auto_play(),
                      onchange: move |evt| {
                          crow_auto_play.set(evt.value() == "true");
                      },
                    }
                  }
                }
              }
            }
          }
        }
      }
    }
}
