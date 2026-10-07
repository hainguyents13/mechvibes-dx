//! Startup wiring shared by the GUI and headless launch paths.
//!
//! Input capture is the one part of startup both modes need to be identical.
//! Duplicating it would let the two drift - and the way it drifts is
//! predictable: whoever writes the second copy reaches for the simplest
//! listener that compiles, which on Windows is the rdev fallback, and ships a
//! mode that eats dead keys and delays clicks (phase 06). One function, called
//! from both, makes that impossible.
//!
//! Only capture lives here. The window, tray, ambiance player, telemetry and
//! update checker are GUI-only and stay in `main()`.

use crossbeam_channel::Sender;

/// Starts platform input capture, feeding the three channels the audio engine
/// reads.
///
/// Returns immediately; events begin arriving once the platform listener is
/// up. Must be called after `spawn_engine`, so nothing sends into a channel
/// with no receiver.
///
/// `window_focused` is the shared focus flag the X11/macOS hybrid listeners
/// consult to decide which of the two is authoritative. Headless has no
/// window, so it passes a flag that is permanently false - "never focused" is
/// exactly right there: it is the state in which the unfocused listener, the
/// one that works without a window, is the one that reports.
#[cfg_attr(
    target_os = "windows",
    allow(unused_variables, reason = "the Raw Input worker is focus-independent")
)]
pub fn start_input_capture_with_focus(
    keyboard_tx: Sender<String>,
    mouse_tx: Sender<String>,
    hotkey_tx: Sender<String>,
    window_focused: std::sync::Arc<std::sync::Mutex<bool>>
) {
    #[cfg(target_os = "linux")]
    {
        use crate::libs::input_listener::start_unified_input_listener;
        use crate::libs::focused_input_listener::start_focused_keyboard_listener;

        let display_server = std::env
            ::var("XDG_SESSION_TYPE")
            .unwrap_or_else(|_| "x11".to_string());
        crate::debug_print!("🔍 Detected display server: {}", display_server);

        if display_server == "wayland" {
            // On Wayland, evdev reads the device nodes directly, so it works
            // focused or not, and it carries hotkey detection with it.
            crate::debug_print!("🎮 Starting evdev keyboard listener (Wayland mode)...");
            crate::libs::evdev_input_listener::start_evdev_keyboard_listener(
                keyboard_tx.clone(),
                mouse_tx.clone(),
                hotkey_tx.clone(),
                window_focused.clone()
            );

            // rdev covers mouse only here. It is handed a permanently-focused
            // flag so its keyboard branch stays quiet and cannot double every
            // keystroke evdev already reported.
            crate::debug_print!("🎮 Starting unified input listener for mouse events (Wayland mode)...");
            let always_focused = std::sync::Arc::new(std::sync::Mutex::new(true));
            start_unified_input_listener(keyboard_tx, mouse_tx, hotkey_tx, Some(always_focused));
        } else {
            // X11: the hybrid. rdev handles keyboard while unfocused,
            // device_query while focused.
            crate::debug_print!("🎮 Starting unified input listener (X11 mode - unfocused)...");
            start_unified_input_listener(
                keyboard_tx.clone(),
                mouse_tx,
                hotkey_tx,
                Some(window_focused.clone())
            );

            crate::debug_print!("🎮 Starting focused keyboard listener (X11 mode - focused)...");
            start_focused_keyboard_listener(keyboard_tx, window_focused);
        }
    }

    // Windows: capture runs in a separate worker process using Raw Input, so
    // events arrive regardless of which window has focus. It cannot run in
    // the UI process - tao/wry takes the process-wide Raw Input registration
    // once the webview is built (see rawinput_listener.rs). If the worker
    // cannot be kept alive we fall back to the rdev + device_query hybrid,
    // which works while unfocused only.
    #[cfg(target_os = "windows")]
    {
        use crate::libs::input_listener::start_unified_input_listener;
        use crate::libs::focused_input_listener::start_focused_keyboard_listener;

        let fallback_keyboard_tx = keyboard_tx.clone();
        let fallback_mouse_tx = mouse_tx.clone();
        let fallback_hotkey_tx = hotkey_tx.clone();
        let fallback_focus = window_focused;

        crate::debug_print!("🎮 Starting Raw Input worker process...");
        crate::libs::input_worker_host::start_input_worker_host(
            keyboard_tx,
            mouse_tx,
            hotkey_tx,
            Box::new(move || {
                start_unified_input_listener(
                    fallback_keyboard_tx.clone(),
                    fallback_mouse_tx,
                    fallback_hotkey_tx,
                    Some(fallback_focus.clone())
                );
                start_focused_keyboard_listener(fallback_keyboard_tx, fallback_focus);
            })
        );
    }

    // macOS: hybrid approach (rdev + device_query) - rdev handles keyboard
    // when unfocused, device_query when focused.
    #[cfg(target_os = "macos")]
    {
        use crate::libs::input_listener::start_unified_input_listener;
        use crate::libs::focused_input_listener::start_focused_keyboard_listener;

        crate::debug_print!("🎮 Starting unified input listener (unfocused)...");
        start_unified_input_listener(
            keyboard_tx.clone(),
            mouse_tx,
            hotkey_tx,
            Some(window_focused.clone())
        );

        crate::debug_print!("🎮 Starting focused keyboard listener (focused)...");
        start_focused_keyboard_listener(keyboard_tx, window_focused);
    }
}

/// Starts input capture for a run with no window.
///
/// The focus flag is pinned false: with no window there is nothing to focus,
/// and on the platforms that consult it that selects the listener which does
/// not need one.
pub fn start_input_capture(
    keyboard_tx: Sender<String>,
    mouse_tx: Sender<String>,
    hotkey_tx: Sender<String>
) {
    start_input_capture_with_focus(
        keyboard_tx,
        mouse_tx,
        hotkey_tx,
        std::sync::Arc::new(std::sync::Mutex::new(false))
    );
}

#[cfg(test)]
mod tests {
    /// Windows headless must never be routed through rdev as its primary
    /// listener. rdev's low-level hooks are what swallow dead keys and delay
    /// the second click of a double-click (phase 06); the Raw Input worker is
    /// free of both, and remains the only primary path here.
    ///
    /// A source assertion because the alternative is spawning a real worker
    /// process and typing at it. What it pins is structural and worth pinning:
    /// rdev may appear in this file only inside the worker's fallback closure.
    #[test]
    fn windows_capture_starts_the_raw_input_worker_not_rdev() {
        const SOURCE: &str = include_str!("bootstrap.rs");
        let runtime = SOURCE.split("#[cfg(test)]").next().expect("runtime code precedes tests");

        let windows_block = runtime
            .split("#[cfg(target_os = \"windows\")]")
            .nth(1)
            .expect("the Windows branch must exist");

        let worker_start = windows_block
            .find("start_input_worker_host")
            .expect("Windows capture must go through the Raw Input worker");
        let fallback_start = windows_block
            .find("Box::new(move ||")
            .expect("the rdev fallback must be a closure passed to the worker host");

        assert!(
            worker_start < fallback_start,
            "the worker must be started first, with rdev only as its fallback"
        );

        let before_worker = &windows_block[..worker_start];
        assert!(
            !before_worker.contains("start_unified_input_listener("),
            "no rdev listener may be started ahead of the Raw Input worker"
        );
    }

    /// Both launch modes must reach the same wiring.
    #[test]
    fn the_headless_entry_point_delegates_to_the_shared_wiring() {
        const SOURCE: &str = include_str!("bootstrap.rs");
        let runtime = SOURCE.split("#[cfg(test)]").next().expect("runtime code precedes tests");

        let headless_fn = runtime
            .split("pub fn start_input_capture(")
            .nth(1)
            .expect("the headless entry point must exist");

        assert!(
            headless_fn.contains("start_input_capture_with_focus("),
            "headless must not carry its own copy of the platform wiring"
        );
    }
}
