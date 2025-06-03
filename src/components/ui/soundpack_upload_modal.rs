use dioxus::prelude::*;
use lucide_dioxus::{Upload, X};

#[component]
pub fn SoundpackUploadModal(
    show: Signal<bool>,
    progress: Signal<String>,
    error: Signal<String>,
    success: Signal<String>,
    on_upload: EventHandler<MouseEvent>,
) -> Element {
    if !show() {
        return rsx! {
            div {}
        };
    }

    rsx! {
        div { class: "fixed inset-0 z-50 flex items-center justify-center",
            // Backdrop
            div {
                class: "absolute inset-0 bg-black/50",
                onclick: move |_| {
                    if progress().is_empty() {
                        show.set(false);
                    }
                },
            }

            // Modal content
            div { class: "relative bg-base-100 rounded-lg shadow-xl p-6 w-full max-w-md mx-4",
                // Header
                div { class: "flex items-center justify-between mb-4",
                    h3 { class: "text-lg font-semibold text-base-content", "Upload Soundpack" }
                    if progress().is_empty() {
                        button {
                            class: "btn btn-ghost btn-sm btn-circle",
                            onclick: move |_| show.set(false),
                            X { class: "w-4 h-4" }
                        }
                    }
                }

                // Content
                div { class: "space-y-4",
                    // Instructions
                    if progress().is_empty() && error().is_empty() && success().is_empty() {
                        div { class: "text-sm text-base-content/70",
                            "Select a ZIP file containing a soundpack to install it."
                            br {}
                            "Supports both V1 and V2 soundpack formats."
                        }
                    }

                    // Progress
                    if !progress().is_empty() {
                        div { class: "flex items-center gap-3",
                            span { class: "loading loading-spinner loading-sm" }
                            span { class: "text-sm text-base-content",
                                "{progress()}"
                            }
                        }
                    }

                    // Error
                    if !error().is_empty() {
                        div { class: "alert alert-error",
                            div { class: "text-sm",
                                "{error()}"
                            }
                        }
                    }

                    // Success
                    if !success().is_empty() {
                        div { class: "alert alert-success",
                            div { class: "text-sm",
                                "{success()}"
                            }
                        }
                    }

                    // Upload button
                    if progress().is_empty() && success().is_empty() {
                        div { class: "flex justify-end gap-2",
                            button {
                                class: "btn btn-ghost",
                                onclick: move |_| show.set(false),
                                "Cancel"
                            }
                            button {
                                class: "btn btn-primary",
                                onclick: move |evt| on_upload.call(evt),
                                Upload { class: "w-4 h-4 mr-2" }
                                "Select File"
                            }
                        }
                    }
                }
            }
        }
    }
}
