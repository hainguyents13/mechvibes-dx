use dioxus::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Theme {
    Dark,
    Light,
    System,
}

impl Theme {
    /// Convert to DaisyUI theme name
    pub fn to_daisy_theme(&self) -> &'static str {
        match self {
            Theme::Dark => "dark",
            Theme::Light => "light",
            Theme::System => "light", // Default to light for now, could detect system preference
        }
    }
}

// Global theme context
pub static THEME: GlobalSignal<Theme> = Signal::global(|| Theme::System);

pub fn use_theme() -> Signal<Theme> {
    THEME.signal()
}
