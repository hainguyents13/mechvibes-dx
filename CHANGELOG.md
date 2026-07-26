# Changelog

## v0.4.3 - 2026-07-26

### Code Cleanup & Refactoring
- **Compiler Warnings Cleanup**: Resolved 100% of compiler warnings (from 101 to 0 warnings) by removing orphan code and applying `#[allow(dead_code)]` annotations to reserved public API surfaces.
- **Orphan File Removal**: Removed unreferenced placeholder and debug files (`src/state/rodio_music.rs`, `src/libs/file_server.rs`, `src/bin/test_dq.rs`).
- **Unused UI Variables**: Fixed unused variable declarations in `soundpack_table.rs` and `device_selector.rs`.

### Testing & Quality Assurance
- **Unit Test Suite**: Created a unit test suite covering soundpack manifest validation (`soundpack_validator.rs`), legacy config keycode conversion (`config_converter.rs`), and system path resolution (`paths.rs`).

### Documentation
- **Comprehensive Docs**: Created all core documentation guides under `./docs/` (`project-overview-pdr.md`, `code-standards.md`, `codebase-summary.md`, `design-guidelines.md`, `deployment-guide.md`, `system-architecture.md`, `project-roadmap.md`).

## v0.4.2 - 2026-07-14

### Performance Improvements

#### Zero Disk I/O on Keypress
- **Root cause:** The `AppConfig::load()` function was reading from disk and system registry on every single keypress and mouse click to check if sounds were enabled.
- **Fix:** Introduced global `AtomicBool` flags (`SOUND_ENABLED`, `KEYBOARD_SOUND_ENABLED`, `MOUSE_SOUND_ENABLED`) in `audio_context.rs`. These flags are synced on app startup and whenever settings change, completely eliminating disk reads from the audio hot path.

#### Faster Soundpack Downloads
- **Root cause:** An artificial `tokio::time::sleep` of 15ms per chunk was slowing down soundpack downloads significantly.
- **Fix:** Removed the sleep delay, allowing downloads to process at full network speed.

### Bug Fixes

#### Tray Menu Download Improvements
- **Tray Index Mismatch:** Fixed an issue where downloading a soundpack from the tray menu could download the wrong pack because the menu relied on a sorted index instead of the pack ID.
- **Loading Label:** Fixed the "Loading..." label in the tray menu which was not displaying during downloads due to an ID comparison mismatch.
- **Auto-Activation:** Downloaded soundpacks from the tray are now correctly prefixed with `keyboard/` or `mouse/` so they instantly show up as active in the UI.

#### General Fixes
- **Ghost Keys:** Fixed a bug where keys could get "stuck" (ghost keys) when switching focus away from the app. The `pressed_keys` tracker is now properly cleared on focus transitions.
- **Safe Soundpack Deletion:** Deleting a currently active soundpack now safely unsets it from the active config, preventing corrupted config states and dangling references.
- **Platform-Specific Restarts:** Fixed the "Restart App" functionality on Windows by falling back to `cmd.exe /C ping` instead of relying on `sh` (which is unavailable natively on Windows). Paths are now safely sanitized.
- **Adaptive Settings Labels:** The "Start with Windows" setting now dynamically shows "Start at Login" on macOS and "Launch at Startup" on Linux.## v0.4.1 - 2026-07-13

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
