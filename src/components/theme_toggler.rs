use crate::libs::theme::{use_theme, Theme};
use crate::state::config::AppConfig;
use crate::state::config_utils::use_config;
use dioxus::prelude::*;
use lucide_dioxus::{Computer, Moon, Palette, Pencil, Plus, Sun, Trash2};

#[component]
pub fn ThemeToggler() -> Element {
    // Get the config and update_config function
    let (config, update_config) = use_config();

    // Theme state - use theme context
    let mut theme = use_theme();

    let custom_themes = config().list_custom_themes();

    rsx! {
      div { class: "space-y-4",
        // Built-in themes
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
            Moon { class: "w-4 h-4 mr-1" }
            {Theme::Dark.display_name()}
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
            Sun { class: "w-4 h-4 mr-1" }
            {Theme::Light.display_name()}
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
            Computer { class: "w-4 h-4 mr-1" }
            {Theme::System.display_name()}
          }
        }
        // Custom themes section
        if !custom_themes.is_empty() {
          div { class: "space-y-2",
            div { class: "text-sm font-medium text-base-content/70", "Custom Themes" }
            for theme_name in custom_themes.iter() {
              CustomThemeButton {
                name: theme_name.clone(),
                is_active: matches!(*theme.read(), Theme::Custom(ref current) if current == theme_name),
                on_select: {
                    let theme_name = theme_name.clone();
                    let update_fn = update_config.clone();
                    move |_| {
                        let name = theme_name.clone();
                        theme.set(Theme::Custom(name.clone()));
                        update_fn(
                            Box::new(move |config: &mut AppConfig| {
                                config.theme = Theme::Custom(name);
                            }),
                        );
                    }
                },
                on_delete: {
                    let theme_name = theme_name.clone();
                    let update_fn = update_config.clone();
                    move |_| {
                        let name = theme_name.clone();
                        update_fn(
                            Box::new(move |config: &mut AppConfig| {
                                let _ = config.delete_custom_theme(&name);
                            }),
                        );
                    }
                },
              }
            }
          }
        }
        // Create new theme button
        CreateThemeButton {}
      }
    }
}

#[derive(Props, Clone, PartialEq)]
struct CustomThemeButtonProps {
    name: String,
    is_active: bool,
    on_select: EventHandler<MouseEvent>,
    on_delete: EventHandler<MouseEvent>,
}

#[component]
fn CustomThemeButton(props: CustomThemeButtonProps) -> Element {
    let mut show_options = use_signal(|| false);

    rsx! {
      div {
        class: "flex items-center gap-2",
        onmouseenter: move |_| show_options.set(true),
        onmouseleave: move |_| show_options.set(false),
        button {
          class: if props.is_active { "btn btn-neutral flex-1 justify-start" } else { "btn btn-soft flex-1 justify-start" },
          onclick: props.on_select,
          Palette { class: "w-4 h-4 mr-2" }
          {props.name.clone()}
        }
        if show_options() {
          div { class: "flex items-center gap-1",
            button {
              class: "btn btn-ghost btn-sm w-8 h-8 p-0",
              onclick: move |_| {},
              Pencil { class: "w-3 h-3" }
            }
            button {
              class: "btn btn-ghost btn-sm w-8 h-8 p-0 text-error hover:bg-error/20",
              onclick: props.on_delete,
              Trash2 { class: "w-3 h-3" }
            }
          }
        }
      }
    }
}

#[component]
fn CreateThemeButton() -> Element {
    let mut show_modal = use_signal(|| false);

    rsx! {
      div {
        button {
          class: "btn btn-soft btn-sm w-full",
          onclick: move |_| show_modal.set(true),
          Plus { class: "w-4 h-4 mr-1" }
          "Create custom theme"
        }
        if show_modal() {
          ThemeCreatorModal { on_close: move |_| show_modal.set(false) }
        }
      }
    }
}

#[derive(Props, Clone, PartialEq)]
struct ThemeCreatorModalProps {
    on_close: EventHandler<MouseEvent>,
}

#[component]
fn ThemeCreatorModal(props: ThemeCreatorModalProps) -> Element {
    let (_, update_config) = use_config();
    let mut theme_name = use_signal(String::new);
    let mut theme_css = use_signal(|| {
        String::from(
            r#"
                /* Define your custom theme variables here */
                --color-base-100: oklch(98% 0.02 240);
                --color-base-200: oklch(95% 0.03 240);
                --color-base-300: oklch(92% 0.04 240);
                --color-base-content: oklch(20% 0.05 240);
                --color-primary: oklch(55% 0.3 240);
                --color-primary-content: oklch(98% 0.01 240);
                --color-secondary: oklch(70% 0.25 200);
                --color-secondary-content: oklch(98% 0.01 200);
                --color-accent: oklch(65% 0.25 160);
                --color-accent-content: oklch(98% 0.01 160);
                --color-neutral: oklch(50% 0.05 240);
                --color-neutral-content: oklch(98% 0.01 240);
                --color-info: oklch(70% 0.2 220);
                --color-info-content: oklch(98% 0.01 220);
                --color-success: oklch(65% 0.25 140);
                --color-success-content: oklch(98% 0.01 140);
                --color-warning: oklch(80% 0.25 80);
                --color-warning-content: oklch(20% 0.05 80);
                --color-error: oklch(65% 0.3 30);
                --color-error-content: oklch(98% 0.01 30);

                /* border radius */
                --radius-selector: 1rem;
                --radius-field: 0.25rem;
                --radius-box: 0.5rem;

                /* base sizes */
                --size-selector: 0.25rem;
                --size-field: 0.25rem;

                /* border size */
                --border: 1px;

                /* effects */
                --depth: 1;
                --noise: 0;
          "#,
        )
    });
    let mut saving = use_signal(|| false);
    let mut error = use_signal(String::new);
    let on_save = {
        let theme_name = theme_name.clone();
        let theme_css = theme_css.clone();
        let update_config = update_config.clone();

        move |_| {
            let name = theme_name().trim().to_string();
            let css = theme_css();

            if name.is_empty() {
                error.set("Theme name cannot be empty".to_string());
                return;
            }

            saving.set(true);
            error.set(String::new());

            update_config(Box::new(move |config: &mut AppConfig| {
                // Replace {theme_name} placeholder in CSS
                let processed_css = css.replace("{theme_name}", &name);
                match config.save_custom_theme(name, processed_css) {
                    Ok(()) => {
                        saving.set(false);
                    }
                    Err(e) => {
                        error.set(e);
                        saving.set(false);
                    }
                }
            }));
        }
    };

    rsx! {
      div {
        class: "fixed inset-0 bg-black/50 flex items-center justify-center z-50",
        onclick: props.on_close,
        div {
          class: "bg-base-100 p-6 rounded-lg max-w-2xl w-full mx-4 max-h-[80vh] overflow-auto",
          onclick: |e| e.stop_propagation(),
          div { class: "flex items-center justify-between mb-4",
            h3 { class: "text-lg font-bold", "Create custom theme" }
            button {
              class: "btn btn-ghost btn-sm",
              onclick: props.on_close,
              "✕"
            }
          }
          div { class: "space-y-4",
            div { class: "form-control",
              label { class: "label",
                span { class: "label-text", "Theme name" }
              }
              input {
                class: "input input-bordered",
                r#type: "text",
                placeholder: "My custom theme",
                value: theme_name(),
                oninput: move |e| theme_name.set(e.value()),
              }
            }
            fieldset { class: "fieldset",
              legend { class: "fieldset-legend", "Custom CSS" }
              textarea {
                class: "textarea w-full h-64 font-mono text-sm",
                placeholder: "Enter your custom CSS here...",
                value: theme_css(),
                oninput: move |e| theme_css.set(e.value()),
              }
              div { class: "label", "Use DaisyUI CSS variables to style your theme" }
            }
            if !error().is_empty() {
              div { class: "alert alert-error", {error()} }
            }
            div { class: "flex justify-end gap-2",
              button {
                class: "btn btn-ghost",
                onclick: props.on_close,
                "Cancel"
              }
              button {
                class: "btn btn-primary",
                disabled: saving() || theme_name().trim().is_empty(),
                onclick: on_save,
                if saving() {
                  span { class: "loading loading-spinner loading-sm mr-2" }
                  "Saving..."
                } else {
                  "Create Theme"
                }
              }
            }
          }
        }
      }
    }
}
