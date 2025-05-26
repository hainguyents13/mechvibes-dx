use crate::components::volume_slider::VolumeSlider;
use crate::libs::AudioContext;
use dioxus::prelude::*;
use std::sync::Arc;

#[component]
pub fn SettingsPage(audio_ctx: Arc<AudioContext>) -> Element {
    // Settings state
    let mut volume = use_signal(|| 1.0f32);
    let mut enable_sound = use_signal(|| true);
    let mut auto_start = use_signal(|| false);
    let mut show_notifications = use_signal(|| true);

    // Update audio system volume
    let ctx = audio_ctx.clone();
    use_effect(move || {
        ctx.set_volume(if enable_sound() { volume() } else { 0.0 });
    });

    rsx! {
      div { class: "container mx-auto p-16 text-center flex flex-col gap-6",
        // Page header
        div { class: "mb-8",
          h1 { class: "text-3xl font-bold text-white mb-4", "⚙️ Settings" }
          p { class: "text-gray-300 text-lg", "Customize your Mechvibes DX experience." }
        }
        // Settings sections
        div { class: "space-y-6 max-w-2xl mx-auto",
          // Audio Settings
          div { class: "bg-gray-800 p-6 rounded-lg text-left",
            h3 { class: "text-xl font-semibold text-white mb-4 flex items-center gap-2",
              span { "🔊" }
              "Audio Settings"
            }
            div { class: "space-y-4",
              // Enable/Disable Sound
              div { class: "flex items-center justify-between",
                label { class: "text-gray-300", "Enable Sound Effects" }
                input {
                  r#type: "checkbox",
                  class: "toggle toggle-primary",
                  checked: enable_sound(),
                  onchange: move |evt| enable_sound.set(evt.value() == "true"),
                }
              }
              // Volume Control
              div { class: "space-y-2",
                label { class: "text-gray-300 block", "Volume" }
                VolumeSlider { volume }
              }
            }
          }
          // Application Settings
          div { class: "bg-gray-800 p-6 rounded-lg text-left",
            h3 { class: "text-xl font-semibold text-white mb-4 flex items-center gap-2",
              span { "💻" }
              "Application Settings"
            }
            div { class: "space-y-4",
              // Auto Start
              div { class: "flex items-center justify-between",
                div {
                  label { class: "text-gray-300 block", "Start with Windows" }
                  p { class: "text-gray-500 text-sm",
                    "Automatically start Mechvibes DX when Windows boots"
                  }
                }
                input {
                  r#type: "checkbox",
                  class: "toggle toggle-primary",
                  checked: auto_start(),
                  onchange: move |evt| auto_start.set(evt.value() == "true"),
                }
              }
              // Notifications
              div { class: "flex items-center justify-between",
                div {
                  label { class: "text-gray-300 block", "Show Notifications" }
                  p { class: "text-gray-500 text-sm",
                    "Display system notifications for important events"
                  }
                }
                input {
                  r#type: "checkbox",
                  class: "toggle toggle-primary",
                  checked: show_notifications(),
                  onchange: move |evt| show_notifications.set(evt.value() == "true"),
                }
              }
            }
          }
          // About Section
          div { class: "bg-gray-800 p-6 rounded-lg text-left",
            h3 { class: "text-xl font-semibold text-white mb-4 flex items-center gap-2",
              span { "ℹ️" }
              "About"
            }
            div { class: "space-y-2 text-gray-300",
              p {
                span { class: "font-semibold", "Mechvibes DX " }
                span { class: "text-gray-500", "v0.1.0" }
              }
              p {
                "A modern mechanical keyboard sound simulator built with Dioxus and Rust."
              }
              p { class: "text-blue-400",
                "Created with ❤️ by the Mechvibes community"
              }
              // Links
              div { class: "flex gap-4 mt-4",
                button { class: "bg-blue-600 hover:bg-blue-700 px-4 py-2 rounded text-white text-sm transition-colors",
                  "Check for Updates"
                }
                button { class: "bg-gray-600 hover:bg-gray-700 px-4 py-2 rounded text-white text-sm transition-colors",
                  "View License"
                }
              }
            }
          }
          // Reset Settings
          div { class: "bg-red-900 bg-opacity-20 border border-red-500 p-6 rounded-lg text-left",
            h3 { class: "text-xl font-semibold text-red-400 mb-4", "⚠️ Danger Zone" }
            p { class: "text-gray-300 mb-4",
              "Reset all settings to their default values."
            }
            button {
              class: "bg-red-600 hover:bg-red-700 px-4 py-2 rounded text-white text-sm transition-colors",
              onclick: move |_| {
                  volume.set(1.0);
                  enable_sound.set(true);
                  auto_start.set(false);
                  show_notifications.set(true);
              },
              "Reset to Defaults"
            }
          }
        }
      }
    }
}
