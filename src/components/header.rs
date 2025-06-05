use crate::libs::theme::{use_theme, Theme};
use crate::utils::theme::use_themes;
use dioxus::document::eval;
use dioxus::prelude::*;

const FAVICON: Asset = asset!("/assets/icon.ico");
const GLOBAL_STYLES: Asset = asset!("/assets/style.css");

#[component]
pub fn Header() -> Element {
    use crate::utils::config::use_config;

    let (config, _) = use_config();
    let (themes, _) = use_themes();
    let theme = use_theme();

    // Use effect to inject only dynamic CSS (custom theme CSS and custom CSS)
    use_effect(move || {
        let custom_css = config().custom_css.clone();
        // Get custom theme CSS if current theme is custom
        let custom_theme_css = if let Theme::Custom(theme_id) = &theme() {
            if let Some(theme_data) = themes().get_theme_by_id(theme_id) {
                // Wrap custom theme CSS with proper data-theme selectors
                format!(
                    ":root:has(input.theme-controller[value=custom-{}]:checked),[data-theme=\"custom-{}\"] {{\n{}\n}}",
                    theme_id,
                    theme_id,
                    theme_data.css
                )
            } else {
                String::new()
            }
        } else {
            String::new()
        };

        // Only combine dynamic CSS parts
        let dynamic_css = format!("{}\n{}", custom_theme_css, custom_css);

        // Inject only dynamic CSS using eval
        let script = format!(
            r#"
              // Remove existing custom style if any
              const existingStyle = document.getElementById('mechvibes-custom-styles');
              if (existingStyle) {{
                  existingStyle.remove();
              }}
              
              // Create new style element for dynamic CSS
              const style = document.createElement('style');
              style.id = 'mechvibes-custom-styles';
              style.textContent = `{}`;
              document.head.appendChild(style);
            "#,
            dynamic_css.replace('`', r#"\`"#).replace("${", r#"\${"#)
        );

        eval(&script);
    });
    rsx! {
      document::Link { rel: "icon", href: FAVICON }
      document::Link { rel: "stylesheet", href: GLOBAL_STYLES }
    }
}
