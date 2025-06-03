use crate::state::soundpack_cache::SoundpackMetadata;
use dioxus::prelude::*;
use lucide_dioxus::{FolderOpen, Music, Plus, Trash};

#[component]
pub fn SoundpackTable(
    soundpacks: Vec<SoundpackMetadata>,
    soundpack_type: &'static str,
    on_add_click: Option<EventHandler<MouseEvent>>,
) -> Element {
    // Search state
    let mut search_query = use_signal(String::new);

    // Clone soundpacks for use in memo
    let soundpacks_clone = soundpacks.clone();

    // Filter soundpacks based on search query
    let filtered_soundpacks = use_memo(move || {
        let query = search_query().to_lowercase();
        if query.is_empty() {
            soundpacks_clone.clone()
        } else {
            soundpacks_clone
                .iter()
                .filter(|pack| {
                    pack.name.to_lowercase().contains(&query)
                        || pack.id.to_lowercase().contains(&query)
                        || pack
                            .author
                            .as_ref()
                            .map_or(false, |author| author.to_lowercase().contains(&query))
                        || pack
                            .tags
                            .iter()
                            .any(|tag| tag.to_lowercase().contains(&query))
                })
                .cloned()
                .collect()
        }
    });

    if soundpacks.is_empty() {
        rsx! {
            div { class: "p-4 text-center text-base-content/70",
                "No {soundpack_type} soundpack found!"
            }
        }
    } else {
        rsx! {
            div { class: "space-y-4",
                // Search field
                div { class: "flex items-center px-3 gap-2",
                    input {
                        class: "input w-full",
                        placeholder: "Search {soundpack_type.to_lowercase()} soundpacks...",
                        value: "{search_query}",
                        oninput: move |evt| search_query.set(evt.value()),
                    }
                    if let Some(add_handler) = on_add_click {
                        button {
                            class: "btn btn-primary",
                            onclick: move |evt| add_handler.call(evt),
                            Plus { class: "w-4 h-4 mr-2" }
                            "Add"
                        }
                    }
                }

                // Table
                div { class: "overflow-x-auto max-h-[calc(100vh-500px)]",
                    if filtered_soundpacks().is_empty() {
                        div { class: "p-4 text-center text-base-content/70",
                            "No result match your search!"
                        }
                    } else {
                        table { class: "table table-sm w-full",
                            tbody {
                                for pack in filtered_soundpacks() {
                                    SoundpackTableRow { soundpack: pack }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
pub fn SoundpackTableRow(soundpack: SoundpackMetadata) -> Element {
    rsx! {
        tr { class: "hover:bg-base-100",
            td { class: "flex items-center gap-4",
                // Icon
                div { class: "flex items-center justify-center",
                    if let Some(icon) = &soundpack.icon {
                        if !icon.is_empty() {
                            div { class: "w-8 h-8 rounded-lg overflow-hidden",
                                img {
                                    class: "w-full h-full object-cover",
                                    src: "{icon}",
                                    alt: "{soundpack.name}",
                                }
                            }
                        } else {
                            div { class: "w-8 h-8 rounded bg-base-300 flex items-center justify-center",
                                Music { class: "w-4 h-4 text-base-content/40" }
                            }
                        }
                    } else {
                        div { class: "w-8 h-8 rounded bg-base-300 flex items-center justify-center",
                            Music { class: "w-4 h-4 text-base-content/40" }
                        }
                    }
                }
                // Name
                div {
                    div { class: "font-medium text-sm text-base-content line-clamp-1",
                        "{soundpack.name}"
                    }
                    if let Some(author) = &soundpack.author {
                        div { class: "text-xs text-base-content/50", "by {author}" }
                    }
                }
            }
            // Actions
            td {
                div { class: "flex items-center justify-end gap-2",
                    button {
                        class: "btn btn-soft btn-xs",
                        title: "Open soundpack folder",
                        FolderOpen { class: "w-4 h-4" }
                    }
                    button {
                        class: "btn btn-error btn-xs",
                        title: "Delete this soundpack",
                        Trash { class: "w-4 h-4" }
                    }
                }
            }
        }
    }
}
