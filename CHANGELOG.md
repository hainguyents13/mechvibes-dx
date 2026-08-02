# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

## [0.6.0] - 2026-08-02

### Added

- **Sound keeps working while the MechvibesDX window is focused (Windows)**: keyboard and mouse capture moved to a dedicated worker process using the Raw Input API, removing the old focused-window workaround (100Hz polling with ~10ms latency and missed keys during focus changes). Typing into the app window itself now sounds identical to typing anywhere else, and the Ctrl+Alt+M hotkey works regardless of focus.
- **Per-device input filtering on Windows**: disabling a specific keyboard or mouse in Settings now actually silences that device (and only that device), effective immediately without a restart. Previously the setting existed but had no effect.
- **Automatic fallback when the audio device disappears**: if the output device in use is unplugged (even mid-typing), sound automatically moves to the system default within a few seconds instead of going silent. Your saved device choice is kept on disk, so restarting the app returns to it once the device is back.

### Changed

- **Audio playback moved to a dedicated engine thread**: switching the output device in Settings now applies instantly at runtime (previously it only took effect after a restart, and could crash). The ambiance player follows the selected device too, including on startup. Input-to-sound latency is also slightly improved by replacing polling loops with blocking channel reads.
- The input worker process supervises itself: if it crashes it restarts automatically, if the app exits it shuts down with it, and if it can't be sustained the app falls back to the previous capture method so sound never fully stops.
- Removed the unused built-in music player (dead code — it was never reachable from the UI).

### Fixed

- Selecting an audio output device in Settings previously only saved the choice without applying it; ambiance sounds resume correctly after a device switch.
- Hardened the Windows input path: fixed a supervisor crash that could permanently stop input in debug builds after a healthy worker restart, a stuck-key state when disabling a keyboard while a key was held, undefined behavior in raw-input buffer handling, and missing system message cleanup.

## [0.5.2] - 2026-07-31

### Fixed

- **Sound cutting off / clicking when typing fast**: keyboard and mouse sounds now use a proper voice pool (oldest-first eviction) instead of a hashmap keyed by key name, so rapid repeated keys no longer cut each other's tails off. Added short fade-in/fade-out (2ms/5ms) to eliminate clicks/pops at segment boundaries, and soft (ramped) eviction instead of a hard cut when the voice pool is full.
- **Sound continuing to play after releasing all keys ("ghost typing")**: the keyboard/mouse/hotkey event loops now drain their entire backlog every tick instead of processing one event at a time, so a fast burst of keystrokes can no longer queue up and keep playing sound after the user has already lifted their hands.
- **Poor audio quality from realtime resampling**: soundpacks are now resampled once at load time to the output device's sample rate (using a high-quality sinc resampler) instead of relying on the audio backend's realtime linear resampling.
- Removed a redundant device probe on every soundpack load; the output device's sample rate is now probed once at startup and cached, avoiding unnecessary device enumeration (which could briefly interrupt audio on Linux/ALSA).
- Sample-rate lookup failures no longer fall back to a hardcoded 44100 Hz guess (which could cause audio to be resampled twice); they now skip resampling and keep the file's native rate instead.

### Changed

- Increased the keyboard/mouse voice pool limit (`max_voices`) from 20 to 32 to give more headroom for overlapping sound tails during fast typing.

## [0.5.1] and earlier

See git history for changes prior to this changelog's introduction.

