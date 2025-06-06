use dioxus::prelude::*;
use lucide_dioxus::{Check, TriangleAlert};

#[component]
pub fn ConfirmDeleteModal(
    modal_id: String,
    soundpack_name: String,
    on_confirm: EventHandler<()>,
) -> Element {
    rsx! {
      dialog { class: "modal", id: "{modal_id}",
        div { class: "modal-box",
          form { method: "dialog",
            button { class: "btn btn-sm btn-circle btn-ghost absolute right-2 top-2",
              "✕"
            }
          }
          h3 { class: "text-lg font-bold", "Delete soundpack" }

          // Content
          div { class: "space-y-4 mt-6",
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
            }            // Action buttons
            div { class: "flex justify-end gap-2 pt-2",
              form { method: "dialog",
                button { class: "btn btn-ghost", "Cancel" }
              }              form { method: "dialog",
                button {
                  class: "btn btn-error",
                  r#type: "submit",
                  onclick: move |_| {
                      on_confirm.call(());
                  },
                  Check { class: "w-4 h-4 mr-1" }
                  "Delete"
                }
              }
            }
          }
        }
        form { method: "dialog", class: "modal-backdrop",
          button { "close" }
        }
      }
    }
}
