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
    pub cache_version: u32, // Add version to force regeneration when format changes
}

impl SoundpackCache {
    fn cache_file() -> String {
        paths::data::soundpack_metadata_cache_json()
            .to_string_lossy()
            .to_string()
    }

    pub fn load() -> Self {
        let cache_file = Self::cache_file();
        // Load metadata cache
        let mut cache = match fs::read_to_string(&cache_file) {
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

        // Auto-refresh if cache is empty, outdated, or version mismatch
        if cache.soundpacks.is_empty() || cache.last_scan == 0 || cache.cache_version < 2 {
            if cache.cache_version < 2 {
                println!(
                    "🔄 Cache version outdated (v{} -> v2), refreshing...",
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
            cache_version: 2, // Current version with data URI support
        }
    }

    pub fn save(&self) {
        let cache_file = Self::cache_file();
        // Ensure parent directory exists
        if let Some(parent) = Path::new(&cache_file).parent() {
            if let Err(e) = fs::create_dir_all(parent) {
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

        match serde_json::to_string_pretty(self) {
            Ok(content) => {
                if let Err(e) = fs::write(&cache_file, content) {
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

        let soundpacks_dir = paths::utils::get_soundpacks_dir_absolute();
        match std::fs::read_dir(&soundpacks_dir) {
            Ok(entries) => {
                self.soundpacks.clear(); // Clear all existing entries

                for entry in entries.filter_map(|e| e.ok()) {
                    if let Some(soundpack_id) = entry.file_name().to_str() {
                        println!("🔄 Regenerating metadata for: {}", soundpack_id);
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
            icon: {
                // Check if icon file exists and convert to base64 data URI or empty string
                if let Some(icon_filename) = config.get("icon").and_then(|v| v.as_str()) {
                    let icon_path = format!(
                        "{}/{}",
                        paths::soundpacks::soundpack_dir(soundpack_id),
                        icon_filename
                    );
                    println!(
                        "🔍 Checking icon for {}: {} -> exists: {}",
                        soundpack_id,
                        icon_path,
                        std::path::Path::new(&icon_path).exists()
                    );
                    if std::path::Path::new(&icon_path).exists() {
                        // Convert to base64 data URI for Dioxus WebView
                        match Self::convert_image_to_data_uri(&icon_path) {
                            Ok(data_uri) => {
                                println!("✅ Icon converted to data URI for {}", soundpack_id);
                                Some(data_uri)
                            }
                            Err(e) => {
                                println!("❌ Failed to convert icon for {}: {}", soundpack_id, e);
                                Some(String::new())
                            }
                        }
                    } else {
                        println!(
                            "❌ Icon not found for {}, setting empty string",
                            soundpack_id
                        );
                        Some(String::new()) // Empty string if icon file not found
                    }
                } else {
                    println!("ℹ️  No icon specified for {}", soundpack_id);
                    Some(String::new()) // Empty string if no icon specified
                }
            },
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

    // Convert image file to base64 data URI for WebView compatibility
    fn convert_image_to_data_uri(image_path: &str) -> Result<String, String> {
        // Read the image file
        let image_data =
            std::fs::read(image_path).map_err(|e| format!("Failed to read image file: {}", e))?;

        // Determine MIME type based on file extension
        let mime_type = match std::path::Path::new(image_path)
            .extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| ext.to_lowercase())
            .as_deref()
        {
            Some("jpg") | Some("jpeg") => "image/jpeg",
            Some("png") => "image/png",
            Some("gif") => "image/gif",
            Some("webp") => "image/webp",
            Some("avif") => "image/avif",
            Some("svg") => "image/svg+xml",
            Some("bmp") => "image/bmp",
            Some("ico") => "image/x-icon",
            _ => "image/png", // Default fallback
        };

        // Convert to base64
        let base64_data =
            base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &image_data);

        // Create data URI
        Ok(format!("data:{};base64,{}", mime_type, base64_data))
    }
}
