/// Global input manager to handle input channels between main and UI
use std::sync::{ mpsc, Arc, Mutex, OnceLock };

/// Static global holder for input channels
static INPUT_CHANNELS: OnceLock<InputChannels> = OnceLock::new();

/// Struct to hold input event channels
pub struct InputChannels {
    pub keyboard_rx: Arc<Mutex<mpsc::Receiver<String>>>,
    pub mouse_rx: Arc<Mutex<mpsc::Receiver<String>>>,
    pub hotkey_rx: Arc<Mutex<mpsc::Receiver<String>>>,
}

/// Initialize input channels (called from main)
pub fn init_input_channels(
    keyboard_rx: mpsc::Receiver<String>,
    mouse_rx: mpsc::Receiver<String>,
    hotkey_rx: mpsc::Receiver<String>
) {
    let channels = InputChannels {
        keyboard_rx: Arc::new(Mutex::new(keyboard_rx)),
        mouse_rx: Arc::new(Mutex::new(mouse_rx)),
        hotkey_rx: Arc::new(Mutex::new(hotkey_rx)),
    };

    let _ = INPUT_CHANNELS.set(channels);
}

/// Get input channels (called from UI)
pub fn get_input_channels() -> &'static InputChannels {
    INPUT_CHANNELS.get().expect("Input channels not initialized")
}
