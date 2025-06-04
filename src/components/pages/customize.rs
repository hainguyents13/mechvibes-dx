use crate::components::theme_toggler::ThemeToggler;
use crate::components::ui::PageHeader;
use crate::utils::config_utils::use_config;
use dioxus::{document::eval, prelude::*};
use lucide_dioxus::{Check, ChevronDown, Palette, RotateCcw};

#[component]
pub fn CustomizePage() -> Element {
    let (config, update_config) = use_config();
    let custom_css = use_memo(move || config().custom_css.clone());
    let mut css_input = use_signal(|| custom_css());
    let mut saving = use_signal(|| false);

    let on_save = move |_| {
        let css = css_input().clone();
        update_config(Box::new(move |cfg| {
            cfg.custom_css = css;
        }));
        saving.set(true);
        spawn(async move {
            futures_timer::Delay::new(std::time::Duration::from_millis(1500)).await;
            saving.set(false);
        });
    };
    rsx! {
      div { class: "p-12 pb-32",
        PageHeader {
          title: "Customize".to_string(),
          subtitle: Some("Change themes, styles, and more".to_string()),
          icon: Some(rsx! {
            Palette { class: "w-8 h-8 mx-auto" }
          }),
        }
        // Settings sections
        div { class: "space-y-4 mt-8",
          // Theme Section
          div { class: "collapse collapse-arrow border border-base-300 bg-base-200 text-base-content",
            input {
              r#type: "radio",
              name: "customize-accordion",
              checked: true,
            }
            div { class: "collapse-title font-semibold", "Themes" }
            div { class: "collapse-content text-sm text-base-content/70",
              div { "Choose your preferred theme or create custom ones" }
              // Built-in theme toggler
              ThemeToggler {}
            }
          }
          div { class: "collapse collapse-arrow border border-base-300 bg-base-200 text-base-content",
            input { r#type: "radio", name: "customize-accordion" }
            div { class: "collapse-title font-semibold", "Logo" }
            div { class: "collapse-content overflow-visible text-base-content/70",
              div { class: "mb-4", "Customize the Mechvibes logo appearance" }
              LogoCustomizationPanel {}
            }
          }
          // Custom CSS Section
          div { class: "collapse collapse-arrow border border-base-300 bg-base-200 text-base-content",
            input { r#type: "radio", name: "customize-accordion" }
            div { class: "collapse-title font-semibold", "Custom CSS" }
            div { class: "collapse-content",
              fieldset { class: "fieldset mb-2",
                legend { class: "fieldset-legend", "Add your custom CSS here" }
                textarea {
                  class: "textarea w-full h-32 font-mono text-sm",
                  value: css_input(),
                  oninput: move |evt| css_input.set(evt.value()),
                }
                div { class: "label",
                  "Apply your own styles to customize the look and feel of the app."
                }
              }
              button {
                class: "btn btn-neutral btn-sm",
                r#type: "button",
                disabled: saving(),
                onclick: on_save,
                if saving() {
                  span { class: "loading loading-spinner loading-sm mr-2" }
                }
                "Save"
              }
            }
          }
        }
      }
    }
}

#[component]
fn LogoCustomizationPanel() -> Element {
    let (config, update_config) = use_config();
    let logo_customization = use_memo(move || config().logo_customization.clone());

    let mut border_color = use_signal(|| logo_customization().border_color);
    let mut text_color = use_signal(|| logo_customization().text_color);
    let mut shadow_color = use_signal(|| logo_customization().shadow_color);
    let mut background_color = use_signal(|| logo_customization().background_color);
    let mut saving = use_signal(|| false);
    // Theme-based color options using CSS variables
    let color_options = vec![
        ("Base Content", "var(--color-base-content)"),
        ("Base 100", "var(--color-base-100)"),
        ("Base 200", "var(--color-base-200)"),
        ("Base 300", "var(--color-base-300)"),
        ("Primary", "var(--color-primary)"),
        ("Primary Content", "var(--color-primary-content)"),
        ("Secondary", "var(--color-secondary)"),
        ("Secondary Content", "var(--color-secondary-content)"),
        ("Accent", "var(--color-accent)"),
        ("Accent Content", "var(--color-accent-content)"),
        ("Neutral", "var(--color-neutral)"),
        ("Neutral Content", "var(--color-neutral-content)"),
        ("Info", "var(--color-info)"),
        ("Info Content", "var(--color-info-content)"),
        ("Success", "var(--color-success)"),
        ("Success Content", "var(--color-success-content)"),
        ("Warning", "var(--color-warning)"),
        ("Warning Content", "var(--color-warning-content)"),
        ("Error", "var(--color-error)"),
        ("Error Content", "var(--color-error-content)"),
    ];
    let on_save = {
        let update_config_clone = update_config.clone();
        move |_| {
            let border = border_color();
            let text = text_color();
            let shadow = shadow_color();
            let background = background_color();

            update_config_clone(Box::new(move |cfg| {
                cfg.logo_customization.border_color = border;
                cfg.logo_customization.text_color = text;
                cfg.logo_customization.shadow_color = shadow;
                cfg.logo_customization.background_color = background;
            }));

            saving.set(true);
            spawn(async move {
                futures_timer::Delay::new(std::time::Duration::from_millis(1500)).await;
                saving.set(false);
            });
        }
    };

    let on_reset = move |_| {
        let default_logo = crate::state::config::LogoCustomization::default();
        border_color.set(default_logo.border_color.clone());
        text_color.set(default_logo.text_color.clone());
        shadow_color.set(default_logo.shadow_color.clone());
        background_color.set(default_logo.background_color.clone());

        update_config(Box::new(move |cfg| {
            cfg.logo_customization = default_logo;
        }));
    };

    // Update local state when config changes
    use_effect(move || {
        let logo = logo_customization();
        border_color.set(logo.border_color);
        text_color.set(logo.text_color);
        shadow_color.set(logo.shadow_color);
        background_color.set(logo.background_color);
    });

    rsx! {
      div { class: "space-y-4",
        // Preview
        div { class: "space-y-2",
          div { class: "text-sm text-base-content", "Preview" }
          div { class: "p-4 bg-base-100 rounded-box border border-base-300",
            div {
              class: "select-none border-4 font-black py-2 px-4 text-2xl rounded-box flex justify-center items-center w-fit mx-auto",
              style: format!(
                  "border-color: {}; color: {}; background-color: {}; box-shadow: 0 3px 0 {}",
                  border_color(),
                  text_color(),
                  background_color(),
                  shadow_color(),
              ),
              "Mechvibes"
            }
          }
        }
        // Border Color
        div { class: "space-y-2",
          div { class: "text-sm text-base-content", "Border Color" }
          div { class: "grid grid-cols-2 gap-2",
            ColorDropdown {
              selected_value: border_color(),
              options: color_options.clone(),
              placeholder: "Select a color...".to_string(),
              on_change: move |color: String| border_color.set(color),
              field: "border_color".to_string(),
            }
            input {
              r#type: "text",
              class: "input",
              placeholder: "Or enter custom color (e.g., #ff0000, rgb(255,0,0))",
              value: if color_options.iter().any(|(_, c)| *c == border_color()) { "" } else { border_color() },
              oninput: move |evt| border_color.set(evt.value()),
            }
          }
        }
        // Text Color
        div { class: "space-y-2",
          div { class: "text-sm text-base-content", "Text Color" }
          div { class: "grid grid-cols-2 gap-2",
            ColorDropdown {
              selected_value: text_color(),
              options: color_options.clone(),
              on_change: move |value| text_color.set(value),
              placeholder: "Select a color...".to_string(),
              field: "text_color".to_string(),
            }
            input {
              r#type: "text",
              class: "input",
              placeholder: "Or enter custom color (e.g., #ff0000, rgb(255,0,0))",
              value: if color_options.iter().any(|(_, c)| *c == text_color()) { "" } else { text_color() },
              oninput: move |evt| text_color.set(evt.value()),
            }
          }
        }
        // Shadow Color
        div { class: "space-y-2",
          div { class: "text-sm text-base-content", "Shadow Color" }
          div { class: "grid grid-cols-2 gap-2",
            ColorDropdown {
              selected_value: shadow_color(),
              options: color_options.clone(),
              on_change: move |value| shadow_color.set(value),
              placeholder: "Select a color...".to_string(),
              field: "shadow_color".to_string(),
            }
            input {
              r#type: "text",
              class: "input",
              placeholder: "Or enter custom color (e.g., #ff0000, rgb(255,0,0))",
              value: if color_options.iter().any(|(_, c)| *c == shadow_color()) { "" } else { shadow_color() },
              oninput: move |evt| shadow_color.set(evt.value()),
            }
          }
        }
        // Background Color
        div { class: "space-y-2",
          div { class: "text-sm text-base-content", "Background Color" }
          div { class: "grid grid-cols-2 gap-2",
            ColorDropdown {
              selected_value: background_color(),
              options: color_options.clone(),
              on_change: move |value| background_color.set(value),
              placeholder: "Select a color...".to_string(),
              field: "background_color".to_string(),
            }
            input {
              r#type: "text",
              class: "input",
              placeholder: "Or enter custom color (e.g., #ff0000, rgb(255,0,0))",
              value: if color_options.iter().any(|(_, c)| *c == background_color()) { "" } else { background_color() },
              oninput: move |evt| background_color.set(evt.value()),
            }
          }
        }
      }

      // Action buttons
      div { class: "flex gap-2 pt-4",
        button {
          class: "btn btn-neutral btn-sm",
          disabled: saving(),
          onclick: on_save,
          if saving() {
            span { class: "loading loading-spinner loading-sm mr-2" }
          } else {
            Check { class: "w-4 h-4 mr-1" }
          }
          "Save changes"
        }
        button { class: "btn btn-ghost btn-sm", onclick: on_reset,
          RotateCcw { class: "w-4 h-4 mr-1" }
          "Reset"
        }
      }
    }
}

#[component]
fn ColorDropdown(
    selected_value: String,
    options: Vec<(&'static str, &'static str)>,
    on_change: EventHandler<String>,
    placeholder: String,
    field: String,
) -> Element {
    let mut is_open = use_signal(|| false);

    // Find the display name for the selected value
    let selected_display = options
        .iter()
        .find(|(_, value)| *value == selected_value)
        .map(|(name, _)| *name)
        .unwrap_or(&placeholder);

    rsx! {
      div {
        class: format!(
            "dropdown w-full {}",
            if field == "background_color" || field == "shadow_color" {
                "dropdown-top"
            } else {
                "dropdown-bottom"
            },
        ),
        button {
          class: "btn btn-soft w-full justify-between",
          "tabindex": "0",
          "role": "button",
          div { class: "flex items-center gap-2",
            // Color circle indicator
            div {
              class: "w-4 h-4 rounded-full border border-base-300 flex-shrink-0",
              style: format!("background-color: {}", selected_value),
            }
            span { class: "text-left truncate w-26", "{selected_display}" }
          }
          ChevronDown { class: "w-4 h-4 " }
        }
        ul {
          class: "dropdown-content bg-base-100 rounded-box z-1 flex-col p-2 h-52 overflow-y-auto w-full shadow-sm",
          "tabindex": "0",
          for (name , color) in options.iter() {
            li { class: "w-full",
              a {
                class: format!(
                    "flex w-full cursor-pointer items-center gap-2 p-2 rounded hover:bg-base-200 text-left {}",
                    if *color == selected_value { "bg-primary/10" } else { "" },
                ),
                onclick: {
                    let color = color.to_string();
                    move |_| {
                        on_change.call(color.clone());
                        is_open.set(false);
                        eval("document.activeElement.blur()");
                    }
                },
                div {
                  class: "w-4 h-4 rounded-full border border-base-300 flex-shrink-0",
                  style: format!("background-color: {}", color),
                }
                span { class: "truncate text-sm", "{name}" }
              }
            }
          }
        }
      }
    }
}
