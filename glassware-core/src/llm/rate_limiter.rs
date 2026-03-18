//! Rate Limiter for LLM API Calls
//!
//! Implements a token bucket rate limiter to respect API
//! - Requests per minute (RPM)
//! - Tokens per minute (TPM)
//!
//! Token Bucket Algorithm:
//! - Bucket starts full with `max_tokens`
//! - Each request consumes 1 token (for RPM) or N tokens (for TPM)
//! - Tokens refill at a constant rate
//! - If bucket is empty, requests block until tokens available

use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// Token bucket rate limiter
pub struct TokenBucket {
    /// Maximum tokens in the bucket
    max_tokens: f64,
    /// Current tokens in the bucket
    tokens: f64,
    /// Token refill rate (tokens per second)
    refill_rate: f64,
    /// Last time tokens were refilled
    last_refill: Instant,
}

impl TokenBucket {
    /// Create a new token bucket
    ///
    /// # Arguments
    /// * `max_tokens` - Maximum tokens the bucket can hold
    /// * `refill_rate` - Tokens added per second
    pub fn new(max_tokens: f64, refill_rate: f64) -> Self {
        Self {
            max_tokens,
            tokens: max_tokens, // Start with full bucket
            refill_rate,
            last_refill: Instant::now(),
        }
    }

    /// Refill tokens based on elapsed time
    fn refill(&mut self) {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_refill).as_secs_f64();
        self.tokens = (self.tokens + elapsed * self.refill_rate).min(self.max_tokens);
        self.last_refill = now;
    }

    /// Try to consume tokens, blocking if necessary
    ///
    /// # Arguments
    /// * `tokens` - Number of tokens to consume
    ///
    /// Returns when tokens are available
    pub fn consume(&mut self, tokens: f64) {
        loop {
            self.refill();

            if self.tokens >= tokens {
                self.tokens -= tokens;
                return;
            }

            // Calculate wait time for enough tokens
            let tokens_needed = tokens - self.tokens;
            let wait_secs = tokens_needed / self.refill_rate;
            let wait_duration = Duration::from_secs_f64(wait_secs);

            std::thread::sleep(wait_duration);
        }
    }

    /// Get current token count (for debugging)
    #[allow(dead_code)]
    pub fn tokens_available(&mut self) -> f64 {
        self.refill();
        self.tokens
    }
}

/// Rate limiter for LLM API calls
///
/// Manages both RPM and TPM limits using separate token buckets
pub struct RateLimiter {
    /// RPM token bucket (1 token per request)
    rpm_bucket: Arc<Mutex<TokenBucket>>,
    /// TPM token bucket (1 token per ~4 chars, estimated)
    tpm_bucket: Arc<Mutex<TokenBucket>>,
}

impl RateLimiter {
    /// Create a new rate limiter
    ///
    /// # Arguments
    /// * `rpm` - Maximum requests per minute
    /// * `tpm` - Maximum tokens per minute (1 token ≈ 4 characters)
    pub fn new(rpm: u32, tpm: u32) -> Self {
        // Convert per-minute limits to per-second refill rates
        let rpm_refill = rpm as f64 / 60.0;
        let tpm_refill = tpm as f64 / 60.0;

        Self {
            rpm_bucket: Arc::new(Mutex::new(TokenBucket::new(rpm as f64, rpm_refill))),
            tpm_bucket: Arc::new(Mutex::new(TokenBucket::new(tpm as f64, tpm_refill))),
        }
    }

    /// Acquire permission to make a request
    ///
    /// Blocks until both RPM and TPM limits allow the request
    ///
    /// # Arguments
    /// * `estimated_tokens` - Estimated token count for the request (default: 1000)
    pub fn acquire(&self, estimated_tokens: u32) {
        // Acquire RPM token (1 per request)
        if let Ok(mut bucket) = self.rpm_bucket.lock() {
            bucket.consume(1.0);
        }

        // Acquire TPM tokens (based on estimated token count)
        if let Ok(mut bucket) = self.tpm_bucket.lock() {
            bucket.consume(estimated_tokens as f64);
        }
    }

    /// Create a rate limiter from environment variables
    ///
    /// Uses these env vars:
    /// - `GLASSWARE_LLM_RPM` - Requests per minute (default: 30)
    /// - `GLASSWARE_LLM_TPM` - Tokens per minute (default: 60000)
    pub fn from_env() -> Self {
        let rpm: u32 = std::env::var("GLASSWARE_LLM_RPM")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(30);

        let tpm: u32 = std::env::var("GLASSWARE_LLM_TPM")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(60_000);

        Self::new(rpm, tpm)
    }
}

impl Default for RateLimiter {
    fn default() -> Self {
        Self::from_env()
    }
}

/// Clone implementation for sharing rate limiter across threads
impl Clone for RateLimiter {
    fn clone(&self) -> Self {
        Self {
            rpm_bucket: Arc::clone(&self.rpm_bucket),
            tpm_bucket: Arc::clone(&self.tpm_bucket),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_bucket_refill() {
        let mut bucket = TokenBucket::new(10.0, 1.0); // 10 max, 1 token/sec
        
        // Consume 5 tokens
        bucket.consume(5.0);
        assert!((bucket.tokens_available() - 5.0).abs() < 0.1);

        // Wait 2 seconds (should refill 2 tokens)
        std::thread::sleep(Duration::from_secs(2));
        let tokens = bucket.tokens_available();
        assert!(tokens >= 6.5 && tokens <= 7.5, "Expected ~7 tokens, got {}", tokens);
    }

    #[test]
    fn test_token_bucket_consumption() {
        let mut bucket = TokenBucket::new(10.0, 10.0); // 10 max, 10 tokens/sec
        
        bucket.consume(5.0);
        assert!((bucket.tokens_available() - 5.0).abs() < 0.5);

        bucket.consume(3.0);
        assert!((bucket.tokens_available() - 2.0).abs() < 0.5);
    }

    #[test]
    fn test_rate_limiter_creation() {
        let limiter = RateLimiter::new(30, 60_000);
        assert!(Arc::strong_count(&limiter.rpm_bucket) == 1);
        assert!(Arc::strong_count(&limiter.tpm_bucket) == 1);
    }

    #[test]
    fn test_rate_limiter_from_env() {
        std::env::set_var("GLASSWARE_LLM_RPM", "60");
        std::env::set_var("GLASSWARE_LLM_TPM", "120000");

        let limiter = RateLimiter::from_env();
        
        // Can't directly check internal values, but ensure it doesn't panic
        limiter.acquire(1000);

        std::env::remove_var("GLASSWARE_LLM_RPM");
        std::env::remove_var("GLASSWARE_LLM_TPM");
    }

    #[test]
    fn test_rate_limiter_default() {
        let limiter = RateLimiter::default();
        limiter.acquire(1000); // Should not panic
    }

    #[test]
    fn test_rate_limiter_clone() {
        let limiter = RateLimiter::new(30, 60_000);
        let cloned = limiter.clone();

        // Both should share the same buckets
        assert!(Arc::strong_count(&limiter.rpm_bucket) == 2);
        assert!(Arc::strong_count(&cloned.tpm_bucket) == 2);
    }
}
