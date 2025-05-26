use crate::components::volume_slider::VolumeSlider;
use crate::libs::audio::AudioContext;
use crate::libs::theme::{use_effective_theme, use_theme, Theme};
use crate::state::config::AppConfig;
use dioxus::prelude::*;
use std::sync::Arc;

#[component]
pub fn SettingsPage(audio_ctx: Arc<AudioContext>) -> Element {
    // Load settings from config file
    let mut config = use_signal(|| AppConfig::load());

    // Settings state - initialize from config
    let mut volume = use_signal(|| config().volume);
    let mut enable_sound = use_signal(|| config().enable_sound);
    let mut auto_start = use_signal(|| config().auto_start);
    let mut show_notifications = use_signal(|| config().show_notifications);

    // Theme state - use theme context and initialize from config
    let mut theme = use_theme();
    let effective_theme = use_effective_theme();

    // Initialize theme from config on first load
    use_effect(move || {
        theme.set(config().theme.clone());
    });

    // Update audio system volume
    let ctx = audio_ctx.clone();
    use_effect(move || {
        ctx.set_volume(if enable_sound() { volume() } else { 0.0 });
    });
    rsx! {
      div {
        class: format!("container mx-auto p-16 text-center flex flex-col gap-6 {}", effective_theme.bg_primary()),
        // Page header
        div { class: "mb-8",
          h1 {
            class: format!("text-3xl font-bold {} mb-4", effective_theme.text_primary()),
            "⚙️ Settings"
          }
          p {
            class: format!("text-lg {}", effective_theme.text_secondary()),
            "Customize your Mechvibes DX experience."
          }
        }        // Settings sections
        div { class: "space-y-6 max-w-2xl mx-auto",
          // Audio Settings
          div {
            class: format!("{} p-6 rounded-lg text-left", effective_theme.bg_secondary()),
            h3 {
              class: format!("text-xl font-semibold {} mb-4 flex items-center gap-2", effective_theme.text_primary()),
              span { "🔊" }
              "Audio Settings"
            }
            div { class: "space-y-4",
              // Enable/Disable Sound
              div { class: "flex items-center justify-between",
                label {
                  class: effective_theme.text_secondary(),
                  "Enable Sound Effects"
                }                input {
                  r#type: "checkbox",
                  class: "toggle toggle-primary",
                  checked: enable_sound(),
                  onchange: move |evt| {
                    enable_sound.set(evt.value() == "true");
                    // Save config
                    let mut new_config = config();
                    new_config.enable_sound = enable_sound();
                    new_config.volume = volume();
                    new_config.auto_start = auto_start();
                    new_config.show_notifications = show_notifications();
                    new_config.theme = theme.read().clone();
                    new_config.last_updated = chrono::Utc::now();
                    let _ = new_config.save();
                    config.set(new_config);
                  },
                }
              }              // Volume Control
              div { class: "space-y-2",
                label {
                  class: format!("{} block", effective_theme.text_secondary()),
                  "Volume"
                }
                VolumeSlider {
                  volume,
                  on_change: move |new_volume: f32| {
                    // Save config when volume changes
                    let mut new_config = config();
                    new_config.volume = new_volume;
                    new_config.enable_sound = enable_sound();
                    new_config.auto_start = auto_start();
                    new_config.show_notifications = show_notifications();
                    new_config.theme = theme.read().clone();
                    new_config.last_updated = chrono::Utc::now();
                    let _ = new_config.save();
                    config.set(new_config);
                  }
                }
              }
            }
          }          // Application Settings
          div { class: format!("{} p-6 rounded-lg text-left", effective_theme.bg_secondary()),
            h3 { class: format!("text-xl font-semibold {} mb-4 flex items-center gap-2", effective_theme.text_primary()),
              span { "💻" }
              "Application Settings"
            }
            div { class: "space-y-4",
              // Theme Settings
              div { class: "space-y-2",
                label { class: format!("{} block", effective_theme.text_secondary()), "Theme" }
                p { class: format!("{} text-sm mb-3", effective_theme.text_tertiary()),
                  "Choose your preferred color scheme"                }
                div { class: "flex gap-2",                  button {
                    class: {
                      if matches!(*theme.read(), Theme::Dark) {
                        "px-4 py-2 rounded-lg text-sm transition-colors bg-blue-600 text-white".to_string()
                      } else {
                        format!("px-4 py-2 rounded-lg text-sm transition-colors {} {} {}",
                                effective_theme.bg_tertiary(),
                                effective_theme.text_secondary(),
                                effective_theme.bg_hover())
                      }
                    },
                    onclick: move |_| {
                      theme.set(Theme::Dark);
                      // Save config
                      let mut new_config = config();
                      new_config.enable_sound = enable_sound();
                      new_config.volume = volume();
                      new_config.auto_start = auto_start();
                      new_config.show_notifications = show_notifications();
                      new_config.theme = Theme::Dark;
                      new_config.last_updated = chrono::Utc::now();
                      let _ = new_config.save();
                      config.set(new_config);
                    },
                    "🌙 Dark"
                  }                  button {
                    class: {
                      if matches!(*theme.read(), Theme::Light) {
                        "px-4 py-2 rounded-lg text-sm transition-colors bg-blue-600 text-white".to_string()
                      } else {
                        format!("px-4 py-2 rounded-lg text-sm transition-colors {} {} {}",
                                effective_theme.bg_tertiary(),
                                effective_theme.text_secondary(),
                                effective_theme.bg_hover())
                      }
                    },
                    onclick: move |_| {
                      theme.set(Theme::Light);
                      // Save config
                      let mut new_config = config();
                      new_config.enable_sound = enable_sound();
                      new_config.volume = volume();
                      new_config.auto_start = auto_start();
                      new_config.show_notifications = show_notifications();
                      new_config.theme = Theme::Light;
                      new_config.last_updated = chrono::Utc::now();
                      let _ = new_config.save();
                      config.set(new_config);
                    },
                    "☀️ Light"
                  }                  button {
                    class: {
                      if matches!(*theme.read(), Theme::System) {
                        "px-4 py-2 rounded-lg text-sm transition-colors bg-blue-600 text-white".to_string()
                      } else {
                        format!("px-4 py-2 rounded-lg text-sm transition-colors {} {} {}",
                                effective_theme.bg_tertiary(),
                                effective_theme.text_secondary(),
                                effective_theme.bg_hover())
                      }
                    },
                    onclick: move |_| {
                      theme.set(Theme::System);
                      // Save config
                      let mut new_config = config();
                      new_config.enable_sound = enable_sound();
                      new_config.volume = volume();
                      new_config.auto_start = auto_start();
                      new_config.show_notifications = show_notifications();
                      new_config.theme = Theme::System;
                      new_config.last_updated = chrono::Utc::now();
                      let _ = new_config.save();
                      config.set(new_config);
                    },
                    "🖥️ System"
                  }
                }
              }              // Auto Start
              div { class: "flex items-center justify-between",
                div {
                  label { class: format!("{} block", effective_theme.text_secondary()), "Start with Windows" }
                  p { class: format!("{} text-sm", effective_theme.text_tertiary()),
                    "Automatically start Mechvibes DX when Windows boots"
                  }
                }                input {
                  r#type: "checkbox",
                  class: "toggle toggle-primary",
                  checked: auto_start(),
                  onchange: move |evt| {
                    auto_start.set(evt.value() == "true");
                    // Save config
                    let mut new_config = config();
                    new_config.enable_sound = enable_sound();
                    new_config.volume = volume();
                    new_config.auto_start = auto_start();
                    new_config.show_notifications = show_notifications();
                    new_config.theme = theme.read().clone();
                    new_config.last_updated = chrono::Utc::now();
                    let _ = new_config.save();
                    config.set(new_config);
                  },
                }
              }
              // Notifications
              div { class: "flex items-center justify-between",
                div {
                  label { class: format!("{} block", effective_theme.text_secondary()), "Show Notifications" }
                  p { class: format!("{} text-sm", effective_theme.text_tertiary()),
                    "Display system notifications for important events"
                  }
                }                input {
                  r#type: "checkbox",
                  class: "toggle toggle-primary",
                  checked: show_notifications(),
                  onchange: move |evt| {
                    show_notifications.set(evt.value() == "true");
                    // Save config
                    let mut new_config = config();
                    new_config.enable_sound = enable_sound();
                    new_config.volume = volume();
                    new_config.auto_start = auto_start();
                    new_config.show_notifications = show_notifications();
                    new_config.theme = theme.read().clone();
                    new_config.last_updated = chrono::Utc::now();
                    let _ = new_config.save();
                    config.set(new_config);
                  },
                }
              }
            }
          }          // About Section
          div { class: format!("{} p-6 rounded-lg text-left", effective_theme.bg_secondary()),
            h3 { class: format!("text-xl font-semibold {} mb-4 flex items-center gap-2", effective_theme.text_primary()),
              span { "ℹ️" }
              "About"
            }
            div { class: format!("space-y-2 {}", effective_theme.text_secondary()),
              p {
                span { class: "font-semibold", "Mechvibes DX " }
                span { class: effective_theme.text_tertiary(), "v0.1.0" }
              }
              p {
                "A modern mechanical keyboard sound simulator built with Dioxus and Rust."
              }
              p { class: effective_theme.text_primary(),
                "Created with ❤️ by the Mechvibes community"
              }
              // Links
              div { class: "flex gap-4 mt-4",
                button { class: format!("{} {} px-4 py-2 rounded {} text-sm transition-colors",
                    effective_theme.bg_secondary(),
                    effective_theme.text_primary(),
                    effective_theme.bg_hover()),
                  "Check for Updates"
                }
                button { class: format!("{} {} px-4 py-2 rounded {} text-sm transition-colors",
                    effective_theme.bg_tertiary(),
                    effective_theme.text_primary(),
                    effective_theme.bg_hover_secondary()),
                  "View License"
                }
              }
            }
          }
          // Reset Settings
          div { class: format!("{} border {} p-6 rounded-lg text-left",
              effective_theme.bg_secondary(),
              effective_theme.border()),            h3 { class: format!("text-xl font-semibold {} mb-4", effective_theme.text_primary()), "⚠️ Danger Zone" }
            p { class: format!("{} mb-4", effective_theme.text_secondary()),
              "Reset all settings to their default values."
            }            button {
              class: "bg-red-600 hover:bg-red-700 px-4 py-2 rounded text-white text-sm transition-colors",
              onclick: move |_| {
                  // Reset all settings to defaults
                  volume.set(1.0);
                  enable_sound.set(true);
                  auto_start.set(false);
                  show_notifications.set(true);
                  theme.set(Theme::System); // Default is now System

                  // Save config with reset values
                  let mut new_config = config();
                  new_config.volume = 1.0;
                  new_config.enable_sound = true;
                  new_config.auto_start = false;
                  new_config.show_notifications = true;
                  new_config.theme = Theme::System;
                  new_config.last_updated = chrono::Utc::now();
                  let _ = new_config.save();
                  config.set(new_config);
              },
              "Reset to Defaults"
            }
          }
        }
      }
    }
}
