use crate::libs::theme::{use_theme, Theme};
use crate::state::app::use_app_state;
use crate::state::config_utils::use_config;
use dioxus::prelude::*;
use std::sync::Arc;

#[component]
pub fn SettingsPage() -> Element {
    // Get access to audio context for reloading soundpacks
    let audio_ctx: Arc<crate::libs::audio::AudioContext> = use_context();
    // Use shared config hook
    let (config, update_config) = use_config();
    let mut enable_sound = use_signal(|| config().enable_sound);
    let mut auto_start = use_signal(|| config().auto_start);
    let mut show_notifications = use_signal(|| config().show_notifications);
    // Get access to app state for soundpack operations
    let app_state = use_app_state();
    // UI state for notification and loading
    let mut refreshing_soundpacks = use_signal(|| false);
    let mut refresh_soundpacks_cache = move || {
        println!("🔄 Refreshing soundpack cache from settings page...");

        // Set loading state to true - directly update the signal
        refreshing_soundpacks.set(true);
        println!("🌻 Loading state set to: {}", refreshing_soundpacks());

        // Clone necessary variables for the async task
        let mut refreshing_signal = refreshing_soundpacks.clone();
        let audio_ctx_clone = audio_ctx.clone();
        let mut app_state_clone = app_state.clone();

        // Perform the refresh operation in a separate task to not block the UI
        spawn(async move {
            // Use async sleep instead of std::thread::sleep
            use futures_timer::Delay;
            use std::time::Duration;

            // Add a small artificial delay to make loading state more visible
            Delay::new(Duration::from_millis(800)).await;

            println!("Starting soundpack refresh operation...");

            // This will create a new SoundpackCache instance that will refresh from directory
            let mut fresh_cache = crate::state::soundpack_cache::SoundpackCache::load();
            fresh_cache.refresh_from_directory();
            fresh_cache.save();

            // Update the app state with new cache
            app_state_clone.write().optimized_cache = Arc::new(fresh_cache);

            // Reload current soundpacks to apply any changes
            crate::state::app::reload_current_soundpacks(&audio_ctx_clone);

            // Add another small delay before changing the loading state back
            Delay::new(Duration::from_millis(200)).await;

            // Reset loading state
            refreshing_signal.set(false);
            println!("🌻 Loading state reset to: {}", refreshing_signal());

            println!("✅ Soundpack refresh complete");
        });
    };

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
          // Soundpack Management Section
          div { class: "collapse collapse-arrow border border-base-300 bg-base-200 text-base-content",
            input { r#type: "radio", name: "setting-accordion" }
            div { class: "collapse-title font-semibold", "Soundpack management" }
            div { class: "collapse-content text-sm",
              div { class: "space-y-4 mt-2",
                p { class: "text-base-content/70",
                  "Refresh soundpack list to detect newly added or removed soundpacks."
                }
                div { class: "flex flex-col gap-2",
                  div { class: "flex items-center gap-4",
                    button {
                      class: "btn btn-soft btn-sm",
                      onclick: move |_| {
                          refresh_soundpacks_cache();
                      },
                      disabled: refreshing_soundpacks(),
                      if refreshing_soundpacks() {
                        span { class: "loading loading-spinner loading-xs mr-2" }
                        "Refreshing..."
                      } else {
                        "Refresh soundpacks"
                      }
                    }
                    // Last scan info
                    if app_state.read().optimized_cache.last_scan > 0 {
                      div { class: "text-xs text-base-content/60",
                        "Last scan "
                        {
                            let last_scan = app_state.read().optimized_cache.last_scan;
                            let now = std::time::SystemTime::now()
                                .duration_since(std::time::SystemTime::UNIX_EPOCH)
                                .unwrap_or_default()
                                .as_secs();
                            let diff = now.saturating_sub(last_scan);
                            if diff < 60 {
                                ": just now".to_string()
                            } else if diff < 3600 {
                                format!("{} min ago", diff / 60)
                            } else if diff < 86400 {
                                format!("{} hr ago", diff / 3600)
                            } else {
                                format!("{} days ago", diff / 86400)
                            }
                        }
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
