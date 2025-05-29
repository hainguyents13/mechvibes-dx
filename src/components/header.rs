use dioxus::prelude::*;

const FAVICON: Asset = asset!("/assets/icon.ico");
const MAIN_CSS: &str = include_str!("../../assets/main.css");
const TAILWIND_CSS: &str = include_str!("../../assets/tailwind.css");

#[component]
pub fn Header() -> Element {
    rsx! {
      document::Link { rel: "icon", href: FAVICON }
      document::Style { {TAILWIND_CSS} }
      document::Style { {MAIN_CSS} }
    }
}
