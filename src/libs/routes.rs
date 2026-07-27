use dioxus::prelude::*;

use crate::{ libs::theme::use_theme, utils::config::use_config };

#[derive(Clone, Routable, Debug, PartialEq)]
pub enum Route {
    #[layout(Layout)] #[route("/")] Home {},
    #[route("/customize")] Customize {},
    #[route("/soundpacks")] Soundpacks {},
    #[route("/mood")] Mood {},
    #[route("/settings")] Settings {},
}

#[component]
pub fn Layout() -> Element {
    let (config_signal, _set_config) = use_config();
    let nav = navigator();

    // Theme state - use theme context and initialize from config
    let mut theme = use_theme();

    // Initialize theme from config on first load
    use_effect(move || {
        theme.set(config_signal.read().theme.clone());
    });

    // Auto-navigate to Soundpacks > Get Packs when requested from tray
    use_effect(move || {
        spawn(async move {
            loop {
                if crate::libs::tray_service::peek_open_get_packs() {
                    nav.push(crate::libs::routes::Route::Soundpacks {});
                }
                futures_timer::Delay::new(std::time::Duration::from_millis(200)).await;
            }
        });
    });

    // macOS: apply native vibrancy (transparent titlebar + NSVisualEffectView)
    #[cfg(target_os = "macos")]
    crate::libs::macos_vibrancy::use_macos_vibrancy();

    // Convert theme to DaisyUI theme name
    let daisy_theme = theme().to_daisy_theme();

    // Get background customization settings (reactive to config changes)
    let background_style = use_memo(move || {
        let config = config_signal.read();
        if config.enable_background_customization {
            let bg_config = &config.background_customization;
            if bg_config.use_image && bg_config.background_image.is_some() {
                format!(
                    "background: url({}) center center / cover no-repeat;",
                    bg_config.background_image.as_ref().unwrap()
                )
            } else {
                format!("background: {};", bg_config.background_color)
            }
        } else {
            String::new()
        }
    });

    rsx! {
      div {
        class: "h-screen flex flex-col",
        "data-theme": "{daisy_theme}",
        style: "{background_style()}",

        // Custom draggable titlebar (alongside native traffic lights)
        crate::components::titlebar::TitleBar {}

        // macOS privacy banners — Accessibility alone is not enough for letter/number
        // keys in other apps; Input Monitoring is required for the packaged .app.
        if !crate::libs::input_manager::check_accessibility_permissions() {
          div {
            class: "bg-error text-error-content text-center py-1.5 text-xs font-semibold px-4 flex items-center justify-center gap-4 border-b border-error/20 z-50",
            span { "⚠️ Accessibility is required for keyboard sounds." }
            button {
              class: "btn btn-xs btn-neutral btn-ghost border border-base-content/20 text-xs px-2 py-0.5 min-h-0 h-auto",
              onclick: move |_| {
                crate::libs::input_manager::open_accessibility_settings();
              },
              "Open Accessibility"
            }
          }
        } else if !crate::libs::input_manager::check_input_monitoring_permissions() {
          div {
            class: "bg-warning text-warning-content text-center py-1.5 text-xs font-semibold px-4 flex items-center justify-center gap-4 border-b border-warning/20 z-50",
            span {
              "⚠️ Input Monitoring required for letter/number keys in other apps. \
               (Special keys may still work.) Enable MechvibesDX, then quit & reopen."
            }
            button {
              class: "btn btn-xs btn-neutral btn-ghost border border-base-content/20 text-xs px-2 py-0.5 min-h-0 h-auto",
              onclick: move |_| {
                crate::libs::input_manager::open_input_monitoring_settings();
              },
              "Open Input Monitoring"
            }
          }
        }

        // Main content area with padding for title bar + dock
        div { class: "flex-1 overflow-auto {crate::utils::spacing::CONTENT_PADDING}",
          Outlet::<Route> {}
        }
        // Glass dock at the bottom
        crate::components::dock::Dock {}
      }
    }
}

#[component]
pub fn Home() -> Element {
    use crate::libs::AudioContext;
    use std::sync::Arc;

    // Use audio context from the layout provider instead of creating new one
    let audio_context: Arc<AudioContext> = use_context();
    rsx! {
      crate::components::pages::HomePage { audio_ctx: audio_context }
    }
}

#[component]
pub fn Soundpacks() -> Element {
    rsx! {
      crate::components::pages::Soundpacks {}
    }
}

#[component]
pub fn Mood() -> Element {
    rsx! {
      crate::components::pages::MoodPage {}
    }
}

#[component]
pub fn Customize() -> Element {
    rsx! {
      crate::components::pages::CustomizePage {}
    }
}

#[component]
pub fn Settings() -> Element {
    rsx! {
      crate::components::pages::SettingsPage {}
    }
}
