# PR Description: macOS Fixes

## Title
`fix(macos): transition to device_query polling to fix background keyboard sounds and path resolution`

---

## Overview
This PR addresses several critical issues on macOS:
1. Keyboard click sounds not playing when the app is in the background or minimized.
2. Inconsistent/partial keyboard capturing (only modifier keys and backspace working in other applications, while letters/numbers were silent).
3. App data/soundpack path resolution issues when running the application via `cargo run`.

---

## Details & Rationale

### 1. macOS Background Keyboard Sound Fix (Transition to `device_query` Polling)
* **The Problem:** The app previously used a hybrid model (`rdev` for background tracking, `device_query` for focused window tracking). On macOS, WebKit/Wry does not reliably emit window focus-change events when hidden or minimized, leaving the focus state stuck. Additionally, `rdev`'s global `CGEventTap` at the HID-level intercepted system keys and initiated layout translations that interfered with `device_query`'s polling, causing macOS to filter out alphanumeric characters (letters/numbers) while only letting non-character keys (like Backspace) through.
* **The Fix:** Replaced `rdev`'s global event tap for keyboard events with a direct `device_query` keyboard poller that runs continuously on macOS (no focus gating). Implemented a lightweight mouse poller (`src/libs/device_query_mouse_listener.rs`) to query mouse state.
* **Result:** This eliminates the `CGEventTap` overhead entirely on macOS, allowing all keys (including letters/numbers) to play sounds perfectly both in the foreground and background with zero perceptible latency.

### 2. Silent Permission Blocking & Re-connection Loop
* **The Problem:** On macOS Sonoma/Sequoia, launching a global hook without Accessibility Permissions no longer triggers a hard crash—it succeeds but silently filters out keyboard inputs.
* **The Fix:** Integrated the native macOS `AXIsProcessTrusted` API via FFI to dynamically check permission status on each frame. 
* **Result:** Added a warning banner in the Dioxus UI that appears when permissions are missing, with a one-click button opening System Settings. The background hook will now attempt to automatically re-bind every 2 seconds once permissions are granted.

### 3. Soundpacks Loading via `cargo run`
* **The Fix:** Updated `get_app_root()` in `src/state/paths.rs` to recognize `target/debug/` and `target/release/` output paths, allowing soundpacks and resource assets to resolve correctly back to the project root during local testing.

---

## Changes
* `src/main.rs`: Set up macOS to use unified `device_query` polling and check permissions early.
* `src/libs/focused_input_listener.rs`: Modified to support continuous polling on macOS and capture hotkeys.
* `src/libs/device_query_mouse_listener.rs`: Added a polling mouse click listener to replace the mouse hook on macOS.
* `src/libs/input_manager.rs`: Added FFI bindings for `AXIsProcessTrusted` and created a live diagnostics log.
* `src/components/pages/home.rs` & `src/libs/routes.rs`: Added a real-time diagnostics event log and dynamic permission warning banner.
* `src/state/paths.rs`: Updated directory resolution to support Cargo development builds.
* `package_macos.sh`: Automated bundle compilation, ad-hoc codesigning, DMG generation, and TCC permission resets.
