//! The Debug section's live log viewer, export button and verbose toggle.
//!
//! # Why polling rather than a subscription
//!
//! Log lines are produced on threads that cannot touch a Dioxus `Signal` (the
//! audio engine thread, the worker-host reader, detached update/telemetry
//! threads), exactly as with the config in `libs/ui.rs`. So the viewer follows
//! the same house pattern: the buffer bumps an atomic counter on every push,
//! and this component polls that counter. One atomic load per tick, and the
//! lines are only cloned when the counter actually moved.
//!
//! # Cost when the section is closed
//!
//! The poll lives in a `use_future` owned by this component, so it stops when
//! the component unmounts. That unmount is not automatic: the surrounding
//! `Collapse` is a DaisyUI CSS accordion, which normally keeps every section's
//! children in the DOM and only hides them with stylesheet rules. The Debug
//! section therefore passes `lazy_children: true`, which is what actually
//! mounts this component on expand and drops it (and this future) on collapse.
//! Removing that flag would silently leave the poll running for every user.

use crate::components::ui::Toggler;
use crate::utils::delay;
use crate::utils::log_buffer;
use dioxus::prelude::*;

/// How often the viewer checks whether new lines arrived. Fast enough to read
/// like a terminal, slow enough that an idle section costs nothing measurable.
const POLL_INTERVAL_MS: u64 = 250;

#[component]
pub fn DebugLogViewer() -> Element {
    // The rendered tail. Replaced wholesale per refresh, never appended to
    // per line, so a burst of output costs one re-render rather than one per
    // line.
    let mut lines = use_signal(|| log_buffer::recent(log_buffer::VIEWER_LINES));
    let mut verbose = use_signal(log_buffer::verbose_enabled);
    let mut export_status = use_signal(|| None::<Result<String, String>>);

    // Live tail. Mounted only while the section is open (see module docs).
    use_future(move || async move {
        let mut seen = u64::MAX; // forces one refresh on mount
        loop {
            let generation = log_buffer::generation();
            if generation != seen {
                seen = generation;
                lines.set(log_buffer::recent(log_buffer::VIEWER_LINES));
            }
            delay::Delay::ms(POLL_INTERVAL_MS).await;
        }
    });

    rsx! {
      div { class: "space-y-4",
        p { class: "text-sm text-base-content/70",
          "Recent activity from the app, kept in memory only. Installed builds have no console, so this is where log lines go. Nothing is written to disk and nothing leaves your device unless you press Export."
        }

        // The terminal-style tail.
        div {
          class: "mockup-code bg-neutral text-neutral-content text-xs overflow-y-auto overflow-x-auto max-h-64 w-full p-3 rounded-lg",
          // Newest lines sit at the bottom, and the container is pinned there
          // so the view follows output the way a terminal does. `column-reverse`
          // makes the browser keep the scroll anchored at the bottom for free,
          // so no scroll-into-view effect is needed on every refresh.
          style: "display: flex; flex-direction: column-reverse; font-family: ui-monospace, SFMono-Regular, Menlo, Consolas, monospace;",
          div {
            if lines().is_empty() {
              div { class: "opacity-60", "No log lines captured yet." }
            }
            for line in lines() {
              div { class: "whitespace-pre-wrap break-all leading-relaxed", "{line}" }
            }
          }
        }

        div { class: "flex items-center gap-3 flex-wrap",
          button {
            class: "btn btn-soft btn-sm",
            onclick: move |_| {
                match log_buffer::export_to_file() {
                    Ok(path) => {
                        let shown = path.display().to_string();
                        // Best effort: the file exists either way and the
                        // path is shown below, so a file manager that will
                        // not open is not a failed export.
                        log_buffer::reveal_in_file_manager(&path);
                        export_status.set(Some(Ok(shown)));
                    }
                    Err(e) => {
                        export_status.set(Some(Err(e)));
                    }
                }
            },
            "Export logs"
          }
          div { class: "text-xs text-base-content/60",
            "{lines().len()} shown, {log_buffer::len()} kept"
          }
        }

        // Export result. An error is reported inline and never as a crash.
        if let Some(status) = export_status() {
          match status {
              Ok(path) => rsx! {
                div { class: "alert alert-success alert-soft text-xs",
                  div {
                    p { "Log file saved." }
                    p { class: "break-all opacity-80", "{path}" }
                  }
                }
              },
              Err(message) => rsx! {
                div { class: "alert alert-error alert-soft text-xs",
                  "Could not save the log file: {message}"
                }
              },
          }
        }

        div { class: "pt-2 border-t border-base-300",
          Toggler {
            title: "Verbose logging".to_string(),
            description: Some(
                "Adds per-keystroke timing lines, useful for latency reports. Key identities are always masked. Resets to off every time the app starts."
                    .to_string(),
            ),
            checked: verbose(),
            on_change: move |new_value: bool| {
                verbose.set(new_value);
                log_buffer::set_verbose(new_value);
            },
          }
        }
      }
    }
}
