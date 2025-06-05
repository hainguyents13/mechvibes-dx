use crate::state::paths;
use crate::utils::{data, path, soundpack};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

// ===== SOUNDPACK TYPES =====

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SoundpackType {
    Keyboard,
    Mouse,
    Both,
}

// Default function for config_version field
fn default_config_version() -> u32 {
    2
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SoundPack {
    pub id: String,
    pub name: String,
    pub author: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub version: Option<String>,
    #[serde(default)]
    pub tags: Option<Vec<String>>,
    #[serde(default)]
    pub keycap: Option<String>,
    #[serde(default)]
    pub icon: Option<String>,
    #[serde(default)]
    pub source: Option<String>,
    #[serde(default)]
    pub method: Option<String>,
    #[serde(default)]
    pub includes_numpad: Option<bool>,
    #[serde(default)]
    pub mouse: bool, // true for mouse soundpacks, false for keyboard soundpacks
    #[serde(default = "default_config_version")]
    pub config_version: u32, // Configuration version, default to 2
    pub defs: HashMap<String, Vec<[f32; 2]>>,
}

impl SoundPack {}

impl SoundpackType {}

// ===== SOUNDPACK METADATA =====

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SoundpackMetadata {
    pub id: String,
    pub name: String,
    pub author: Option<String>,
    pub description: Option<String>,
    pub version: String,
    pub tags: Vec<String>,
    pub keycap: Option<String>,
    pub icon: Option<String>,
    pub mouse: bool, // true for mouse soundpacks, false for keyboard
    pub last_modified: u64,
    pub last_accessed: u64,
    // Validation fields
    pub config_version: Option<u32>,
    pub is_valid_v2: bool,
    pub validation_status: String,
    pub can_be_converted: bool,
}

// ===== SOUNDPACK CACHE =====

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SoundpackCache {
    pub soundpacks: HashMap<String, SoundpackMetadata>,
    pub last_scan: u64,
    pub cache_version: u32, // Add version to force regeneration when format changes
}

impl SoundpackCache {
    fn cache_file() -> String {
        paths::data::soundpack_cache_json()
            .to_string_lossy()
            .to_string()
    }
    pub fn load() -> Self {
        let cache_file = Self::cache_file();
        // Load metadata cache using data utilities
        let mut cache = match data::load_json_from_file::<SoundpackCache>(
            std::path::Path::new(&cache_file),
        ) {
            Ok(cache) => {
                println!(
                    "📦 Loaded soundpack metadata cache with {} entries",
                    cache.soundpacks.len()
                );
                cache
            }
            Err(e) => {
                eprintln!("⚠️  Failed to load cache file: {}", e);
                Self::new()
            }
        };

        // Auto-refresh if cache is empty, outdated, or version mismatch
        if cache.soundpacks.is_empty() || cache.last_scan == 0 || cache.cache_version < 3 {
            if cache.cache_version < 3 {
                println!(
                    "🔄 Cache version outdated (v{} -> v3), refreshing...",
                    cache.cache_version
                );
            }
            println!("🔄 Cache is empty or outdated, refreshing from directory...");
            cache.refresh_from_directory();
            cache.save();
        }

        cache
    }

    pub fn new() -> Self {
        Self {
            soundpacks: HashMap::new(),
            last_scan: 0,
            cache_version: 3, // Current version with validation support
        }
    }
    pub fn save(&self) {
        let cache_file = Self::cache_file();

        // Ensure parent directory exists
        if let Some(parent) = Path::new(&cache_file).parent() {
            if let Err(e) = path::ensure_directory_exists(&parent.to_string_lossy()) {
                eprintln!("⚠️  Failed to create cache directory: {}", e);
                return;
            }
        }

        // Debug: Log what's being saved
        for (id, metadata) in &self.soundpacks {
            println!(
                "💾 Saving {}: icon = {:?}",
                id,
                if let Some(icon) = &metadata.icon {
                    if icon.is_empty() {
                        "EMPTY".to_string()
                    } else if icon.starts_with("data:") {
                        format!("DATA_URI ({}...)", &icon[..50.min(icon.len())])
                    } else {
                        format!("PATH: {}", icon)
                    }
                } else {
                    "NONE".to_string()
                }
            );
        }

        match data::save_json_to_file(self, std::path::Path::new(&cache_file)) {
            Ok(_) => println!(
                "💾 Saved soundpack metadata cache with {} entries",
                self.soundpacks.len()
            ),
            Err(e) => eprintln!("⚠️  Failed to save metadata cache: {}", e),
        }
    }

    // Add or update soundpack metadata
    pub fn add_soundpack(&mut self, metadata: SoundpackMetadata) {
        self.soundpacks.insert(metadata.id.clone(), metadata);
    }

    // Refresh cache by scanning soundpacks directory
    pub fn refresh_from_directory(&mut self) {
        println!("📂 Scanning soundpacks directory...");

        let soundpacks_dir = paths::utils::get_soundpacks_dir_absolute();
        match std::fs::read_dir(&soundpacks_dir) {
            Ok(entries) => {
                self.soundpacks.clear(); // Clear all existing entries

                for entry in entries.filter_map(|e| e.ok()) {
                    if let Some(soundpack_id) = entry.file_name().to_str() {
                        println!("🔄 Regenerating metadata for: {}", soundpack_id);
                        if let Ok(metadata) = soundpack::load_soundpack_metadata(soundpack_id)
                        {
                            self.soundpacks.insert(soundpack_id.to_string(), metadata);
                        }
                    }
                }

                self.last_scan = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs();

                println!("📦 Loaded {} soundpacks metadata", self.soundpacks.len());
            }
            Err(e) => {
                eprintln!("⚠️  Failed to read soundpacks directory: {}", e);
            }
        }
    }
}
