use rodio::buffer::SamplesBuffer;
use rodio::{ OutputStreamHandle, Sink };
use std::sync::{ Arc, Mutex };
use std::time::Duration;

use super::audio_context::AudioContext;

const FADE_IN_MS: f32 = 2.0;
const FADE_OUT_MS: f32 = 5.0;
const EVICT_RAMP_MS: u64 = 10;

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

    // Scale down fades on very short segments so in/out never overlap-cancel.
    // A segment of 0-1 frames has no room for any fade (half == 0), which the
    // clamp below already expresses correctly.
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

/// Builds a faded sink for `segment` and pushes it onto the voice pool,
/// evicting old voices softly if the pool is full.
fn spawn_voice(
    stream_handle: &OutputStreamHandle,
    mut segment_samples: Vec<f32>,
    channels: u16,
    sample_rate: u32,
    volume: f32,
    sinks: &Arc<Mutex<Vec<Sink>>>,
    max_voices: usize
) {
    apply_fade(&mut segment_samples, channels, sample_rate);
    let segment = SamplesBuffer::new(channels, sample_rate, segment_samples);

    if let Ok(sink) = Sink::try_new(stream_handle) {
        sink.set_volume(volume);
        sink.append(segment);

        let mut pool = sinks.lock().unwrap();
        manage_active_sinks(&mut pool, max_voices);
        pool.push(sink);
    }
}

impl AudioContext {
    pub fn play_key_event_sound(&self, key: &str, is_keydown: bool) {
        // Check enable_sound from cached config (no file I/O in hot path)
        if !self.is_sound_enabled() || !self.is_keyboard_sound_enabled() {
            return;
        }

        let mut pressed = self.key_pressed.lock().unwrap();
        if is_keydown {
            if *pressed.get(key).unwrap_or(&false) {
                return;
            }
            pressed.insert(key.to_string(), true);
        } else {
            if !*pressed.get(key).unwrap_or(&false) {
                return;
            }
            pressed.insert(key.to_string(), false);
        }
        drop(pressed); // Get timestamp and end time
        let key_map = self.key_map.lock().unwrap();
        let (start, end) = match key_map.get(key) {
            Some(arr) if arr.len() == 2 => {
                let idx = if is_keydown { 0 } else { 1 };
                let arr = arr[idx];
                let start = arr[0]; // Keep in milliseconds
                let end = arr[1]; // This is end time
                let duration = end - start; // Calculate duration for validation only

                // Debug logging for problematic keys
                if start < 0.0 || duration <= 0.0 || duration > 10000.0 {
                    eprintln!(
                        "⚠️ Suspicious mapping for key '{}' ({}): start={:.3}ms, end={:.3}ms, duration={:.3}ms (raw: [{}, {}])",
                        key,
                        if is_keydown {
                            "down"
                        } else {
                            "up"
                        },
                        start,
                        end,
                        duration,
                        arr[0],
                        arr[1]
                    );
                }

                (start, end)
            }
            Some(arr) if arr.len() == 1 => {
                // Only keydown mapping available, ignore keyup events
                if !is_keydown {
                    return; // Skip keyup events for keys with only keydown mapping
                }
                let arr = arr[0];
                let start = arr[0]; // Keep in milliseconds
                let end = arr[1]; // This is end time
                let duration = end - start; // Calculate duration for validation only

                // Debug logging for problematic keys
                if start < 0.0 || duration <= 0.0 || duration > 10000.0 {
                    eprintln!(
                        "⚠️ Suspicious mapping for key '{}': start={:.3}ms, end={:.3}ms, duration={:.3}ms (raw: [{}, {}])",
                        key,
                        start,
                        end,
                        duration,
                        arr[0],
                        arr[1]
                    );
                }

                (start, end)
            }
            Some(arr) => {
                eprintln!(
                    "Invalid mapping for key '{}': expected 1-2 elements, got {}",
                    key,
                    arr.len()
                );
                return;
            }
            None => {
                // Silently ignore unmapped keys to reduce noise
                return;
            }
        };
        drop(key_map);

        self.play_sound_segment(key, start, end);
    }
    fn play_sound_segment(&self, key: &str, start: f32, end: f32) {
        // Clone Arc pointer (8 bytes) instead of entire Vec (potentially MBs)
        let pcm_opt = self.keyboard_samples.lock().unwrap().clone();
        if let Some((samples_arc, channels, sample_rate)) = pcm_opt {
            let samples = &**samples_arc; // Deref Arc to access Vec
            // Calculate total audio duration in milliseconds
            let total_duration =
                ((samples.len() as f32) / (sample_rate as f32) / (channels as f32)) * 1000.0;

            // Calculate duration from start and end times
            let duration = end - start;

            // Validate input parameters
            if start < 0.0 || duration <= 0.0 || end <= start {
                eprintln!(
                    "❌ Invalid time parameters for key '{}': start={:.3}ms, end={:.3}ms, duration={:.3}ms",
                    key,
                    start,
                    end,
                    duration
                );
                return;
            }
            // Use epsilon tolerance for floating point comparison (1ms tolerance)
            const EPSILON: f32 = 1.0; // 1ms tolerance

            // Check if start time exceeds audio duration - this is an error condition
            if start >= total_duration + EPSILON {
                eprintln!(
                    "❌ TIMING ERROR: Start time {:.3}ms exceeds audio duration {:.3}ms for key '{}'",
                    start,
                    total_duration,
                    key
                );
                return;
            }

            // Check if end time exceeds audio duration
            if end > total_duration + EPSILON {
                eprintln!(
                    "❌ TIMING ERROR: Audio segment {:.3}ms-{:.3}ms exceeds duration {:.3}ms for key '{}'",
                    start,
                    end,
                    total_duration,
                    key
                );
                return;
            }

            // Calculate sample positions (convert milliseconds to seconds for sample calculation)
            let start_sample = ((start / 1000.0) *
                (sample_rate as f32) *
                (channels as f32)) as usize;
            let end_sample = ((end / 1000.0) * (sample_rate as f32) * (channels as f32)) as usize;

            // Validate sample range with safety checks
            if end_sample > samples.len() {
                // Try to clamp end_sample to available samples
                let max_available_sample = samples.len();
                let clamped_end_sample = max_available_sample;
                let clamped_end_time =
                    ((clamped_end_sample as f32) / (sample_rate as f32) / (channels as f32)) *
                    1000.0;
                let clamped_duration = clamped_end_time - start;

                // Use clamped values if they're reasonable
                if clamped_duration > 1.0 && clamped_end_sample > start_sample {
                    let segment_samples = samples[start_sample..clamped_end_sample].to_vec();
                    spawn_voice(
                        &self.stream_handle,
                        segment_samples,
                        channels,
                        sample_rate,
                        self.get_volume(),
                        &self.key_sinks,
                        self.max_voices
                    );
                    return;
                }

                return;
            }

            // Final validation before extracting samples
            if start_sample >= end_sample || start_sample >= samples.len() {
                eprintln!(
                    "❌ INTERNAL ERROR: Invalid sample range for key '{}': {}..{} (max {})",
                    key,
                    start_sample,
                    end_sample,
                    samples.len()
                );
                eprintln!(
                    "   Audio: {:.3}ms, Channels: {}, Rate: {}",
                    total_duration,
                    channels,
                    sample_rate
                );
                return;
            }

            let segment_samples = samples[start_sample..end_sample].to_vec();
            spawn_voice(
                &self.stream_handle,
                segment_samples,
                channels,
                sample_rate,
                self.get_volume(),
                &self.key_sinks,
                self.max_voices
            );
        } else {
            eprintln!("❌ No keyboard PCM buffer available");
        }
    }

    pub fn play_mouse_event_sound(&self, button: &str, is_buttondown: bool) {
        // Check enable_sound from cached config (no file I/O in hot path)
        if !self.is_sound_enabled() || !self.is_mouse_sound_enabled() {
            return;
        }

        let mut pressed = self.mouse_pressed.lock().unwrap();
        if is_buttondown {
            if *pressed.get(button).unwrap_or(&false) {
                return;
            }
            pressed.insert(button.to_string(), true);
        } else {
            if !*pressed.get(button).unwrap_or(&false) {
                return;
            }
            pressed.insert(button.to_string(), false);
        }
        drop(pressed);

        // Get timestamp and duration
        let mouse_map = self.mouse_map.lock().unwrap();
        let (start, duration) = match mouse_map.get(button) {
            Some(arr) if arr.len() == 2 => {
                let idx = if is_buttondown { 0 } else { 1 };
                let arr = arr[idx];
                let start = arr[0]; // Keep in milliseconds
                let end = arr[1]; // This is actually end time, not duration
                let duration = end - start; // Calculate duration from start and end
                (start, duration)
            }
            Some(arr) if arr.len() == 1 => {
                // Only buttondown mapping available, ignore buttonup events
                if !is_buttondown {
                    return; // Skip buttonup events for buttons with only buttondown mapping
                }
                let arr = arr[0];
                let start = arr[0]; // Keep in milliseconds
                let end = arr[1]; // This is actually end time, not duration
                let duration = end - start; // Calculate duration from start and end
                (start, duration)
            }
            Some(arr) => {
                eprintln!(
                    "Invalid mapping for mouse button '{}': expected 1-2 elements, got {}",
                    button,
                    arr.len()
                );
                return;
            }
            None => {
                // Silently ignore unmapped mouse buttons to reduce noise
                return;
            }
        };
        drop(mouse_map);

        self.play_mouse_sound_segment(button, start, duration);
    }

    fn play_mouse_sound_segment(&self, button: &str, start: f32, duration: f32) {
        // Clone Arc pointer (8 bytes) instead of entire Vec (potentially MBs)
        let pcm_opt = self.mouse_samples.lock().unwrap().clone();
        if let Some((samples_arc, channels, sample_rate)) = pcm_opt {
            let samples = &**samples_arc; // Deref Arc to access Vec
            // Calculate total audio duration in milliseconds
            let total_duration =
                ((samples.len() as f32) / (sample_rate as f32) / (channels as f32)) * 1000.0;

            // Validate input parameters
            if start < 0.0 || duration <= 0.0 {
                eprintln!(
                    "❌ Invalid time parameters for mouse button '{}': start={:.3}ms, duration={:.3}ms",
                    button,
                    start,
                    duration
                );
                return;
            } // Use epsilon tolerance for floating point comparison (1ms tolerance)
            const EPSILON: f32 = 1.0; // 1ms tolerance

            // Check if start time exceeds audio duration - this is an error condition
            if start >= total_duration + EPSILON {
                eprintln!(
                    "❌ TIMING ERROR: Start time {:.3}ms exceeds audio duration {:.3}ms for mouse button '{}'",
                    start,
                    total_duration,
                    button
                );
                return;
            }

            // Check if start + duration exceeds audio duration
            if start + duration > total_duration + EPSILON {
                eprintln!(
                    "❌ TIMING ERROR: Audio segment {:.3}ms-{:.3}ms exceeds duration {:.3}ms for mouse button '{}'",
                    start,
                    start + duration,
                    total_duration,
                    button
                );
                return;
            }

            // Use exact timing - no clamping or fallbacks
            let end_time = start + duration;

            // Calculate sample positions (convert milliseconds to seconds for sample calculation)
            let start_sample = ((start / 1000.0) *
                (sample_rate as f32) *
                (channels as f32)) as usize;
            let end_sample = ((end_time / 1000.0) *
                (sample_rate as f32) *
                (channels as f32)) as usize;

            // Validate sample range
            if end_sample > samples.len() {
                eprintln!("❌ TIMING ERROR: Audio segment exceeds sample buffer for mouse button '{}'", button);
                eprintln!(
                    "   Requested samples: {}..{}, Available: {} samples",
                    start_sample,
                    end_sample,
                    samples.len()
                );
                eprintln!("🔧 SOLUTION: Regenerate the soundpack to fix timing issues.");
                return;
            } // Final validation before extracting samples
            if start_sample >= end_sample || start_sample >= samples.len() {
                eprintln!(
                    "❌ INTERNAL ERROR: Invalid sample range for mouse button '{}': {}..{} (max {})",
                    button,
                    start_sample,
                    end_sample,
                    samples.len()
                );
                eprintln!(
                    "   Audio: {:.3}ms, Channels: {}, Rate: {}",
                    total_duration,
                    channels,
                    sample_rate
                );
                return;
            }

            let segment_samples = samples[start_sample..end_sample].to_vec();
            spawn_voice(
                &self.stream_handle,
                segment_samples,
                channels,
                sample_rate,
                self.get_mouse_volume(),
                &self.mouse_sinks,
                self.max_voices
            );
        } else {
            eprintln!("❌ No mouse PCM buffer available");
        }
    }
}
