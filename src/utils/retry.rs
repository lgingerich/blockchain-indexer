use std::time::Duration;
use tokio_retry::strategy::{jitter, ExponentialBackoff};

pub fn get_retry_strategy() -> impl Iterator<Item = Duration> {
    ExponentialBackoff::from_millis(1000)
        .factor(2)
        .max_delay(Duration::from_millis(60000))
        .map(jitter)
        .take(8)
}