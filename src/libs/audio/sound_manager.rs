use rodio::buffer::SamplesBuffer;
use rodio::Sink;
use std::collections::HashMap;
use rand::Rng;

use super::audio_context::AudioContext;
use crate::state::config::AppConfig;

impl AudioContext {
    pub fn play_key_event_sound(&self, key: &str, is_keydown: bool) {
        // println!(
        //     "⌨️ Key event received: {} ({})",
        //     key,
        //     if is_keydown { "down" } else { "up" }
        // );

        // Check enable_sound from config before playing audio
        let config = AppConfig::load();
        if !config.enable_sound || !config.enable_keyboard_sound {
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
        drop(pressed); // Get timestamp and end time - check if random sounds are enabled
        let key_map = self.key_map.lock().unwrap();
        let (start, end) = if config.enable_random_sounds {
            self.get_random_sound_mapping(&key_map, is_keydown)
        } else {
            self.get_original_sound_mapping(&key_map, key, is_keydown)
        };
        drop(key_map);

        if let (Some(start), Some(end)) = (start, end) {
            self.play_sound_segment(key, start, end, is_keydown);
        }
    }
    fn play_sound_segment(&self, key: &str, start: f32, end: f32, is_keydown: bool) {
        let pcm_opt = self.keyboard_samples.lock().unwrap().clone();
        if let Some((samples, channels, sample_rate)) = pcm_opt {
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
            // eprintln!(
            //     "🔍 Playing sound for key '{}': start={:.3}ms, end={:.3}ms, duration={:.3}ms (total duration: {:.3}ms)",
            //     key,
            //     start,
            //     end,
            //     duration,
            //     total_duration
            // );

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
                    let segment = SamplesBuffer::new(channels, sample_rate, segment_samples);

                    if let Ok(sink) = Sink::try_new(&self.stream_handle) {
                        sink.set_volume(self.get_volume());
                        sink.append(segment);

                        let mut key_sinks = self.key_sinks.lock().unwrap();
                        self.manage_active_sinks(&mut key_sinks);
                        key_sinks.insert(
                            format!("{}-{}", key, if is_keydown { "down" } else { "up" }),
                            sink
                        );
                    }
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
            let segment = SamplesBuffer::new(channels, sample_rate, segment_samples);

            if let Ok(sink) = Sink::try_new(&self.stream_handle) {
                sink.set_volume(self.get_volume());
                sink.append(segment);

                let mut key_sinks = self.key_sinks.lock().unwrap();
                self.manage_active_sinks(&mut key_sinks);
                key_sinks.insert(
                    format!("{}-{}", key, if is_keydown { "down" } else { "up" }),
                    sink
                );
            }
        } else {
            eprintln!("❌ No keyboard PCM buffer available");
        }
    }

    fn manage_active_sinks(&self, key_sinks: &mut std::sync::MutexGuard<HashMap<String, Sink>>) {
        // First, clean up finished sinks (those that have stopped playing)
        let finished_keys: Vec<String> = key_sinks
            .iter()
            .filter(|(_, sink)| sink.empty())
            .map(|(key, _)| key.clone())
            .collect();

        for key in finished_keys {
            key_sinks.remove(&key);
        }

        // Only remove active sinks if we still exceed max_voices after cleanup
        if key_sinks.len() >= self.max_voices {
            // Find the oldest sink (first in iteration order) and remove it
            if let Some((old_key, _)) = key_sinks.iter().next().map(|(k, _)| (k.clone(), ())) {
                key_sinks.remove(&old_key);
                let mut pressed = self.key_pressed.lock().unwrap();
                pressed.insert(old_key, false);
            }
        }
    }

    pub fn play_mouse_event_sound(&self, button: &str, is_buttondown: bool) {
        // Check enable_sound from config before playing audio
        let config = AppConfig::load();
        if !config.enable_sound || !config.enable_mouse_sound {
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

        // Get timestamp and duration - check if random sounds are enabled
        let mouse_map = self.mouse_map.lock().unwrap();
        let (start, duration) = if config.enable_random_sounds {
            self.get_random_mouse_sound_mapping(&mouse_map, is_buttondown)
        } else {
            self.get_original_mouse_sound_mapping(&mouse_map, button, is_buttondown)
        };
        drop(mouse_map);

        if let (Some(start), Some(duration)) = (start, duration) {
            self.play_mouse_sound_segment(button, start, duration, is_buttondown);
        }
    }

    fn play_mouse_sound_segment(
        &self,
        button: &str,
        start: f32,
        duration: f32,
        is_buttondown: bool
    ) {
        let pcm_opt = self.mouse_samples.lock().unwrap().clone();
        if let Some((samples, channels, sample_rate)) = pcm_opt {
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
            let segment = SamplesBuffer::new(channels, sample_rate, segment_samples);

            if let Ok(sink) = Sink::try_new(&self.stream_handle) {
                sink.set_volume(self.get_mouse_volume());
                sink.append(segment);

                let mut mouse_sinks = self.mouse_sinks.lock().unwrap();
                self.manage_active_mouse_sinks(&mut mouse_sinks);
                mouse_sinks.insert(
                    format!("{}-{}", button, if is_buttondown { "down" } else { "up" }),
                    sink
                );
            }
        } else {
            eprintln!("❌ No mouse PCM buffer available");
        }
    }

    fn get_original_sound_mapping(
        &self,
        key_map: &HashMap<String, Vec<[f32; 2]>>,
        key: &str,
        is_keydown: bool
    ) -> (Option<f32>, Option<f32>) {
        match key_map.get(key) {
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

                (Some(start), Some(end))
            }
            Some(arr) if arr.len() == 1 => {
                // Only keydown mapping available, ignore keyup events
                if !is_keydown {
                    return (None, None); // Skip keyup events for keys with only keydown mapping
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

                (Some(start), Some(end))
            }
            Some(arr) => {
                eprintln!(
                    "Invalid mapping for key '{}': expected 1-2 elements, got {}",
                    key,
                    arr.len()
                );
                (None, None)
            }
            None => {
                // Silently ignore unmapped keys to reduce noise
                (None, None)
            }
        }
    }

    fn get_random_sound_mapping(
        &self,
        key_map: &HashMap<String, Vec<[f32; 2]>>,
        is_keydown: bool
    ) -> (Option<f32>, Option<f32>) {
        // Get all available key mappings
        let available_keys: Vec<&String> = key_map.keys().collect();

        if available_keys.is_empty() {
            return (None, None);
        }

        // Choose a random key mapping
        let mut rng = rand::thread_rng();
        let random_index = rng.gen_range(0..available_keys.len());
        let random_key = available_keys[random_index];

        if let Some(arr) = key_map.get(random_key) {
            if arr.len() == 2 {
                let idx = if is_keydown { 0 } else { 1 };
                let arr = arr[idx];
                let start = arr[0]; // Keep in milliseconds
                let end = arr[1]; // This is end time
                let duration = end - start; // Calculate duration for validation only

                // Debug logging for problematic keys
                if start < 0.0 || duration <= 0.0 || duration > 10000.0 {
                    eprintln!(
                        "⚠️ Suspicious random mapping from key '{}' ({}): start={:.3}ms, end={:.3}ms, duration={:.3}ms (raw: [{}, {}])",
                        random_key,
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

                (Some(start), Some(end))
            } else if arr.len() == 1 {
                // Only keydown mapping available, ignore keyup events
                if !is_keydown {
                    return (None, None); // Skip keyup events for keys with only keydown mapping
                }
                let arr = arr[0];
                let start = arr[0]; // Keep in milliseconds
                let end = arr[1]; // This is end time
                let duration = end - start; // Calculate duration for validation only

                // Debug logging for problematic keys
                if start < 0.0 || duration <= 0.0 || duration > 10000.0 {
                    eprintln!(
                        "⚠️ Suspicious random mapping from key '{}': start={:.3}ms, end={:.3}ms, duration={:.3}ms (raw: [{}, {}])",
                        random_key,
                        start,
                        end,
                        duration,
                        arr[0],
                        arr[1]
                    );
                }

                (Some(start), Some(end))
            } else {
                eprintln!(
                    "Invalid random mapping from key '{}': expected 1-2 elements, got {}",
                    random_key,
                    arr.len()
                );
                (None, None)
            }
        } else {
            (None, None)
        }
    }

    fn get_original_mouse_sound_mapping(
        &self,
        mouse_map: &HashMap<String, Vec<[f32; 2]>>,
        button: &str,
        is_buttondown: bool
    ) -> (Option<f32>, Option<f32>) {
        match mouse_map.get(button) {
            Some(arr) if arr.len() == 2 => {
                let idx = if is_buttondown { 0 } else { 1 };
                let arr = arr[idx];
                let start = arr[0]; // Keep in milliseconds
                let end = arr[1]; // This is actually end time, not duration
                let duration = end - start; // Calculate duration from start and end
                (Some(start), Some(duration))
            }
            Some(arr) if arr.len() == 1 => {
                // Only buttondown mapping available, ignore buttonup events
                if !is_buttondown {
                    return (None, None); // Skip buttonup events for buttons with only buttondown mapping
                }
                let arr = arr[0];
                let start = arr[0]; // Keep in milliseconds
                let end = arr[1]; // This is actually end time, not duration
                let duration = end - start; // Calculate duration from start and end
                (Some(start), Some(duration))
            }
            Some(arr) => {
                eprintln!(
                    "Invalid mapping for mouse button '{}': expected 1-2 elements, got {}",
                    button,
                    arr.len()
                );
                (None, None)
            }
            None => {
                // Silently ignore unmapped mouse buttons to reduce noise
                (None, None)
            }
        }
    }

    fn get_random_mouse_sound_mapping(
        &self,
        mouse_map: &HashMap<String, Vec<[f32; 2]>>,
        is_buttondown: bool
    ) -> (Option<f32>, Option<f32>) {
        // Get all available mouse button mappings
        let available_buttons: Vec<&String> = mouse_map.keys().collect();

        if available_buttons.is_empty() {
            return (None, None);
        }

        // Choose a random button mapping
        let mut rng = rand::thread_rng();
        let random_index = rng.gen_range(0..available_buttons.len());
        let random_button = available_buttons[random_index];

        if let Some(arr) = mouse_map.get(random_button) {
            if arr.len() == 2 {
                let idx = if is_buttondown { 0 } else { 1 };
                let arr = arr[idx];
                let start = arr[0]; // Keep in milliseconds
                let end = arr[1]; // This is actually end time, not duration
                let duration = end - start; // Calculate duration from start and end
                (Some(start), Some(duration))
            } else if arr.len() == 1 {
                // Only buttondown mapping available, ignore buttonup events
                if !is_buttondown {
                    return (None, None); // Skip buttonup events for buttons with only buttondown mapping
                }
                let arr = arr[0];
                let start = arr[0]; // Keep in milliseconds
                let end = arr[1]; // This is actually end time, not duration
                let duration = end - start; // Calculate duration from start and end
                (Some(start), Some(duration))
            } else {
                eprintln!(
                    "Invalid random mapping from mouse button '{}': expected 1-2 elements, got {}",
                    random_button,
                    arr.len()
                );
                (None, None)
            }
        } else {
            (None, None)
        }
    }

    fn manage_active_mouse_sinks(
        &self,
        mouse_sinks: &mut std::sync::MutexGuard<HashMap<String, Sink>>
    ) {
        // First, clean up finished sinks (those that have stopped playing)
        let finished_buttons: Vec<String> = mouse_sinks
            .iter()
            .filter(|(_, sink)| sink.empty())
            .map(|(button, _)| button.clone())
            .collect();

        for button in finished_buttons {
            mouse_sinks.remove(&button);
        }

        // Only remove active sinks if we still exceed max_voices after cleanup
        if mouse_sinks.len() >= self.max_voices {
            // Find the oldest sink (first in iteration order) and remove it
            if let Some((old_button, _)) = mouse_sinks.iter().next().map(|(k, _)| (k.clone(), ())) {
                mouse_sinks.remove(&old_button);
                let mut pressed = self.mouse_pressed.lock().unwrap();
                pressed.insert(old_button, false);
            }
        }
    }

    /// Clean up finished sinks to prevent memory leaks and improve performance
    pub fn cleanup_finished_sinks(&self) {
        // Clean up finished keyboard sinks
        if let Ok(mut key_sinks) = self.key_sinks.lock() {
            let finished_keys: Vec<String> = key_sinks
                .iter()
                .filter(|(_, sink)| sink.empty())
                .map(|(key, _)| key.clone())
                .collect();

            for key in finished_keys {
                key_sinks.remove(&key);
            }
        }

        // Clean up finished mouse sinks
        if let Ok(mut mouse_sinks) = self.mouse_sinks.lock() {
            let finished_buttons: Vec<String> = mouse_sinks
                .iter()
                .filter(|(_, sink)| sink.empty())
                .map(|(button, _)| button.clone())
                .collect();

            for button in finished_buttons {
                mouse_sinks.remove(&button);
            }
        }
    }
}
