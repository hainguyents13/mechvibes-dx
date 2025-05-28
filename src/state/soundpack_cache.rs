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

    // Đường dẫn tương đối
    #[serde(default)]
    pub relative_path: String,

    // Full path to icon (if available)
    #[serde(default)]
    pub full_icon_path: Option<String>,

    // Full path to sound file
    #[serde(default)]
    pub full_sound_path: Option<String>,
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
                                        // Lấy tên thư mục (đường dẫn tương đối)
                                        let dir_name = path
                                            .file_name()
                                            .and_then(|name| name.to_str())
                                            .unwrap_or("unknown");

                                        // Create full path for icon if available
                                        let full_icon_path = pack.icon.as_ref().map(|icon_path| {
                                            format!("./soundpacks/{}/{}", dir_name, icon_path)
                                        });

                                        // Create full path for sound file if available
                                        let full_sound_path =
                                            pack.source.as_ref().map(|sound_path| {
                                                format!("./soundpacks/{}/{}", dir_name, sound_path)
                                            });

                                        return Some(SoundpackItem {
                                            soundpack: pack,
                                            relative_path: format!("./soundpacks/{}", dir_name),
                                            full_icon_path,
                                            full_sound_path,
                                        });
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
    pub fn get_soundpack_by_id(&self, id: &str) -> Option<&SoundpackItem> {
        self.soundpacks.iter().find(|item| item.soundpack.id == id)
    }
}

impl Default for SoundpackCache {
    fn default() -> Self {
        Self::rebuild()
    }
}
