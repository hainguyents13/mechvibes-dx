use crate::libs::theme::{use_theme, Theme};
use crate::state::config::AppConfig;
use crate::state::config_utils::use_config;
use dioxus::prelude::*;

#[component]
pub fn ThemeToggler() -> Element {
    // Get the config and update_config function
    let (_, update_config) = use_config();

    // Theme state - use theme context
    let mut theme = use_theme();

    rsx! {
      div { class: "flex items-center justify-between gap-2 w-full",
        button {
          class: if matches!(*theme.read(), Theme::Dark) { "btn btn-neutral flex-1" } else { "btn btn-soft flex-1" },
          onclick: {
              let update_fn = update_config.clone();
              move |_| {
                  theme.set(Theme::Dark);
                  update_fn(
                      Box::new(|config: &mut AppConfig| {
                          config.theme = Theme::Dark;
                      }),
                  );
              }
          },
          "🌙 Dark"
        }
        button {
          class: if matches!(*theme.read(), Theme::Light) { "btn btn-neutral flex-1" } else { "btn btn-soft flex-1" },
          onclick: {
              let update_fn = update_config.clone();
              move |_| {
                  theme.set(Theme::Light);
                  update_fn(
                      Box::new(|config: &mut AppConfig| {
                          config.theme = Theme::Light;
                      }),
                  );
              }
          },
          "☀️ Light"
        }
        button {
          class: if matches!(*theme.read(), Theme::System) { "btn btn-neutral flex-1" } else { "btn btn-soft flex-1" },
          onclick: {
              let update_fn = update_config.clone();
              move |_| {
                  theme.set(Theme::System);
                  update_fn(
                      Box::new(|config: &mut AppConfig| {
                          config.theme = Theme::System;
                      }),
                  );
              }
          },
          "🖥️ System"
        }
      }
    }
}
