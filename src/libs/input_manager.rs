/// Global input manager to handle input channels between main and UI
use std::sync::{ mpsc, Arc, Mutex, OnceLock };

use std::sync::atomic::{AtomicBool, Ordering};

/// Static global holder for input channels
static INPUT_CHANNELS: OnceLock<InputChannels> = OnceLock::new();

/// Static global holder for window focus state
static WINDOW_FOCUS_STATE: OnceLock<Arc<Mutex<bool>>> = OnceLock::new();

/// Static global holder for macOS accessibility permissions status
static HAS_ACCESSIBILITY_PERMISSIONS: AtomicBool = AtomicBool::new(true);

pub fn set_accessibility_permissions(has_permission: bool) {
    HAS_ACCESSIBILITY_PERMISSIONS.store(has_permission, Ordering::Relaxed);
}

#[allow(dead_code)]
pub fn get_accessibility_permissions() -> bool {
    HAS_ACCESSIBILITY_PERMISSIONS.load(Ordering::Relaxed)
}

/// Struct to hold input event channels
#[allow(dead_code)]
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
#[allow(dead_code)]
pub fn init_window_focus_state() {
    let _ = WINDOW_FOCUS_STATE.set(Arc::new(Mutex::new(false)));
}

/// Initialize window focus state with a specific value (called from main)
pub fn init_window_focus_state_with_value(focused: bool) {
    let _ = WINDOW_FOCUS_STATE.set(Arc::new(Mutex::new(focused)));
}

/// Get window focus state (called from UI)
#[allow(dead_code)]
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

#[cfg(target_os = "macos")]
mod macos_privacy {
    // Input Monitoring (macOS 10.15+) — required for letter/number keys from other apps
    #[link(name = "CoreGraphics", kind = "framework")]
    unsafe extern "C" {
        fn CGPreflightListenEventAccess() -> bool;
        fn CGRequestListenEventAccess() -> bool;
    }

    pub fn is_accessibility_trusted() -> bool {
        macos_accessibility_client::accessibility::application_is_trusted()
    }

    /// Prompt the user to allow Accessibility if it hasn't been given.
    pub fn prompt_accessibility_if_needed() -> bool {
        macos_accessibility_client::accessibility::application_is_trusted_with_prompt()
    }

    pub fn is_input_monitoring_trusted() -> bool {
        // Returns true if this process may listen via CGEvent taps for keyboard
        // events from other apps (including letter/number keys).
        unsafe { CGPreflightListenEventAccess() }
    }

    /// Request Input Monitoring (shows system prompt / adds app to the list).
    pub fn request_input_monitoring() -> bool {
        unsafe { CGRequestListenEventAccess() }
    }
}

/// True when Accessibility is granted (or non-macOS).
pub fn check_accessibility_permissions() -> bool {
    #[cfg(target_os = "macos")]
    {
        macos_privacy::is_accessibility_trusted()
    }
    #[cfg(not(target_os = "macos"))]
    {
        get_accessibility_permissions()
    }
}

/// True when Input Monitoring is granted (macOS 10.15+).
/// Without this, CGEventTap often only delivers special keys (Backspace, CapsLock, …)
/// from other apps — letters and numbers are filtered. `cargo run` binaries that were
/// previously approved work; a newly packaged `.app` needs its own grant.
pub fn check_input_monitoring_permissions() -> bool {
    #[cfg(target_os = "macos")]
    {
        macos_privacy::is_input_monitoring_trusted()
    }
    #[cfg(not(target_os = "macos"))]
    {
        true
    }
}

/// True when the process can capture full keyboard events globally.
#[allow(dead_code)]
pub fn check_full_keyboard_capture_permissions() -> bool {
    check_accessibility_permissions() && check_input_monitoring_permissions()
}

/// Prompt for Accessibility + request Input Monitoring. Call once at startup on macOS.
/// Returns whether full capture should work after prompting.
pub fn ensure_macos_input_permissions() -> bool {
    #[cfg(target_os = "macos")]
    {
        let ax = if macos_privacy::is_accessibility_trusted() {
            true
        } else {
            println!("🔒 Requesting macOS Accessibility permission (system prompt)...");
            macos_privacy::prompt_accessibility_if_needed()
        };

        let listen = if macos_privacy::is_input_monitoring_trusted() {
            true
        } else {
            println!(
                "🔒 Requesting macOS Input Monitoring permission \
                 (needed for letter/number keys in other apps)..."
            );
            macos_privacy::request_input_monitoring()
        };

        set_accessibility_permissions(ax && listen);

        println!(
            "🔒 Permissions — Accessibility: {}, Input Monitoring: {}",
            if ax { "yes" } else { "NO" },
            if listen { "yes" } else { "NO" }
        );

        if !ax || !listen {
            eprintln!(
                "⚠️ Full keyboard capture needs BOTH permissions:\n\
                 • System Settings → Privacy & Security → Accessibility → enable MechvibesDX\n\
                 • System Settings → Privacy & Security → Input Monitoring → enable MechvibesDX\n\
                 Then quit and reopen the app. (cargo run uses a different binary path, so\n\
                 permissions granted there do NOT apply to the .app / DMG build.)"
            );
        }

        ax && listen
    }
    #[cfg(not(target_os = "macos"))]
    {
        true
    }
}

/// Open System Settings to Accessibility privacy pane.
pub fn open_accessibility_settings() {
    let _ = crate::utils::path::open_path(
        "x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility",
    );
}

/// Open System Settings to Input Monitoring privacy pane.
pub fn open_input_monitoring_settings() {
    // Ventura+ deep link; falls back gracefully if ignored by older macOS
    let _ = crate::utils::path::open_path(
        "x-apple.systempreferences:com.apple.preference.security?Privacy_ListenEvent",
    );
}

static LAST_EVENTS: OnceLock<Mutex<Vec<String>>> = OnceLock::new();

pub fn get_last_events() -> Vec<String> {
    LAST_EVENTS.get_or_init(|| Mutex::new(Vec::new())).lock().unwrap().clone()
}

pub fn add_last_event(event_str: String) {
    let mut events = LAST_EVENTS.get_or_init(|| Mutex::new(Vec::new())).lock().unwrap();
    events.insert(0, event_str);
    if events.len() > 8 {
        events.truncate(8);
    }
}
