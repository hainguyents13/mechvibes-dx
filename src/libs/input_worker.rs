//! Input worker process entry point (Windows only).
//!
//! The same executable, launched with `--input-worker`, runs this instead
//! of the Dioxus UI. It owns the Raw Input registration (`rawinput_listener`)
//! and streams every key/button transition to its parent over stdout, one
//! line per event.
//!
//! It exists as a separate process because Windows only allows one Raw Input
//! target window per device class per process, and tao/wry claims that slot
//! in the UI process - see `rawinput_listener.rs` module docs. This process
//! never builds a webview, so nothing competes for the registration.
//!
//! ## Wire format (worker -> host, one line per event)
//!
//! ```text
//! K\t{device_id}\t{code}\t{down|up}    keyboard
//! M\t{device_id}\t{code}\t{down|up}    mouse
//! ```
//!
//! `device_id` is the hash `InputDeviceManager` computes for that physical
//! device, or `-` when the lookup failed. `code` is the same string the
//! rdev/evdev listeners produce (`"KeyA"`, `"MouseLeft"`), so the audio
//! engine's parser is untouched. Text rather than a binary encoding so the
//! stream can be read by eye when debugging: run the worker from a terminal
//! and watch the lines.
//!
//! Keyboard events with no source device never reach the wire at all - see
//! `is_physical_keyboard_event`.
//!
//! Deliberately NOT done here: device filtering and hotkey detection. Config
//! lives in the UI process; keeping the only copy there avoids syncing it
//! across the pipe (see `input_worker_host.rs`).
//!
//! ## Lifetime
//!
//! The worker exits when its stdin reaches EOF, which Windows delivers as
//! soon as the parent's pipe handle closes - on clean exit *and* on crash.
//! That is the whole orphan-prevention mechanism; no Job Object needed.
#![cfg(target_os = "windows")]

use std::io::{ BufWriter, Read, Write };

use crate::libs::rawinput_listener::{ run_rawinput_loop, EventKind, RawInputEvent };

/// Command-line flag that selects this mode. Checked in `main()` before any
/// other initialization.
pub const WORKER_ARG: &str = "--input-worker";

/// Runs the worker until stdin closes or the message loop dies. Never
/// returns to normal app startup - callers should exit the process after.
pub fn run() {
    spawn_stdin_lifeline();

    let mut out = BufWriter::new(std::io::stdout());

    let result = run_rawinput_loop(
        Box::new(move |event: RawInputEvent| {
            if !is_physical_keyboard_event(event.kind, event.device_id.as_deref()) {
                return;
            }

            let kind = match event.kind {
                EventKind::Keyboard => 'K',
                EventKind::Mouse => 'M',
            };
            let device_id = event.device_id.as_deref().unwrap_or("-");
            let direction = if event.is_down { "down" } else { "up" };

            // Flushed per event on purpose: a BufWriter that only flushes
            // when full would hold keystrokes back until the next burst,
            // which is exactly the latency this architecture exists to
            // remove. The write is a few dozen bytes into a pipe.
            if
                writeln!(out, "{}\t{}\t{}\t{}", kind, device_id, event.code, direction).is_err() ||
                out.flush().is_err()
            {
                // Parent went away mid-write; nothing left to serve.
                std::process::exit(0);
            }
        })
    );

    if let Err(e) = result {
        eprintln!("[worker] Raw Input listener failed: {}", e);
        std::process::exit(1);
    }
}

/// Whether a keyboard event came from a real keyboard, rather than being
/// injected by software.
///
/// Raw Input reports a NULL `hDevice` for synthetic keyboard input
/// (`SendInput` and friends); a real keystroke always carries the handle of
/// the device that produced it. `device_id` is `None` exactly when that
/// handle would not resolve, so "no source device" is the injection signal.
/// Verified on Windows 11: `GetRawInputDeviceInfoW(NULL, RIDI_DEVICENAME, ..)`
/// fails with `ERROR_INVALID_HANDLE`, while every attached keyboard resolves
/// to a name.
///
/// This exists for Vietnamese (and other) IMEs. Typing `dd` in Telex makes the
/// IME rewrite the buffer by injecting backspaces plus the replacement
/// character within a few milliseconds; unfiltered, that burst played as a
/// machine-gun clump of sounds for keys the user never pressed. The user
/// already heard the physical keystroke that triggered the correction, so the
/// correction itself should be silent.
///
/// Mouse events are passed through untouched: no IME rewrites mouse input, and
/// the same NULL handle appears for legitimate synthetic clicks that users do
/// expect to hear.
///
/// Known trade-off, deliberately not configurable for now: input from an
/// on-screen keyboard, AutoHotkey, or a remote-desktop client is also
/// injected, so it is silent too. A setting can be added if anyone asks.
fn is_physical_keyboard_event(kind: EventKind, device_id: Option<&str>) -> bool {
    match kind {
        EventKind::Keyboard => device_id.is_some(),
        EventKind::Mouse => true,
    }
}

/// Exits the process when the parent's end of our stdin pipe closes.
///
/// This is the orphan guard: if the UI process exits or crashes, Windows
/// closes its handles, our stdin hits EOF, and we go away with it. Runs on
/// its own thread because the Raw Input message loop owns the main one.
///
/// Note when debugging by hand: launching the worker without giving it a
/// live stdin (e.g. `Start-Process` with stdout redirected but no stdin
/// pipe) makes it exit immediately, because that stdin reports EOF at once.
/// That is the guard working, not a crash - to watch its output, spawn it
/// with stdin piped and hold the handle open, the way the host does.
fn spawn_stdin_lifeline() {
    std::thread::spawn(|| {
        let mut stdin = std::io::stdin();
        let mut buf = [0u8; 64];
        loop {
            match stdin.read(&mut buf) {
                Ok(0) => break, // EOF - parent is gone
                Ok(_) => {} // host currently sends nothing; ignore and keep waiting
                Err(_) => break,
            }
        }
        std::process::exit(0);
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn injected_keystrokes_are_dropped() {
        // Raw Input reports no source device for software-injected keys. This
        // is the IME case: the correction burst that follows "dd" in Telex
        // arrives this way and must never reach the wire.
        assert!(!is_physical_keyboard_event(EventKind::Keyboard, None));
    }

    #[test]
    fn real_keystrokes_still_pass() {
        // A resolved device id means a real keyboard produced the event.
        assert!(is_physical_keyboard_event(EventKind::Keyboard, Some("a1b2c3d4")));
    }

    #[test]
    fn per_device_filtering_is_unaffected() {
        // The host filters presses by device id. Every id it could match on
        // must still arrive, whatever its value - the drop rule keys off the
        // absence of an id, never off which device it names.
        for device_id in ["a1b2c3d4", "0", "ffffffff"] {
            assert!(
                is_physical_keyboard_event(EventKind::Keyboard, Some(device_id)),
                "{device_id} must reach the host so it can apply the user's filter"
            );
        }
    }

    #[test]
    fn mouse_events_are_never_dropped() {
        // No IME rewrites mouse input, and synthetic clicks are something
        // users do expect to hear - so the rule is keyboard-only.
        assert!(is_physical_keyboard_event(EventKind::Mouse, None));
        assert!(is_physical_keyboard_event(EventKind::Mouse, Some("a1b2c3d4")));
    }
}
