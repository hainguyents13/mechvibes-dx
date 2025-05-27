use crate::libs::audio::AudioContext;
use crate::libs::theme::{use_theme, Theme};
use crate::state::config_utils::use_config;
use dioxus::prelude::*;
use std::sync::Arc;

#[component]
pub fn SettingsPage(audio_ctx: Arc<AudioContext>) -> Element {
    // Use shared config hook
    let (config, update_config) = use_config();

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
                    onchange: {
                        let update_config = update_config.clone();
                        move |evt: Event<FormData>| {
                            enable_sound.set(evt.value() == "true");
                            update_config(
                                Box::new(move |config| {
                                    config.enable_sound = evt.value() == "true";
                                }),
                            );
                        }
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
                      onclick: {
                          let update_config = update_config.clone();
                          move |_| {
                              theme.set(Theme::Dark);
                              update_config(
                                  Box::new(|config| {
                                      config.theme = Theme::Dark;
                                  }),
                              );
                          }
                      },
                      "🌙 Dark"
                    }
                    button {
                      class: if matches!(*theme.read(), Theme::Light) { "btn btn-primary flex-1" } else { "btn btn-outline flex-1" },
                      onclick: {
                          let update_config = update_config.clone();
                          move |_| {
                              theme.set(Theme::Light);
                              update_config(
                                  Box::new(|config| {
                                      config.theme = Theme::Light;
                                  }),
                              );
                          }
                      },
                      "☀️ Light"
                    }
                    button {
                      class: if matches!(*theme.read(), Theme::System) { "btn btn-primary flex-1" } else { "btn btn-outline flex-1" },
                      onclick: {
                          let update_config = update_config.clone();
                          move |_| {
                              theme.set(Theme::System);
                              update_config(
                                  Box::new(|config| {
                                      config.theme = Theme::System;
                                  }),
                              );
                          }
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
                      onchange: {
                          let update_config = update_config.clone();
                          move |evt: Event<FormData>| {
                              auto_start.set(evt.value() == "true");
                              update_config(
                                  Box::new(move |config| {
                                      config.auto_start = evt.value() == "true";
                                  }),
                              );
                          }
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
                      onchange: {
                          let update_config = update_config.clone();
                          move |evt: Event<FormData>| {
                              show_notifications.set(evt.value() == "true");
                              update_config(
                                  Box::new(move |config| {
                                      config.show_notifications = evt.value() == "true";
                                  }),
                              );
                          }
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
                  onclick: {
                      let update_config = update_config.clone();
                      move |_| {
                          volume.set(1.0);
                          enable_sound.set(true);
                          auto_start.set(false);
                          show_notifications.set(true);
                          theme.set(Theme::System);
                          update_config(
                              Box::new(|config| {
                                  config.volume = 1.0;
                                  config.enable_sound = true;
                                  config.auto_start = false;
                                  config.show_notifications = true;
                                  config.theme = Theme::System;
                              }),
                          );
                      }
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
