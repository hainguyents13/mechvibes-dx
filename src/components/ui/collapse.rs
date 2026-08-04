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
    /// Only build `children` while this section is expanded.
    ///
    /// This is a DaisyUI CSS accordion: the radio input drives visibility
    /// through stylesheet rules, so by default every section's children are
    /// mounted in the DOM at all times and merely hidden. That is fine for
    /// static content, but a section whose content runs a poll loop or holds
    /// other live state would keep paying for it while collapsed. Opting in
    /// here makes the content mount on expand and unmount on collapse, which
    /// drops any `use_future` it owns.
    #[props(default = false)]
    pub lazy_children: bool,
}

/// Whether `children` should be built this render.
///
/// Split out from the component so the rule can be asserted directly: an
/// ordinary section always renders (preserving the CSS-accordion behavior every
/// existing caller relies on), and a lazy one renders only while expanded.
fn should_render_children(lazy: bool, is_open: bool) -> bool {
    !lazy || is_open
}

#[component]
pub fn Collapse(props: CollapseProps) -> Element {
    let container_class = format!(
        "collapse relative w-full collapse-arrow {} {}",
        props.variant,
        props.class
    );

    // Only tracked for lazy sections; eager ones render exactly as before.
    let mut is_open = use_signal(|| props.default_open);
    let render_children = should_render_children(props.lazy_children, is_open());

    rsx! {
        div { class: "{container_class}",
            input {
                r#type: "radio",
                name: "{props.group_name}",
                checked: props.default_open,
                // The radio drives the CSS either way. For a lazy section we
                // additionally mirror its state into a signal so the content
                // can be mounted and unmounted with it. `onchange` fires only
                // when this input becomes the selected one; the sibling that
                // gets deselected is not notified, so collapsing is handled by
                // `onclick` on the title below.
                onchange: move |_| {
                    if props.lazy_children {
                        is_open.set(true);
                    }
                },
            }
            div {
                class: "{props.title_class}",
                // Clicking the title of the section that is already open
                // collapses it, and the radio reports no change for that. Track
                // it here so the content unmounts rather than lingering.
                onclick: move |_| {
                    if props.lazy_children {
                        let open = is_open();
                        is_open.set(!open);
                    }
                },
                span { class: "flex items-center gap-2",
                    "{props.title}"
                    if props.show_indicator {
                        span { class: "w-2 h-2 rounded-full bg-primary" }
                    }
                }
            }
            div { class: "{props.content_class}",
                if render_children {
                    {props.children}
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::should_render_children;

    #[test]
    fn ordinary_sections_always_render_their_children() {
        // Every pre-existing caller depends on this: the CSS accordion hides
        // collapsed content rather than unmounting it, and changing that for
        // non-opted-in sections would alter how they behave.
        assert!(should_render_children(false, true));
        assert!(should_render_children(false, false));
    }

    #[test]
    fn lazy_sections_render_only_while_expanded() {
        // What makes the Debug viewer's poll loop stop when the section is
        // closed. If this ever returns true while collapsed, that loop runs
        // for every user forever.
        assert!(should_render_children(true, true));
        assert!(!should_render_children(true, false));
    }
}
