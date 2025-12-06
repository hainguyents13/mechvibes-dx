/// Global input manager to handle input channels between main and UI
use std::sync::{ mpsc, Arc, Mutex, OnceLock };

/// Static global holder for input channels
static INPUT_CHANNELS: OnceLock<InputChannels> = OnceLock::new();

/// Static global holder for window focus state
static WINDOW_FOCUS_STATE: OnceLock<Arc<Mutex<bool>>> = OnceLock::new();

/// Struct to hold input event channels
pub struct InputChannels {
    pub keyboard_rx: Arc<Mutex<mpsc::Receiver<String>>>,
    pub mouse_rx: Arc<Mutex<mpsc::Receiver<String>>>,
    pub hotkey_rx: Arc<Mutex<mpsc::Receiver<String>>>,
    pub keyboard_tx: Arc<Mutex<mpsc::Sender<String>>>,
    pub mouse_tx: Arc<Mutex<mpsc::Sender<String>>>,
    pub hotkey_tx: Arc<Mutex<mpsc::Sender<String>>>,
}

/// Initialize input channels (called from main)
pub fn init_input_channels(
    keyboard_rx: mpsc::Receiver<String>,
    mouse_rx: mpsc::Receiver<String>,
    hotkey_rx: mpsc::Receiver<String>,
    keyboard_tx: mpsc::Sender<String>,
    mouse_tx: mpsc::Sender<String>,
    hotkey_tx: mpsc::Sender<String>
) {
    let channels = InputChannels {
        keyboard_rx: Arc::new(Mutex::new(keyboard_rx)),
        mouse_rx: Arc::new(Mutex::new(mouse_rx)),
        hotkey_rx: Arc::new(Mutex::new(hotkey_rx)),
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
