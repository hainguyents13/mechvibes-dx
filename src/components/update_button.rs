//! The "Download & install" button and its state machine.
//!
//! Downloading is a deliberate user action: a background check only ever
//! *notifies*, and nothing is transferred until this button is clicked. The
//! whole flow is surfaced inside the button itself rather than in separate
//! alerts, so there is exactly one place to look:
//!
//! ```text
//! Idle       "Download & install v0.7.0"
//!   click -> Downloading  spinner + "Downloading new version..." (disabled)
//!         -> Ready        "Restart to finish update" + "Later"
//!   click -> Installing   "Installing..." (disabled), app exits
//!         -> Failed       short reason + "Open download page"
//! ```
//!
//! `Failed` never leaves the user stuck: the browser link that predates the
//! staged-download flow is always the way out.

use crate::utils::auto_updater::{
    download_and_stage_update,
    get_update_stage,
    install_staged_update,
    AutoUpdater,
    UpdateInfo,
    UpdateStage,
};
use dioxus::desktop::use_window;
use dioxus::prelude::*;
use lucide_dioxus::{ Download, RefreshCw };

/// Renders the install control for an available update.
///
/// On platforms without a silent installer this collapses to a plain link to
/// the release page - there is no in-place upgrade path to offer there.
#[component]
pub fn UpdateInstallButton(info: UpdateInfo) -> Element {
    let window = use_window();

    // The stage is written from an async task with no handle on any signal,
    // so it is polled. Half a second is well under the time any step takes
    // and costs a single Mutex read.
    let mut stage = use_signal(get_update_stage);
    let mut deferred = use_signal(|| false);

    use_future(move || async move {
        loop {
            let current = get_update_stage();
            if *stage.peek() != current {
                stage.set(current);
            }
            futures_timer::Delay::new(std::time::Duration::from_millis(500)).await;
        }
    });

    let version = info.latest_version.clone();
    let release_page = AutoUpdater::releases_page_url(&version);

    // No silent installer on this platform: link out, never offer a download
    // that could not be applied anyway.
    if !crate::utils::update_installer::silent_update_supported() {
        return rsx! {
          a {
            href: "{release_page}",
            target: "_blank",
            class: "btn btn-success btn-sm",
            Download { class: "w-4 h-4 mr-1" }
            "Get v{version}"
          }
        };
    }

    let fallback_link = rsx! {
      a {
        href: "{release_page}",
        target: "_blank",
        class: "link link-hover text-xs",
        "Open download page"
      }
    };

    match stage() {
        UpdateStage::Downloading { version } =>
            rsx! {
          button { class: "btn btn-success btn-sm", disabled: true,
            span { class: "loading loading-spinner loading-xs mr-1" }
            "Downloading new version..."
          }
          div { class: "text-xs text-base-content/50 mt-1", "v{version}" }
        },

        // Verified and on disk. Confirming the restart is a second, explicit
        // click - the app is never torn down from under the user.
        UpdateStage::Ready { version, .. } if !deferred() =>
            rsx! {
          div { class: "flex items-center gap-2",
            button {
              class: "btn btn-success btn-sm",
              onclick: move |_| {
                  match install_staged_update() {
                      Ok(()) => {
                          // Installer is running detached; close the app the
                          // same way the tray Exit does, which also drops the
                          // input worker's stdin and takes that child with it.
                          window.close();
                      }
                      Err(e) => {
                          crate::always_eprint!("❌ Could not start the update installer: {}", e);
                      }
                  }
              },
              RefreshCw { class: "w-4 h-4 mr-1" }
              "Restart to finish update"
            }
            button {
              class: "btn btn-ghost btn-sm",
              onclick: move |_| {
                  // Session-local only. The verified file stays staged, so
                  // returning to this button (or restarting the app) picks it
                  // up without downloading again.
                  deferred.set(true);
              },
              "Later"
            }
          }
          div { class: "text-xs text-base-content/50 mt-1",
            "v{version} downloaded and verified."
          }
        },

        // "Later" was chosen: offer the restart again without re-downloading.
        UpdateStage::Ready { version, .. } =>
            rsx! {
          button {
            class: "btn btn-success btn-soft btn-sm",
            onclick: move |_| {
                deferred.set(false);
            },
            RefreshCw { class: "w-4 h-4 mr-1" }
            "Install v{version} now"
          }
          div { class: "text-xs text-base-content/50 mt-1",
            "Already downloaded - installing takes a few seconds."
          }
        },

        UpdateStage::Failed { reason, .. } =>
            rsx! {
          div { class: "space-y-1",
            div { class: "text-sm text-warning", "{reason}" }
            {fallback_link}
          }
        },

        UpdateStage::Idle =>
            rsx! {
          button {
            class: "btn btn-success btn-sm",
            onclick: {
                let info = info.clone();
                move |_| {
                    let info = info.clone();
                    spawn(async move {
                        download_and_stage_update(&info).await;
                    });
                }
            },
            Download { class: "w-4 h-4 mr-1" }
            "Download & install v{version}"
          }
        },
    }
}
