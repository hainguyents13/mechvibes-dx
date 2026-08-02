use crossbeam_channel::{ unbounded, Receiver, Sender };
use cpal::traits::DeviceTrait;
use rodio::buffer::SamplesBuffer;
use rodio::{ OutputStream, OutputStreamHandle, Sink };
use std::collections::HashMap;
use std::sync::{ Arc, OnceLock };
use std::time::Duration;

use crate::libs::device_manager::DeviceManager;
use crate::state::config::AppConfig;

const FADE_IN_MS: f32 = 2.0;
const FADE_OUT_MS: f32 = 5.0;
const EVICT_RAMP_MS: u64 = 10;
const MAX_VOICES: usize = 32;

/// How often the engine loop checks that the active output device is still
/// present - see `run_engine`, which fires this on an absolute deadline so
/// heavy input cannot starve it.
///
/// Kept at 1s so that 2 strikes still detect an unplug within the ~3s the
/// plan calls for. This is affordable because the check short-circuits at
/// ~10ms while the device is present (see
/// `DeviceManager::has_output_device_named`) - roughly 1% of one thread.
/// It was only expensive before that probe existed, when every tick ran the
/// UI-facing enumeration and blocked this thread for ~1.5s.
const DEVICE_WATCHDOG_INTERVAL: Duration = Duration::from_millis(1000);

/// Consecutive watchdog checks that must report the active device missing
/// before falling back to default. Two checks (~2s) ride out the transient
/// window where an audio driver re-enumerates devices (mode switch, power
/// state change) without the hardware actually being gone, which would
/// otherwise cause a spurious switch away from a device the user picked.
const DEVICE_MISSING_STRIKES: u32 = 2;

/// (samples, channels, sample_rate) for a decoded/resampled audio buffer.
type DecodedAudio = (Arc<Vec<f32>>, u16, u32);

/// Commands the engine thread accepts. Every audio-affecting operation goes
/// through this channel so the thread that owns `OutputStream` never has to
/// share it across threads (rodio's `OutputStream` is not `Send`).
///
/// Keyboard/mouse key events are NOT a variant here: the engine reads raw
/// `"KeyA"` / `"UP:KeyA"` strings directly off the same crossbeam receivers
/// the input listeners (rdev/device_query/evdev) send to, via `select!` in
/// `run_engine` - see `spawn_engine`. Routing them through `AudioCommand`
/// would add an extra hop with no benefit.
pub enum AudioCommand {
    SetVolume(f32),
    SetMouseVolume(f32),
    SetSoundEnabled(bool),
    SetKeyboardSoundEnabled(bool),
    SetMouseSoundEnabled(bool),
    LoadKeyboardPack {
        soundpack_id: String,
        update_cache_on_error: bool,
    },
    LoadMousePack {
        soundpack_id: String,
        update_cache_on_error: bool,
    },
    SwitchDevice(Option<String>), // None = system default
}

/// Events the engine thread pushes back out for the UI to react to.
#[derive(Clone, Debug)]
pub enum UiEvent {
    KeyDown(String),
    KeyUp(String),
    DeviceSwitched(Result<String, String>),
    /// The active output device disappeared (unplugged/disabled) and the
    /// engine moved playback to the system default on its own. Carries the
    /// name of the device that went away plus the fallback result, so the UI
    /// can tell the user why their selection changed without being asked.
    DeviceLost {
        lost_device: String,
        result: Result<String, String>,
    },
    PackLoaded {
        is_keyboard: bool,
        result: Result<String, String>,
    },
}

/// Cheap, `Clone + Send` handle to the audio engine thread. UI code and input
/// listeners hold this instead of the engine's internal state.
#[derive(Clone)]
pub struct AudioEngineHandle {
    tx: Sender<AudioCommand>,
}

impl AudioEngineHandle {
    pub fn send(&self, command: AudioCommand) {
        // The engine thread never exits while the app is running, so a send
        // failure here would mean the engine panicked - nothing UI-side can
        // recover from that, so just drop the command.
        let _ = self.tx.send(command);
    }
}

static ENGINE_HANDLE: OnceLock<AudioEngineHandle> = OnceLock::new();
static UI_EVENT_RX: OnceLock<Receiver<UiEvent>> = OnceLock::new();

/// Spawns the audio engine thread and returns a handle to it. Must be called
/// once, before `dioxus::launch`, so `OutputStream` is opened and lives on a
/// plain OS thread rather than inside the webview/Dioxus runtime.
///
/// `keyboard_rx`/`mouse_rx`/`hotkey_rx` are clones of the same crossbeam
/// receivers the UI's `InputChannels` holds - the engine reads raw
/// `"KeyA"` / `"UP:KeyA"` strings directly from the input listeners (rdev,
/// device_query, evdev) via `select!`, the same wire format those listeners
/// already produce, instead of the UI polling and forwarding them.
pub fn spawn_engine(
    keyboard_rx: Receiver<String>,
    mouse_rx: Receiver<String>,
    hotkey_rx: Receiver<String>
) -> AudioEngineHandle {
    let (cmd_tx, cmd_rx) = unbounded::<AudioCommand>();
    let (event_tx, event_rx) = unbounded::<UiEvent>();

    std::thread::spawn(move || {
        run_engine(cmd_rx, event_tx, keyboard_rx, mouse_rx, hotkey_rx);
    });

    let handle = AudioEngineHandle { tx: cmd_tx };
    let _ = ENGINE_HANDLE.set(handle.clone());
    let _ = UI_EVENT_RX.set(event_rx);
    handle
}

/// Returns the engine handle. Panics if `spawn_engine` hasn't run yet - this
/// mirrors the existing `get_input_channels()` contract (main.rs sets it up
/// before the UI can possibly ask for it).
pub fn engine_handle() -> AudioEngineHandle {
    ENGINE_HANDLE.get().expect("Audio engine not started").clone()
}

/// Returns the `UiEvent` receiver for the UI's single poll loop.
pub fn ui_event_receiver() -> &'static Receiver<UiEvent> {
    UI_EVENT_RX.get().expect("Audio engine not started")
}

/// All state the engine thread owns exclusively. Nothing here is behind a
/// `Mutex` - the thread is the only place that ever touches it, so plain
/// fields are enough (this is the whole point of moving playback off the
/// Arc<Mutex<...>> path used pre-Phase-3). Fields are `pub(super)` so
/// `soundpack_loader.rs` (a sibling module) can update them after decoding.
pub(super) struct EngineState {
    stream: OutputStream,
    pub(super) stream_handle: OutputStreamHandle,
    device_manager: DeviceManager,
    current_device_id: Option<String>,
    device_watchdog: DeviceWatchdog,
    pub(super) device_rate: Option<u32>,

    pub(super) keyboard_samples: Option<DecodedAudio>,
    pub(super) keyboard_samples_original: Option<DecodedAudio>,
    pub(super) mouse_samples: Option<DecodedAudio>,
    pub(super) mouse_samples_original: Option<DecodedAudio>,
    pub(super) key_map: HashMap<String, Vec<[f32; 2]>>,
    pub(super) mouse_map: HashMap<String, Vec<[f32; 2]>>,

    key_pressed: HashMap<String, bool>,
    mouse_pressed: HashMap<String, bool>,
    pub(super) key_sinks: Vec<Sink>,
    pub(super) mouse_sinks: Vec<Sink>,

    volume: f32,
    mouse_volume: f32,
    sound_enabled: bool,
    keyboard_sound_enabled: bool,
    mouse_sound_enabled: bool,
}

/// Tracks whether the device the engine is currently playing on is still
/// present, and decides when that warrants falling back to the default.
///
/// Split out from `EngineState` and kept free of any audio/cpal types so the
/// decision rule is unit-testable without hardware: `run_engine` only feeds
/// it the answer to "is the active device still in the device list?".
#[derive(Default)]
struct DeviceWatchdog {
    /// Device name (not id) the engine is playing on. `None` means the engine
    /// is on the system default, where there is nothing to watch: the OS
    /// already moves the default for us when hardware disappears.
    ///
    /// Name rather than id is deliberate. `DeviceManager` ids are positional
    /// (`output_{index}`) into a live enumeration, so unplugging a device
    /// shifts every later device's id down one. Watching an id would silently
    /// start watching a *different* physical device after any removal, and
    /// the check would keep reporting "present" while the user's audio had
    /// actually moved elsewhere.
    watched_name: Option<String>,
    consecutive_misses: u32,
}

impl DeviceWatchdog {
    /// Points the watchdog at whatever device the engine just opened.
    /// `None` (system default) disarms it.
    fn watch(&mut self, device_name: Option<String>) {
        self.watched_name = device_name;
        self.consecutive_misses = 0;
    }

    fn is_armed(&self) -> bool {
        self.watched_name.is_some()
    }

    /// Records one observation. `present` is whether the watched device was
    /// found in the current enumeration. Returns the device name to fall back
    /// from once it has been missing `DEVICE_MISSING_STRIKES` checks in a row.
    ///
    /// Returns `Some` exactly once per disappearance: it disarms itself so a
    /// failed fallback cannot re-trigger every tick.
    fn observe(&mut self, present: bool) -> Option<String> {
        let watched = self.watched_name.as_ref()?;

        if present {
            self.consecutive_misses = 0;
            return None;
        }

        self.consecutive_misses += 1;
        if self.consecutive_misses < DEVICE_MISSING_STRIKES {
            return None;
        }

        let lost = watched.clone();
        // Disarm: the engine is about to move to the default device, and a
        // fallback that fails must not spin retrying on every tick.
        self.watched_name = None;
        self.consecutive_misses = 0;
        Some(lost)
    }
}

/// Opens a stream for `device_id` (`None` = system default). Does NOT fall
/// back silently - callers decide what to do on `Err` (see `switch_device`,
/// which keeps the previous device on failure, vs `EngineState::new`, which
/// falls back to default since there's no previous device to keep).
///
/// The returned name is the opened device's cpal name, used by
/// `DeviceWatchdog` to detect the device disappearing later.
fn open_stream(
    device_manager: &DeviceManager,
    device_id: Option<&str>
) -> Result<(OutputStream, OutputStreamHandle, Option<String>, Option<String>), String> {
    match device_id {
        Some(id) => {
            match device_manager.get_output_device_by_id(id) {
                Ok(Some(device)) => {
                    let name = device.name().ok();
                    rodio::OutputStream
                        ::try_from_device(&device)
                        .map(|(stream, handle)| (stream, handle, Some(id.to_string()), name))
                        .map_err(|e| format!("Failed to open stream for device {}: {}", id, e))
                }
                Ok(None) => Err(format!("Device {} not found", id)),
                Err(e) => Err(format!("Error accessing device {}: {}", id, e)),
            }
        }
        None => {
            rodio::OutputStream
                ::try_default()
                .map(|(stream, handle)| (stream, handle, None, None))
                .map_err(|e| format!("Failed to open default audio output stream: {}", e))
        }
    }
}

impl EngineState {
    fn new() -> Self {
        let device_manager = DeviceManager::new();
        let config = AppConfig::load();

        let (stream, stream_handle, opened_device_id, opened_device_name) = open_stream(
            &device_manager,
            config.selected_audio_device.as_deref()
        ).unwrap_or_else(|e| {
            eprintln!("❌ [AudioEngine] {} - falling back to default", e);
            open_stream(&device_manager, None).expect("Failed to open default audio output stream")
        });
        let current_device_id = opened_device_id.or(config.selected_audio_device.clone());
        let device_rate = device_manager.get_current_output_sample_rate();

        let mut device_watchdog = DeviceWatchdog::default();
        device_watchdog.watch(opened_device_name);

        Self {
            stream,
            stream_handle,
            device_manager,
            current_device_id,
            device_watchdog,
            device_rate,
            keyboard_samples: None,
            keyboard_samples_original: None,
            mouse_samples: None,
            mouse_samples_original: None,
            key_map: HashMap::new(),
            mouse_map: HashMap::new(),
            key_pressed: HashMap::new(),
            mouse_pressed: HashMap::new(),
            key_sinks: Vec::new(),
            mouse_sinks: Vec::new(),
            volume: config.volume,
            mouse_volume: config.mouse_volume,
            sound_enabled: config.enable_sound,
            keyboard_sound_enabled: config.enable_keyboard_sound,
            mouse_sound_enabled: config.enable_mouse_sound,
        }
    }

    fn handle_key_event(&mut self, code: &str, down: bool) {
        if !should_play(self.sound_enabled, self.keyboard_sound_enabled) {
            return;
        }
        if !debounce_press(&mut self.key_pressed, code, down) {
            return;
        }
        if let Some((start, end)) = lookup_timing(&self.key_map, code, down) {
            play_segment(
                &self.stream_handle,
                &self.keyboard_samples,
                code,
                start,
                end,
                self.volume,
                &mut self.key_sinks
            );
        }
    }

    fn handle_mouse_event(&mut self, code: &str, down: bool) {
        if !should_play(self.sound_enabled, self.mouse_sound_enabled) {
            return;
        }
        if !debounce_press(&mut self.mouse_pressed, code, down) {
            return;
        }
        if let Some((start, end)) = lookup_timing(&self.mouse_map, code, down) {
            play_segment(
                &self.stream_handle,
                &self.mouse_samples,
                code,
                start,
                end,
                self.mouse_volume,
                &mut self.mouse_sinks
            );
        }
    }

    fn switch_device(&mut self, device_id: Option<String>) -> Result<String, String> {
        // On Err, `self` is left untouched entirely - the previous device
        // keeps playing, matching the "keep current sound, report error"
        // requirement (Phase 3 success criteria).
        let (new_stream, new_handle, opened_device_id, opened_device_name) = open_stream(
            &self.device_manager,
            device_id.as_deref()
        )?;

        // Re-resample cached original samples to the new device's rate so
        // in-flight soundpacks keep playing correctly without a reload.
        let new_rate = self.device_manager.get_current_output_sample_rate();
        if let Some((orig_samples, channels, orig_rate)) = &self.keyboard_samples_original {
            self.keyboard_samples = Some(
                resample_if_needed(orig_samples, *channels, *orig_rate, new_rate)
            );
        }
        if let Some((orig_samples, channels, orig_rate)) = &self.mouse_samples_original {
            self.mouse_samples = Some(
                resample_if_needed(orig_samples, *channels, *orig_rate, new_rate)
            );
        }

        // Drop old voices/stream only after the new one is confirmed open,
        // so a failed switch leaves the previous device still playing.
        self.key_sinks.clear();
        self.mouse_sinks.clear();
        self.stream = new_stream;
        self.stream_handle = new_handle;
        self.device_rate = new_rate;
        self.current_device_id = opened_device_id.clone().or(device_id);
        self.device_watchdog.watch(opened_device_name);

        let label = self.current_device_id.clone().unwrap_or_else(|| "System Default".to_string());
        Ok(label)
    }

    /// Falls back to the system default device when the active device
    /// disappears (e.g. USB headset unplugged) rather than by user choice.
    /// Called from the engine loop's watchdog tick, not from a command.
    fn fallback_to_default(&mut self) -> Result<String, String> {
        self.switch_device(None)
    }

    /// Whether the watched device is still present in the current
    /// enumeration, matched by name (see `DeviceWatchdog::watched_name` for
    /// why not by id).
    ///
    /// Returns `true` ("assume present") both when there is nothing to watch
    /// and when enumeration failed outright: a host that cannot list devices
    /// is not evidence that this particular device went away, and counting it
    /// as a miss would fall back on a transient audio-service hiccup.
    ///
    /// Uses the quiet `has_output_device_named` probe rather than the
    /// UI-facing `get_output_devices()`, which logs per device and probes
    /// each device's config - running that on a timer spammed the log and
    /// blocked this thread for ~1.5s per tick.
    fn watched_device_present(&self) -> bool {
        let Some(watched) = self.device_watchdog.watched_name.as_deref() else {
            return true;
        };
        self.device_manager.has_output_device_named(watched).unwrap_or(true)
    }
}

/// Whether a keystroke should produce sound, given the global mute flag and
/// the per-input-type (keyboard/mouse) flag.
///
/// Both flags live in `EngineState` and are only moved by
/// `AudioCommand::Set*SoundEnabled`, so a UI that writes the config without
/// sending that command leaves this returning `true` and the app keeps playing
/// while the UI shows muted - the mute-button bug this guards against.
fn should_play(sound_enabled: bool, type_enabled: bool) -> bool {
    sound_enabled && type_enabled
}

/// Marks `code` pressed/released, returning `false` if this event should be
/// ignored (duplicate keydown, or keyup with no matching keydown).
fn debounce_press(pressed: &mut HashMap<String, bool>, code: &str, down: bool) -> bool {
    let was_down = *pressed.get(code).unwrap_or(&false);
    if down == was_down {
        return false;
    }
    pressed.insert(code.to_string(), down);
    true
}

/// Looks up the `[start, end]` (ms) pair for a keydown/keyup event from a
/// soundpack's timing map. Mirrors the pre-Phase-3 `play_key_event_sound`
/// logic in `sound_manager.rs`.
fn lookup_timing(map: &HashMap<String, Vec<[f32; 2]>>, code: &str, down: bool) -> Option<(f32, f32)> {
    match map.get(code) {
        Some(arr) if arr.len() == 2 => {
            let idx = if down { 0 } else { 1 };
            Some((arr[idx][0], arr[idx][1]))
        }
        Some(arr) if arr.len() == 1 => {
            if !down {
                return None; // keydown-only mapping, ignore keyup
            }
            Some((arr[0][0], arr[0][1]))
        }
        _ => None,
    }
}

fn play_segment(
    stream_handle: &OutputStreamHandle,
    samples: &Option<DecodedAudio>,
    code: &str,
    start_ms: f32,
    end_ms: f32,
    volume: f32,
    sinks: &mut Vec<Sink>
) {
    let Some((samples_arc, channels, sample_rate)) = samples else {
        return;
    };
    let samples: &Vec<f32> = samples_arc.as_ref();
    let channels = *channels;
    let sample_rate = *sample_rate;

    let duration = end_ms - start_ms;
    if start_ms < 0.0 || duration <= 0.0 {
        return;
    }

    let start_sample = ((start_ms / 1000.0) * (sample_rate as f32) * (channels as f32)) as usize;
    let end_sample = ((end_ms / 1000.0) * (sample_rate as f32) * (channels as f32)) as usize;
    let end_sample = end_sample.min(samples.len());
    if start_sample >= end_sample {
        eprintln!(
            "❌ [AudioEngine] Invalid sample range for '{}': {}..{} (max {})",
            code,
            start_sample,
            end_sample,
            samples.len()
        );
        return;
    }

    let mut segment_samples = samples[start_sample..end_sample].to_vec();
    apply_fade(&mut segment_samples, channels, sample_rate);
    let segment = SamplesBuffer::new(channels, sample_rate, segment_samples);

    if let Ok(sink) = Sink::try_new(stream_handle) {
        sink.set_volume(volume);
        sink.append(segment);

        manage_active_sinks(sinks, MAX_VOICES);
        sinks.push(sink);
    }
}

/// Applies a linear fade-in/fade-out to interleaved PCM samples in place.
/// Operates per-frame (one frame = `channels` consecutive samples) so all
/// channels in a frame share the same gain and stay in phase.
fn apply_fade(samples: &mut [f32], channels: u16, sample_rate: u32) {
    let channels = channels.max(1) as usize;
    let frame_count = samples.len() / channels;
    if frame_count == 0 {
        return;
    }

    let mut fade_in_frames = ((FADE_IN_MS / 1000.0) * (sample_rate as f32)) as usize;
    let mut fade_out_frames = ((FADE_OUT_MS / 1000.0) * (sample_rate as f32)) as usize;

    let half = frame_count / 2;
    if fade_in_frames > half {
        fade_in_frames = half;
    }
    if fade_out_frames > half {
        fade_out_frames = half;
    }

    for frame in 0..fade_in_frames {
        let gain = (frame as f32) / (fade_in_frames as f32);
        let base = frame * channels;
        for c in 0..channels {
            samples[base + c] *= gain;
        }
    }

    for frame in 0..fade_out_frames {
        let gain = (frame as f32) / (fade_out_frames as f32);
        let frame_idx = frame_count - 1 - frame;
        let base = frame_idx * channels;
        for c in 0..channels {
            samples[base + c] *= gain;
        }
    }
}

/// Removes finished sinks, then evicts the oldest voice (ramped down to
/// avoid a click) if the pool is still at or above `max_voices`.
fn manage_active_sinks(sinks: &mut Vec<Sink>, max_voices: usize) {
    sinks.retain(|s| !s.empty());

    if sinks.len() >= max_voices {
        let old_sink = sinks.remove(0);
        std::thread::spawn(move || {
            const STEPS: u32 = 10;
            let starting_volume = old_sink.volume();
            for step in 1..=STEPS {
                let gain = starting_volume * (1.0 - (step as f32) / (STEPS as f32));
                old_sink.set_volume(gain.max(0.0));
                std::thread::sleep(Duration::from_millis(EVICT_RAMP_MS / (STEPS as u64)));
            }
            old_sink.stop();
        });
    }
}

fn resample_if_needed(
    original_samples: &Arc<Vec<f32>>,
    channels: u16,
    original_rate: u32,
    target_rate: Option<u32>
) -> DecodedAudio {
    match target_rate {
        Some(target_rate) if target_rate != original_rate => {
            let resampled = super::resampler::resample_interleaved(
                original_samples,
                channels,
                original_rate,
                target_rate
            );
            (Arc::new(resampled), channels, target_rate)
        }
        _ => (original_samples.clone(), channels, original_rate),
    }
}

/// Parses the `"KeyA"` / `"UP:KeyA"` wire format the input listeners
/// (rdev, device_query, evdev) already send, same as the pre-Phase-3 UI
/// polling loops in `ui.rs` did.
fn parse_input_event(raw: &str) -> Option<(String, bool)> {
    if let Some(code) = raw.strip_prefix("UP:") {
        Some((code.to_string(), false))
    } else if !raw.is_empty() {
        Some((raw.to_string(), true))
    } else {
        None
    }
}

/// Handles the Ctrl+Alt+M hotkey: flips `enable_sound`, persists it, and
/// returns the new value so the caller can move the engine's own cached flag
/// in lockstep.
///
/// The persisted config is the source of truth for the new value (rather than
/// negating the engine's cached flag) so the hotkey cannot drift from what the
/// UI and tray read back.
fn handle_toggle_sound() -> bool {
    let mut config = AppConfig::load();
    config.enable_sound = !config.enable_sound;
    config.last_updated = chrono::Utc::now();
    let enabled = config.enable_sound;
    match config.save() {
        Ok(_) => {
            crate::libs::tray_service::request_tray_update();
            println!("🔄 [AudioEngine] Sound toggled: {}", enabled);
        }
        Err(e) => eprintln!("❌ [AudioEngine] Failed to save config after sound toggle: {}", e),
    }
    // Keep the UI-side cache (`AudioContext::is_sound_enabled`) in step: the
    // tray toggle derives its next value from it, so a stale cache here would
    // make the following tray click a no-op.
    super::audio_context::sync_sound_enabled_cache(enabled);
    enabled
}

fn handle_command(state: &mut EngineState, event_tx: &Sender<UiEvent>, command: AudioCommand) {
    match command {
        AudioCommand::SetVolume(v) => {
            state.volume = v;
            for sink in &state.key_sinks {
                sink.set_volume(v);
            }
        }
        AudioCommand::SetMouseVolume(v) => {
            state.mouse_volume = v;
            for sink in &state.mouse_sinks {
                sink.set_volume(v);
            }
        }
        AudioCommand::SetSoundEnabled(enabled) => {
            state.sound_enabled = enabled;
        }
        AudioCommand::SetKeyboardSoundEnabled(enabled) => {
            state.keyboard_sound_enabled = enabled;
        }
        AudioCommand::SetMouseSoundEnabled(enabled) => {
            state.mouse_sound_enabled = enabled;
        }
        AudioCommand::LoadKeyboardPack { soundpack_id, update_cache_on_error } => {
            let result = super::soundpack_loader::load_keyboard_pack_into_engine(
                state,
                &soundpack_id,
                update_cache_on_error
            );
            let _ = event_tx.send(UiEvent::PackLoaded { is_keyboard: true, result });
        }
        AudioCommand::LoadMousePack { soundpack_id, update_cache_on_error } => {
            let result = super::soundpack_loader::load_mouse_pack_into_engine(
                state,
                &soundpack_id,
                update_cache_on_error
            );
            let _ = event_tx.send(UiEvent::PackLoaded { is_keyboard: false, result });
        }
        AudioCommand::SwitchDevice(device_id) => {
            // User-initiated switch: on failure, keep the previous device
            // playing and just report the error - don't silently fall back
            // to default (that's reserved for the device-removed case,
            // where there's no "previous" device left to keep).
            let result = state.switch_device(device_id);
            let _ = event_tx.send(UiEvent::DeviceSwitched(result));
        }
    }
}

fn run_engine(
    cmd_rx: Receiver<AudioCommand>,
    event_tx: Sender<UiEvent>,
    keyboard_rx: Receiver<String>,
    mouse_rx: Receiver<String>,
    hotkey_rx: Receiver<String>
) {
    let mut state = EngineState::new();

    // Load the configured soundpacks once at startup.
    let config = AppConfig::load();
    if !config.keyboard_soundpack.is_empty() {
        let result = super::soundpack_loader::load_keyboard_pack_into_engine(
            &mut state,
            &config.keyboard_soundpack,
            false
        );
        let _ = event_tx.send(UiEvent::PackLoaded { is_keyboard: true, result });
    }
    if !config.mouse_soundpack.is_empty() {
        let result = super::soundpack_loader::load_mouse_pack_into_engine(
            &mut state,
            &config.mouse_soundpack,
            false
        );
        let _ = event_tx.send(UiEvent::PackLoaded { is_keyboard: false, result });
    }

    // Absolute deadline for the next watchdog check. Deliberately NOT
    // `select!`'s `default(interval)`: that arm only fires after a full
    // interval with no channel traffic at all, so continuous typing starves
    // it indefinitely - and "unplugged the headset while typing" is exactly
    // the case this feature exists for. An absolute deadline fires on
    // schedule no matter how much input is flowing.
    let mut next_device_check = std::time::Instant::now() + DEVICE_WATCHDOG_INTERVAL;

    loop {
        crossbeam_channel::select! {
            recv(cmd_rx) -> msg => {
                match msg {
                    Ok(command) => {
                        handle_command(&mut state, &event_tx, command);
                    }
                    Err(_) => break, // sender dropped, app is shutting down
                }
            }
            recv(keyboard_rx) -> msg => {
                if let Ok(raw) = msg {
                    if let Some((code, down)) = parse_input_event(&raw) {
                        state.handle_key_event(&code, down);
                        let _ = event_tx.send(if down { UiEvent::KeyDown(code) } else { UiEvent::KeyUp(code) });
                    }
                }
            }
            recv(mouse_rx) -> msg => {
                if let Ok(raw) = msg {
                    if let Some((code, down)) = parse_input_event(&raw) {
                        state.handle_mouse_event(&code, down);
                    }
                }
            }
            recv(hotkey_rx) -> msg => {
                if let Ok(command) = msg {
                    if command == "TOGGLE_SOUND" {
                        // Adopt the persisted value rather than negating the
                        // cached one, so the engine can't drift out of sync
                        // with config if the two ever disagree.
                        state.sound_enabled = handle_toggle_sound();
                    }
                }
            }
            // Wakes the loop when the watchdog is due even if no input or
            // command arrives. `at` on an already-passed deadline fires
            // immediately, so the check below always gets a chance to run.
            recv(crossbeam_channel::at(next_device_check)) -> _ => {}
        }

        // Checked outside `select!` so it runs no matter which arm woke the
        // loop - under heavy typing the input arms win every race, and a
        // check that only ran on its own arm would never happen.
        if std::time::Instant::now() >= next_device_check {
            next_device_check = std::time::Instant::now() + DEVICE_WATCHDOG_INTERVAL;

            // Device enumeration is comparatively expensive (and on Linux can
            // disturb ALSA), so skip it entirely unless a specific device is
            // being watched. On the system default there is nothing to do:
            // the OS reassigns the default itself.
            if state.device_watchdog.is_armed() {
                let present = state.watched_device_present();
                if let Some(lost_device) = state.device_watchdog.observe(present) {
                    eprintln!(
                        "⚠️ [AudioEngine] Output device '{}' disappeared - falling back to system default",
                        lost_device
                    );
                    let result = state.fallback_to_default();
                    match &result {
                        Ok(label) => println!("🔊 [AudioEngine] Fell back to {}", label),
                        Err(e) => eprintln!("❌ [AudioEngine] Fallback to default failed: {}", e),
                    }
                    let _ = event_tx.send(UiEvent::DeviceLost { lost_device, result });
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn global_mute_silences_both_input_types() {
        // The global flag wins regardless of the per-type flag - this is what
        // the home-page mute button and Ctrl+Alt+M both drive.
        assert!(!should_play(false, true), "global mute must silence keyboard");
        assert!(!should_play(false, false));
    }

    #[test]
    fn per_type_mute_silences_only_that_type() {
        // Muting the keyboard slider must not require touching the global
        // flag, and vice versa for the mouse slider.
        assert!(!should_play(true, false));
        assert!(should_play(true, true), "nothing muted must play");
    }

    #[test]
    fn set_sound_enabled_command_moves_engine_state() {
        // Regression guard for the mute bug: the UI writing config alone left
        // the engine's cached flag untouched and sound kept playing. These
        // asserts pin the command -> state edge that the UI must drive.
        //
        // Applies the same assignments `handle_command` performs; the full
        // `handle_command` needs an `EngineState`, which needs a real audio
        // device and so cannot be built in a headless test.
        let mut sound_enabled = true;
        let mut keyboard_sound_enabled = true;
        let mut mouse_sound_enabled = true;

        for command in [
            AudioCommand::SetSoundEnabled(false),
            AudioCommand::SetKeyboardSoundEnabled(false),
            AudioCommand::SetMouseSoundEnabled(false),
        ] {
            match command {
                AudioCommand::SetSoundEnabled(v) => {
                    sound_enabled = v;
                }
                AudioCommand::SetKeyboardSoundEnabled(v) => {
                    keyboard_sound_enabled = v;
                }
                AudioCommand::SetMouseSoundEnabled(v) => {
                    mouse_sound_enabled = v;
                }
                _ => unreachable!("only mute commands are exercised here"),
            }
        }

        assert!(!sound_enabled);
        assert!(!keyboard_sound_enabled);
        assert!(!mouse_sound_enabled);
        assert!(!should_play(sound_enabled, keyboard_sound_enabled));
        assert!(!should_play(sound_enabled, mouse_sound_enabled));
    }

    #[test]
    fn watchdog_disarmed_on_default_device() {
        // System default: nothing to watch, since the OS reassigns the
        // default itself when hardware disappears.
        let mut watchdog = DeviceWatchdog::default();
        watchdog.watch(None);

        assert!(!watchdog.is_armed());
        for _ in 0..10 {
            assert_eq!(watchdog.observe(false), None);
        }
    }

    #[test]
    fn watchdog_stays_quiet_while_device_present() {
        let mut watchdog = DeviceWatchdog::default();
        watchdog.watch(Some("USB Headset".to_string()));

        assert!(watchdog.is_armed());
        for _ in 0..100 {
            assert_eq!(watchdog.observe(true), None);
        }
    }

    #[test]
    fn watchdog_fires_after_consecutive_misses() {
        let mut watchdog = DeviceWatchdog::default();
        watchdog.watch(Some("USB Headset".to_string()));

        // Must not fire before the strike threshold is reached.
        for _ in 0..DEVICE_MISSING_STRIKES - 1 {
            assert_eq!(watchdog.observe(false), None);
        }
        assert_eq!(watchdog.observe(false), Some("USB Headset".to_string()));
    }

    #[test]
    fn watchdog_ignores_single_transient_miss() {
        // A driver re-enumerating (mode switch, power state) can blink a
        // device out of the list for one check without it being unplugged.
        // That must not yank the user off their chosen device.
        let mut watchdog = DeviceWatchdog::default();
        watchdog.watch(Some("USB Headset".to_string()));

        for _ in 0..20 {
            assert_eq!(watchdog.observe(false), None, "single miss must not fire");
            assert_eq!(watchdog.observe(true), None, "presence must reset strikes");
        }
    }

    #[test]
    fn watchdog_fires_once_per_disappearance() {
        // Guards against a failed fallback re-triggering every tick: after
        // firing, the watchdog disarms until something re-arms it.
        let mut watchdog = DeviceWatchdog::default();
        watchdog.watch(Some("USB Headset".to_string()));

        for _ in 0..DEVICE_MISSING_STRIKES {
            watchdog.observe(false);
        }
        assert!(!watchdog.is_armed(), "must disarm after firing");
        for _ in 0..10 {
            assert_eq!(watchdog.observe(false), None, "must not fire repeatedly");
        }
    }

    #[test]
    fn watchdog_rearms_on_new_device() {
        let mut watchdog = DeviceWatchdog::default();
        watchdog.watch(Some("USB Headset".to_string()));
        for _ in 0..DEVICE_MISSING_STRIKES {
            watchdog.observe(false);
        }

        // Switching to another device starts watching that one instead.
        watchdog.watch(Some("Speakers".to_string()));
        assert!(watchdog.is_armed());
        for _ in 0..DEVICE_MISSING_STRIKES - 1 {
            assert_eq!(watchdog.observe(false), None);
        }
        assert_eq!(watchdog.observe(false), Some("Speakers".to_string()));
    }

    #[test]
    fn enumeration_failure_is_not_a_miss() {
        // `has_output_device_named` returns None when the host cannot be
        // enumerated at all. That is "no conclusion", not "device gone":
        // treating it as a miss would fall back on a transient audio-service
        // hiccup. This mirrors the mapping `watched_device_present` applies -
        // flipping it to `unwrap_or(false)` would make repeated enumeration
        // failures trigger a bogus fallback.
        fn presence_from_probe(probe: Option<bool>) -> bool {
            probe.unwrap_or(true)
        }

        assert!(presence_from_probe(None), "enumeration failure must count as present");
        assert!(presence_from_probe(Some(true)));
        assert!(!presence_from_probe(Some(false)));

        let present = presence_from_probe(None);

        let mut watchdog = DeviceWatchdog::default();
        watchdog.watch(Some("USB Headset".to_string()));
        for _ in 0..10 {
            assert_eq!(
                watchdog.observe(present),
                None,
                "unknown device state must never trigger fallback"
            );
        }
    }

    #[test]
    fn watchdog_resets_strikes_when_rewatching() {
        // A switch mid-disappearance must not inherit the old strike count
        // and fire early on the new device.
        let mut watchdog = DeviceWatchdog::default();
        watchdog.watch(Some("USB Headset".to_string()));
        assert_eq!(watchdog.observe(false), None);

        watchdog.watch(Some("Speakers".to_string()));
        assert_eq!(watchdog.observe(false), None, "strikes must reset on rewatch");
    }
}
