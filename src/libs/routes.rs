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

    // Theme state - use theme context and initialize from config
    let mut theme = use_theme();

    // Adopt the theme the config was saved with.
    //
    // The guard keeps a re-render of this component from cascading. `theme` is
    // a global signal, so this effect re-runs whenever the config signal moves,
    // and it would otherwise write the value it already holds. `Signal::set`
    // has no equality gate and notifies every subscriber regardless, which
    // scheduled a second render here and re-ran every effect subscribed to the
    // theme - including the font/CSS injection in `Header`, i.e. a webview
    // round trip producing the frame already on screen. Only publish a real
    // change.
    use_effect(move || {
        let from_config = config_signal.read().theme.clone();
        if *theme.peek() != from_config {
            theme.set(from_config);
        }
    });

    // Convert theme to DaisyUI theme name
    let daisy_theme = theme().to_daisy_theme();
    crate::debug_print!(
        "🎨 Layout rendering with theme: {:?} -> DaisyUI: {}",
        theme(),
        daisy_theme
    );

    // Get background customization settings (reactive to config changes)
    let background_style = use_memo(move || {
        let config = config_signal.read();
        if config.enable_background_customization {
            let bg_config = &config.background_customization;
            if bg_config.use_image && bg_config.background_image.is_some() {
                // Use background image
                format!(
                    "background: url({}) center center / cover no-repeat;",
                    bg_config.background_image.as_ref().unwrap()
                )
            } else {
                // Use background color
                format!("background: {};", bg_config.background_color)
            }
        } else {
            // Default background (let theme handle it)
            String::new()
        }
    });

    rsx! {
      div {
        class: "h-screen flex flex-col",
        "data-theme": "{daisy_theme}",
        style: "{background_style()}",
        // Custom title bar for window controls
        crate::components::titlebar::TitleBar {}

        // Main content area with padding to account for title bar
        div { class: "flex-1 overflow-auto {crate::utils::spacing::CONTENT_PADDING}",
          // Outlet for nested routes
          Outlet::<Route> {}
        }
        // Dock at the bottom
        crate::components::dock::Dock {}

        // Renders nothing; exists so that reading the route subscribes only
        // this leaf to navigation rather than the whole layout.
        TabLogger {}
      }
    }
}

/// Logs every tab switch with a divider, so console output between two
/// navigations is attributable to the tab that produced it.
///
/// This lives in its own component on purpose. `use_route` subscribes its
/// caller to the router, so reading the route directly in `Layout` re-rendered
/// the entire chrome - title bar, dock and background - on every tab switch.
/// Isolating the read here keeps the log while leaving `Layout` untouched by
/// navigation, so only this leaf and the `Outlet` re-render.
#[component]
fn TabLogger() -> Element {
    let tab_name = match use_route::<Route>() {
        Route::Home {} => "Home",
        Route::Customize {} => "Customize",
        Route::Soundpacks {} => "Soundpacks",
        Route::Mood {} => "Mood",
        Route::Settings {} => "Settings",
    };
    use_effect(use_reactive!(|tab_name| {
        println!("──────────────────────────────────────");
        println!("📍 Tab: {tab_name}");
    }));

    rsx! {}
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
