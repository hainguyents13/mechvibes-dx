use dioxus::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Theme {
    Dark,
    Light,
    System,
    Custom(String), // Custom theme with name
}

impl Theme {
    /// Convert to DaisyUI theme name
    pub fn to_daisy_theme(&self) -> String {
        match self {
            Theme::Dark => "dark".to_string(),
            Theme::Light => "light".to_string(),
            Theme::System => "light".to_string(), // Default to light for now
            Theme::Custom(name) => format!("custom-{}", name),
        }
    }

    /// Get display name for the theme
    pub fn display_name(&self) -> String {
        match self {
            Theme::Dark => "Dark".to_string(),
            Theme::Light => "Light".to_string(),
            Theme::System => "System".to_string(),
            Theme::Custom(name) => name.clone(),
        }
    }
}

// Global theme context
pub static THEME: GlobalSignal<Theme> = Signal::global(|| Theme::System);

pub fn use_theme() -> Signal<Theme> {
    THEME.signal()
}
