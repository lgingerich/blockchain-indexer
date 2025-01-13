use anyhow::{anyhow, Error, Result};
use std::{error::Error as StdError, future::Future, time::Duration};
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
            max_attempts: 5,
            base_delay_ms: 500,
            max_delay_ms: 30_000,
            exponential: 2.0
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
                    return Err(anyhow!(e).context(format!("Failed after {} attempts", attempt)));
                }

                warn!(
                    "Attempt {}/{} for '{}' failed: {}. Retrying in {}ms...",
                    attempt, config.max_attempts, context, e, delay
                );

                sleep(Duration::from_millis(delay)).await;

                // Exponential backoff with jitter
                delay = std::cmp::min(
                    config.max_delay_ms,
                    (delay as f64 * config.exponential + (fastrand::f64() * 100.0)) as u64,
                );
                attempt += 1;
            }
        }
    }
}
