use crate::state::app::use_update_info;
use dioxus::prelude::*;
use lucide_dioxus::Download;

#[component]
pub fn TitleBar() -> Element {
    let update_info = use_update_info();

    rsx! {
      div {
        class: "fixed inset-x-0 top-0 h-8 z-999 flex items-center select-none",
        style: "-webkit-app-region: drag; app-region: drag;",

        // Right side — update badge only
        div {
          class: "flex items-center ml-auto pr-3 gap-2",
          style: "-webkit-app-region: no-drag; app-region: no-drag;",
          if let Some(update) = update_info.clone() {
            if update.update_available {
              div {
                class: "tooltip tooltip-bottom",
                "data-tip": "New version {update.latest_version} available!",
                if let Some(url) = &update.download_url {
                  button {
                    class: "btn btn-success btn-xs",
                    onclick: {
                        let url = url.clone();
                        move |_| {
                            if let Err(e) = open::that(&url) {
                              eprintln!("Failed to open URL: {}", e);
                            }
                        }
                    },
                    Download { class: "w-3 h-3" }
                    "{update.latest_version}"
                  }
                }
              }
            }
          }
        }
      }
    }
}
