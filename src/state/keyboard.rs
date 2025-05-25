//! Keyboard state management for tracking key press events
//! This module handles the global state of keyboard interactions

/// Represents the current state of keyboard interactions
/// - `key_pressed`: Whether any key is currently being pressed
/// - `last_key`: The most recent key that was pressed
#[derive(Debug, PartialEq, Clone)]
pub struct KeyboardState {
    pub key_pressed: bool,
    pub last_key: String,
}

impl KeyboardState {
    /// Creates a new keyboard state with default values:
    /// - No key pressed
    /// - No last key recorded
    pub fn new() -> Self {
        Self {
            key_pressed: false,
            last_key: String::new(),
        }
    }
}
