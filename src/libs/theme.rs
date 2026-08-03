use dioxus::prelude::*;
use serde::{Deserialize, Serialize};
use strum::{EnumIter, IntoEnumIterator};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Theme {
    BuiltIn(BuiltInTheme),
    Custom(String), // Custom theme with name
}

/// Defaults to following the OS, matching what a fresh `AppConfig` picks.
/// Having a `Default` lets a damaged `theme` entry in a hand-edited config
/// fall back to this one setting instead of failing the whole document.
///
/// Written out rather than derived: `#[default]` only applies to unit
/// variants, and the default here carries a `BuiltInTheme` payload.
impl Default for Theme {
    fn default() -> Self {
        Theme::BuiltIn(BuiltInTheme::default())
    }
}


#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, EnumIter, Default)]
pub enum BuiltInTheme {
    Light,
    Dark,
    #[default]
    System,
    // DaisyUI themes
    Cupcake,
    Bumblebee,
    Emerald,
    Corporate,
    Synthwave,
    Retro,
    Cyberpunk,
    Valentine,
    Halloween,
    Garden,
    Forest,
    Aqua,
    Lofi,
    Pastel,
    Fantasy,
    Wireframe,
    Black,
    Luxury,
    Dracula,
    Cmyk,
    Autumn,
    Business,
    Acid,
    Lemonade,
    Night,
    Coffee,
    Winter,
    Dim,
    Nord,
    Sunset,
    Abyss,
    Silk,
    Caramellatte,
}

impl BuiltInTheme {
    /// Convert to DaisyUI theme name
    pub fn to_daisy_theme(&self) -> String {
        match self {
            BuiltInTheme::System => "light".to_string(), // Default to light for system theme
            _ => format!("{:?}", self).to_lowercase(),
        }
    }
    /// Get all available built-in themes
    pub fn all() -> Vec<BuiltInTheme> {
        BuiltInTheme::iter().collect()
    }
}

impl Theme {
    /// Convert to DaisyUI theme name
    pub fn to_daisy_theme(&self) -> String {
        match self {
            Theme::BuiltIn(builtin) => builtin.to_daisy_theme(),
            Theme::Custom(name) => format!("custom-{}", name),
        }
    }
}

// Global theme context
pub static THEME: GlobalSignal<Theme> = Signal::global(|| Theme::BuiltIn(BuiltInTheme::System));

pub fn use_theme() -> Signal<Theme> {
    THEME.signal()
}
