use crate::state::paths;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

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
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SoundpackCache {
    pub soundpacks: HashMap<String, SoundpackMetadata>,
    pub last_scan: u64,
}

impl SoundpackCache {
    const CACHE_FILE: &'static str = paths::data::SOUNDPACK_METADATA_CACHE_JSON;

    pub fn load() -> Self {
        // Load metadata cache
        let mut cache = match fs::read_to_string(Self::CACHE_FILE) {
            Ok(content) => match serde_json::from_str::<SoundpackCache>(&content) {
                Ok(cache) => {
                    println!(
                        "📦 Loaded soundpack metadata cache with {} entries",
                        cache.soundpacks.len()
                    );
                    cache
                }
                Err(e) => {
                    eprintln!("⚠️  Failed to parse cache file: {}", e);
                    Self::new()
                }
            },
            Err(_) => {
                println!("📦 Creating new soundpack metadata cache");
                Self::new()
            }
        };

        // Auto-refresh if cache is empty or outdated
        if cache.soundpacks.is_empty() || cache.last_scan == 0 {
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
        }
    }

    pub fn save(&self) {
        // Ensure parent directory exists
        if let Some(parent) = Path::new(Self::CACHE_FILE).parent() {
            if let Err(e) = fs::create_dir_all(parent) {
                eprintln!("⚠️  Failed to create cache directory: {}", e);
                return;
            }
        }

        match serde_json::to_string_pretty(self) {
            Ok(content) => {
                if let Err(e) = fs::write(Self::CACHE_FILE, content) {
                    eprintln!("⚠️  Failed to save metadata cache: {}", e);
                } else {
                    println!(
                        "💾 Saved soundpack metadata cache with {} entries",
                        self.soundpacks.len()
                    );
                }
            }
            Err(e) => eprintln!("⚠️  Failed to serialize cache: {}", e),
        }
    }

    // Add or update soundpack metadata
    pub fn add_soundpack(&mut self, metadata: SoundpackMetadata) {
        self.soundpacks.insert(metadata.id.clone(), metadata);
    }

    // Refresh cache by scanning soundpacks directory
    pub fn refresh_from_directory(&mut self) {
        println!("📂 Scanning soundpacks directory...");

        match std::fs::read_dir(paths::soundpacks::DIR) {
            Ok(entries) => {
                self.soundpacks.clear();

                for entry in entries.filter_map(|e| e.ok()) {
                    if let Some(soundpack_id) = entry.file_name().to_str() {
                        if let Ok(metadata) = self.load_soundpack_metadata(soundpack_id) {
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

    // Load soundpack metadata from config.json
    fn load_soundpack_metadata(&self, soundpack_id: &str) -> Result<SoundpackMetadata, String> {
        let config_path = paths::soundpacks::config_json(soundpack_id);

        let content = std::fs::read_to_string(&config_path)
            .map_err(|e| format!("Failed to read config: {}", e))?;

        let config: serde_json::Value =
            serde_json::from_str(&content).map_err(|e| format!("Failed to parse config: {}", e))?;

        let name = config
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or(soundpack_id)
            .to_string();

        let version = config
            .get("version")
            .and_then(|v| v.as_str())
            .unwrap_or("1.0.0")
            .to_string();

        let tags = config
            .get("tags")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect()
            })
            .unwrap_or_default();

        // Get file stats
        let metadata = std::fs::metadata(&config_path)
            .map_err(|e| format!("Failed to get metadata: {}", e))?;

        Ok(SoundpackMetadata {
            id: soundpack_id.to_string(),
            name,
            author: config
                .get("author")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            description: config
                .get("description")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            version,
            tags,
            keycap: config
                .get("keycap")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            icon: config
                .get("icon")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            mouse: config
                .get("mouse")
                .and_then(|v| v.as_bool())
                .unwrap_or(false), // Default to false (keyboard soundpack)
            last_modified: metadata
                .modified()
                .unwrap_or(std::time::SystemTime::UNIX_EPOCH)
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            last_accessed: 0, // Will be updated when accessed
        })
    }
}
