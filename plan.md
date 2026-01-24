# Default Output Device Change Handling - Implementation Plan

Goal: When `selected_audio_device` is `None` (system default), automatically switch audio output to the new system default without restarting the app.

## Scope
- Windows: event-driven detection via Core Audio (IMMNotificationClient).
- Non-Windows: optional polling fallback (low frequency).
- Only apply auto-rebuild when user has not explicitly selected a specific device.

## Current State (Key Files)
- `src/libs/audio/audio_context.rs`: creates `OutputStream::try_default()` once at startup.
- `src/libs/audio/music_player.rs`: creates `OutputStream::try_default()` once at startup.
- `src/state/ambiance.rs`: creates `OutputStream::try_default()` when starting ambiance sounds; keeps sinks alive.
- `src/libs/ui.rs`: owns `AudioContext` and provides it via `use_context_provider`.
- `src/state/config.rs`: `selected_audio_device: Option<String>` (`None` = system default).

## High-Level Design
1) Detect default output device change.
2) Dispatch event to app loop.
3) If `selected_audio_device == None`, rebuild audio subsystems:
   - Recreate `AudioContext` and reload soundpacks.
   - Restart ambiance sinks.
   - Recreate music player and resume current track if playing.

## Implementation Steps

### 1) Add Device Change Watcher (Windows)
- New file: `src/libs/audio/device_change_watcher.rs`
- **Current implementation**: cross-platform polling via `cpal` every ~2s.
- Event-driven Core Audio (IMMNotificationClient) is a follow-up optimization.
- Expose a `start_default_device_watcher() -> Receiver<DefaultDeviceEvent>`.

Notes:
- Polling keeps implementation simple and reliable across platforms.

### 2) Optional Polling Fallback (Non-Windows)
- New file: `src/libs/audio/device_poll_watcher.rs`
- Every 1–2s, check `cpal` default output device name/ID.
- If changed, send the same `DefaultDeviceEvent`.
- Enable only for non-Windows via `cfg`.

### 3) Event Wiring in UI
- In `src/libs/ui.rs`, add a `use_future` task that listens for device-change events.
- When event arrives:
  - Load config; if `selected_audio_device.is_some()`, ignore.
  - Otherwise trigger audio rebuild sequence.

### 4) Rebuild AudioContext
- Introduce a signal/slot for the current `AudioContext` in `src/libs/ui.rs`:
  - Replace `let audio_context = use_hook(|| Arc::new(AudioContext::new()));`
  - With `let audio_context = use_signal(|| Arc::new(AudioContext::new()));`
  - Provide `audio_context()` via `use_context_provider`.
- On rebuild:
  - Create `Arc::new(AudioContext::create_with_device(None)?)`.
  - Call `soundpack_loader::load_soundpack(&new_ctx)`.
  - Update signal to new `Arc` so UI uses new stream.

### 5) Restart Ambiance
- In `src/state/ambiance.rs`, add a helper:
  - `pub fn restart_ambiance_on_device_change()`
    - Stop all sinks in `GLOBAL_AMBIANCE_SINKS`.
    - Recreate active sounds using `start_all_active_sounds()`.
- Call this from the device-change handler.

### 6) Rebuild Music Player
- Track current music state (playing, current track URL) in `MusicPlayerState`.
- Add a `rebuild_music_player()` function in `src/state/music.rs`:
  - Drop old player instance.
  - Create a new `RodioMusicPlayer`.
  - If previously playing, call `Play(current_url)`.
- Trigger on device-change event when `selected_audio_device == None`.

### 7) Guard Conditions
- Only act when:
  - `selected_audio_device == None`
  - Device-change event is for output render devices

### 8) Logging & Error Handling
- Log when device changes and when rebuild succeeds/fails.
- Failures should not crash; keep old audio active if rebuild fails.

## Testing Plan
- Manual:
  - Set audio output to default in app.
  - Change Windows default output device while app is running.
  - Verify key/mouse sounds switch to new device.
  - Verify ambiance and music also switch.
- Regression:
  - Explicitly selected device should not change on default switch.

## Notes / Risks
- Ensure COM is initialized in watcher thread (CoInitializeEx).
- Make sure Dioxus context updates are thread-safe; dispatch rebuild onto async task.
- Avoid tight polling; use event-based on Windows.
