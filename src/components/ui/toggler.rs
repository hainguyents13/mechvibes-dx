use dioxus::prelude::*;

#[derive(Props, Clone, PartialEq)]
pub struct TogglerProps {
    /// The main label/title for the toggle
    pub title: String,
    /// Optional description text below the title
    pub description: Option<String>,
    /// Whether the toggle is currently checked
    pub checked: bool,
    /// Callback when toggle state changes
    pub on_change: EventHandler<bool>,
    /// Optional size variant (sm, md, lg)
    #[props(default = "sm")]
    pub size: &'static str,
    /// Optional color variant (primary, secondary, accent, etc.)
    #[props(default = "")]
    pub variant: &'static str,
    /// Whether the toggle is disabled
    #[props(default = false)]
    pub disabled: bool,
    /// Custom CSS classes for the container
    #[props(default = "")]
    pub class: &'static str,
}

#[component]
pub fn Toggler(props: TogglerProps) -> Element {
    let toggle_class = format!(
        "toggle toggle-{} {}",
        props.size,
        if !props.variant.is_empty() {
            format!("toggle-{}", props.variant)
        } else {
            String::new()
        }
    );

    rsx! {
      label { class: format!("label w-full justify-between cursor-pointer {}", props.class),
        div { class: "space-y-1 flex-1 pr-4",
          div { class: "text-sm font-medium text-base-content", "{props.title}" }
          if let Some(description) = &props.description {
            div { class: "text-xs whitespace-break-spaces text-base-content/70",
              "{description}"
            }
          }
        }
        input {
          r#type: "checkbox",
          class: "{toggle_class}",
          checked: props.checked,
          disabled: props.disabled,
          onchange: move |evt| {
              props.on_change.call(evt.checked());
          },
        }
      }
    }
}
