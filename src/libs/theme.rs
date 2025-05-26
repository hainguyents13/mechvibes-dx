use dioxus::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Theme {
    Dark,
    Light,
    System,
}

impl Theme {
    pub fn get_effective_theme(&self) -> EffectiveTheme {
        match self {
            Theme::Dark => EffectiveTheme::Dark,
            Theme::Light => EffectiveTheme::Light,
            Theme::System => {
                // For now, default to light. In a real app, you'd check system preference
                EffectiveTheme::Light
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum EffectiveTheme {
    Dark,
    Light,
}

impl EffectiveTheme {
    pub fn bg_primary(&self) -> &'static str {
        match self {
            EffectiveTheme::Dark => "bg-gray-900",
            EffectiveTheme::Light => "bg-white",
        }
    }

    pub fn bg_secondary(&self) -> &'static str {
        match self {
            EffectiveTheme::Dark => "bg-gray-800",
            EffectiveTheme::Light => "bg-gray-100",
        }
    }

    pub fn text_primary(&self) -> &'static str {
        match self {
            EffectiveTheme::Dark => "text-white",
            EffectiveTheme::Light => "text-gray-900",
        }
    }

    pub fn text_secondary(&self) -> &'static str {
        match self {
            EffectiveTheme::Dark => "text-gray-300",
            EffectiveTheme::Light => "text-gray-600",
        }
    }

    pub fn text_tertiary(&self) -> &'static str {
        match self {
            EffectiveTheme::Dark => "text-gray-400",
            EffectiveTheme::Light => "text-gray-500",
        }
    }

    pub fn border(&self) -> &'static str {
        match self {
            EffectiveTheme::Dark => "border-gray-600",
            EffectiveTheme::Light => "border-gray-300",
        }
    }

    pub fn bg_hover(&self) -> &'static str {
        match self {
            EffectiveTheme::Dark => "hover:bg-gray-700",
            EffectiveTheme::Light => "hover:bg-gray-50",
        }
    }
}

// Global theme context
pub static THEME: GlobalSignal<Theme> = Signal::global(|| Theme::System);

pub fn use_theme() -> Signal<Theme> {
    THEME.signal()
}

pub fn use_effective_theme() -> EffectiveTheme {
    let theme = THEME.read();
    theme.get_effective_theme()
}
