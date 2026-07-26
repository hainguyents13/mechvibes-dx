//! macOS native vibrancy and transparent titlebar setup.

use dioxus::desktop::use_wry_event_handler;
use dioxus::desktop::tao::event::{ Event as TaoEvent, StartCause };
use std::sync::OnceLock;

static APPLIED: OnceLock<()> = OnceLock::new();

pub fn use_macos_vibrancy() {
    use_wry_event_handler(move |event, _window_target| {
        if APPLIED.get().is_some() {
            return;
        }
        if let TaoEvent::NewEvents(StartCause::Init) = event {
            apply_vibrancy();
        }
    });
}

#[cfg(target_os = "macos")]
#[allow(deprecated, unexpected_cfgs)]
fn apply_vibrancy() {
    use cocoa::base::{ id, nil };
    use objc::sel;
    use objc::sel_impl;

    if APPLIED.get().is_some() {
        return;
    }

    unsafe {
        let ns_app: id = objc::msg_send![objc::class!(NSApplication), sharedApplication];
        let windows: id = objc::msg_send![ns_app, windows];
        let count: usize = objc::msg_send![windows, count];

        if count == 0 {
            return;
        }

        let ns_window: id = objc::msg_send![windows, objectAtIndex: 0];

        // Transparent titlebar
        let _: () = objc::msg_send![ns_window, setTitlebarAppearsTransparent: true];

        // Content extends behind title bar (NSFullSizeContentViewWindowMask = 1 << 3)
        let style_mask: u64 = objc::msg_send![ns_window, styleMask];
        let _: () = objc::msg_send![ns_window, setStyleMask: style_mask | (1 << 3)];

        // Non-opaque + clear background
        let _: () = objc::msg_send![ns_window, setOpaque: false];
        let _: () = objc::msg_send![ns_window, setBackgroundColor: cocoa::appkit::NSColor::clearColor(nil)];

        // Hide title text (NSTitleVisibilityHidden = 1)
        let _: () = objc::msg_send![ns_window, setTitleVisibility: 1_u64];

        // NSVisualEffectView for liquid glass
        let content_view: id = objc::msg_send![ns_window, contentView];
        let effect_view: id = objc::msg_send![objc::class!(NSVisualEffectView), alloc];
        let effect_view: id = objc::msg_send![effect_view, init];

        if effect_view != nil {
            let _: () = objc::msg_send![effect_view, setBlendingMode: 0_u64];
            let _: () = objc::msg_send![effect_view, setMaterial: 7_u64];
            let _: () = objc::msg_send![effect_view, setState: 0_u64];
            // NSViewWidthSizable | NSViewHeightSizable
            let _: () = objc::msg_send![effect_view, setAutoresizingMask: 18_u64];

            // Get content view bounds and apply to effect view
            let frame: NSRectFFI = objc::msg_send![content_view, frame];
            let _: () = objc::msg_send![effect_view, setFrame: frame];
            // Insert below webview (NSWindowBelow = -1)
            let _: () = objc::msg_send![content_view, addSubview: effect_view positioned: (-1_isize) relativeTo: nil];
        }

        let _ = APPLIED.set(());
    }
}

/// FFI-compatible NSRect matching cocoa's layout
#[cfg(target_os = "macos")]
#[repr(C)]
#[derive(Copy, Clone)]
struct NSRectFFI {
    origin_x: f64,
    origin_y: f64,
    width: f64,
    height: f64,
}

#[cfg(not(target_os = "macos"))]
fn apply_vibrancy() {}
