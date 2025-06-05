use dioxus::prelude::*;
use lucide_dioxus::{TriangleAlert, X};

#[component]
pub fn ConfirmDeleteModal(
    show: Signal<bool>,
    soundpack_name: String,
    on_confirm: EventHandler<()>,
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
          onclick: move |_| show.set(false),
        }

        // Modal content
        div { class: "relative bg-base-100 rounded-box shadow-xl p-6 w-full max-w-md mx-4",
          // Header
          div { class: "flex items-center justify-between mb-4",
            h3 { class: "text-lg font-semibold text-base-content", "Delete Soundpack" }
            button {
              class: "btn btn-ghost btn-sm btn-circle",
              onclick: move |_| show.set(false),
              X { class: "w-4 h-4" }
            }
          }

          // Content
          div { class: "space-y-4",
            // Warning icon and message
            div { class: "flex items-start gap-3",
              div { class: "flex-shrink-0 w-10 h-10 rounded-full bg-error/10 flex items-center justify-center",
                TriangleAlert { class: "w-5 h-5 text-error" }
              }
              div { class: "flex-1",
                div { class: "font-medium text-base-content mb-1",
                  "Are you sure you want to delete this soundpack?"
                }
                div { class: "text-sm text-base-content/70 mb-3",
                  "This action cannot be undone. The soundpack \"{soundpack_name}\" and all its files will be permanently removed."
                }
              }
            }

            // Action buttons
            div { class: "flex justify-end gap-2 pt-2",
              button {
                class: "btn btn-ghost",
                onclick: move |_| show.set(false),
                "Cancel"
              }
              button {
                class: "btn btn-error",
                onclick: move |_| {
                    show.set(false);
                    on_confirm.call(());
                },
                "Delete"
              }
            }
          }
        }
      }
    }
}
