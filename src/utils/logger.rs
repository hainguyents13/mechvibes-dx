/// Debug logging utility
///
/// Every macro here does two things with one formatted string: it prints to the
/// console exactly as it always did, and it tees a timestamped copy into the
/// in-RAM ring buffer that backs the Debug section in Settings
/// (`crate::utils::log_buffer`). Console behavior is deliberately unchanged, so
/// `dx serve` output looks identical to before the buffer existed.
///
/// The tee is what makes installed release builds diagnosable at all: they are
/// built with `#![windows_subsystem = "windows"]` and have no console, so
/// without the buffer a user's logs simply do not exist anywhere.
use std::sync::OnceLock;

static DEBUG_ENABLED: OnceLock<bool> = OnceLock::new();

/// Initialize debug logging - always enabled
pub fn init_debug_logging() {
    let _ = DEBUG_ENABLED.set(true);
    crate::always_print!("🐛 Debug logging enabled");
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
            // Formatted once, then used twice: the console keeps its existing
            // output and the ring buffer gets a timestamped copy.
            let line = format!($($arg)*);
            println!("{}", line);
            $crate::utils::log_buffer::push(&line);
        }
    };
}

/// Debug error print macro - only prints if debug console is enabled
#[macro_export]
macro_rules! debug_eprint {
    ($($arg:tt)*) => {
        if $crate::utils::logger::is_debug_enabled() {
            let line = format!($($arg)*);
            eprintln!("{}", line);
            $crate::utils::log_buffer::push(&line);
        }
    };
}

/// Always print macro - for critical messages that should always show
#[macro_export]
macro_rules! always_print {
    ($($arg:tt)*) => {
        {
            let line = format!($($arg)*);
            println!("{}", line);
            $crate::utils::log_buffer::push(&line);
        }
    };
}

/// Always error print macro - for critical errors that should always show
#[macro_export]
macro_rules! always_eprint {
    ($($arg:tt)*) => {
        {
            let line = format!($($arg)*);
            eprintln!("{}", line);
            $crate::utils::log_buffer::push(&line);
        }
    };
}

#[cfg(test)]
mod tests {
    /// The macros must keep capturing into the buffer, since that is the only
    /// place an installed release build's logs exist.
    #[test]
    fn always_print_reaches_the_ring_buffer() {
        // Serialized with every other buffer-asserting test: without this,
        // log_buffer's rotation test can flood the shared buffer and evict
        // this test's line before the assertion reads it back.
        let _guard = crate::utils::log_buffer::buffer_test_guard();

        let before = crate::utils::log_buffer::generation();
        crate::always_print!("logger test marker {}", 42);
        assert!(
            crate::utils::log_buffer::generation() > before,
            "a logged line must land in the buffer"
        );

        let recent = crate::utils::log_buffer::recent(50).join("\n");
        assert!(recent.contains("logger test marker 42"), "{}", recent);
    }

    /// Formatting happens once and the argument expression must not be
    /// evaluated twice - a logged counter would otherwise double-increment.
    #[test]
    fn arguments_are_evaluated_exactly_once() {
        let mut calls = 0;
        let mut bump = || {
            calls += 1;
            calls
        };
        crate::always_print!("evaluated {} time(s)", bump());
        assert_eq!(calls, 1, "a side-effecting argument must run once");
    }
}
