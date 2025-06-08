/// Application constants used throughout the application
/// This file centralizes all application naming and branding constants

/// The display name of the application (with proper casing)
pub const APP_NAME: &str = "MechvibesDX";

/// The display name with spaces for better readability
pub const APP_NAME_DISPLAY: &str = "MechvibesDX";

/// The lowercase version for file names, URLs, etc.
pub const APP_NAME_LOWERCASE: &str = "mechvibes-dx";

/// The identifier for the application (used in protocol registration, etc.)
pub const APP_IDENTIFIER: &str = "com.hainguyents13.mechvibesdx";

/// The protocol scheme for deep linking
pub const APP_PROTOCOL: &str = "mechvibes";

/// The protocol URL prefix
pub const APP_PROTOCOL_URL: &str = "mechvibes://";

/// Short description of the application
pub const APP_DESCRIPTION_SHORT: &str = "MechvibesDX";

/// Long description of the application
pub const APP_DESCRIPTION_LONG: &str =
    "MechvibesDX is a modern, cross-platform soundboard for mechanical keyboards. It supports a wide range of keyboard sounds and is designed to be user-friendly and customizable.";

/// Version of the application (should match Cargo.toml)
pub const APP_VERSION: &str = env!("CARGO_PKG_VERSION");

/// CSS ID prefix for custom styles
pub const CSS_ID_PREFIX: &str = "mechvibes";
