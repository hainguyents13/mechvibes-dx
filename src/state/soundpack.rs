use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
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
    pub def: HashMap<String, Vec<[f32; 2]>>,
}

impl SoundPack {
    pub fn load_from_path(path: &str) -> Option<Self> {
        let config_path = format!("{}/config.json", path);
        let config_file = Path::new(&config_path);

        if !config_file.exists() {
            eprintln!("❌ Soundpack config file not found: {}", config_path);
            return None;
        }

        match fs::read_to_string(config_file) {
            Ok(content) => {
                // Trước tiên parse thành Value để kiểm tra
                match serde_json::from_str::<serde_json::Value>(&content) {
                    Ok(raw_value) => {
                        if !raw_value.is_object() {
                            eprintln!("❌ Invalid soundpack config: not a JSON object");
                            return None;
                        }

                        // Kiểm tra các field bắt buộc
                        let obj = raw_value.as_object().unwrap();
                        if !obj.contains_key("id")
                            || !obj.contains_key("name")
                            || !obj.contains_key("author")
                        {
                            eprintln!("❌ Invalid soundpack config: missing required fields (id, name, or author)");
                            return None;
                        }

                        // Parse thành SoundPack
                        match serde_json::from_value::<SoundPack>(raw_value) {
                            Ok(soundpack) => {
                                println!("✅ [soundpack.rs] Loaded soundpack: {}", soundpack.name);
                                Some(soundpack)
                            }
                            Err(e) => {
                                eprintln!("❌ Failed to parse soundpack config: {}", e);
                                eprintln!("❌ Config file: {}", config_path);
                                None
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("❌ Invalid JSON in soundpack config: {}", e);
                        eprintln!("❌ Config file: {}", config_path);
                        None
                    }
                }
            }
            Err(e) => {
                eprintln!("❌ Failed to read soundpack config: {}", e);
                eprintln!("❌ Config file: {}", config_path);
                None
            }
        }
    }
}
