use crate::components::theme_toggler::ThemeToggler;
use crate::components::ui::{Collapse, ColorPicker, PageHeader, Toggler};
use crate::utils::config::use_config;
use dioxus::prelude::*;
use lucide_dioxus::{Check, Palette, RotateCcw};

#[component]
pub fn CustomizePage() -> Element {
    // let (config, update_config) = use_config();
    // let mut saving = use_signal(|| false);

    // let custom_css = use_memo(move || config().custom_css.clone());
    // let mut css_input = use_signal(|| custom_css());
    // let on_save = move |_| {
    //     let css = css_input().clone();
    //     update_config(Box::new(move |cfg| {
    //         cfg.custom_css = css;
    //     }));
    //     saving.set(true);
    //     spawn(async move {
    //         futures_timer::Delay::new(std::time::Duration::from_millis(1500)).await;
    //         saving.set(false);
    //     });
    // };
    rsx! {
      div { class: "p-12 pb-32",
        PageHeader {
          title: "Customize".to_string(),
          subtitle: "Vibe it your way!".to_string(),
          icon: Some(rsx! {
            Palette { class: "w-8 h-8 mx-auto" }
          }),
        }
        // Settings sections
        div { class: "space-y-4 mt-8",          // Theme Section
          Collapse {
            title: "Themes".to_string(),
            group_name: "customize-accordion".to_string(),
            default_open: true,
            variant: "border border-base-300 bg-base-200 text-base-content",
            content_class: "collapse-content text-sm text-base-content/70",
            children: rsx! {
              div { "Choose your preferred theme or create custom ones" }
              // Built-in theme toggler
              ThemeToggler {}
            },
          }          Collapse {
            title: "Logo".to_string(),
            group_name: "customize-accordion".to_string(),
            variant: "border border-base-300 bg-base-200 text-base-content",
            content_class: "collapse-content overflow-visible text-base-content/70",
            children: rsx! {
              LogoCustomizationSection {}
            },
          }
                // Custom CSS Section
        // div { class: "collapse collapse-arrow border border-base-300 bg-base-200 text-base-content",
        //   input { r#type: "radio", name: "customize-accordion" }
        //   div { class: "collapse-title font-semibold", "Custom CSS" }
        //   div { class: "collapse-content",
        //     fieldset { class: "fieldset mb-2",
        //       legend { class: "fieldset-legend", "Add your custom CSS here" }
        //       textarea {
        //         class: "textarea w-full h-32 font-mono text-sm",
        //         value: css_input(),
        //         oninput: move |evt| css_input.set(evt.value()),
        //       }
        //       div { class: "label",
        //         "Apply your own styles to customize the look and feel of the app."
        //       }
        //     }
        //     button {
        //       class: "btn btn-neutral btn-sm",
        //       r#type: "button",
        //       disabled: saving(),
        //       onclick: on_save,
        //       if saving() {
        //         span { class: "loading loading-spinner loading-sm mr-2" }
        //       }
        //       "Save"
        //     }
        //   }
        // }
        }
      }
    }
}

#[component]
fn LogoCustomizationSection() -> Element {
    let (config, update_config) = use_config();
    let enable_logo_customization = use_memo(move || config().enable_logo_customization);

    // Create a local signal that syncs with config
    let mut local_enable = use_signal(|| enable_logo_customization());

    // Update local state when config changes
    use_effect(move || {
        local_enable.set(enable_logo_customization());
    });
    rsx! {
      div { class: "space-y-4",        // Toggle switch for logo customization
        Toggler {
          title: "Enable Logo Customization".to_string(),
          description: Some("Customize border, text, shadow and background colors".to_string()),
          checked: local_enable(),
          on_change: move |new_value: bool| {
            local_enable.set(new_value);
            update_config(
                Box::new(move |cfg| {
                    cfg.enable_logo_customization = new_value;
                }),
            );
          },
        }
        // Show LogoCustomizationPanel only when enabled
        if local_enable() {
          LogoCustomizationPanel {}
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
    let mut muted_background = use_signal(|| logo_customization().muted_background);
    let mut dimmed_when_muted = use_signal(|| logo_customization().dimmed_when_muted);
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
            let muted_bg = muted_background();
            let dimmed = dimmed_when_muted();

            update_config_clone(Box::new(move |cfg| {
                cfg.logo_customization.border_color = border;
                cfg.logo_customization.text_color = text;
                cfg.logo_customization.shadow_color = shadow;
                cfg.logo_customization.background_color = background;
                cfg.logo_customization.muted_background = muted_bg;
                cfg.logo_customization.dimmed_when_muted = dimmed;
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
        muted_background.set(default_logo.muted_background.clone());
        dimmed_when_muted.set(default_logo.dimmed_when_muted);

        update_config(Box::new(move |cfg| {
            cfg.logo_customization = default_logo;
        }));
    }; // Update local state when config changes
    use_effect(move || {
        let logo = logo_customization();
        border_color.set(logo.border_color);
        text_color.set(logo.text_color);
        shadow_color.set(logo.shadow_color);
        background_color.set(logo.background_color);
        muted_background.set(logo.muted_background);
        dimmed_when_muted.set(logo.dimmed_when_muted);
    });

    rsx! {
      div { class: "space-y-4",
        // Preview
        div { class: "space-y-2",
          div { class: "text-sm text-base-content", "Preview" }
          div { class: "grid grid-cols-2 gap-2 p-4 bg-base-100 rounded-box border border-base-300 space-y-3",
            // Normal state preview
            div {
              div { class: "text-xs text-base-content/70", "Normal" }
              div {
                class: "select-none border-3 font-black py-2 px-4 text-2xl rounded-box flex justify-center items-center w-full mt-1",
                style: format!(
                    "border-color: {}; color: {}; background: {}; box-shadow: 0 3px 0 {}",
                    border_color(),
                    text_color(),
                    background_color(),
                    shadow_color(),
                ),
                "Mechvibes"
              }
            }
            // Muted state preview
            div {
              div { class: "text-xs text-base-content/70", "Muted" }
              div {
                class: format!(
                    "select-none border-3 font-black py-2 px-4 text-2xl rounded-box flex justify-center items-center w-full mx-auto mt-1{}",
                    if dimmed_when_muted() { " opacity-50" } else { "" },
                ),
                style: format!(
                    "border-color: {}; color: {}; background: {}",
                    border_color(),
                    text_color(),
                    muted_background(),
                ),
                "Mechvibes"
              }
            }
          }
        }
        // Border Color
        ColorPicker {
          label: "Border Color".to_string(),
          selected_value: border_color(),
          options: color_options.clone(),
          placeholder: "Select a color...".to_string(),
          on_change: move |color: String| border_color.set(color),
          field: "border_color".to_string(),
          description: None,
        }
        // Text Color
        ColorPicker {
          label: "Text Color".to_string(),
          selected_value: text_color(),
          options: color_options.clone(),
          placeholder: "Select a color...".to_string(),
          on_change: move |value| text_color.set(value),
          field: "text_color".to_string(),
          description: None,
        }
        // Shadow Color
        ColorPicker {
          label: "Shadow Color".to_string(),
          selected_value: shadow_color(),
          options: color_options.clone(),
          placeholder: "Select a color...".to_string(),
          on_change: move |value| shadow_color.set(value),
          field: "shadow_color".to_string(),
          description: None,
        }
        // Background Color
        ColorPicker {
          label: "Background".to_string(),
          selected_value: background_color(),
          options: color_options.clone(),
          placeholder: "Select a color...".to_string(),
          on_change: move |value| background_color.set(value),
          field: "background_color".to_string(),
          description: None,
        }
        // Muted Background Color
        ColorPicker {
          label: "Muted Background".to_string(),
          selected_value: muted_background(),
          options: color_options.clone(),
          placeholder: "Select a color...".to_string(),
          on_change: move |value| muted_background.set(value),
          field: "muted_background".to_string(),
          description: Some("Background color when sound is disabled".to_string()),
        }        // Dimmed logo when muted option
        Toggler {
          title: "Dimmed logo when muted".to_string(),
          description: Some("Applies opacity to the logo when sound is disabled".to_string()),
          checked: dimmed_when_muted(),
          on_change: move |new_value: bool| {
            dimmed_when_muted.set(new_value);
          },
        }
      }

      // Action buttons
      div { class: "flex gap-2 mt-3",
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
      div { class: "text-sm text-base-content/50 mt-3",
        "When you reset the logo customization, it will revert to the selected theme colors."
      }
    }
}
