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
}

#[component]
pub fn Collapse(props: CollapseProps) -> Element {
    let container_class = format!("collapse collapse-arrow {} {}", props.variant, props.class);

    rsx! {
        div { class: "{container_class}",
            input {
                r#type: "radio",
                name: "{props.group_name}",
                checked: props.default_open,
            }
            div { class: "{props.title_class}",
                "{props.title}"
            }
            div { class: "{props.content_class}",
                {props.children}
            }
        }
    }
}
