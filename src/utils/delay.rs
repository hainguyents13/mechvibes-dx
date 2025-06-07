use std::time::Duration;

/// Utility functions for common delays used throughout the application
pub struct Delay;

impl Delay {
    pub async fn key_event() {
        futures_timer::Delay::new(Duration::from_millis(1)).await;
    }
    /// Custom delay with specified duration in milliseconds
    pub async fn ms(ms: u64) {
        futures_timer::Delay::new(Duration::from_millis(ms)).await;
    }
}
