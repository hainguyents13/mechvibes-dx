use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::time::UNIX_EPOCH;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SoundpackMetadata {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub version: String,
    pub tags: Vec<String>,
    pub keycap: Option<String>,
    pub icon: Option<String>,
    pub last_modified: u64,
    pub file_size: u64,
    pub last_accessed: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizedSoundpackCache {
    pub soundpacks: HashMap<String, SoundpackMetadata>,
    pub last_scan: u64,
}

impl OptimizedSoundpackCache {
    const CACHE_FILE: &'static str = "data/soundpack_metadata_cache.json";
    const CACHE_DIR: &'static str = "data/cache/soundpacks";

    pub fn load() -> Self {
        // Create cache directory if it doesn't exist
        if let Err(e) = fs::create_dir_all(Self::CACHE_DIR) {
            eprintln!("⚠️  Failed to create cache directory: {}", e);
        } // Load metadata cache
        match fs::read_to_string(Self::CACHE_FILE) {
            Ok(content) => match serde_json::from_str::<OptimizedSoundpackCache>(&content) {
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
        }
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

    // Check if individual soundpack cache exists and is valid
    pub fn has_audio_cache(&self, soundpack_id: &str) -> bool {
        let cache_path = format!("{}/{}.audio_cache", Self::CACHE_DIR, soundpack_id);
        Path::new(&cache_path).exists()
    }

    // Load individual soundpack audio cache
    pub fn load_audio_cache(&self, soundpack_id: &str) -> Option<Vec<u8>> {
        let cache_path = format!("{}/{}.audio_cache", Self::CACHE_DIR, soundpack_id);
        match fs::read(&cache_path) {
            Ok(data) => {
                // Update last accessed time
                self.update_last_accessed(soundpack_id);
                println!(
                    "🚀 Loaded audio cache for {} ({} bytes)",
                    soundpack_id,
                    data.len()
                );
                Some(data)
            }
            Err(_) => None,
        }
    }

    // Save individual soundpack audio cache
    pub fn save_audio_cache(&self, soundpack_id: &str, audio_data: &[u8]) {
        let cache_path = format!("{}/{}.audio_cache", Self::CACHE_DIR, soundpack_id);
        if let Err(e) = fs::write(&cache_path, audio_data) {
            eprintln!("⚠️  Failed to save audio cache for {}: {}", soundpack_id, e);
        } else {
            println!(
                "💾 Saved audio cache for {} ({} bytes)",
                soundpack_id,
                audio_data.len()
            );
            self.update_last_accessed(soundpack_id);
        }
    } // Update last accessed time for a soundpack
    fn update_last_accessed(&self, _soundpack_id: &str) {
        // In a real implementation, we'd want to make this mutable
        // For now, we'll update the access time when saving cache
    }

    // Clear unused audio caches (LRU cleanup)
    pub fn cleanup_old_caches(&mut self, keep_recent: usize) {
        println!(
            "🧹 Starting cache cleanup, keeping {} most recent",
            keep_recent
        );

        // Get all cache files
        let cache_dir = Path::new(Self::CACHE_DIR);
        if !cache_dir.exists() {
            return;
        }

        let mut cache_files: Vec<_> = match fs::read_dir(cache_dir) {
            Ok(entries) => entries
                .filter_map(|entry| entry.ok())
                .filter(|entry| {
                    entry
                        .path()
                        .extension()
                        .and_then(|ext| ext.to_str())
                        .map(|ext| ext == "audio_cache")
                        .unwrap_or(false)
                })
                .collect(),
            Err(_) => return,
        };

        // Sort by last modified time (newest first)
        cache_files.sort_by(|a, b| {
            let a_time = a
                .metadata()
                .and_then(|m| m.modified())
                .unwrap_or(UNIX_EPOCH);
            let b_time = b
                .metadata()
                .and_then(|m| m.modified())
                .unwrap_or(UNIX_EPOCH);
            b_time.cmp(&a_time) // Reverse order for newest first
        });

        // Remove old cache files
        if cache_files.len() > keep_recent {
            let to_remove = &cache_files[keep_recent..];
            let mut removed_count = 0;

            for file in to_remove {
                if let Err(e) = fs::remove_file(file.path()) {
                    eprintln!("⚠️  Failed to remove cache file {:?}: {}", file.path(), e);
                } else {
                    removed_count += 1;
                }
            }

            if removed_count > 0 {
                println!("🗑️  Removed {} old cache files", removed_count);
            }
        }
    }

    // Get cache statistics
    pub fn get_cache_stats(&self) -> CacheStats {
        let cache_dir = Path::new(Self::CACHE_DIR);
        let mut total_size = 0u64;
        let mut file_count = 0usize;

        if let Ok(entries) = fs::read_dir(cache_dir) {
            for entry in entries.filter_map(|e| e.ok()) {
                if let Ok(metadata) = entry.metadata() {
                    if metadata.is_file()
                        && entry
                            .path()
                            .extension()
                            .and_then(|ext| ext.to_str())
                            .map(|ext| ext == "audio_cache")
                            .unwrap_or(false)
                    {
                        total_size += metadata.len();
                        file_count += 1;
                    }
                }
            }
        }
        CacheStats {
            total_size,
            file_count,
            metadata_count: self.soundpacks.len(),
        }
    }

    // Refresh cache by scanning soundpacks directory
    pub fn refresh_from_directory(&mut self) {
        println!("📂 Scanning soundpacks directory...");

        match std::fs::read_dir("./soundpacks") {
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
        let config_path = format!("./soundpacks/{}/config.json", soundpack_id);

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
            last_modified: metadata
                .modified()
                .unwrap_or(std::time::SystemTime::UNIX_EPOCH)
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            file_size: metadata.len(),
            last_accessed: 0, // Will be updated when accessed
        })
    }

    // Background cache cleanup task
    pub fn start_cache_cleanup_task() {
        std::thread::spawn(|| {
            println!("🧹 Starting background cache cleanup task...");

            loop {
                // Sleep for 1 hour
                std::thread::sleep(std::time::Duration::from_secs(3600));

                let mut cache = OptimizedSoundpackCache::load();
                cache.cleanup_old_caches(10); // Keep 10 most recent
                cache.save();

                println!("🧹 Performed automatic cache cleanup");
            }
        });
    }
}

#[derive(Debug)]
pub struct CacheStats {
    pub total_size: u64,
    pub file_count: usize,
    pub metadata_count: usize,
}

impl CacheStats {
    pub fn format_size(&self) -> String {
        if self.total_size < 1024 {
            format!("{} B", self.total_size)
        } else if self.total_size < 1024 * 1024 {
            format!("{:.2} KB", self.total_size as f64 / 1024.0)
        } else if self.total_size < 1024 * 1024 * 1024 {
            format!("{:.2} MB", self.total_size as f64 / (1024.0 * 1024.0))
        } else {
            format!(
                "{:.2} GB",
                self.total_size as f64 / (1024.0 * 1024.0 * 1024.0)
            )
        }
    }
}

// Background cache cleanup task
pub fn start_cache_cleanup_task() {
    std::thread::spawn(|| {
        println!("🧹 Starting background cache cleanup task...");

        loop {
            // Sleep for 1 hour
            std::thread::sleep(std::time::Duration::from_secs(3600));

            let mut cache = OptimizedSoundpackCache::load();
            cache.cleanup_old_caches(10); // Keep 10 most recent
            cache.save();

            println!("🧹 Performed automatic cache cleanup");
        }
    });
}
