//! "Update ready — Restart now / Later" prompt.
//!
//! Rendered from the titlebar so it is reachable from every page without a
//! toast system (the app has none, and adding one is out of scope). It is a
//! DaisyUI modal driven entirely by [`UpdateStage`]; when staging failed or is
//! unsupported it renders nothing, and the plain download link in the titlebar
//! remains the way out.
//!
//! Choosing "Later" only hides the modal for this session - the verified
//! installer stays on disk and `restore_staged_update()` brings the prompt
//! back on the next launch.

use crate::utils::auto_updater::{
    dismiss_staged_update,
    get_update_stage,
    install_staged_update,
    UpdateStage,
};
use dioxus::desktop::use_window;
use dioxus::prelude::*;
use lucide_dioxus::{ CircleCheck, RefreshCw };

/// Modal shown when a verified installer is waiting.
#[component]
pub fn UpdatePrompt() -> Element {
    let window = use_window();

    // Polled rather than pushed: the stage is written from background tasks
    // that have no handle on any Dioxus signal, and this is a once-a-second
    // read of a Mutex<enum> - far cheaper than plumbing a channel into the
    // component tree for an event that fires at most once per session.
    let mut stage = use_signal(get_update_stage);
    let mut dismissed = use_signal(|| false);
    let mut install_error = use_signal(|| None::<String>);

    use_future(move || async move {
        loop {
            let current = get_update_stage();
            if *stage.peek() != current {
                stage.set(current);
            }
            futures_timer::Delay::new(std::time::Duration::from_secs(1)).await;
        }
    });

    let UpdateStage::Ready { version, .. } = stage() else {
        return rsx! {};
    };

    if dismissed() {
        return rsx! {};
    }

    let restart_now = move |_| {
        match install_staged_update() {
            Ok(()) => {
                // The installer is running detached and will close whatever
                // still holds the binary, then relaunch us. Close the window
                // ourselves anyway so the shutdown is orderly: this is the
                // same path the tray's Exit uses, and it drops the input
                // worker's stdin pipe, which makes that child exit too.
                window.close();
            }
            Err(e) => {
                eprintln!("❌ Could not start the update installer: {}", e);
                install_error.set(
                    Some(
                        format!(
                            "Could not start the installer ({}). You can download the update manually instead.",
                            e
                        )
                    )
                );
            }
        }
    };

    rsx! {
      div { class: "modal modal-open z-1000",
        div { class: "modal-box",
          h3 { class: "font-bold text-lg flex items-center gap-2",
            CircleCheck { class: "w-5 h-5 text-success" }
            "Update {version} is ready"
          }
          p { class: "py-3 text-sm text-base-content/80",
            "The update has been downloaded and verified. Restarting takes a few seconds; your settings and soundpacks are kept."
          }
          if let Some(error) = install_error() {
            div { class: "alert alert-error text-sm mb-2", "{error}" }
          }
          div { class: "modal-action",
            button {
              class: "btn btn-ghost btn-sm",
              onclick: move |_| {
                  // Session-local only: the file stays staged and the prompt
                  // returns next launch.
                  dismissed.set(true);
              },
              "Later"
            }
            button {
              class: "btn btn-success btn-sm",
              onclick: restart_now,
              RefreshCw { class: "w-4 h-4 mr-1" }
              "Restart now"
            }
          }
          div { class: "text-xs text-base-content/50 mt-2",
            button {
              class: "link link-hover",
              onclick: move |_| {
                  // Explicit opt-out: throw the download away entirely.
                  dismiss_staged_update();
                  dismissed.set(true);
              },
              "Discard this download"
            }
          }
        }
      }
    }
}
