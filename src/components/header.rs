use crate::libs::theme::{ use_theme, Theme };
use crate::utils::theme::use_themes;
use crate::utils::constants::CSS_ID_PREFIX;
use dioxus::document::eval;
use dioxus::prelude::*;

const FAVICON: Asset = asset!("/assets/icon.ico");
const GLOBAL_STYLES_CSS: &str = include_str!("../../assets/style.css");

// Font assets
const LATO_REGULAR: Asset = asset!("/assets/fonts/Lato-Regular.ttf");
const LATO_BOLD: Asset = asset!("/assets/fonts/Lato-Bold.ttf");
const LATO_ITALIC: Asset = asset!("/assets/fonts/Lato-Italic.ttf");
const LATO_BOLD_ITALIC: Asset = asset!("/assets/fonts/Lato-BoldItalic.ttf");
const LATO_BLACK: Asset = asset!("/assets/fonts/Lato-Black.ttf");

#[component]
pub fn Header() -> Element {
    use crate::utils::config::use_config;

    let (config, _) = use_config();
    let (themes, _) = use_themes();
    let theme = use_theme();

    // Use effect to inject fonts and dynamic CSS
    use_effect(move || {
        println!("🎨 Header: Injecting fonts and dynamic CSS");
        let custom_css = config().custom_css.clone();

        // Create font-face declarations using Manganis assets
        let font_css = format!(
            r#"
            @font-face {{
                font-family: "Lato";
                src: url("{}") format("truetype");
                font-weight: 400;
                font-style: normal;
            }}
            @font-face {{
                font-family: "Lato";
                src: url("{}") format("truetype");
                font-weight: 700;
                font-style: normal;
            }}
            @font-face {{
                font-family: "Lato";
                src: url("{}") format("truetype");
                font-weight: 400;
                font-style: italic;
            }}
            @font-face {{
                font-family: "Lato";
                src: url("{}") format("truetype");
                font-weight: 700;
                font-style: italic;
            }}
            @font-face {{
                font-family: "Lato";
                src: url("{}") format("truetype");
                font-weight: 900;
                font-style: normal;
            }}
            body {{
                font-family: "Lato", sans-serif;
            }}
            "#,
            LATO_REGULAR,
            LATO_BOLD,
            LATO_ITALIC,
            LATO_BOLD_ITALIC,
            LATO_BLACK
        );

        // Get custom theme CSS if current theme is custom
        let custom_theme_css = if let Theme::Custom(theme_id) = &theme() {
            if let Some(theme_data) = themes().get_theme_by_id(theme_id) {
                // Wrap custom theme CSS with proper data-theme selectors
                format!("[data-theme=\"custom-{}\"] {{\n{}\n}}", theme_id, theme_data.css)
            } else {
                String::new()
            }
        } else {
            String::new()
        };

        // Combine all dynamic CSS parts
        let dynamic_css = format!("{}\n{}\n{}", font_css, custom_theme_css, custom_css); // Inject only dynamic CSS using eval
        let script = format!(
            r#"
              // Remove existing custom style if any
              const existingStyle = document.getElementById('{}-custom-styles');
              if (existingStyle) {{
                  existingStyle.remove();
              }}
              
              // Create new style element for dynamic CSS
              const style = document.createElement('style');
              style.id = '{}-custom-styles';
              style.textContent = `{}`;
              document.head.appendChild(style);
            "#,
            CSS_ID_PREFIX,
            CSS_ID_PREFIX,
            dynamic_css.replace('`', r#"\`"#).replace("${", r#"\${"#)
        );

        eval(&script);
    });

    rsx! {
      // prettier-ignore
      document::Link { rel: "icon", r#type: "image/x-icon", href: FAVICON }

      // Inline global styles since asset!() for CSS doesn't work in desktop apps
      style { dangerous_inner_html: GLOBAL_STYLES_CSS }
    }
}
