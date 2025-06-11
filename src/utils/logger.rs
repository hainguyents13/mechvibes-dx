/// Debug logging utility
use std::sync::OnceLock;

static DEBUG_ENABLED: OnceLock<bool> = OnceLock::new();

/// Initialize debug logging - always enabled
pub fn init_debug_logging() {
    let _ = DEBUG_ENABLED.set(true);
    println!("🐛 Debug logging enabled");
}

/// Check if debug logging is enabled
pub fn is_debug_enabled() -> bool {
    *DEBUG_ENABLED.get().unwrap_or(&false)
}

/// Debug print macro - only prints if debug console is enabled
#[macro_export]
macro_rules! debug_print {
    ($($arg:tt)*) => {
        if $crate::utils::logger::is_debug_enabled() {
            println!($($arg)*);
        }
    };
}

/// Debug error print macro - only prints if debug console is enabled
#[macro_export]
macro_rules! debug_eprint {
    ($($arg:tt)*) => {
        if $crate::utils::logger::is_debug_enabled() {
            eprintln!($($arg)*);
        }
    };
}

/// Always print macro - for critical messages that should always show
#[macro_export]
macro_rules! always_print {
    ($($arg:tt)*) => {
        println!($($arg)*)
    };
}

/// Always error print macro - for critical errors that should always show
#[macro_export]
macro_rules! always_eprint {
    ($($arg:tt)*) => {
        eprintln!($($arg)*)
    };
}
