use crate::components::ui::PageHeader;
use crate::state::config_utils::use_config;
use dioxus::prelude::*;
use lucide_dioxus::Palette;

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
          div { class: "collapse collapse-arrow border border-base-300 bg-base-200 text-base-content",
            input {
              r#type: "radio",
              name: "customize-accordion",
              checked: true,
            }
            div { class: "collapse-title font-semibold", "Logo" }
            div { class: "collapse-content text-base-content/70", "Custom Mechvibes logo" }
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
