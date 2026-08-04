# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

## [0.7.3] - 2026-08-04

### Fixed

- **"Start with Windows" no longer switches itself off.** The app compared its registry entry against the wrong form of the path whenever "Start minimized" was also enabled, concluded autostart was off, and helpfully saved that conclusion over your setting. Both toggles now work, so Task Scheduler workarounds are no longer needed.
- **Keys used as global hotkeys by other apps now play sound.** If another program (Rainmeter, macro tools, and similar) claims a key like F1 as a hotkey, Windows hides the key press from us but still delivers the release; the app now recognizes that pattern and plays the sound anyway. Mechvibes classic behavior restored.
- **Launching the app while it is already running now brings up the running window** instead of doing nothing.
- **A damaged config file no longer wipes your settings.** Unknown or invalid fields fall back to sensible defaults for that field only, and if the file is truly unreadable it is preserved as `config.json.corrupt` instead of being overwritten, so nothing is lost. Hand-editors: a soundpack slot set to `""` is accepted and simply loads no pack for that slot.
- **Muting with Ctrl+Alt+M now updates the window immediately**, matching the tray and in-app buttons.
- The classic-Mechvibes converter backs up an existing converted audio file before overwriting it, and stops instead of proceeding when any backup fails.

### Changed

- **Every settings write now goes through a single writer with atomic file saves.** This is the structural fix behind the "my settings reverted" saga: previously twelve different places in the app could each save a whole copy of the settings, and whichever finished last silently undid the others. That mistake is now impossible to write, not merely discouraged, and a half-written settings file can no longer exist even if the app is killed mid-save.

## [0.7.2] - 2026-08-03

### Added

- **macOS DMG (experimental)**: the macOS build now ships as a proper `.dmg` with a drag-to-Applications app bundle instead of a bare-binary archive. Still unsigned and not notarized: right-click the app and choose Open the first time. Settings are stored in `~/Library/Application Support/Mechvibes`. Untested on real hardware; reports welcome.

### Fixed

- **Settings really stop reverting now.** The 0.7.1 fix covered one path but missed the main one: a background service kept a copy of your settings from the moment the app started and wrote it back seconds later (and again every 24 hours), undoing whatever you had changed in between. Whichever setting you touched first after opening the app was the one you would see reset, which is why reports about "which setting resets" never agreed. Verified against the running app this time, not only in tests.

## [0.7.1] - 2026-08-03

### Added

- **Anonymous usage statistics**: the app now sends one anonymous ping per launch (OS and app version, nothing else) so we can tell how many people use MechvibesDX daily. No keystrokes, no personal data, no persistent identifiers. You can turn it off in Settings, Privacy section; details in the README.

### Fixed

- **Settings no longer revert on their own shortly after starting the app.** The background update check kept a copy of your settings from launch time and wrote it back after checking for updates, silently undoing anything you changed in between (volume, theme, toggles). This mostly hit people on slower connections who start the app once a day, which is why it was hard to reproduce.
- **Volume changes on the Home page could be lost when switching tabs quickly.** The change is now applied immediately; only the disk write is delayed.
- **Muting from the tray menu now updates the window immediately** instead of showing a stale icon until something else changed.

## [0.7.0] - 2026-08-03

### Fixed

- **Severe input lag when a specific audio output device was selected**: the app checked device presence inside the audio engine loop every second, and that check could take up to ~2 seconds depending on your device's position in the system list — delaying both the sound and the logo animation by that much, seemingly at random. Sound now plays within ~15ms of a keystroke regardless of which output device is selected.
- **Vietnamese input methods (Telex/UniKey/EVKey) no longer cause bursts of clumped sounds**: when an IME rewrites text (e.g. "dd" → "đ", or restoring a mistyped syllable), it synthetically injects backspaces and replacement keys — each of which used to play a sound all at once. Software-injected keystrokes are now ignored; only keys you physically press make sound. Note this also means on-screen keyboards and macro tools no longer trigger sounds.
- **Logo animation keeps up with fast typing**: the pressed-state transition was too slow to render each stroke at speed; each keystroke now shows a distinct pulse.

### Changed

- **Removed the automatic switch to the system default output when the selected device disappears.** The background polling this required is gone completely — zero periodic device probing while you type. If you unplug the device you selected, the app stays silent until you pick another one in Settings (or restart); your saved choice is kept. Selecting "System Default" is unaffected — the OS handles device changes itself.
- Settings → Devices note updated: device changes apply immediately, no restart needed (restart remains a troubleshooting option).

### Added

- **Diagnostic trace mode**: launch with the environment variable `MECHVIBES_TRACE=1` to print one line per keystroke with per-stage timings (input capture → audio engine → sound → UI) — useful when reporting latency issues.

## [0.6.3] - 2026-08-03

### Fixed

- **The app no longer offers an update to the version you are already running.** A version recorded before a manual upgrade could linger and make the app advertise itself as an update; the saved value is now cleared at startup and every update prompt double-checks the version before showing anything.
- **The app now relaunches itself after a one-click update.** Previously "Restart to finish update" closed the app and the freshly installed version never started — an over-cautious installer guard was skipping the relaunch step. Verified end to end: close, silent install, automatic restart.

## [0.6.2] - 2026-08-03

### Added

- **One-click update install (Windows)**: when a new version is available, the Settings page now shows a "Download & install" button that downloads the installer in the background, verifies its SHA-256 checksum against the release's `SHA256SUMS.txt`, and — after you confirm — installs silently and relaunches the app. Nothing is downloaded until you click, and choosing "Later" keeps the verified download ready for next time. If anything fails (offline, checksum mismatch, older release without checksums), the button falls back to opening the download page as before.
- **Linux `.deb` package**: releases now include an installable Debian/Ubuntu package. Note: it does not add your user to the `input` group — run `sudo usermod -a -G input $USER` and re-log once after installing.
- **macOS build (experimental)**: an unsigned, untested arm64 build now ships with each release for adventurous testers; see the bundled README for Gatekeeper and Accessibility steps.
- Releases now include a `SHA256SUMS.txt` covering every asset.

### Changed

- **Tray icon dims while muted**, and the tray's mute entry is now a fixed-label "Mute sounds" item with a check mark instead of swapping text. The correct state also shows immediately when the app starts already muted.
- The update notification in the title bar now takes you to Settings instead of opening a browser download directly, so every install path goes through checksum verification.
- On Linux and macOS, the updater no longer offers the Windows installer; it links to the releases page instead.

### Fixed

- Removed noisy window-focus logging and dead internal plumbing left over from the pre-worker input architecture.

## [0.6.1] - 2026-08-02

### Fixed

- **Mute buttons had no effect on sound**: the mute toggles on the home page, in Settings, and in the tray menu only saved the preference without telling the running audio engine, and the icon could get out of sync and toggle back on the next click. All mute paths now apply immediately and stay consistent with the Ctrl+Alt+M hotkey.
- **Soundpack selector dropdown was transparent/unreadable**: a stale CSS variable from an older theme version made the dropdown background compute to transparent.
- **Device list disappeared when switching tabs**: the enumerated audio/input device list is now remembered for the rest of the session instead of resetting to the "refresh" placeholder every time you leave and re-enter Settings.
- **Reset to Defaults** now applies volume and mute state to the running engine immediately instead of requiring a restart.
- Removed constant config file read/write churn (the app was rewriting its config about once per second while typing) and noisy window-focus logging in release builds.

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







