use rodio::buffer::SamplesBuffer;
use rodio::Sink;
use std::collections::HashMap;

use super::audio_context::AudioContext;
use crate::state::config::AppConfig;

impl AudioContext {
    pub fn play_key_event_sound(&self, key: &str, is_keydown: bool) {
        println!(
            "⌨️ Key event received: {} ({})",
            key,
            if is_keydown { "down" } else { "up" }
        );

        // Check enable_sound from config before playing audio
        let config = AppConfig::load();
        if !config.enable_sound {
            println!("🔇 Sound disabled in config, skipping key event");
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
        drop(pressed);
        // Get timestamp and duration
        let key_map = self.key_map.lock().unwrap();
        let (start, duration) = match key_map.get(key) {
            Some(arr) if arr.len() == 2 => {
                let idx = if is_keydown { 0 } else { 1 };
                let arr = arr[idx];
                (arr[0] / 1000.0, arr[1] / 1000.0)
            }
            Some(arr) => {
                eprintln!(
                    "Invalid mapping for key '{}': expected 2 elements, got {}",
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

        self.play_sound_segment(key, start, duration, is_keydown);
    }
    fn play_sound_segment(&self, key: &str, start: f32, duration: f32, is_keydown: bool) {
        let pcm_opt = self.keyboard_samples.lock().unwrap().clone();

        if let Some((samples, channels, sample_rate)) = pcm_opt {
            let start_sample = (start * sample_rate as f32 * channels as f32) as usize;
            let end_sample = ((start + duration) * sample_rate as f32 * channels as f32) as usize;
            let end_sample = end_sample.min(samples.len());

            if start_sample >= end_sample || start_sample >= samples.len() {
                eprintln!(
                    "❌ Invalid sample range for key '{}': {}..{} (max {})",
                    key,
                    start_sample,
                    end_sample,
                    samples.len()
                );
                return;
            }

            let segment_samples = samples[start_sample..end_sample].to_vec();
            let segment = SamplesBuffer::new(channels, sample_rate, segment_samples);

            if let Ok(sink) = Sink::try_new(&self.stream_handle) {
                sink.set_volume(self.get_volume() * 5.0);
                sink.append(segment);

                let mut key_sinks = self.key_sinks.lock().unwrap();
                self.manage_active_sinks(&mut key_sinks);
                key_sinks.insert(
                    format!("{}-{}", key, if is_keydown { "down" } else { "up" }),
                    sink,
                );
            }
        } else {
            eprintln!("❌ No keyboard PCM buffer available");
        }
    }

    fn manage_active_sinks(&self, key_sinks: &mut std::sync::MutexGuard<HashMap<String, Sink>>) {
        if key_sinks.len() >= self.max_voices {
            if let Some((old_key, _)) = key_sinks.iter().next().map(|(k, _)| (k.clone(), ())) {
                key_sinks.remove(&old_key);
                let mut pressed = self.key_pressed.lock().unwrap();
                pressed.insert(old_key, false);
            }
        }
    }
    pub fn play_mouse_event_sound(&self, button: &str, is_buttondown: bool) {
        println!(
            "🖱️ Mouse event received: {} ({})",
            button,
            if is_buttondown { "down" } else { "up" }
        );

        // Check enable_sound from config before playing audio
        let config = AppConfig::load();
        if !config.enable_sound {
            println!("🔇 Sound disabled in config, skipping mouse event");
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
                (arr[0] / 1000.0, arr[1] / 1000.0)
            }
            Some(arr) => {
                eprintln!(
                    "Invalid mapping for mouse button '{}': expected 2 elements, got {}",
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

        self.play_mouse_sound_segment(button, start, duration, is_buttondown);
    }

    fn play_mouse_sound_segment(
        &self,
        button: &str,
        start: f32,
        duration: f32,
        is_buttondown: bool,
    ) {
        let pcm_opt = self.mouse_samples.lock().unwrap().clone();

        if let Some((samples, channels, sample_rate)) = pcm_opt {
            let start_sample = (start * sample_rate as f32 * channels as f32) as usize;
            let end_sample = ((start + duration) * sample_rate as f32 * channels as f32) as usize;
            let end_sample = end_sample.min(samples.len());

            if start_sample >= end_sample || start_sample >= samples.len() {
                eprintln!(
                    "Invalid sample range for mouse button '{}': {}..{} (max {})",
                    button,
                    start_sample,
                    end_sample,
                    samples.len()
                );
                return;
            }
            let segment_samples = samples[start_sample..end_sample].to_vec();
            let segment = SamplesBuffer::new(channels, sample_rate, segment_samples);

            if let Ok(sink) = Sink::try_new(&self.stream_handle) {
                sink.set_volume(self.get_mouse_volume() * 5.0);
                sink.append(segment);

                let mut mouse_sinks = self.mouse_sinks.lock().unwrap();
                self.manage_active_mouse_sinks(&mut mouse_sinks);
                mouse_sinks.insert(
                    format!("{}-{}", button, if is_buttondown { "down" } else { "up" }),
                    sink,
                );
            }
        } else {
            eprintln!("❌ No mouse PCM buffer available");
        }
    }

    fn manage_active_mouse_sinks(
        &self,
        mouse_sinks: &mut std::sync::MutexGuard<HashMap<String, Sink>>,
    ) {
        if mouse_sinks.len() >= self.max_voices {
            if let Some((old_button, _)) = mouse_sinks.iter().next().map(|(k, _)| (k.clone(), ())) {
                mouse_sinks.remove(&old_button);
                let mut pressed = self.mouse_pressed.lock().unwrap();
                pressed.insert(old_button, false);
            }
        }
    }
}
