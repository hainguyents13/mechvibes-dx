use crate::libs::theme::{use_theme, Theme};
use crate::state::config::AppConfig;
use crate::state::config_utils::use_config;
use dioxus::document::eval;
use dioxus::prelude::*;
use lucide_dioxus::{Check, Computer, Moon, Palette, Pencil, Plus, Sun, Trash2};
use uuid;

#[component]
pub fn ThemeToggler() -> Element {
    // Get the config and update_config function
    let (config, update_config) = use_config();

    // Theme state - use theme context
    let mut theme = use_theme();

    let config_ref = config();
    let custom_themes = config_ref.list_custom_theme_data();

    // State for editing themes
    let mut editing_theme = use_signal(|| None::<String>); // Theme ID being edited

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
        } // Custom themes section
        if !custom_themes.is_empty() {
          div { class: "space-y-2",
            div { class: "text-sm text-base-content", "Custom themes" }
            for theme_data in custom_themes.iter() {
              CustomThemeButton {
                name: theme_data.name.clone(),
                theme_id: theme_data.id.clone(),
                theme_css: theme_data.css.clone(),
                is_active: matches!(*theme.read(), Theme::Custom(ref current) if current == &theme_data.id),
                on_select: {
                    let theme_id = theme_data.id.clone();
                    let update_fn = update_config.clone();
                    move |_| {
                        let theme_id = theme_id.clone();
                        theme.set(Theme::Custom(theme_id.clone()));
                        update_fn(
                            Box::new(move |config: &mut AppConfig| {
                                config.theme = Theme::Custom(theme_id);
                            }),
                        );
                    }
                },
                on_delete: {
                    let theme_id = theme_data.id.clone();
                    let update_fn = update_config.clone();
                    move |_| {
                        let theme_id = theme_id.clone();
                        update_fn(
                            Box::new(move |config: &mut AppConfig| {
                                let _ = config.delete_custom_theme(&theme_id);
                            }),
                        );
                    }
                },
                on_edit: {
                    let theme_id = theme_data.id.clone();
                    move |_| {
                        editing_theme.set(Some(theme_id.clone()));
                        eval("theme_creator_modal.showModal()");
                    }
                },
              }
            }
          }
        } // Create new theme button
        CreateThemeButton { editing_theme_id: editing_theme }
      }
    }
}

#[derive(Props, Clone, PartialEq)]
struct CustomThemeButtonProps {
    name: String,
    theme_css: String,
    theme_id: String,
    is_active: bool,
    on_select: EventHandler<MouseEvent>,
    on_delete: EventHandler<MouseEvent>,
    on_edit: EventHandler<MouseEvent>,
}

#[component]
fn CustomThemeButton(props: CustomThemeButtonProps) -> Element {
    rsx! {
      div { class: "flex w-full items-center gap-2",
        button {
          class: format!(
              "bg-base-100 overflow-hidden border border-primary rounded-lg text-base-content w-full  font-sans transition-all {}",
              if props.is_active {
                  "select-none opacity-20"
              } else {
                  "hover:ring-4 cursor-pointer"
              },
          ),
          style: props.theme_css.clone(),
          disabled: props.is_active,
          onclick: props.on_select,
          div { class: "flex bg-base-100 gap-3 h-9 px-2 items-center justify-between relative",
            div { class: "flex items-center grow-0 justify-center",
              if props.is_active {
                Check { class: "w-5 h-5 text-primary" }
              } else {
                Palette { class: "w-5 h-5 text-primary" }
              }
            }
            div { class: "bg-base-100 grow text-primary font-bold text-left",
              {props.name.clone()}
            }
            div { class: "flex gap-1 items-center",
              div { class: "bg-primary flex aspect-square w-3 rounded-full" }
              div { class: "bg-secondary flex aspect-square w-3 rounded-full" }
              div { class: "bg-accent flex aspect-square w-3 rounded-full" }
              div { class: "bg-neutral flex aspect-square w-3 rounded-full" }
            }
          }
        }
        div { class: "join",
          button {
            class: "btn h-9 join-item btn-outline btn-sm",
            onclick: props.on_edit,
            Pencil { class: "w-4 h-4" }
          }
          button {
            class: "btn h-9 join-item btn-sm btn-outline btn-error",
            onclick: props.on_delete,
            Trash2 { class: "w-4 h-4" }
          }
        }
      }
    }
}

#[derive(Props, Clone, PartialEq)]
struct CreateThemeButtonProps {
    editing_theme_id: Signal<Option<String>>,
}

#[component]
fn CreateThemeButton(props: CreateThemeButtonProps) -> Element {
    rsx! {
      div {
        button {
          class: "btn btn-soft btn-sm w-full",
          onclick: {
              let mut editing_theme_id = props.editing_theme_id;
              move |_| {
                  editing_theme_id.set(None);
                  eval("theme_creator_modal.showModal()");
              }
          },
          Plus { class: "w-4 h-4 mr-1" }
          "Create custom theme"
        }
        ThemeCreatorModal { editing_theme_id: props.editing_theme_id }
      }
    }
}

#[derive(Props, Clone, PartialEq)]
struct ThemeCreatorModalProps {
    editing_theme_id: Signal<Option<String>>,
}

const THEME_DEFAULT_CSS: &str = r#"/* Colors */
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
          "#;

#[component]
fn ThemeCreatorModal(props: ThemeCreatorModalProps) -> Element {
    let (config, update_config) = use_config();
    let mut theme_name = use_signal(String::new);
    let mut theme_css = use_signal(|| String::from(THEME_DEFAULT_CSS));
    let mut saving = use_signal(|| false);
    let mut error = use_signal(String::new);

    // Load existing theme data when editing
    use_effect(move || {
        if let Some(editing_id) = props.editing_theme_id.read().as_ref() {
            if let Some(theme_data) = config().get_custom_theme_by_id(editing_id) {
                theme_name.set(theme_data.name.clone());
                theme_css.set(theme_data.css.clone());
            }
        } else {
            // Reset for new theme
            theme_name.set(String::new());
            theme_css.set(String::from(THEME_DEFAULT_CSS));
        }
    });

    let is_editing = props.editing_theme_id.read().is_some();

    let on_save = {
        let theme_name = theme_name.clone();
        let theme_css = theme_css.clone();
        let update_config = update_config.clone();
        let mut editing_theme_id = props.editing_theme_id;

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
                let result = if let Some(editing_id) = editing_theme_id.read().as_ref() {
                    // Update existing theme
                    config.save_custom_theme(editing_id.clone(), name, css)
                } else {
                    // Create new theme
                    let theme_id = uuid::Uuid::new_v4().to_string();
                    config.save_custom_theme(theme_id, name, css)
                };

                match result {
                    Ok(()) => {
                        saving.set(false);
                        editing_theme_id.set(None); // Clear editing state
                        eval("theme_creator_modal.close()");
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
      dialog { class: "modal", id: "theme_creator_modal",
        div { class: "modal-box",
          form { method: "dialog",
            button { class: "btn btn-sm btn-circle btn-ghost absolute right-2 top-2",
              "✕"
            }
          }
          h3 { class: "font-bold text-lg mb-4",
            if is_editing {
              "Edit custom theme"
            } else {
              "Create custom theme"
            }
          }
          div { class: "space-y-4",
            fieldset { class: "fieldset",
              legend { class: "fieldset-legend",
                span { class: "label-text", "Theme name" }
              }
              input {
                class: "input w-full",
                r#type: "text",
                placeholder: "My custom theme",
                value: theme_name(),
                oninput: move |e| theme_name.set(e.value()),
              }
            }
            fieldset { class: "fieldset",
              legend { class: "fieldset-legend", "CSS" }
              textarea {
                class: "textarea w-full h-64 font-mono text-sm",
                placeholder: "Enter your theme CSS here...",
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
                class: "btn btn-primary btn-sm",
                disabled: saving() || theme_name().trim().is_empty(),
                onclick: on_save,
                if saving() {
                  span { class: "loading loading-spinner loading-sm mr-2" }
                  if is_editing {
                    "Updating..."
                  } else {
                    "Creating..."
                  }
                } else {
                  if is_editing {
                    "Update theme"
                  } else {
                    "Create theme"
                  }
                }
              }
            }
          }
        }
      }
    }
}
