use anyhow::{anyhow, Error, Result};
use std::{future::Future, time::Duration};
use tokio::time::sleep;
use tracing::{error, warn};

use crate::utils::{strip_html, rate_limiter::RateLimiter};
pub struct RetryConfig {
    pub max_attempts: u32,
    pub base_delay_ms: u64,
    pub max_delay_ms: u64,
    pub min_delay_ms: u64,
    pub exponential: f64,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: 8,
            base_delay_ms: 1_000,
            min_delay_ms: 500,
            max_delay_ms: 60_000,
            exponential: 2.0,
        }
    }
}

pub async fn retry<F, Fut, T>(
    operation: F, 
    config: &RetryConfig, 
    context: &str,
    rate_limiter: Option<&RateLimiter>
) -> Result<T, Error>
where
    F: Fn() -> Fut,
    Fut: Future<Output = std::result::Result<T, Error>>,
{
    let mut attempt = 1;
    let mut delay = config.base_delay_ms;

    loop {
        // Acquire rate limit permit if limiter is provided
        let permit = if let Some(limiter) = rate_limiter {
            Some(limiter.acquire().await?)
        } else {
            None
        };

        match operation().await {
            Ok(result) => {
                // Record success if we have a permit
                if let Some(p) = permit {
                    p.record_result(false);
                }
                return Ok(result);
            },
            Err(e) => {
                // Record error if we have a permit
                if let Some(p) = permit {
                    p.record_result(true);
                }
                
                if attempt >= config.max_attempts {
                    error!(
                        "Operation '{}' failed after {} attempts. Final error: {}",
                        context, attempt, e
                    );
                    return Err(anyhow!(strip_html(&e.to_string()))
                        .context(format!("Failed after {} attempts", attempt)));
                }

                warn!(
                    "Attempt {}/{} for '{}' failed: {}. Retrying in {}ms...",
                    attempt,
                    config.max_attempts,
                    context,
                    strip_html(&e.to_string()),
                    delay
                );

                sleep(Duration::from_millis(delay)).await;

                // Decorrelated jitter backoff algorithm
                // This ensures each delay is greater than the previous one
                // Formula: min(max_delay, max(min_delay, random(min_delay, prev_delay * 3)))
                let next_delay = std::cmp::min(
                    config.max_delay_ms,
                    std::cmp::max(
                        config.min_delay_ms,
                        delay + (fastrand::u64(0..=delay * 2))
                    )
                );
                
                delay = next_delay;
                attempt += 1;
            }
        }
    }
}
