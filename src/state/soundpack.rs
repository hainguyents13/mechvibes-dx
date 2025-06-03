use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SoundpackType {
    Keyboard,
    Mouse,
    Both,
}

impl SoundpackType {}

impl SoundPack {}

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
