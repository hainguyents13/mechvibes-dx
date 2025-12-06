use dioxus::prelude::*;

#[derive(Props, Clone, PartialEq)]
pub struct CollapseProps {
    /// The title displayed in the collapse header
    pub title: String,
    /// The content to be displayed when expanded
    pub children: Element,
    /// The radio group name for accordion behavior
    pub group_name: String,
    /// Whether this collapse item is expanded by default
    #[props(default = false)]
    pub default_open: bool,
    /// Optional variant for styling (border, base-300, etc.)
    #[props(default = "border border-base-300 bg-base-200 text-base-content")]
    pub variant: &'static str,
    /// Optional title styling
    #[props(default = "collapse-title font-semibold")]
    pub title_class: &'static str,
    /// Optional content styling
    #[props(default = "collapse-content")]
    pub content_class: &'static str,
    /// Custom CSS classes for the container
    #[props(default = "")]
    pub class: &'static str,
    /// Show indicator dot when enabled
    #[props(default = false)]
    pub show_indicator: bool,
}

#[component]
pub fn Collapse(props: CollapseProps) -> Element {
    let container_class = format!(
        "collapse relative w-full collapse-arrow {} {}",
        props.variant,
        props.class
    );

    rsx! {
        div { class: "{container_class}",
            input {
                r#type: "radio",
                name: "{props.group_name}",
                checked: props.default_open,
            }
            div { class: "{props.title_class}",
                span { class: "flex items-center gap-2",
                    "{props.title}"
                    if props.show_indicator {
                        span { class: "w-2 h-2 rounded-full bg-primary" }
                    }
                }
            }
            div { class: "{props.content_class}",
                {props.children}
            }
        }
    }
}
