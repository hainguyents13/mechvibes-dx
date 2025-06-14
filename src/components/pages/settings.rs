use crate::components::ui::{ Collapse, PageHeader, Toggler };
use crate::components::device_selector::AudioOutputSelector;
use crate::libs::theme::{ use_theme, BuiltInTheme, Theme };
use crate::libs::tray_service::request_tray_update;
use crate::utils::config::use_config;
use crate::utils::constants::{ APP_NAME_DISPLAY, APP_NAME };
use dioxus::prelude::*;
use lucide_dioxus::Settings;

#[component]
pub fn SettingsPage() -> Element {
    // Use shared config hook
    let (config, update_config) = use_config(); // Use computed signals that always reflect current config state
    let enable_sound = use_memo(move || config().enable_sound);
    let auto_start = use_memo(move || config().auto_start);
    let start_minimized = use_memo(move || config().start_minimized);
    let show_notifications = use_memo(move || config().show_notifications);

    // Theme state - use theme context (initialized in Layout component)
    let mut theme = use_theme();
    rsx! {
      div { class: "p-12", // Page header
        PageHeader {
          title: "Settings".to_string(),
          subtitle: format!("Config your {} experience.", APP_NAME_DISPLAY),
          icon: Some(rsx! {
            Settings { class: "w-8 h-8 mx-auto" }
          }),
        }

        // Settings sections
        div { class: "space-y-4",
          // General Settings Section
          Collapse {
            title: "General".to_string(),
            group_name: "setting-accordion".to_string(),
            default_open: true,
            content_class: "collapse-content text-sm",
            children: rsx! {
              div { class: "space-y-6", // Volume Control
                Toggler {
                  title: "Enable all sounds".to_string(),
                  description: Some("You can also use Ctrl+Alt+M to toggle sound on/off".to_string()),
                  checked: enable_sound(),
                  on_change: {
                      let update_config = update_config.clone();
                      move |new_value: bool| {
                          update_config(
                              Box::new(move |config| {
                                  config.enable_sound = new_value;
                              }),
                          );
                          request_tray_update();
                      }
                  },
                }
                // Auto Start
                Toggler {
                  title: "Start with Windows".to_string(),
                  description: Some(format!("Automatically start {} when Windows boots", APP_NAME)),
                  checked: auto_start(),
                  on_change: {
                      let update_config = update_config.clone();
                      move |new_value: bool| {
                          update_config(
                              Box::new(move |config| {
                                  config.auto_start = new_value;
                              }),
                          );
                          spawn(async move {
                              match crate::utils::auto_startup::set_auto_startup(new_value) {
                                  Ok(_) => {
                                      let status = if new_value { "enabled" } else { "disabled" };
                                      println!("✅ Auto startup {}", status);
                                  }
                                  Err(e) => {
                                      eprintln!("❌ Failed to set auto startup: {}", e);
                                  }
                              }
                          });
                      }
                  },
                }
                // Start Minimized (only show when auto start is enabled)
                if auto_start() {
                  Toggler {
                    title: "Start minimized to tray".to_string(),
                    description: Some("When starting with Windows, open minimized to system tray".to_string()),
                    checked: start_minimized(),
                    on_change: {
                        let update_config = update_config.clone();
                        move |new_value: bool| {
                            update_config(
                                Box::new(move |config| {
                                    config.start_minimized = new_value;
                                }),
                            );
                            spawn(async move {
                                if crate::state::config::AppConfig::load().auto_start {
                                    match crate::utils::auto_startup::set_auto_startup(true) {
                                        Ok(_) => {
                                            let status = if new_value {
                                                "with minimized flag"
                                            } else {
                                                "without minimized flag"
                                            };
                                            println!("✅ Auto startup updated {}", status);
                                        }
                                        Err(e) => {
                                            eprintln!("❌ Failed to update auto startup: {}", e);
                                        }
                                    }
                                }
                            });
                        }
                    },
                  }
                }               
                // Notifications
                Toggler {
                  title: "Show notifications".to_string(),
                  description: Some("Display system notifications for important events".to_string()),
                  checked: show_notifications(),
                  on_change: {
                      let update_config = update_config.clone();
                      move |new_value: bool| {
                          update_config(
                              Box::new(move |config| {
                                  config.show_notifications = new_value;
                              }),
                          );
                      }
                  },
                }
              }
            },        
          }          
          // Devices Section
          Collapse {
            title: "Devices (Experimental)".to_string(),
            group_name: "setting-accordion".to_string(),
            content_class: "collapse-content text-sm",           
            children: rsx! {
              div { class: "space-y-2",
                // Audio Output Device
                AudioOutputSelector {}
                
                // Device Information
                div { 
                  div { class: "text-sm font-semibold mb-2", "Device Information" }
                  div { class: "text-xs text-base-content/70 space-y-1",
                    p { "• Audio output devices control where soundpack audio is played" }
                    p { "• Restart the application for changes to take effect" }
                    p { "• If a device becomes unavailable, the system will fall back gracefully" }
                  }
                }
              }
            },
          }
          // App info Section
          Collapse {
            title: "App info".to_string(),
            group_name: "setting-accordion".to_string(),
            content_class: "collapse-content text-sm",
            children: rsx! {
              crate::components::app_info::AppInfoDisplay {}
            },
          }
          // Danger Zone Section
          Collapse {
            title: "Danger zone".to_string(),
            group_name: "setting-accordion".to_string(),
            title_class: "collapse-title font-semibold text-error",
            variant: "border border-base-300 bg-base-200",
            content_class: "collapse-content text-sm",
            children: rsx! {
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
            },
          }
        }
      }
    }
}
