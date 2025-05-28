use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompressedAudioData {
    pub samples: Vec<f32>,
    pub sample_rate: u32,
    pub channels: u16,
    pub key_mappings: HashMap<String, Vec<(f64, f64)>>,
    pub soundpack_info: SoundpackInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SoundpackInfo {
    pub id: String,
    pub name: String,
    pub version: String,
    pub compressed_at: u64,
    pub original_size: u64,
    pub compressed_size: u64,
}

impl CompressedAudioData {
    pub fn new(
        samples: Vec<f32>,
        sample_rate: u32,
        channels: u16,
        key_mappings: HashMap<String, Vec<(f64, f64)>>,
        soundpack_id: String,
        soundpack_name: String,
        version: String,
    ) -> Self {
        let original_size = (samples.len() * std::mem::size_of::<f32>()) as u64;

        Self {
            samples,
            sample_rate,
            channels,
            key_mappings,
            soundpack_info: SoundpackInfo {
                id: soundpack_id,
                name: soundpack_name,
                version,
                compressed_at: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs(),
                original_size,
                compressed_size: 0, // Will be set after compression
            },
        }
    }

    pub fn compress(&self) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        // Use bincode for efficient binary serialization
        let compressed = bincode::serialize(self)?;
        println!(
            "🗜️  Compressed audio data: {} -> {} bytes ({:.1}% reduction)",
            self.soundpack_info.original_size,
            compressed.len(),
            (1.0 - compressed.len() as f64 / self.soundpack_info.original_size as f64) * 100.0
        );
        Ok(compressed)
    }

    pub fn decompress(data: &[u8]) -> Result<Self, Box<dyn std::error::Error>> {
        let mut decompressed: Self = bincode::deserialize(data)?;
        decompressed.soundpack_info.compressed_size = data.len() as u64;
        println!(
            "📦 Decompressed audio data for {}: {} samples",
            decompressed.soundpack_info.name,
            decompressed.samples.len()
        );
        Ok(decompressed)
    }

    pub fn get_compression_ratio(&self) -> f64 {
        if self.soundpack_info.compressed_size > 0 {
            self.soundpack_info.compressed_size as f64 / self.soundpack_info.original_size as f64
        } else {
            1.0
        }
    }

    pub fn get_stats(&self) -> CompressionStats {
        CompressionStats {
            original_size: self.soundpack_info.original_size,
            compressed_size: self.soundpack_info.compressed_size,
            samples_count: self.samples.len(),
            key_mappings_count: self.key_mappings.len(),
            compression_ratio: self.get_compression_ratio(),
        }
    }
}

#[derive(Debug)]
pub struct CompressionStats {
    pub original_size: u64,
    pub compressed_size: u64,
    pub samples_count: usize,
    pub key_mappings_count: usize,
    pub compression_ratio: f64,
}

impl CompressionStats {
    pub fn format_original_size(&self) -> String {
        format_bytes(self.original_size)
    }

    pub fn format_compressed_size(&self) -> String {
        format_bytes(self.compressed_size)
    }

    pub fn format_savings(&self) -> String {
        if self.compressed_size > 0 {
            let savings = (1.0 - self.compression_ratio) * 100.0;
            format!("{:.1}%", savings)
        } else {
            "N/A".to_string()
        }
    }
}

fn format_bytes(bytes: u64) -> String {
    if bytes < 1024 {
        format!("{} B", bytes)
    } else if bytes < 1024 * 1024 {
        format!("{:.2} KB", bytes as f64 / 1024.0)
    } else if bytes < 1024 * 1024 * 1024 {
        format!("{:.2} MB", bytes as f64 / (1024.0 * 1024.0))
    } else {
        format!("{:.2} GB", bytes as f64 / (1024.0 * 1024.0 * 1024.0))
    }
}
