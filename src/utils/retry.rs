// use std::time::Duration;
// use tokio_retry::strategy::{jitter, ExponentialBackoff};
// use tracing::warn;

// /// A retry configuration that provides a sequence of 8 retry delays with exponential backoff:
// /// - Starts at 1 second
// /// - Doubles each time (factor of 2)
// /// - Caps at 60 seconds
// /// - Includes random jitter to prevent thundering herd problems
// ///
// /// The Lazy static ensures this is only computed once when first accessed,
// /// and the same configuration is reused across all retry attempts in the program.
// /// This is typically used with tokio-retry to handle transient failures in network
// /// operations or other async tasks.
// pub fn get_retry_config(context: &str) -> Vec<Duration> {
//     let mut attempt = 1;
//     ExponentialBackoff::from_millis(1000)
//         .factor(2)
//         .max_delay(Duration::from_millis(60000))
//         .map(move |duration| {
//             // Only log when we actually need to retry
//             if attempt > 1 {
//                 warn!(
//                     "Attempt {}/8 failed for '{}'. Retrying in {:.2}s...",
//                     attempt - 1,
//                     context,
//                     duration.as_secs_f64()
//                 );
//             }
//             attempt += 1;
//             duration
//         })
//         .map(jitter)
//         .take(8)
//         .collect()
// }


use anyhow::{anyhow, Error, Result};
use std::{future::Future, time::Duration};
use tokio::time::sleep;
use tracing::{error, warn};


pub struct RetryConfig {
    pub max_attempts: u32,
    pub base_delay_ms: u64,
    pub max_delay_ms: u64,
    pub exponential: f64,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: 8,
            base_delay_ms: 1_000,
            max_delay_ms: 60_000,
            exponential: 2.0,
        }
    }
}

pub async fn retry<F, Fut, T>(operation: F, config: &RetryConfig, context: &str) -> Result<T, Error>
where
    F: Fn() -> Fut,
    Fut: Future<Output = std::result::Result<T, Error>>,
{
    let mut attempt = 1;
    let mut delay = config.base_delay_ms;

    loop {
        match operation().await {
            Ok(result) => return Ok(result),
            Err(e) => {
                if attempt >= config.max_attempts {
                    error!(
                        "Operation '{}' failed after {} attempts. Final error: {}",
                        context, attempt, e
                    );
                    let error_msg = e.to_string();
                    return Err(anyhow!(error_msg)
                        .context(format!("Failed after {} attempts", attempt)));
                }

                warn!(
                    "Attempt {}/{} for '{}' failed: {}. Retrying in {}ms...",
                    attempt,
                    config.max_attempts,
                    context,
                    &e.to_string(),
                    delay
                );

                sleep(Duration::from_millis(delay)).await;

                // Exponential backoff with full jitter
                // https://aws.amazon.com/blogs/architecture/exponential-backoff-and-jitter/
                let next_delay = delay as f64 * config.exponential;
                delay = std::cmp::min(
                    config.max_delay_ms,
                    (fastrand::f64() * next_delay) as u64,
                );
                attempt += 1;
            }
        }
    }
}