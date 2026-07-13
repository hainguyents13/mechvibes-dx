# Changelog

## v0.4.1 - 2026-07-13

### Bug Fixes

#### Keyboard sounds not working outside app window (macOS)
- **Root cause:** `rdev::listen()` created a CGEventTap at `CGEventTapLocation::HID` (highest priority), which intercepted ALL keyboard events system-wide. Even though keyboard events were discarded by the tap, the tap still processed them through expensive keyboard layout translation, interfering with `device_query`'s `CGEventSourceKeyState()` polling. Character keys (letters/numbers) went through input method processing and were missed; non-character keys (backspace, arrows) worked because they bypass input method processing.
- **Fix:** Replaced rdev's HID-level CGEventTap with `device_query`-only polling for both keyboard and mouse on macOS. With no CGEventTap present, `device_query`'s `CGEventSourceKeyState()` polling works correctly for all keys including letters and numbers.
- **Files changed:**
  - `src/libs/device_query_mouse_listener.rs` — New polling-based mouse listener using `device_query` (no CGEventTap)
  - `src/main.rs` — macOS input setup now uses `device_query`-only approach instead of rdev HID tap
  - `src/libs/mod.rs` — Added `device_query_mouse_listener` module

#### Soundpacks failing to load when running via `cargo run`
- **Root cause:** `get_app_root()` in `src/state/paths.rs` did not detect `target/debug/` or `target/release/` output paths, causing soundpack resolution to fail when the app was launched via `cargo run`.
- **Fix:** Extended `get_app_root()` to recognize `target/debug/` and `target/release/` paths and resolve them back to the project root, in addition to the existing `target/dx/` handling.
- **Files changed:**
  - `src/state/paths.rs` — Updated `get_app_root()` path detection logic

### Removed

#### Approach B (Session-level CGEventTap)
- Deleted `src/libs/session_input_listener.rs` (~360 lines of raw CGEventTap FFI code)
- Removed `cocoa` and `core-graphics` macOS-specific dependencies from `Cargo.toml`
- Removed `macos_dq` cargo feature flag

**Rationale:** Approach A (device_query-only) was chosen as the permanent solution over Approach B (session-level CGEventTap) because:
- 7x less code (51 vs 360 lines)
- No raw FFI bindings or CGEventTap overhead
- Fewer dependencies (`cocoa`, `core-graphics` no longer needed)
- No FFI-safety compiler warnings
- Lower system resource usage (polling loop vs event tap + CFRunLoop)
- Imperceptible latency difference (~10ms polling is below human perception for keyboard sounds)

### Summary of Approach A (device_query-only)

| Component | Behavior |
|-----------|----------|
| Keyboard | `device_query` polls `CGEventSourceKeyState()` every 10ms — works for all keys |
| Mouse | `device_query` polls mouse buttons every 10ms (Left/Right/Middle) |
| CGEventTap | None — this is the key difference from the broken rdev approach |
| System impact | Minimal — single polling thread, no event interception |
