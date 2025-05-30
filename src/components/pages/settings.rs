use crate::libs::theme::{use_theme, Theme};
use crate::state::config_utils::use_config;
use dioxus::prelude::*;

#[component]
pub fn SettingsPage() -> Element {
    // Use shared config hook
    let (config, update_config) = use_config();
    let mut enable_sound = use_signal(|| config().enable_sound);
    let mut auto_start = use_signal(|| config().auto_start);
    let mut show_notifications = use_signal(|| config().show_notifications);

    // Theme state - use theme context (initialized in Layout component)
    let mut theme = use_theme();
    rsx! {
      div { class: "p-12 pb-32",
        // Page header
        div { class: "text-center mb-8",
          h1 { class: "text-4xl font-bold mb-4 text-base-content", "Settings" }
          p { class: "text-lg text-base-content", "Customize your MechvibesDX experience." }
        }
        // Settings sections
        div { class: "space-y-4",
          // General Settings Section
          div { class: "collapse collapse-arrow border border-base-300 bg-base-200 text-base-content",
            input {
              r#type: "radio",
              name: "setting-accordion",
              checked: true,
            }
            div { class: "collapse-title font-semibold", "General" }
            div { class: "collapse-content text-sm",
              div { class: "form-control",
                div { class: "space-y-6",
                  // Theme Settings
                  div { class: "form-control",
                    div { class: "label",
                      span { class: "label-text text-base", "Theme" }
                    }
                    div { class: "mt-2",
                      crate::components::theme_toggler::ThemeToggler {}
                    }
                  }
                  label { class: "label cursor-pointer flex items-center justify-between",
                    div {
                      div { class: "label-text text-base", "Enable all sounds" }
                      div { class: "label-text-alt text-xs truncate",
                        "Turn all keyboard sounds on or off"
                      }
                    }
                    input {
                      r#type: "checkbox",
                      class: "toggle toggle-sm toggle-base-100",
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
                  }
                  // Auto Start
                  div { class: "form-control",
                    label { class: "label cursor-pointer flex items-center justify-between",
                      div {
                        div { class: "label-text text-base",
                          "Start with Windows"
                        }
                        div { class: "label-text-alt text-xs truncate",
                          "Automatically start MechvibesDX when Windows boots"
                        }
                      }
                      input {
                        r#type: "checkbox",
                        class: "toggle toggle-sm toggle-base-100",
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
                    label { class: "label cursor-pointer flex items-center justify-between",
                      div {
                        div { class: "label-text text-base",
                          "Show Notifications"
                        }
                        div { class: "label-text-alt",
                          "Display system notifications for important events"
                        }
                      }
                      input {
                        r#type: "checkbox",
                        class: "toggle toggle-sm toggle-base-100",
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
          }
          // App info Section
          div { class: "collapse collapse-arrow border border-base-300 bg-base-200 text-base-content",
            input { r#type: "radio", name: "setting-accordion" }
            div { class: "collapse-title font-semibold", "App info" }
            div { class: "collapse-content text-sm",
              crate::components::app_info::AppInfoDisplay {}
            }
          }
          // Danger Zone Section
          // Reset Settings
          div { class: "collapse collapse-arrow border border-base-300 bg-base-200",
            input { r#type: "radio", name: "setting-accordion" }
            div { class: "collapse-title font-semibold  text-error", "Danger zone" }
            div { class: "collapse-content text-sm",
              p { class: "mb-4 text-base-content/70",
                "Reset all settings to their default values. This action cannot be undone."
              }
              div { class: " justify-start",
                button {
                  class: "btn btn-error btn-soft btn-sm",
                  onclick: {
                      let update_config = update_config.clone();
                      move |_| {
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
