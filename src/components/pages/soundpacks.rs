use dioxus::prelude::*;

#[component]
pub fn SoundpacksPage() -> Element {
    use crate::libs::theme::use_effective_theme;
    let effective_theme = use_effective_theme();

    rsx! {
        div { class: "container mx-auto p-16 text-center flex flex-col gap-6",
            // Page header
            div { class: "mb-8",
                h1 { class: format!("text-3xl font-bold {} mb-4", effective_theme.text_primary()), "🎵 Soundpacks" }
                p { class: format!("{} text-lg", effective_theme.text_secondary()),
                    "Discover and manage your mechanical keyboard sound profiles."
                }
            }

            // Temporary placeholder content
            div { class: "flex-1 flex items-center justify-center min-h-96",
                div { class: "text-center",
                    div { class: "text-6xl mb-4", "🚧" }
                    h2 { class: format!("text-2xl font-semibold {} mb-3", effective_theme.text_primary()), "Coming Soon" }
                    p { class: format!("{} max-w-md", effective_theme.text_tertiary()),
                        "This page will feature a comprehensive soundpack browser, installation manager, and community soundpack discovery. For now, you can manage your soundpacks from the Home page."
                    }
                }
            }
        }
    }
}
