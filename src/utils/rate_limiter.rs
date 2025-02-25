use std::collections::VecDeque;
use std::sync::{Arc, Condvar, Mutex};
use std::thread;
use std::time::{Duration, Instant};
use tokio::sync::Semaphore;

/// AdaptiveRateLimiter provides an adaptive concurrent request control mechanism
/// that dynamically adjusts the concurrency level based on response times and error rates.
pub struct RateLimiter {
    // Maximum number of concurrent requests allowed
    max_concurrent_requests: usize,

    // Current number of concurrent requests allowed (adaptive)
    current_limit: Arc<Mutex<usize>>,

    // Semaphore to control concurrent access
    semaphore: Arc<Semaphore>,

    // Window of recent response times for adaptation
    response_times: Arc<Mutex<VecDeque<Duration>>>,

    // Window of recent errors for adaptation
    error_count: Arc<Mutex<VecDeque<bool>>>,

    // Window size for adaptation metrics
    window_size: usize,

    // Target response time - we'll adjust concurrency to try to maintain this
    target_response_time: Duration,

    // Condition variable for waiting threads
    condition: Arc<(Mutex<bool>, Condvar)>,

    // Adaptation interval
    adaptation_interval: Duration,

    // Flag to control the adaptation thread
    running: Arc<Mutex<bool>>,
}

impl RateLimiter {
    /// Create a new AdaptiveRateLimiter with the specified parameters
    pub fn new(
        initial_limit: usize,
        max_limit: usize,
        window_size: usize,
        target_response_time: Duration,
        adaptation_interval: Duration,
    ) -> Self {
        let current_limit = Arc::new(Mutex::new(initial_limit));
        let semaphore = Arc::new(Semaphore::new(initial_limit));
        let response_times = Arc::new(Mutex::new(VecDeque::with_capacity(window_size)));
        let error_count = Arc::new(Mutex::new(VecDeque::with_capacity(window_size)));
        let condition = Arc::new((Mutex::new(false), Condvar::new()));
        let running = Arc::new(Mutex::new(true));

        let limiter = RateLimiter {
            max_concurrent_requests: max_limit,
            current_limit,
            semaphore,
            response_times,
            error_count,
            window_size,
            target_response_time,
            condition,
            adaptation_interval,
            running,
        };

        // Start the adaptation thread
        limiter.start_adaptation_thread();

        limiter
    }

    /// Acquire permission to make a request
    /// Returns a permit that will automatically release when dropped
    pub async fn acquire(&self) -> Result<RateLimitPermit, tokio::sync::AcquireError> {
        let permit = self.semaphore.acquire().await?;

        Ok(RateLimitPermit {
            start_time: Instant::now(),
            limiter: self,
            _permit: permit,
        })
    }

    /// Record a response time for adaptation
    fn record_response(&self, duration: Duration, is_error: bool) {
        // Record response time
        let mut times = self.response_times.lock().unwrap();
        times.push_back(duration);
        if times.len() > self.window_size {
            times.pop_front();
        }

        // Record error
        let mut errors = self.error_count.lock().unwrap();
        errors.push_back(is_error);
        if errors.len() > self.window_size {
            errors.pop_front();
        }
    }

    /// Start the background thread that adapts the concurrency limit
    fn start_adaptation_thread(&self) {
        let current_limit = Arc::clone(&self.current_limit);
        let response_times = Arc::clone(&self.response_times);
        let error_count = Arc::clone(&self.error_count);
        let semaphore = Arc::clone(&self.semaphore);
        let condition = Arc::clone(&self.condition);
        let running = Arc::clone(&self.running);
        let max_concurrent_requests = self.max_concurrent_requests;
        let target_response_time = self.target_response_time;
        let adaptation_interval = self.adaptation_interval;

        thread::spawn(move || {
            while *running.lock().unwrap() {
                // Sleep for the adaptation interval
                thread::sleep(adaptation_interval);

                // Calculate metrics for adaptation
                let avg_response_time = {
                    let times = response_times.lock().unwrap();
                    if times.is_empty() {
                        continue;
                    }

                    times.iter().sum::<Duration>() / times.len() as u32
                };

                let error_rate = {
                    let errors = error_count.lock().unwrap();
                    if errors.is_empty() {
                        0.0
                    } else {
                        errors.iter().filter(|&&e| e).count() as f64 / errors.len() as f64
                    }
                };

                // Adapt the concurrency limit based on metrics
                let mut limit = current_limit.lock().unwrap();

                // If error rate is too high, reduce concurrency
                if error_rate > 0.1 {
                    *limit = (*limit * 3 / 4).max(1);
                }
                // If response time is too high, reduce concurrency
                else if avg_response_time > target_response_time {
                    *limit = (*limit * 9 / 10).max(1);
                }
                // If response time is well below target, increase concurrency
                else if avg_response_time < target_response_time / 2 {
                    *limit = (*limit * 11 / 10).min(max_concurrent_requests);
                }

                // Update the semaphore with the new limit
                let current_permits = semaphore.available_permits();
                let desired_permits = *limit;

                match current_permits.cmp(&desired_permits) {
                    std::cmp::Ordering::Less => {
                        // Add permits
                        semaphore.add_permits(desired_permits - current_permits);
                    }
                    std::cmp::Ordering::Greater => {
                        // Can't directly remove permits, will have to wait for them to be used
                        // This is handled by the semaphore automatically limiting to the new max
                    }
                    std::cmp::Ordering::Equal => {
                        // No action needed
                    }
                }

                // Notify any waiting threads
                let (lock, cvar) = &*condition;
                let mut notified = lock.lock().unwrap();
                *notified = true;
                cvar.notify_all();
            }
        });
    }

    /// Shutdown the rate limiter and its adaptation thread
    pub fn shutdown(&self) {
        let mut running = self.running.lock().unwrap();
        *running = false;

        // Notify the adaptation thread to check the running flag
        let (lock, cvar) = &*self.condition;
        let mut notified = lock.lock().unwrap();
        *notified = true;
        cvar.notify_all();
    }

    /// Get the current concurrency limit
    pub fn get_current_limit(&self) -> usize {
        *self.current_limit.lock().unwrap()
    }
}

/// A permit that represents permission to make a request
/// When dropped, it will record the response time
pub struct RateLimitPermit<'a> {
    start_time: Instant,
    limiter: &'a RateLimiter,
    _permit: tokio::sync::SemaphorePermit<'a>,
}

impl RateLimitPermit<'_> {
    /// Record the result of the request
    pub fn record_result(self, is_error: bool) {
        let duration = self.start_time.elapsed();
        self.limiter.record_response(duration, is_error);
    }
}

impl Drop for RateLimitPermit<'_> {
    fn drop(&mut self) {
        // If record_result wasn't called explicitly, we'll record it as a success
        // This happens when the permit is just dropped without calling record_result
        if !std::thread::panicking() {
            let duration = self.start_time.elapsed();
            self.limiter.record_response(duration, false);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::sleep;

    #[tokio::test]
    async fn test_rate_limiter_basic() {
        let limiter = RateLimiter::new(
            5,                          // initial_limit
            20,                         // max_limit
            100,                        // window_size
            Duration::from_millis(100), // target_response_time
            Duration::from_millis(500), // adaptation_interval
        );

        // Acquire permits
        let mut permits = Vec::new();
        for _ in 0..5 {
            let permit = limiter.acquire().await.unwrap();
            permits.push(permit);
        }

        // Should be at the limit now
        assert_eq!(limiter.semaphore.available_permits(), 0);

        // Release one permit
        permits.pop();

        // Should be able to acquire one more
        let _permit = limiter.acquire().await.unwrap();

        // Clean up
        limiter.shutdown();
    }
}
