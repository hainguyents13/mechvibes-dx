use dioxus::prelude::*;

const FAVICON: Asset = asset!("/assets/favicon.ico");
const MAIN_CSS: &str = include_str!("../../assets/main.css");
const TAILWIND_CSS: &str = include_str!("../../assets/tailwind.css");

#[component]
pub fn app() -> Element {
    rsx! {
      // The Stylesheet component inserts a style link into the head of the document
      document::Link { rel: "icon", href: FAVICON }
      document::Style { {MAIN_CSS} }
      document::Style { {TAILWIND_CSS} }
    }
}
