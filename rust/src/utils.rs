use rand::Rng;
use tokio::time::{sleep, Duration};

// Constants for retry configuration
const MAX_RETRIES: u32 = 5;
const BASE_DELAY_MS: u64 = 100;
const MAX_DELAY_MS: u64 = 5000;

/// Implements exponential backoff with jitter
pub async fn exponential_backoff(attempt: u32, max_retries: u32) {
    let max_shift = std::cmp::min(MAX_DELAY_MS, BASE_DELAY_MS * (1 << attempt));
    let jitter = rand::thread_rng().gen_range(0..=50);
    let delay = Duration::from_millis(max_shift + jitter);
    sleep(delay).await;
}
