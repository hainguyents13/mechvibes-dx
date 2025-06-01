use dioxus::prelude::*;

const FAVICON: Asset = asset!("/assets/icon.ico");
const MAIN_CSS: &str = include_str!("../../assets/main.css");
const TAILWIND_CSS: &str = include_str!("../../assets/tailwind.css");

#[component]
pub fn Header() -> Element {
    use crate::state::config_utils::use_config;
    let (config, _) = use_config();
    let custom_css = config().custom_css.clone();

    rsx! {
      document::Link { rel: "icon", href: FAVICON }
      document::Style { {TAILWIND_CSS} }
      document::Style { {MAIN_CSS} }
      if !custom_css.is_empty() {
        document::Style { {custom_css} }
      }
    }
}
