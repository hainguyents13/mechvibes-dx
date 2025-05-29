use dioxus::prelude::*;

#[component]
pub fn CustomizePage() -> Element {
    rsx! {
      div { class: "container mx-auto p-16 text-center flex flex-col gap-6",
        // Page header
        div { class: "mb-8",
          h1 { class: "text-3xl font-bold text-base-content mb-4", "🎨 Themes" }
          p { class: "text-base-content/70 text-lg",
            "Discover and manage your application's themes."
          }
        }

        // Temporary placeholder content
        div { class: "flex-1 flex items-center justify-center min-h-96",
          div { class: "text-center",
            div { class: "text-6xl mb-4", "🚧" }
            h2 { class: "text-2xl font-semibold text-base-content mb-3",
              "Coming Soon"
            }
            p { class: "text-base-content/60 max-w-md",
              "This page will feature a comprehensive theme browser, installation manager, and community theme discovery. For now, you can manage your themes from the Home page."
            }
          }
        }
      }
    }
}
