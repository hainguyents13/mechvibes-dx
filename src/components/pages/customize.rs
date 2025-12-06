use crate::components::theme_toggler::ThemeToggler;
use crate::components::ui::{ Collapse, ColorPicker, PageHeader, Toggler };
use crate::utils::config::use_config;
use crate::utils::delay;
use crate::utils::path;
use dioxus::prelude::*;
use lucide_dioxus::{ Check, Palette, RotateCcw, Upload };

/// Reusable image picker component with file dialog and URL input
#[component]
fn ImagePicker(
    label: String,
    value: Option<String>,
    on_change: EventHandler<Option<String>>,
    dialog_title: String,
) -> Element {
    rsx! {
        div { class: "space-y-2",
            div {
                div { class: "text-sm font-medium text-base-content", "{label}" }
                div { class: "text-xs text-base-content/50", "Select an image file or enter a URL" }
            }
            div { class: "flex gap-2",
                input {
                    r#type: "text",
                    placeholder: "Enter image URL or path...",
                    class: "input w-full input-sm",
                    value: value.unwrap_or_default(),
                    oninput: move |evt| {
                        let new_value = if evt.value().is_empty() { None } else { Some(evt.value()) };
                        on_change.call(new_value);
                    },
                }
                button {
                    class: "btn btn-neutral btn-sm",
                    onclick: move |_| {
                        let on_change = on_change.clone();
                        let title = dialog_title.clone();
                        spawn(async move {
                            let file_dialog = rfd::AsyncFileDialog::new()
                                .add_filter("Images", &["png", "jpg", "jpeg", "gif", "webp", "bmp"])
                                .set_title(&title)
                                .pick_file()
                                .await;

                            if let Some(file_handle) = file_dialog {
                                let source_path = file_handle.path().to_string_lossy().to_string();

                                // Copy image to custom images directory and get asset URL
                                match path::copy_to_custom_images(&source_path) {
                                    Ok(asset_url) => {
                                        on_change.call(Some(asset_url));
                                    }
                                    Err(e) => {
                                        eprintln!("Failed to copy image: {}", e);
                                    }
                                }
                            }
                        });
                    },
                    Upload { class: "w-4 h-4" }
                }
            }
        }
    }
}

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
      div { class: "",
        PageHeader {
          title: "Customize".to_string(),
          subtitle: "Vibe it your way!".to_string(),
          icon: Some(rsx! {
            Palette { class: "w-8 h-8 mx-auto" }
          }),
        }
        // Settings sections
        div { class: "{crate::utils::spacing::SECTION_SPACING} mt-8", // Theme Section
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
          }
          LogoCollapseSection {}
          BackgroundCollapseSection {}
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
fn LogoCollapseSection() -> Element {
    let (config, _) = use_config();
    let enable_logo_customization = use_memo(move || config().enable_logo_customization);

    rsx! {
        Collapse {
            title: "Logo".to_string(),
            group_name: "customize-accordion".to_string(),
            variant: "border border-base-300 bg-base-200 text-base-content",
            content_class: "collapse-content overflow-visible text-base-content/70",
            show_indicator: enable_logo_customization(),
            children: rsx! {
                LogoCustomizationSection {}
            },
        }
    }
}

#[component]
fn BackgroundCollapseSection() -> Element {
    let (config, _) = use_config();
    let enable_background_customization = use_memo(move || config().enable_background_customization);

    rsx! {
        Collapse {
            title: "Background".to_string(),
            group_name: "customize-accordion".to_string(),
            variant: "border border-base-300 bg-base-200 text-base-content",
            content_class: "collapse-content overflow-visible text-base-content/70",
            show_indicator: enable_background_customization(),
            children: rsx! {
                BackgroundCustomizationSection {}
            },
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
      div { class: "space-y-4", // Toggle switch for logo customization
        Toggler {
          title: "Enable logo customization".to_string(),
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
    let mut background_image = use_signal(|| logo_customization().background_image.clone());
    let mut use_background_image = use_signal(|| logo_customization().use_background_image);
    let mut muted_background = use_signal(|| logo_customization().muted_background);
    let mut muted_background_image = use_signal(||
        logo_customization().muted_background_image.clone()
    );
    let mut use_muted_background_image = use_signal(
        || logo_customization().use_muted_background_image
    );
    let mut dimmed_when_muted = use_signal(|| logo_customization().dimmed_when_muted);
    let mut saving = use_signal(|| false);
    // Theme-based color options using CSS variables
    let color_options = vec![
        ("Primary", "var(--color-primary)"),
        ("Primary Content", "var(--color-primary-content)"),
        ("Secondary", "var(--color-secondary)"),
        ("Secondary Content", "var(--color-secondary-content)"),
        ("Base Content", "var(--color-base-content)"),
        ("Base 100", "var(--color-base-100)"),
        ("Base 200", "var(--color-base-200)"),
        ("Base 300", "var(--color-base-300)"),
        ("Success", "var(--color-success)"),
        ("Success Content", "var(--color-success-content)"),
        ("Warning", "var(--color-warning)"),
        ("Warning Content", "var(--color-warning-content)"),
        ("Error", "var(--color-error)"),
        ("Error Content", "var(--color-error-content)")
    ];
    let on_save = {
        let update_config_clone = update_config.clone();
        move |_| {
            let border = border_color();
            let text = text_color();
            let shadow = shadow_color();
            let background = background_color();
            let bg_image = background_image();
            let use_bg_image = use_background_image();
            let muted_bg = muted_background();
            let muted_bg_image = muted_background_image();
            let use_muted_bg_image = use_muted_background_image();
            let dimmed = dimmed_when_muted();

            update_config_clone(
                Box::new(move |cfg| {
                    cfg.logo_customization.border_color = border;
                    cfg.logo_customization.text_color = text;
                    cfg.logo_customization.shadow_color = shadow;
                    cfg.logo_customization.background_color = background;
                    cfg.logo_customization.background_image = bg_image;
                    cfg.logo_customization.use_background_image = use_bg_image;
                    cfg.logo_customization.muted_background = muted_bg;
                    cfg.logo_customization.muted_background_image = muted_bg_image;
                    cfg.logo_customization.use_muted_background_image = use_muted_bg_image;
                    cfg.logo_customization.dimmed_when_muted = dimmed;
                })
            );
            saving.set(true);
            spawn(async move {
                delay::Delay::ms(500).await;
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
        background_image.set(default_logo.background_image.clone());
        use_background_image.set(default_logo.use_background_image);
        muted_background.set(default_logo.muted_background.clone());
        muted_background_image.set(default_logo.muted_background_image.clone());
        use_muted_background_image.set(default_logo.use_muted_background_image);
        dimmed_when_muted.set(default_logo.dimmed_when_muted);

        update_config(
            Box::new(move |cfg| {
                cfg.logo_customization = default_logo;
            })
        );
    }; // Update local state when config changes
    use_effect(move || {
        let logo = logo_customization();
        border_color.set(logo.border_color);
        text_color.set(logo.text_color);
        shadow_color.set(logo.shadow_color);
        background_color.set(logo.background_color);
        background_image.set(logo.background_image);
        use_background_image.set(logo.use_background_image);
        muted_background.set(logo.muted_background);
        muted_background_image.set(logo.muted_background_image);
        use_muted_background_image.set(logo.use_muted_background_image);
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
                    "border-color: {}; color: {}; {}; box-shadow: 0 3px 0 {}",
                    border_color(),
                    text_color(),
                    if use_background_image() {
                        if let Some(ref img) = background_image() {
                            format!(
                                "background-image: url('{}'); background-size: cover; background-position: center",
                                img,
                            )
                        } else {
                            format!("background: {}", background_color())
                        }
                    } else {
                        format!("background: {}", background_color())
                    },
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
                    "border-color: {}; color: {}; {}",
                    border_color(),
                    text_color(),
                    if use_muted_background_image() {
                        if let Some(ref img) = muted_background_image() {
                            format!(
                                "background-image: url('{}'); background-size: cover; background-position: center",
                                img,
                            )
                        } else {
                            format!("background: {}", muted_background())
                        }
                    } else {
                        format!("background: {}", muted_background())
                    },
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
        } // Shadow Color
        ColorPicker {
          label: "Shadow Color".to_string(),
          selected_value: shadow_color(),
          options: color_options.clone(),
          placeholder: "Select a color...".to_string(),
          on_change: move |value| shadow_color.set(value),
          field: "shadow_color".to_string(),
          description: None,
        }
        // Background Section
        div { class: "space-y-3 p-3 border border-base-300 rounded-box bg-base-100",
          h4 { class: "text-sm font-semibold text-base-content", "Background (Normal)" }
          // Toggle between color and image for normal background
          Toggler {
            title: "Use image".to_string(),
            description: Some("Use image instead of solid color".to_string()),
            checked: use_background_image(),
            on_change: move |new_value: bool| {
                use_background_image.set(new_value);
            },
          }
          // Background Color Picker (shown when not using image)
          if !use_background_image() {
            ColorPicker {
              label: "Color".to_string(),
              selected_value: background_color(),
              options: color_options.clone(),
              placeholder: "Select a color...".to_string(),
              on_change: move |value| background_color.set(value),
              field: "background_color".to_string(),
              description: None,
            }
          }
          // Background Image Selector (shown when using image)
          if use_background_image() {
            ImagePicker {
              label: "Background Image".to_string(),
              value: background_image(),
              on_change: move |value| background_image.set(value),
              dialog_title: "Select Background Image".to_string(),
            }
          }
        }

        // Muted Background Section
        div { class: "space-y-3 p-3 border border-base-300 rounded-box bg-base-100",
          h4 { class: "text-sm font-semibold text-base-content", "Background (Muted)" }
          // Toggle between color and image for muted background
          Toggler {
            title: "Use image".to_string(),
            description: Some("Use image instead of solid color".to_string()),
            checked: use_muted_background_image(),
            on_change: move |new_value: bool| {
                use_muted_background_image.set(new_value);
            },
          }
          // Muted Background Color Picker (shown when not using image)
          if !use_muted_background_image() {
            ColorPicker {
              label: "Color".to_string(),
              selected_value: muted_background(),
              options: color_options.clone(),
              placeholder: "Select a color...".to_string(),
              on_change: move |value| muted_background.set(value),
              field: "muted_background".to_string(),
              description: Some("Background color when sound is disabled".to_string()),
            }
          }
          // Muted Background Image Selector (shown when using image)
          if use_muted_background_image() {
            ImagePicker {
              label: "Image".to_string(),
              value: muted_background_image(),
              on_change: move |value| muted_background_image.set(value),
              dialog_title: "Select Muted Background Image".to_string(),
            }
          }
        }
        // Dimmed logo when muted option
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

#[component]
fn BackgroundCustomizationSection() -> Element {
    let (config, update_config) = use_config();
    let enable_background_customization = use_memo(
        move || config().enable_background_customization
    );

    // Create a local signal that syncs with config
    let mut local_enable = use_signal(|| enable_background_customization());

    // Update local state when config changes
    use_effect(move || {
        local_enable.set(enable_background_customization());
    });

    rsx! {
      div { class: "space-y-4",
        // Toggle switch for background customization
        Toggler {
          title: "Enable background customization".to_string(),
          description: Some("Customize app background with colors or images".to_string()),
          checked: local_enable(),
          on_change: move |new_value: bool| {
              local_enable.set(new_value);
              update_config(
                  Box::new(move |cfg| {
                      cfg.enable_background_customization = new_value;
                  }),
              );
          },
        }
        // Show BackgroundCustomizationPanel only when enabled
        if local_enable() {
          BackgroundCustomizationPanel {}
        }
      }
    }
}

#[component]
fn BackgroundCustomizationPanel() -> Element {
    let (config, update_config) = use_config();
    let background_customization = use_memo(move || config().background_customization.clone());
    let mut background_color = use_signal(|| background_customization().background_color);
    let mut background_image = use_signal(|| background_customization().background_image.clone());
    let mut use_image = use_signal(|| background_customization().use_image);
    let mut saving = use_signal(|| false);

    // Theme-based color options using CSS variables (same as logo)
    let color_options = vec![
        ("Primary", "var(--color-primary)"),
        ("Primary Content", "var(--color-primary-content)"),
        ("Secondary", "var(--color-secondary)"),
        ("Secondary Content", "var(--color-secondary-content)"),
        ("Base Content", "var(--color-base-content)"),
        ("Base 100", "var(--color-base-100)"),
        ("Base 200", "var(--color-base-200)"),
        ("Base 300", "var(--color-base-300)"),
        ("Success", "var(--color-success)"),
        ("Success Content", "var(--color-success-content)"),
        ("Warning", "var(--color-warning)"),
        ("Warning Content", "var(--color-warning-content)"),
        ("Error", "var(--color-error)"),
        ("Error Content", "var(--color-error-content)")
    ];

    let on_save = {
        let update_config_clone = update_config.clone();
        move |_| {
            let color = background_color();
            let image = background_image();
            let use_img = use_image();

            update_config_clone(
                Box::new(move |cfg| {
                    cfg.background_customization.background_color = color;
                    cfg.background_customization.background_image = image;
                    cfg.background_customization.use_image = use_img;
                })
            );
            saving.set(true);
            spawn(async move {
                delay::Delay::ms(500).await;
                saving.set(false);
            });
        }
    };

    let on_reset = move |_| {
        let default_bg = crate::state::config::BackgroundCustomization::default();
        background_color.set(default_bg.background_color.clone());
        background_image.set(default_bg.background_image.clone());
        use_image.set(default_bg.use_image);

        update_config(
            Box::new(move |cfg| {
                cfg.background_customization = default_bg;
            })
        );
    };

    // Update local state when config changes
    use_effect(move || {
        let bg = background_customization();
        background_color.set(bg.background_color);
        background_image.set(bg.background_image);
        use_image.set(bg.use_image);
    });

    rsx! {
      div { class: "space-y-4",
        // Toggle between color and image
        Toggler {
          title: "Use image".to_string(),
          description: Some("Use image instead of solid color".to_string()),
          checked: use_image(),
          on_change: move |new_value: bool| {
              use_image.set(new_value);
          },
        }

        // Background Color Picker (shown when not using image)
        if !use_image() {
          ColorPicker {
            label: "Background Color".to_string(),
            selected_value: background_color(),
            options: color_options.clone(),
            placeholder: "Select a color...".to_string(),
            on_change: move |color: String| background_color.set(color),
            field: "background_color".to_string(),
            description: None,
          }
        }
        // Background Image Selector (shown when using image)
        if use_image() {
          ImagePicker {
            label: "Background Image".to_string(),
            value: background_image(),
            on_change: move |value| background_image.set(value),
            dialog_title: "Select Background Image".to_string(),
          }
        }

        // Action buttons
        div { class: "flex gap-2 mt-4",
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
}
