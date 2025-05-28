use rodio::buffer::SamplesBuffer;
use rodio::Sink;
use std::collections::HashMap;

use super::audio_context::AudioContext;
use crate::state::config::AppConfig;

impl AudioContext {
    pub fn play_key_event_sound(&self, key: &str, is_keydown: bool) {
        // Check enable_sound from config before playing audio
        let config = AppConfig::load();
        if !config.enable_sound {
            println!("🔕 Sound disabled in config, skipping key event: '{}'", key);
            return;
        }

        println!(
            "🔍 Handling key event: '{}' ({})",
            key,
            if is_keydown { "down" } else { "up" }
        );

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
                println!("✅ Found mapping for key '{}': {:?}", key, arr);
                let idx = if is_keydown { 0 } else { 1 };
                let arr = arr[idx];
                (arr[0] / 1000.0, arr[1] / 1000.0)
            }
            Some(arr) => {
                println!(
                    "⚠️ Invalid mapping for key '{}': {:?} (expected 2 elements)",
                    key, arr
                );
                return;
            }
            None => {
                println!("❌ No mapping for key '{}' in current soundpack", key);
                println!(
                    "   Available keys: {:?}",
                    key_map.keys().take(10).collect::<Vec<_>>()
                );
                return;
            }
        };
        drop(key_map);

        self.play_sound_segment(key, start, duration, is_keydown);
    }

    fn play_sound_segment(&self, key: &str, start: f32, duration: f32, is_keydown: bool) {
        let pcm_opt = self.cached_samples.lock().unwrap().clone();

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
            eprintln!("❌ No cached PCM buffer available");
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
}
