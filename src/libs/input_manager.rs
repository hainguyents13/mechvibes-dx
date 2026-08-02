/// Shared window-focus state for the hybrid input listeners.
///
/// This used to also hold the input event channels, back when the UI polled
/// them every ~1ms. Since the audio engine thread took over with its own
/// receiver clones (see `spawn_engine` in main.rs), only the focus state is
/// left: the rdev (unfocused) / device_query (focused) listeners read it to
/// hand off to each other — the primary path on macOS/Linux-X11, and the
/// fallback path on Windows when the Raw Input worker can't be sustained.
use std::sync::{ Arc, Mutex, OnceLock };

/// Static global holder for window focus state
static WINDOW_FOCUS_STATE: OnceLock<Arc<Mutex<bool>>> = OnceLock::new();

/// Initialize window focus state with a specific value (called from main)
pub fn init_window_focus_state_with_value(focused: bool) {
    let _ = WINDOW_FOCUS_STATE.set(Arc::new(Mutex::new(focused)));
}

/// Get window focus state (called from UI)
pub fn get_window_focus_state() -> Arc<Mutex<bool>> {
    WINDOW_FOCUS_STATE.get().expect("Window focus state not initialized").clone()
}

/// Set window focus state (called from UI event handler)
///
/// The state itself is still needed on every platform: the rdev (unfocused) /
/// device_query (focused) hybrid listeners use it to hand off to each other.
/// On Windows that hybrid is only reached when the Raw Input worker process
/// can't be sustained (see `input_worker_host.rs`), but that fallback is live,
/// so the plumbing stays.
///
/// Deliberately unlogged: focus flips on every alt-tab and click-away, which
/// drowns out everything else in the console. When diagnosing the fallback
/// listeners' handoff, read the state where it's consumed instead.
pub fn set_window_focus(focused: bool) {
    if let Some(state) = WINDOW_FOCUS_STATE.get() {
        *state.lock().unwrap() = focused;
    }
}
