use crate::state::soundpack::SoundPack;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SoundpackItem {
    // Just use the full SoundPack struct directly - no duplication
    #[serde(flatten)]
    pub soundpack: SoundPack,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SoundpackCache {
    pub last_updated: DateTime<Utc>,
    pub soundpacks: Vec<SoundpackItem>,
}

impl SoundpackCache {
    const CACHE_FILE: &'static str = "./data/soundpacks.json";
    pub fn load() -> Self {
        // Ensure data directory exists
        if let Err(_) = fs::create_dir_all("./data") {
            eprintln!("Warning: Could not create data directory");
        }

        let cache_path = PathBuf::from(Self::CACHE_FILE);
        if let Ok(contents) = fs::read_to_string(cache_path) {
            match serde_json::from_str::<SoundpackCache>(&contents) {
                Ok(cache) => {
                    println!(
                        "📦 Loaded soundpack cache with {} soundpacks (last updated: {})",
                        cache.soundpacks.len(),
                        cache.last_updated.format("%Y-%m-%d %H:%M:%S")
                    );
                    return cache;
                }
                Err(e) => {
                    eprintln!(
                        "Warning: Failed to parse soundpack cache: {}. Rebuilding.",
                        e
                    );
                }
            }
        } else {
            println!("📦 No soundpack cache found, building initial cache...");
        }

        // Build new cache
        Self::rebuild()
    }

    pub fn rebuild() -> Self {
        println!("🔄 Rebuilding soundpack cache...");
        let soundpacks = Self::scan_soundpacks();
        let cache = Self {
            last_updated: Utc::now(),
            soundpacks,
        };

        // Save cache to disk
        let _ = cache.save();

        println!(
            "✅ Soundpack cache rebuilt with {} soundpacks",
            cache.soundpacks.len()
        );
        cache
    }
    fn scan_soundpacks() -> Vec<SoundpackItem> {
        std::fs::read_dir("./soundpacks")
            .map(|entries| {
                entries
                    .filter_map(|entry| {
                        entry.ok().and_then(|e| {
                            let path = e.path();
                            if path.join("config.json").exists() {
                                if let Ok(content) =
                                    std::fs::read_to_string(path.join("config.json"))
                                {
                                    if let Ok(pack) = serde_json::from_str::<SoundPack>(&content) {
                                        return Some(SoundpackItem { soundpack: pack });
                                    }
                                }
                            }
                            None
                        })
                    })
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default()
    }

    fn save(&self) -> Result<(), String> {
        let contents = serde_json::to_string_pretty(self)
            .map_err(|e| format!("Failed to serialize soundpack cache: {}", e))?;
        fs::write(Self::CACHE_FILE, contents)
            .map_err(|e| format!("Failed to write soundpack cache file: {}", e))?;
        Ok(())
    }
    pub fn get_soundpacks(&self) -> &Vec<SoundpackItem> {
        &self.soundpacks
    }
    pub fn get_soundpack_by_id(&self, id: &str) -> Option<&SoundPack> {
        self.soundpacks
            .iter()
            .find(|item| item.soundpack.id == id)
            .map(|item| &item.soundpack)
    }
}

impl Default for SoundpackCache {
    fn default() -> Self {
        Self::rebuild()
    }
}
