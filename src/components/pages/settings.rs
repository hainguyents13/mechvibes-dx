use crate::components::ui::PageHeader;
use crate::libs::theme::{use_theme, BuiltInTheme, Theme};
use crate::utils::config_utils::use_config;
use dioxus::prelude::*;
use lucide_dioxus::Settings;

#[component]
pub fn SettingsPage() -> Element {
    // Use shared config hook
    let (config, update_config) = use_config();
    // Use computed signals that always reflect current config state
    let enable_sound = use_memo(move || config().enable_sound);
    let auto_start = use_memo(move || config().auto_start);
    let show_notifications = use_memo(move || config().show_notifications);

    // Theme state - use theme context (initialized in Layout component)
    let mut theme = use_theme();
    rsx! {
      div { class: "p-12 pb-32",
        // Page header
        PageHeader {
          title: "Settings".to_string(),
          subtitle: "Config your MechvibesDX experience.".to_string(),
          icon: Some(rsx! {
            Settings { class: "w-8 h-8 mx-auto" }
          }),
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
              div { class: "space-y-6",
                // Volume Control
                label { class: " cursor-pointer flex items-center justify-between",
                  div {
                    div { class: "", "Enable all sounds" }
                    div { class: "text-base-content/70 text-xs truncate",
                      span { "You can also use " }
                      span { class: "kbd kbd-xs font-mono text-base",
                        "Ctrl+Alt+M"
                      }
                      span { " to toggle sound on/off" }
                    }
                  }

                  input {
                    r#type: "checkbox",
                    class: "toggle toggle-sm toggle-base-100",
                    checked: enable_sound(),
                    onchange: {
                        let update_config = update_config.clone();
                        move |evt: Event<FormData>| {
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
                div { class: "",
                  label { class: " cursor-pointer flex items-center justify-between",
                    div {
                      div { class: "", "Start with Windows" }
                      div { class: "text-xs text-base-content/70 truncate",
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
                div { class: "",
                  label { class: "cursor-pointer flex items-center justify-between",
                    div {
                      div { class: "", "Show Notifications" }
                      div { class: "text-base-content/70 text-xs truncate",
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
                          theme.set(Theme::BuiltIn(BuiltInTheme::System));
                          update_config(
                              Box::new(|config| {
                                  config.volume = 1.0;
                                  config.enable_sound = true;
                                  config.auto_start = false;
                                  config.show_notifications = true;
                                  config.theme = Theme::BuiltIn(BuiltInTheme::System);
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
