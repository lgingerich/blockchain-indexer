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
                    return Err(
                        anyhow!(error_msg).context(format!("Failed after {} attempts", attempt))
                    );
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

                // Equal Jitter implementation
                // https://aws.amazon.com/blogs/architecture/exponential-backoff-and-jitter/
                let next_delay = (delay as f64 * config.exponential) as u64;
                let next_delay = std::cmp::min(config.max_delay_ms, next_delay);
                let half_delay = next_delay as f64 / 2.0;
                delay = half_delay as u64 + (fastrand::f64() * half_delay) as u64;
                attempt += 1;
            }
        }
    }
}
