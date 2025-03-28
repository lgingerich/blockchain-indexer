use once_cell::sync::Lazy;
use std::time::Duration;
use tokio_retry::strategy::{jitter, ExponentialBackoff};

/// A static retry configuration that initializes only when first accessed.
///
/// This configuration provides a sequence of 8 retry delays with exponential backoff:
/// - Starts at 1 second
/// - Doubles each time (factor of 2)
/// - Caps at 60 seconds
/// - Includes random jitter to prevent thundering herd problems
///
/// The Lazy static ensures this is only computed once when first accessed,
/// and the same configuration is reused across all retry attempts in the program.
/// This is typically used with tokio-retry to handle transient failures in network
/// operations or other async tasks.
pub static RETRY_CONFIG: Lazy<Vec<Duration>> = Lazy::new(|| {
    ExponentialBackoff::from_millis(1000)
        .factor(2)
        .max_delay(Duration::from_millis(60000))
        .map(jitter)
        .take(8)
        .collect()
});
