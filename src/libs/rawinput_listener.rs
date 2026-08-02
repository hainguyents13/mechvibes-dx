//! Windows Raw Input capture - runs in the **input worker process**
//! (`input_worker.rs`), never in the UI process.
//!
//! Delivers `WM_INPUT` to a hidden window registered with `RIDEV_INPUTSINK`
//! so keyboard/mouse events arrive regardless of which window has focus -
//! the fix for "no sound while the app itself is focused" on Windows.
//!
//! Why a separate process: Windows allows only ONE Raw Input target window
//! per device class per process, and tao/wry registers keyboard+mouse raw
//! input for its own window once the Dioxus UI is built, overwriting any
//! registration we make in that process. Verified empirically in Phase 4:
//! immediately after startup the registration points at our HWND with
//! `RIDEV_INPUTSINK` (flags=0x100); ~5s later (UI up) it points at tao's
//! HWND with flags=0, and our message loop then receives zero messages -
//! not just zero `WM_INPUT`, zero messages of any kind. The limit is
//! per-process, so a worker process with no webview in it is unaffected.
//!
//! Two findings from that debugging worth keeping:
//!   1. A message-only (`HWND_MESSAGE`) window NEVER receives `WM_INPUT`,
//!      even though `RegisterRawInputDevices` reports success. Must be a
//!      normal window that simply is never shown.
//!   2. Running the listener on a spawned thread is fine - the message loop
//!      does not have to be on the process's main thread.
//!
//! Windows-only. macOS/Linux keep using `input_listener.rs` +
//! `focused_input_listener.rs` (rdev/device_query/evdev), unaffected by
//! this file.
#![cfg(target_os = "windows")]

use std::collections::{ HashMap, HashSet };
use std::ptr::null_mut;

use winapi::shared::minwindef::{ LPARAM, LRESULT, UINT, WPARAM };
use winapi::shared::windef::HWND;
use winapi::um::libloaderapi::GetModuleHandleW;
use winapi::um::winuser::*;

use crate::libs::input_device_manager::InputDeviceManager;

/// One captured input event, before it is turned into a wire line.
///
/// `device_id` is the same hash of the Raw Input device name that
/// `InputDeviceManager` produces for the Settings device list, so the UI
/// process can match it against `AppConfig.enabled_keyboards`/`enabled_mice`
/// without the worker needing to read config at all.
pub struct RawInputEvent<'a> {
    pub kind: EventKind,
    pub device_id: Option<String>,
    pub code: &'a str,
    pub is_down: bool,
}

#[derive(Clone, Copy, PartialEq)]
pub enum EventKind {
    Keyboard,
    Mouse,
}

/// Per-thread state the WndProc uses. Only ever touched from this one
/// thread's message loop - Raw Input delivers `WM_INPUT` synchronously to
/// the window that registered for it, on that window's thread.
struct ListenerState {
    sink: Box<dyn FnMut(RawInputEvent)>,
    pressed_keys: HashSet<String>,
    pressed_buttons: HashSet<String>,
    /// Resolving a device id costs three `GetRawInputDeviceInfoW` round
    /// trips plus a registry lookup for the friendly name, which is far too
    /// much to repeat inside the WndProc for every keystroke. Handles are
    /// stable for as long as a device is attached, so the answer is cached.
    /// A stale entry for an unplugged device is harmless - the handle is
    /// never reused for a different device while this process lives.
    device_ids: HashMap<usize, Option<String>>,
}

impl ListenerState {
    fn device_id(&mut self, handle: winapi::shared::ntdef::HANDLE) -> Option<String> {
        self.device_ids
            .entry(handle as usize)
            .or_insert_with(|| InputDeviceManager::device_id_for_handle(handle).ok())
            .clone()
    }
}

thread_local! {
    static STATE: std::cell::RefCell<Option<ListenerState>> = const {
        std::cell::RefCell::new(None)
    };
}

/// Runs the Raw Input message loop on the calling thread, handing every
/// key/button transition to `sink`. Blocks until the message loop ends, so
/// callers that need to keep working should run this on its own thread.
///
/// Device filtering and hotkey detection are deliberately NOT done here:
/// the worker process has no config, the UI process owns both (see
/// `input_worker_host.rs`). Key-repeat suppression IS done here, to keep
/// held-key traffic off the IPC pipe.
pub fn run_rawinput_loop(sink: Box<dyn FnMut(RawInputEvent)>) -> Result<(), String> {
    STATE.with(|s| {
        *s.borrow_mut() = Some(ListenerState {
            sink,
            pressed_keys: HashSet::new(),
            pressed_buttons: HashSet::new(),
            device_ids: HashMap::new(),
        });
    });

    run_message_loop()
}

fn run_message_loop() -> Result<(), String> {
    let class_name = to_wide("MechvibesRawInputListener");

    unsafe {
        let hinstance = GetModuleHandleW(null_mut());

        let wnd_class = WNDCLASSEXW {
            cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
            style: 0,
            lpfnWndProc: Some(wnd_proc),
            cbClsExtra: 0,
            cbWndExtra: 0,
            hInstance: hinstance,
            hIcon: null_mut(),
            hCursor: null_mut(),
            hbrBackground: null_mut(),
            lpszMenuName: null_mut(),
            lpszClassName: class_name.as_ptr(),
            hIconSm: null_mut(),
        };

        if RegisterClassExW(&wnd_class) == 0 {
            let err = winapi::um::errhandlingapi::GetLastError();
            return Err(format!("RegisterClassExW failed (GetLastError={})", err));
        }

        // NOT a message-only window (HWND_MESSAGE): those are excluded from
        // Windows' input system and never receive WM_INPUT, even with a
        // successful RegisterRawInputDevices call (verified empirically -
        // registration succeeded but zero WM_INPUT arrived). A normal
        // top-level window that is simply never shown (no WS_VISIBLE) and
        // kept off the taskbar (WS_EX_TOOLWINDOW) is invisible to the user
        // while still being a valid Raw Input target.
        let hwnd = CreateWindowExW(
            WS_EX_TOOLWINDOW,
            class_name.as_ptr(),
            null_mut(),
            WS_POPUP,
            0,
            0,
            0,
            0,
            null_mut(),
            null_mut(),
            hinstance,
            null_mut()
        );

        if hwnd.is_null() {
            let err = winapi::um::errhandlingapi::GetLastError();
            return Err(format!("CreateWindowExW failed (GetLastError={})", err));
        }

        register_raw_input_devices(hwnd)?;

        eprintln!("[worker] Raw Input registered, listening (focus-independent)");

        let mut msg: MSG = std::mem::zeroed();
        // Blocks until a message arrives - no polling, no fixed-interval
        // wakeups. Returns 0 on WM_QUIT, -1 on error, >0 otherwise.
        loop {
            match GetMessageW(&mut msg, null_mut(), 0, 0) {
                0 => {
                    break;
                } // WM_QUIT
                -1 => {
                    let err = winapi::um::errhandlingapi::GetLastError();
                    return Err(format!("GetMessageW failed (GetLastError={})", err));
                }
                _ => {
                    TranslateMessage(&msg);
                    DispatchMessageW(&msg);
                }
            }
        }
    }

    Ok(())
}

fn register_raw_input_devices(hwnd: HWND) -> Result<(), String> {
    let devices = [
        RAWINPUTDEVICE {
            usUsagePage: 0x01, // Generic Desktop Controls
            usUsage: 0x06, // Keyboard
            dwFlags: RIDEV_INPUTSINK,
            hwndTarget: hwnd,
        },
        RAWINPUTDEVICE {
            usUsagePage: 0x01,
            usUsage: 0x02, // Mouse
            dwFlags: RIDEV_INPUTSINK,
            hwndTarget: hwnd,
        },
    ];

    let ok = unsafe {
        RegisterRawInputDevices(
            devices.as_ptr(),
            devices.len() as u32,
            std::mem::size_of::<RAWINPUTDEVICE>() as u32
        )
    };

    if ok == 0 {
        let err = unsafe { winapi::um::errhandlingapi::GetLastError() };
        return Err(format!("RegisterRawInputDevices failed (GetLastError={})", err));
    }

    Ok(())
}

unsafe extern "system" fn wnd_proc(
    hwnd: HWND,
    msg: UINT,
    wparam: WPARAM,
    lparam: LPARAM
) -> LRESULT {
    match msg {
        WM_INPUT => {
            handle_wm_input(lparam);
            0
        }
        WM_DESTROY => {
            unsafe {
                PostQuitMessage(0);
            }
            0
        }
        _ => unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) },
    }
}

fn handle_wm_input(lparam: LPARAM) {
    let mut size: u32 = 0;
    unsafe {
        GetRawInputData(
            lparam as HRAWINPUT,
            RID_INPUT,
            null_mut(),
            &mut size,
            std::mem::size_of::<RAWINPUTHEADER>() as u32
        );
    }
    if size == 0 {
        return;
    }

    let mut buffer = vec![0u8; size as usize];
    let read = unsafe {
        GetRawInputData(
            lparam as HRAWINPUT,
            RID_INPUT,
            buffer.as_mut_ptr() as *mut _,
            &mut size,
            std::mem::size_of::<RAWINPUTHEADER>() as u32
        )
    };
    if read != size {
        return;
    }

    let raw = unsafe { &*(buffer.as_ptr() as *const RAWINPUT) };

    match raw.header.dwType {
        RIM_TYPEKEYBOARD => handle_keyboard(raw),
        RIM_TYPEMOUSE => handle_mouse(raw),
        _ => {}
    }
}

fn handle_keyboard(raw: &RAWINPUT) {
    let kbd = unsafe { raw.data.keyboard() };

    // RI_KEY_TERMSRV_SET_LED/SHADOW are synthetic (terminal services), not
    // physical key events - skip them same as the flags rdev filters.
    if (kbd.Flags as u32) & (RI_KEY_TERMSRV_SET_LED | RI_KEY_TERMSRV_SHADOW) != 0 {
        return;
    }

    let is_up = (kbd.Flags as u32) & RI_KEY_BREAK != 0;
    let is_e0 = (kbd.Flags as u32) & RI_KEY_E0 != 0;

    let Some(code) = map_vkey_to_code(kbd.VKey, kbd.MakeCode, is_e0) else {
        return;
    };

    STATE.with(|s| {
        let mut state_ref = s.borrow_mut();
        let Some(state) = state_ref.as_mut() else {
            return;
        };

        // Key-repeat: Raw Input resends WM_INPUT for held keys. Only the
        // first down (and the eventual up) should go over the pipe, same
        // as rdev's pressed_keys tracking in input_listener.rs.
        if is_up {
            state.pressed_keys.remove(&code);
        } else {
            if !state.pressed_keys.insert(code.clone()) {
                return;
            }
        }

        let device_id = state.device_id(raw.header.hDevice);
        (state.sink)(RawInputEvent {
            kind: EventKind::Keyboard,
            device_id,
            code: &code,
            is_down: !is_up,
        });
    });
}

fn handle_mouse(raw: &RAWINPUT) {
    let mouse = unsafe { raw.data.mouse() };
    let flags = mouse.usButtonFlags;
    if flags == 0 {
        return; // pure movement event, no button state change
    }

    STATE.with(|s| {
        let mut state_ref = s.borrow_mut();
        let Some(state) = state_ref.as_mut() else {
            return;
        };

        for (code, down_flag, up_flag) in mouse_button_flag_pairs() {
            let is_down = if flags & down_flag != 0 {
                if !state.pressed_buttons.insert(code.to_string()) {
                    continue;
                }
                true
            } else if flags & up_flag != 0 {
                state.pressed_buttons.remove(code);
                false
            } else {
                continue;
            };

            let device_id = state.device_id(raw.header.hDevice);
            (state.sink)(RawInputEvent {
                kind: EventKind::Mouse,
                device_id,
                code,
                is_down,
            });
        }
    });
}

/// `RAWMOUSE.usButtonFlags` only carries 5 button pairs (Left/Right/Middle/
/// XBUTTON1/XBUTTON2) per the Win32 Raw Input spec - there is no
/// `RI_MOUSE_BUTTON_6/7/8`. The old rdev listener's `Mouse6`/`Mouse7`/
/// `Mouse8` codes (`input_listener.rs::map_button_to_code`, `Button::Unknown`
/// arm) can't be reproduced through standard Raw Input; mice with more than
/// 5 buttons report the extras via a separate HID usage/RAWHID report, out
/// of scope for this phase (no known soundpack currently maps these - see
/// phase-04-raw-input-windows.md Risk Assessment).
fn mouse_button_flag_pairs() -> [(&'static str, u16, u16); 5] {
    [
        ("MouseLeft", RI_MOUSE_LEFT_BUTTON_DOWN, RI_MOUSE_LEFT_BUTTON_UP),
        ("MouseRight", RI_MOUSE_RIGHT_BUTTON_DOWN, RI_MOUSE_RIGHT_BUTTON_UP),
        ("MouseMiddle", RI_MOUSE_MIDDLE_BUTTON_DOWN, RI_MOUSE_MIDDLE_BUTTON_UP),
        ("Mouse4", RI_MOUSE_BUTTON_4_DOWN, RI_MOUSE_BUTTON_4_UP),
        ("Mouse5", RI_MOUSE_BUTTON_5_DOWN, RI_MOUSE_BUTTON_5_UP),
    ]
}

fn to_wide(s: &str) -> Vec<u16> {
    use std::os::windows::ffi::OsStrExt;
    std::ffi::OsStr::new(s).encode_wide().chain(std::iter::once(0)).collect()
}

/// Maps a Raw Input VKey + scancode + extended-key flags to the same key
/// code strings `input_listener.rs::map_key_to_code` produces from rdev, so
/// soundpack timing maps (keyed by these strings) work unchanged regardless
/// of which listener produced the event. VKey alone is ambiguous for
/// Left/Right modifier pairs and numpad-vs-navigation keys sharing a VKey
/// (e.g. VK_RETURN is both Enter and NumpadEnter) - the E0 "extended key"
/// flag disambiguates those, matching how Win32 itself distinguishes them.
fn map_vkey_to_code(vkey: u16, scancode: u16, is_e0: bool) -> Option<String> {
    use winapi::um::winuser::*;

    let code = match vkey as i32 {
        VK_BACK => "Backspace",
        VK_TAB => "Tab",
        VK_RETURN => {
            if is_e0 {
                "NumpadEnter"
            } else {
                "Enter"
            }
        }
        // Ctrl+Pause sends E1 and a different scancode (NumLock's), but
        // still carries VK_PAUSE - maps to Pause either way, same as
        // rdev's Key::Pause.
        VK_PAUSE => "Pause",
        VK_CAPITAL => "CapsLock",
        VK_ESCAPE => "Escape",
        VK_SPACE => "Space",
        VK_PRIOR => "PageUp",
        VK_NEXT => "PageDown",
        VK_END => "End",
        VK_HOME => "Home",
        VK_LEFT => "ArrowLeft",
        VK_UP => "ArrowUp",
        VK_RIGHT => "ArrowRight",
        VK_DOWN => "ArrowDown",
        VK_INSERT => "Insert",
        VK_DELETE => "Delete",

        // VK_0-VK_9 and VK_A-VK_Z are not defined constants in Win32 - per
        // MSDN, alphanumeric virtual-key codes equal their ASCII value
        // ('0'-'9' = 0x30-0x39, 'A'-'Z' = 0x41-0x5A), so match on the ASCII
        // literal directly (winapi crate omits these consts, matching the
        // Win32 header).
        0x30 => "Digit0",
        0x31 => "Digit1",
        0x32 => "Digit2",
        0x33 => "Digit3",
        0x34 => "Digit4",
        0x35 => "Digit5",
        0x36 => "Digit6",
        0x37 => "Digit7",
        0x38 => "Digit8",
        0x39 => "Digit9",

        0x41 => "KeyA",
        0x42 => "KeyB",
        0x43 => "KeyC",
        0x44 => "KeyD",
        0x45 => "KeyE",
        0x46 => "KeyF",
        0x47 => "KeyG",
        0x48 => "KeyH",
        0x49 => "KeyI",
        0x4a => "KeyJ",
        0x4b => "KeyK",
        0x4c => "KeyL",
        0x4d => "KeyM",
        0x4e => "KeyN",
        0x4f => "KeyO",
        0x50 => "KeyP",
        0x51 => "KeyQ",
        0x52 => "KeyR",
        0x53 => "KeyS",
        0x54 => "KeyT",
        0x55 => "KeyU",
        0x56 => "KeyV",
        0x57 => "KeyW",
        0x58 => "KeyX",
        0x59 => "KeyY",
        0x5a => "KeyZ",

        // Numpad digits/operators only fire without E0; E0+these VKeys
        // don't occur from the numpad (navigation cluster uses VK_PRIOR
        // etc, handled above), but guard anyway for safety.
        VK_NUMPAD0 => "Numpad0",
        VK_NUMPAD1 => "Numpad1",
        VK_NUMPAD2 => "Numpad2",
        VK_NUMPAD3 => "Numpad3",
        VK_NUMPAD4 => "Numpad4",
        VK_NUMPAD5 => "Numpad5",
        VK_NUMPAD6 => "Numpad6",
        VK_NUMPAD7 => "Numpad7",
        VK_NUMPAD8 => "Numpad8",
        VK_NUMPAD9 => "Numpad9",
        VK_MULTIPLY => "NumpadMultiply",
        VK_ADD => "NumpadAdd",
        VK_SUBTRACT => "NumpadSubtract",
        VK_DECIMAL => "NumpadDecimal",
        VK_DIVIDE => "NumpadDivide",

        VK_F1 => "F1",
        VK_F2 => "F2",
        VK_F3 => "F3",
        VK_F4 => "F4",
        VK_F5 => "F5",
        VK_F6 => "F6",
        VK_F7 => "F7",
        VK_F8 => "F8",
        VK_F9 => "F9",
        VK_F10 => "F10",
        VK_F11 => "F11",
        VK_F12 => "F12",

        VK_NUMLOCK => "NumLock",
        VK_SCROLL => "ScrollLock",
        VK_SNAPSHOT => "PrintScreen",

        // Modifier pairs: VK_SHIFT/CONTROL/MENU are the ambiguous
        // "either side" codes Windows sends for older apps; the extended
        // (0xA0-0xA5) VKeys disambiguate directly when present. Prefer the
        // extended VKey when Windows provides it (it usually does for Raw
        // Input), fall back to scancode-based E0 check for VK_SHIFT (Shift
        // has no E0 flag distinction the way Ctrl/Alt do - right shift is
        // identified by scancode 0x36 vs left's 0x2A).
        VK_LSHIFT => "ShiftLeft",
        VK_RSHIFT => "ShiftRight",
        VK_LCONTROL => "ControlLeft",
        VK_RCONTROL => "ControlRight",
        VK_LMENU => "AltLeft",
        VK_RMENU => "AltRight",
        VK_SHIFT => {
            if scancode == 0x36 {
                "ShiftRight"
            } else {
                "ShiftLeft"
            }
        }
        VK_CONTROL => {
            if is_e0 {
                "ControlRight"
            } else {
                "ControlLeft"
            }
        }
        VK_MENU => {
            if is_e0 {
                "AltRight"
            } else {
                "AltLeft"
            }
        }
        VK_LWIN => "MetaLeft",
        VK_RWIN => "MetaRight",

        VK_OEM_MINUS => "Minus",
        VK_OEM_PLUS => "Equal",
        VK_OEM_COMMA => "Comma",
        VK_OEM_PERIOD => "Period",
        VK_OEM_1 => "Semicolon", // US layout ;:
        VK_OEM_2 => "Slash", // US layout /?
        VK_OEM_3 => "Backquote", // US layout `~
        VK_OEM_4 => "BracketLeft", // US layout [{
        VK_OEM_5 => "Backslash", // US layout \|
        VK_OEM_6 => "BracketRight", // US layout ]}
        VK_OEM_7 => "Quote", // US layout '"
        VK_OEM_102 => "IntlBackslash", // extra key on ISO keyboards

        _ => {
            return None;
        }
    };

    Some(code.to_string())
}
