use crate::libs::theme::{use_theme, Theme};
use dioxus::document::eval;
use dioxus::prelude::*;

const FAVICON: Asset = asset!("/assets/icon.ico");
const MAIN_CSS: &str = include_str!("../../assets/main.css");
const TAILWIND_CSS: &str = include_str!("../../assets/tailwind.css");

#[component]
pub fn Header() -> Element {
    use crate::state::config_utils::use_config;

    let (config, _) = use_config();
    let theme = use_theme();

    // Use effect to inject CSS when theme or config changes
    use_effect(move || {
        let custom_css = config().custom_css.clone();

        // Get custom theme CSS if current theme is custom
        let custom_theme_css = if let Theme::Custom(theme_name) = &theme() {
            config()
                .get_custom_theme(theme_name)
                .map(|theme_data| theme_data.css.clone())
                .unwrap_or_default()
        } else {
            String::new()
        };

        // Combine all CSS
        let combined_css = format!(
            "{}\n{}\n{}\n{}",
            TAILWIND_CSS, MAIN_CSS, custom_theme_css, custom_css
        );

        // Inject CSS using eval (works in Dioxus desktop)
        let script = format!(
            r#"
            // Remove existing custom style if any
            const existingStyle = document.getElementById('mechvibes-custom-styles');
            if (existingStyle) {{
                existingStyle.remove();
            }}
            
            // Create new style element
            const style = document.createElement('style');
            style.id = 'mechvibes-custom-styles';
            style.textContent = `{}`;
            document.head.appendChild(style);
            "#,
            combined_css.replace('`', r#"\`"#).replace("${", r#"\${"#)
        );

        eval(&script);
    });

    rsx! {
      document::Link { rel: "icon", href: FAVICON }
    }
}
