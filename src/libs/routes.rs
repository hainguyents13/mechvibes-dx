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

    // Report the theme only when it actually moves. Navigating remounts this
    // component (the router builds a separate `rsx! { Layout {} }` per route,
    // so each carries its own template identity and gets replaced rather than
    // diffed), which re-runs this line with an unchanged theme and made the
    // console read as though every tab switch re-themed the app.
    {
        use std::sync::Mutex;
        static LAST_LOGGED: Mutex<Option<String>> = Mutex::new(None);
        let mut last = LAST_LOGGED.lock().unwrap_or_else(|e| e.into_inner());
        if last.as_deref() != Some(daisy_theme.as_str()) {
            *last = Some(daisy_theme.clone());
            crate::debug_print!(
                "🎨 Theme applied: {:?} -> DaisyUI: {}",
                theme(),
                daisy_theme
            );
        }
    }

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
      }
    }
}

/// Navigate to `route`, logging a divider and the destination first.
///
/// The log has to happen here rather than in a component, because it must come
/// out *before* the render work the navigation triggers. Rendering is where
/// every other line in the console originates, and a component - even one
/// placed ahead of the `Outlet` in the layout's `rsx!` - is not guaranteed to
/// run before its siblings: sibling order in `rsx!` is not render order, and a
/// log written from `use_effect` lands after the whole pass. Emitting at the
/// call site is the only point that provably precedes all of it, and it covers
/// every navigation because it is the one way the app changes route.
pub fn navigate(route: &'static str) {
    println!("──────────────────────────────────────");
    println!("📍 Tab: {}", tab_name_for(route));
    navigator().push(route);
}

/// The human-readable name for a route path, falling back to the path itself.
fn tab_name_for(route: &str) -> &str {
    match route {
        "/" => "Home",
        "/customize" => "Customize",
        "/soundpacks" => "Soundpacks",
        "/mood" => "Mood",
        "/settings" => "Settings",
        other => other,
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    /// Every path `navigate` is called with must parse back to a real route and
    /// must produce a tab name rather than falling through to the raw path.
    ///
    /// `navigate` takes a `&str` because it is called from `rsx!` event
    /// handlers, so a typo would otherwise be a silent 404 at runtime with a
    /// divider logged for a tab the user never reached.
    #[test]
    fn every_navigable_path_maps_to_a_route_and_a_tab_name() {
        let expected = [
            ("/", "Home"),
            ("/customize", "Customize"),
            ("/soundpacks", "Soundpacks"),
            ("/mood", "Mood"),
            ("/settings", "Settings"),
        ];

        for (path, tab) in expected {
            assert!(
                Route::from_str(path).is_ok(),
                "{path} is passed to navigate() but does not parse as a route"
            );
            assert_eq!(tab_name_for(path), tab, "wrong tab name logged for {path}");
        }
    }

    /// The five paths above are the whole routing table; if a route is added
    /// without teaching `navigate` about it, the divider would log a raw URL.
    #[test]
    fn the_tab_name_table_covers_every_route() {
        for route in [
            Route::Home {},
            Route::Customize {},
            Route::Soundpacks {},
            Route::Mood {},
            Route::Settings {},
        ] {
            let path = route.to_string();
            let name = tab_name_for(&path);
            assert!(
                !name.starts_with('/'),
                "{path} has no tab name - navigate() would log the raw path"
            );
        }
    }
}
