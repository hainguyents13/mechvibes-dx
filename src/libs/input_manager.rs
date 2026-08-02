/// Global input manager to handle input channels between main and the audio engine
use crossbeam_channel::{ Receiver, Sender };
use std::sync::{ Arc, Mutex, OnceLock };

/// Static global holder for input channels
static INPUT_CHANNELS: OnceLock<InputChannels> = OnceLock::new();

/// Static global holder for window focus state
static WINDOW_FOCUS_STATE: OnceLock<Arc<Mutex<bool>>> = OnceLock::new();

/// Struct to hold input event channels. Crossbeam receivers are `Clone`, so
/// the engine thread can hold its own receiver directly instead of sharing
/// one behind a `Mutex` (kept here only for the sender clones UI code uses).
#[allow(dead_code)]
pub struct InputChannels {
    pub keyboard_rx: Receiver<String>,
    pub mouse_rx: Receiver<String>,
    pub hotkey_rx: Receiver<String>,
    pub keyboard_tx: Arc<Mutex<Sender<String>>>,
    pub mouse_tx: Arc<Mutex<Sender<String>>>,
    pub hotkey_tx: Arc<Mutex<Sender<String>>>,
}

/// Initialize input channels (called from main)
pub fn init_input_channels(
    keyboard_rx: Receiver<String>,
    mouse_rx: Receiver<String>,
    hotkey_rx: Receiver<String>,
    keyboard_tx: Sender<String>,
    mouse_tx: Sender<String>,
    hotkey_tx: Sender<String>
) {
    let channels = InputChannels {
        keyboard_rx,
        mouse_rx,
        hotkey_rx,
        keyboard_tx: Arc::new(Mutex::new(keyboard_tx)),
        mouse_tx: Arc::new(Mutex::new(mouse_tx)),
        hotkey_tx: Arc::new(Mutex::new(hotkey_tx)),
    };

    let _ = INPUT_CHANNELS.set(channels);
}

/// Get input channels (called from UI)
pub fn get_input_channels() -> &'static InputChannels {
    INPUT_CHANNELS.get().expect("Input channels not initialized")
}

/// Initialize window focus state (called from main)
#[allow(dead_code)]
pub fn init_window_focus_state() {
    let _ = WINDOW_FOCUS_STATE.set(Arc::new(Mutex::new(false)));
}

/// Initialize window focus state with a specific value (called from main)
pub fn init_window_focus_state_with_value(focused: bool) {
    let _ = WINDOW_FOCUS_STATE.set(Arc::new(Mutex::new(focused)));
}

/// Get window focus state (called from UI)
pub fn get_window_focus_state() -> Arc<Mutex<bool>> {
    WINDOW_FOCUS_STATE.get().expect("Window focus state not initialized").clone()
}

/// Set window focus state (called from UI event handler)
pub fn set_window_focus(focused: bool) {
    if let Some(state) = WINDOW_FOCUS_STATE.get() {
        *state.lock().unwrap() = focused;
        println!("🔍 Window focus state changed: {}", if focused { "FOCUSED" } else { "UNFOCUSED" });
    }
}
