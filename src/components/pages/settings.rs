use crate::components::ui::{ Collapse, PageHeader, Toggler };
use crate::components::device_selector::AudioOutputSelector;
use crate::libs::AudioContext;
use crate::libs::theme::{ use_theme, BuiltInTheme, Theme };
use crate::libs::tray_service::request_tray_update;
use crate::utils::config::use_config;
use crate::utils::constants::{ APP_NAME_DISPLAY, APP_NAME };
use crate::utils::auto_updater::{ check_for_updates_simple, UpdateInfo };
use crate::state::app::use_update_info_setter;
use crate::utils::time::format_relative_time;
use dioxus::prelude::*;
use lucide_dioxus::{ PartyPopper, Settings };
use std::sync::Arc;

#[component]
pub fn SettingsPage() -> Element {
    // Use shared config hook
    let (config, update_config) = use_config();

    // Mute/volume changes must reach the audio engine thread, which caches
    // these flags in its own state - a config write alone does not move them.
    let audio_ctx = use_context::<Arc<AudioContext>>();

    // Use computed signals that always reflect current config state
    let enable_sound = use_memo(move || config().enable_sound);
    let enable_volume_boost = use_memo(move || config().enable_volume_boost);
    let auto_start = use_memo(move || config().auto_start);
    let start_minimized = use_memo(move || config().start_minimized);
    let auto_update_config = use_memo(move || config().auto_update.clone());

    // Update states
    let mut update_info = use_signal(|| None::<UpdateInfo>);
    let mut is_checking_updates = use_signal(|| false);
    let mut check_error = use_signal(|| None::<String>);
    let update_info_setter = use_update_info_setter();

    // Load saved update info on component mount
    use_effect(move || {
        if let Some(saved_update) = crate::utils::auto_updater::get_saved_update_info() {
            update_info.set(Some(saved_update));
        }
    });

    // Get current version for display
    let current_version = crate::utils::constants::APP_VERSION;

    // Theme state - use theme context (initialized in Layout component)
    let mut theme = use_theme();
    rsx! {
      div { class: "", // Page header
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
              div { class: "{crate::utils::spacing::SECTION_SPACING_LG}",
                // Volume Control
                Toggler {
                  title: "Enable all sounds".to_string(),
                  description: Some("You can also use Ctrl+Alt+M to toggle sound on/off".to_string()),
                  checked: enable_sound(),
                  on_change: {
                      let update_config = update_config.clone();
                      let audio_ctx = audio_ctx.clone();
                      move |new_value: bool| {
                          // The engine caches this flag; a config write alone
                          // would leave it playing until restart. Go through
                          // the audio context so the engine is notified too.
                          audio_ctx.set_sound_enabled(new_value);
                          update_config(
                              Box::new(move |config| {
                                  config.enable_sound = new_value;
                              }),
                          );
                          request_tray_update();
                      }
                  },
                }
                // Volume Boost
                Toggler {
                  title: "Volume boost (200% max)".to_string(),
                  description: Some(
                      "Allow volume sliders to go up to 200%. May cause audio distortion at high levels."
                          .to_string(),
                  ),
                  checked: enable_volume_boost(),
                  on_change: {
                      let update_config = update_config.clone();
                      move |new_value: bool| {
                          update_config(
                              Box::new(move |config| {
                                  config.enable_volume_boost = new_value;
                                  if !new_value {
                                      if config.volume > 1.0 {
                                          config.volume = 1.0;
                                      }
                                      if config.mouse_volume > 1.0 {
                                          config.mouse_volume = 1.0;
                                      }
                                  }
                              }),
                          );
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
              }
            },
          }
          // Devices Section
          Collapse {
            title: "Devices".to_string(),
            group_name: "setting-accordion".to_string(),
            content_class: "collapse-content text-sm",
            children: rsx! {
              div { class: "space-y-2",
                // Audio Output Device
                AudioOutputSelector {}
                // Device Information
                div { class: "text-xs mt-2 text-base-content/70",
                  "Device changes apply immediately — no restart needed. If audio ever stops working (e.g. after unplugging a device), restarting the app can help."
                }
              }
            },
          }
          // Auto-Update Section
          Collapse {
            title: "Updates".to_string(),
            group_name: "setting-accordion".to_string(),
            content_class: "collapse-content text-sm",
            children: rsx! {
              div { class: "space-y-4",
                p { class: "text-sm text-base-content/70",
                  if crate::utils::update_installer::silent_update_supported() {
                    "Update checking runs every 24 hours. Nothing is downloaded until you choose to install."
                  } else {
                    "Update checking runs every 24 hours in the background. Updates are installed manually from the releases page."
                  }
                }
                div { class: "flex items-center gap-3",
                  button {
                    class: "btn btn-soft btn-sm",
                    disabled: is_checking_updates(),
                    onclick: move |_| {
                        println!("Manual update check requested");
                        is_checking_updates.set(true);
                        check_error.set(None);
                        let mut update_info = update_info.clone();
                        let mut is_checking_updates = is_checking_updates.clone();
                        let mut check_error = check_error.clone();
                        let update_info_setter = update_info_setter.clone();
                        spawn(async move {
                            match check_for_updates_simple().await {
                                Ok(info) => {
                                    update_info.set(Some(info.clone()));
                                    let mut app_config = crate::state::config::AppConfig::load();
                                    if info.update_available {
                                        app_config.auto_update.available_version = Some(
                                            info.latest_version.clone(),
                                        );
                                        app_config.auto_update.available_download_url = info
                                            .download_url
                                            .clone();
                                    } else {
                                        app_config.auto_update.available_version = None;
                                        app_config.auto_update.available_download_url = None;
                                    }
                                    app_config.auto_update.last_check = Some(
                                        std::time::SystemTime::now()
                                            .duration_since(std::time::SystemTime::UNIX_EPOCH)
                                            .unwrap_or_default()
                                            .as_secs(),
                                    );
                                    let _ = app_config.save();
                                    if info.update_available {
                                        update_info_setter.call(Some(info));
                                    } else {
                                        update_info_setter.call(None);
                                    }
                                    check_error.set(None);
                                }
                                Err(e) => {
                                    check_error.set(Some(format!("Failed to check for updates: {}", e)));
                                    update_info.set(None);
                                    update_info_setter.call(None);
                                }
                            }
                            is_checking_updates.set(false);
                        });
                    },
                    if is_checking_updates() {
                      span { class: "loading loading-spinner loading-xs mr-1" }
                    }
                    if is_checking_updates() {
                      "Checking..."
                    } else {
                      "Check for Updates"
                    }
                  } // Display last check time
                  if let Some(last_check) = auto_update_config().last_check {
                    div { class: "text-xs text-base-content/60",
                      "Last checked: {format_relative_time(last_check)}"
                    }
                  } else {
                    div { class: "text-xs text-base-content/60", "Never checked" }
                  }
                }
                // Display update status
                if let Some(error) = check_error() {
                  div { class: "alert alert-error text-sm", "❌ {error}" }
                } else if let Some(info) = update_info() {
                  if info.update_available {
                    div { class: "alert alert-success alert-soft text-sm",
                      PartyPopper { class: "w-6 h-6 mr-2" }
                      div {
                        p { "Update available: v{info.latest_version}" }
                        div { class: "mt-2 space-y-2 text-sm",
                          // Owns the whole download -> verify -> restart
                          // sequence, including its own error states.
                          crate::components::update_button::UpdateInstallButton {
                            info: info.clone(),
                          }
                          if let Some(release_url) = &info.release_notes {
                            a {
                              href: "{release_url}",
                              target: "_blank",
                              class: "link link-neutral link-hover",
                              "View release notes"
                            }
                          }
                        }
                      }
                    }
                  } else {
                    div { class: "alert alert-success alert-soft ",
                      "You're running the latest version (v{info.current_version})"
                    }
                  }
                } else {
                  // A saved update from a previous session. Read through
                  // get_saved_update_info(), never straight off the config:
                  // it drops any entry that is not actually newer than the
                  // running build. Reading the raw field here is what once
                  // made an upgraded app advertise an update to itself.
                  if let Some(saved) = crate::utils::auto_updater::get_saved_update_info() {
                    div { class: "alert alert-success text-sm",
                      div {
                        p { "🎉 Update available: v{saved.latest_version}" }
                        div { class: "mt-2 space-y-2",
                          crate::components::update_button::UpdateInstallButton {
                            info: saved.clone(),
                          }
                          a {
                            href: "https://github.com/hainguyents13/mechvibes-dx/releases/tag/v{saved.latest_version}",
                            target: "_blank",
                            class: "link link-secondary",
                            "View Release Notes"
                          }
                        }
                        p { class: "text-xs text-base-content/60 mt-1",
                          "Current version: v{current_version}"
                        }
                      }
                    }
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
                      let audio_ctx = audio_ctx.clone();
                      move |_| {
                          theme.set(Theme::BuiltIn(BuiltInTheme::System));
                          // Push the audio-affecting defaults through the
                          // engine as well, otherwise a reset would restore
                          // the config but leave the running engine muted /
                          // at the old volume until the next restart.
                          audio_ctx.set_sound_enabled(true);
                          audio_ctx.set_volume(1.0);
                          update_config(
                              Box::new(|config| {
                                  config.volume = 1.0;
                                  config.enable_sound = true;
                                  config.enable_volume_boost = false;
                                  config.auto_start = false;
                                  config.theme = Theme::BuiltIn(BuiltInTheme::System);
                              }),
                          );
                          request_tray_update();
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
