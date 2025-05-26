use crate::libs::audio::AudioContext;
use crate::libs::theme::{use_theme, Theme};
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
      div { class: "container mx-auto p-8 mb-16",
        // Page header
        div { class: "text-center mb-8",
          h1 { class: "text-4xl font-bold mb-4", "Settings" }
          p { class: "text-lg opacity-70", "Customize your Mechvibes DX experience." }
        }
        // Settings sections
        div { class: "space-y-4",
          div { class: "collapse collapse-arrow bg-base-100 border border-primary",
            input {
              r#type: "radio",
              name: "setting-accodion",
              checked: true,
            }
            div { class: "collapse-title font-semibold", "General" }
            div { class: "collapse-content text-sm",
              div { class: "form-control",
                label { class: "label cursor-pointer",
                  input {
                    r#type: "checkbox",
                    class: "toggle toggle-primary",
                    checked: enable_sound(),
                    onchange: move |evt| {
                        enable_sound.set(evt.value() == "true");
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
                  span { class: "label-text text-base", "Enable all sounds" }
                }
              }
            }
          }
          div { class: "collapse collapse-arrow bg-base-100 border border-primary",
            input { r#type: "radio", name: "setting-accodion" }
            div { class: "collapse-title font-semibold", "Customize" }
            div { class: "collapse-content text-sm",
              div { class: "space-y-6",
                // Theme Settings
                div { class: "form-control",
                  label { class: "label",
                    span { class: "label-text text-base", "Theme" }
                  }
                  div { class: "label",
                    span { class: "label-text-alt opacity-70",
                      "Choose your preferred color scheme"
                    }
                  }
                  div { class: "btn-group btn-group-horizontal w-full mt-2",
                    button {
                      class: if matches!(*theme.read(), Theme::Dark) { "btn btn-primary flex-1" } else { "btn btn-outline flex-1" },
                      onclick: move |_| {
                          theme.set(Theme::Dark);
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
                    }
                    button {
                      class: if matches!(*theme.read(), Theme::Light) { "btn btn-primary flex-1" } else { "btn btn-outline flex-1" },
                      onclick: move |_| {
                          theme.set(Theme::Light);
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
                    }
                    button {
                      class: if matches!(*theme.read(), Theme::System) { "btn btn-primary flex-1" } else { "btn btn-outline flex-1" },
                      onclick: move |_| {
                          theme.set(Theme::System);
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
                }
                // Auto Start
                div { class: "form-control",
                  label { class: "label cursor-pointer",
                    div {
                      div { class: "label-text text-base", "Start with Windows" }
                      div { class: "label-text-alt opacity-70",
                        "Automatically start Mechvibes DX when Windows boots"
                      }
                    }
                    input {
                      r#type: "checkbox",
                      class: "toggle toggle-primary",
                      checked: auto_start(),
                      onchange: move |evt| {
                          auto_start.set(evt.value() == "true");
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
                // Notifications
                div { class: "form-control",
                  label { class: "label cursor-pointer",
                    div {
                      div { class: "label-text text-base", "Show Notifications" }
                      div { class: "label-text-alt opacity-70",
                        "Display system notifications for important events"
                      }
                    }
                    input {
                      r#type: "checkbox",
                      class: "toggle toggle-primary",
                      checked: show_notifications(),
                      onchange: move |evt| {
                          show_notifications.set(evt.value() == "true");
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
              }
            }
          }
          // App Info Section as DaisyUI collapse
          div { class: "collapse collapse-arrow bg-base-100 border border-primary",
            input {
              r#type: "checkbox",
              name: "setting-accordion-appinfo",
            }
            div { class: "collapse-title font-semibold", "App Info" }
            div { class: "collapse-content text-sm",
              crate::components::app_info::AppInfoDisplay {}
            }
          }
          div { class: "collapse collapse-arrow bg-base-100 border border-red-500",
            input { r#type: "radio", name: "setting-accodion" }
            div { class: "collapse-title font-semibold", "Danger zone" }
            div { class: "collapse-content text-sm",
              p { class: "opacity-80 mb-4",
                "Reset all settings to their default values. This action cannot be undone."
              }
              div { class: " justify-start",
                button {
                  class: "btn btn-error btn-sm",
                  onclick: move |_| {
                      volume.set(1.0);
                      enable_sound.set(true);
                      auto_start.set(false);
                      show_notifications.set(true);
                      theme.set(Theme::System);
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
    }
}
