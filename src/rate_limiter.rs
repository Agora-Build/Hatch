use std::time::{Duration, Instant};

/// Token-bucket rate limiter for HTTP/S3 requests.
/// Enforces a minimum interval between `acquire()` calls.
pub struct RateLimiter {
    last: Option<Instant>,
    interval: Duration,
}

impl RateLimiter {
    /// Create a limiter allowing `max_per_sec` calls per second.
    pub fn new(max_per_sec: u32) -> Self {
        Self {
            last: None,
            interval: Duration::from_millis(1000 / max_per_sec as u64),
        }
    }

    /// Wait if needed to stay within the rate limit, then record this call.
    pub async fn acquire(&mut self) {
        if let Some(last) = self.last {
            let elapsed = last.elapsed();
            if elapsed < self.interval {
                tokio::time::sleep(self.interval - elapsed).await;
            }
        }
        self.last = Some(Instant::now());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn interval_for_5_per_sec_is_200ms() {
        let rl = RateLimiter::new(5);
        assert_eq!(rl.interval, Duration::from_millis(200));
    }

    #[test]
    fn interval_for_1_per_sec_is_1000ms() {
        let rl = RateLimiter::new(1);
        assert_eq!(rl.interval, Duration::from_millis(1000));
    }

    #[test]
    fn interval_for_10_per_sec_is_100ms() {
        let rl = RateLimiter::new(10);
        assert_eq!(rl.interval, Duration::from_millis(100));
    }

    #[tokio::test]
    async fn first_acquire_does_not_sleep() {
        let mut rl = RateLimiter::new(5);
        let start = Instant::now();
        rl.acquire().await;
        // First call should complete essentially immediately (< 50ms)
        assert!(start.elapsed() < Duration::from_millis(50));
    }

    #[tokio::test]
    async fn rapid_second_acquire_waits() {
        let mut rl = RateLimiter::new(5); // 200ms interval
        rl.acquire().await; // first: no wait
        let start = Instant::now();
        rl.acquire().await; // second: should wait ~200ms
        let elapsed = start.elapsed();
        // Should have waited at least 150ms (give 50ms slack for timing)
        assert!(elapsed >= Duration::from_millis(150), "elapsed was {:?}", elapsed);
    }
}
