//! Retry logic with exponential backoff and jitter.
//!
//! This module provides configurable retry logic for transient failures.
//!
//! Features:
//! - Configurable retries
//! - Exponential backoff
//! - Jitter to prevent thundering herd
//! - Retry on specific error types only

use std::time::Duration;
use tracing::{debug, warn};

use crate::error::{OrchestratorError, Result};

/// Retry configuration.
#[derive(Debug, Clone)]
pub struct RetryConfig {
    /// Maximum number of retries.
    pub max_retries: u32,
    /// Base delay for exponential backoff.
    pub base_delay: Duration,
    /// Maximum delay between retries.
    pub max_delay: Duration,
    /// Multiplier for exponential backoff.
    pub multiplier: f32,
    /// Add jitter to delays.
    pub jitter: bool,
    /// Retry only on these error types.
    pub retryable_errors: Vec<RetryableError>,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            base_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(30),
            multiplier: 2.0,
            jitter: true,
            retryable_errors: vec![
                RetryableError::Http,
                RetryableError::RateLimit,
                RetryableError::Timeout,
                RetryableError::Network,
            ],
        }
    }
}

impl RetryConfig {
    /// Create a new retry config with default values.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set maximum retries.
    pub fn with_max_retries(mut self, max: u32) -> Self {
        self.max_retries = max;
        self
    }

    /// Set base delay.
    pub fn with_base_delay(mut self, delay: Duration) -> Self {
        self.base_delay = delay;
        self
    }

    /// Set maximum delay.
    pub fn with_max_delay(mut self, delay: Duration) -> Self {
        self.max_delay = delay;
        self
    }

    /// Set multiplier.
    pub fn with_multiplier(mut self, mult: f32) -> Self {
        self.multiplier = mult;
        self
    }

    /// Enable or disable jitter.
    pub fn with_jitter(mut self, jitter: bool) -> Self {
        self.jitter = jitter;
        self
    }

    /// Set retryable errors.
    pub fn with_retryable_errors(mut self, errors: Vec<RetryableError>) -> Self {
        self.retryable_errors = errors;
        self
    }

    /// Calculate delay for a given attempt.
    pub fn delay_for_attempt(&self, attempt: u32) -> Duration {
        let delay = self.base_delay.as_secs_f64()
            * (self.multiplier as f64).powi(attempt as i32);

        let delay = Duration::from_secs_f64(delay.min(self.max_delay.as_secs_f64()));

        if self.jitter {
            // Add jitter: delay * (0.5 + random * 0.5)
            let jitter_factor = 0.5 + rand::random::<f64>() * 0.5;
            Duration::from_secs_f64(delay.as_secs_f64() * jitter_factor)
        } else {
            delay
        }
    }

    /// Check if an error is retryable.
    pub fn is_retryable(&self, error: &OrchestratorError) -> bool {
        for retryable in &self.retryable_errors {
            match retryable {
                RetryableError::Http => {
                    if matches!(error, OrchestratorError::http_error(_)) {
                        return true;
                    }
                }
                RetryableError::RateLimit => {
                    if matches!(error, OrchestratorError::RateLimitExceeded { .. }) {
                        return true;
                    }
                }
                RetryableError::Timeout => {
                    if matches!(error, OrchestratorError::Timeout(_)) {
                        return true;
                    }
                }
                RetryableError::Network => {
                    if matches!(error, OrchestratorError::io_error(_)) {
                        return true;
                    }
                }
                RetryableError::Database => {
                    if matches!(error, OrchestratorError::database_error(_)) {
                        return true;
                    }
                }
                RetryableError::Custom => {
                    // Custom errors are always retryable
                    return true;
                }
            }
        }
        false
    }
}

/// Types of retryable errors.
#[derive(Debug, Clone)]
pub enum RetryableError {
    /// HTTP errors.
    Http,
    /// Rate limit errors.
    RateLimit,
    /// Timeout errors.
    Timeout,
    /// Network/IO errors.
    Network,
    /// Database errors.
    Database,
    /// Custom errors.
    Custom,
}

/// Retry state for tracking attempts.
pub struct RetryState {
    /// Current attempt number (0-indexed).
    pub attempt: u32,
    /// Total attempts (max_retries + 1).
    pub total_attempts: u32,
    /// Last error encountered.
    pub last_error: Option<OrchestratorError>,
    /// Total time spent retrying.
    pub total_wait_time: Duration,
}

impl RetryState {
    /// Create new retry state.
    pub fn new(max_retries: u32) -> Self {
        Self {
            attempt: 0,
            total_attempts: max_retries + 1,
            last_error: None,
            total_wait_time: Duration::ZERO,
        }
    }

    /// Check if more retries are available.
    pub fn can_retry(&self) -> bool {
        self.attempt < self.total_attempts
    }

    /// Get the next delay.
    pub fn next_delay(&self, config: &RetryConfig) -> Duration {
        config.delay_for_attempt(self.attempt)
    }

    /// Record an attempt.
    pub fn record_attempt(&mut self, error: OrchestratorError, wait_time: Duration) {
        self.attempt += 1;
        self.last_error = Some(error);
        self.total_wait_time += wait_time;
    }
}

/// Execute a future with retry logic.
pub async fn with_retry<F, Fut, T>(config: &RetryConfig, mut f: F) -> Result<T>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = Result<T>>,
{
    let mut state = RetryState::new(config.max_retries);
    let mut last_error: Option<OrchestratorError> = None;

    while state.can_retry() {
        match f().await {
            Ok(result) => return Ok(result),
            Err(e) => {
                // Check if error is retryable
                if !config.is_retryable(&e) {
                    debug!("Error is not retryable: {}", e);
                    return Err(e);
                }

                last_error = Some(e);

                if state.can_retry() {
                    let delay = state.next_delay(config);
                    warn!(
                        "Retry attempt {}/{} after {}ms",
                        state.attempt + 1,
                        config.max_retries,
                        delay.as_millis()
                    );

                    tokio::time::sleep(delay).await;
                    state.record_attempt(last_error.take().unwrap(), delay);
                }
            }
        }
    }

    // All retries exhausted
    Err(last_error.unwrap_or_else(|| {
        OrchestratorError::max_retries_exceeded("Unknown error".to_string())
    }))
}

/// Execute a future with retry logic and custom error handling.
pub async fn with_retry_and_handler<F, Fut, T, H>(
    config: &RetryConfig,
    mut f: F,
    mut handler: H,
) -> Result<T>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = Result<T>>,
    H: FnMut(&OrchestratorError, u32),
{
    let mut state = RetryState::new(config.max_retries);
    let mut last_error: Option<OrchestratorError> = None;

    while state.can_retry() {
        match f().await {
            Ok(result) => return Ok(result),
            Err(e) => {
                if !config.is_retryable(&e) {
                    debug!("Error is not retryable: {}", e);
                    return Err(e);
                }

                handler(&e, state.attempt);
                last_error = Some(e);

                if state.can_retry() {
                    let delay = state.next_delay(config);
                    tokio::time::sleep(delay).await;
                    state.record_attempt(last_error.take().unwrap(), delay);
                }
            }
        }
    }

    Err(last_error.unwrap_or_else(|| {
        OrchestratorError::max_retries_exceeded("Unknown error".to_string())
    }))
}

/// Retry with exponential backoff (simplified version using tokio-retry if available).
#[cfg(feature = "retry")]
pub mod tokio_retry_impl {
    use super::*;
    use ::tokio_retry::{
        strategy::{jitter, ExponentialBackoff},
        Retry,
    };

    /// Execute a future with tokio-retry.
    pub async fn with_retry<F, Fut, T>(config: &RetryConfig, f: F) -> Result<T>
    where
        F: Fn() -> Fut,
        Fut: std::future::Future<Output = Result<T>>,
    {
        let retry_strategy = ExponentialBackoff::from_millis(config.base_delay.as_millis() as u64)
            .map(jitter)
            .take(config.max_retries as usize);

        Retry::spawn(retry_strategy, || {
            let result = f();
            async move {
                result.await.map_err(|e| {
                    if config.is_retryable(&e) {
                        e
                    } else {
                        // Convert non-retryable errors to a special variant
                        OrchestratorError::cancelled(format!("Non-retryable error: {}", e))
                    }
                })
            }
        })
        .await
        .map_err(|e| {
            if matches!(e, OrchestratorError::cancelled(_)) {
                // Extract original error from Cancelled wrapper
                let msg = e.to_string();
                if let Some(original) = msg.strip_prefix("Non-retryable error: ") {
                    OrchestratorError::config_error(original.to_string())
                } else {
                    e
                }
            } else {
                OrchestratorError::max_retries_exceeded(e.to_string())
            }
        })
    }
}

/// Builder for creating retry configurations.
pub struct RetryConfigBuilder {
    config: RetryConfig,
}

impl RetryConfigBuilder {
    /// Create a new builder.
    pub fn new() -> Self {
        Self {
            config: RetryConfig::default(),
        }
    }

    /// Set maximum retries.
    pub fn max_retries(mut self, max: u32) -> Self {
        self.config.max_retries = max;
        self
    }

    /// Set base delay.
    pub fn base_delay(mut self, delay: Duration) -> Self {
        self.config.base_delay = delay;
        self
    }

    /// Set maximum delay.
    pub fn max_delay(mut self, delay: Duration) -> Self {
        self.config.max_delay = delay;
        self
    }

    /// Set multiplier.
    pub fn multiplier(mut self, mult: f32) -> Self {
        self.config.multiplier = mult;
        self
    }

    /// Enable jitter.
    pub fn jitter(mut self, jitter: bool) -> Self {
        self.config.jitter = jitter;
        self
    }

    /// Add a retryable error type.
    pub fn retryable_error(mut self, error: RetryableError) -> Self {
        self.config.retryable_errors.push(error);
        self
    }

    /// Build the retry configuration.
    pub fn build(self) -> RetryConfig {
        self.config
    }
}

impl Default for RetryConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[test]
    fn test_retry_config_default() {
        let config = RetryConfig::default();
        assert_eq!(config.max_retries, 3);
        assert_eq!(config.base_delay, Duration::from_millis(100));
        assert_eq!(config.max_delay, Duration::from_secs(30));
        assert_eq!(config.multiplier, 2.0);
        assert!(config.jitter);
    }

    #[test]
    fn test_retry_config_builder() {
        let config = RetryConfigBuilder::new()
            .max_retries(5)
            .base_delay(Duration::from_millis(50))
            .max_delay(Duration::from_secs(60))
            .multiplier(3.0)
            .jitter(false)
            .build();

        assert_eq!(config.max_retries, 5);
        assert_eq!(config.base_delay, Duration::from_millis(50));
        assert!(!config.jitter);
    }

    #[test]
    fn test_delay_calculation() {
        let config = RetryConfigBuilder::new()
            .jitter(false)
            .build();

        // Attempt 0: 100ms * 2^0 = 100ms
        assert_eq!(config.delay_for_attempt(0), Duration::from_millis(100));
        
        // Attempt 1: 100ms * 2^1 = 200ms
        assert_eq!(config.delay_for_attempt(1), Duration::from_millis(200));
        
        // Attempt 2: 100ms * 2^2 = 400ms
        assert_eq!(config.delay_for_attempt(2), Duration::from_millis(400));
    }

    #[test]
    fn test_delay_max_cap() {
        let config = RetryConfigBuilder::new()
            .base_delay(Duration::from_millis(100))
            .max_delay(Duration::from_secs(1))
            .multiplier(10.0)
            .jitter(false)
            .build();

        // Attempt 3 would be 100ms * 10^3 = 100s, but capped at 1s
        assert!(config.delay_for_attempt(3) <= Duration::from_secs(1));
    }

    #[test]
    fn test_is_retryable() {
        let config = RetryConfig::default();

        assert!(config.is_retryable(&OrchestratorError::Http(reqwest::Error::from(
            std::io::Error::new(std::io::ErrorKind::Other, "test")
        ))));

        assert!(config.is_retryable(&OrchestratorError::RateLimitExceeded { retry_after: 60 }));

        // Config error is not retryable by default
        assert!(!config.is_retryable(&OrchestratorError::config_error("test".to_string())));
    }

    #[test]
    fn test_retry_state() {
        let mut state = RetryState::new(3);
        
        assert!(state.can_retry());
        assert_eq!(state.attempt, 0);
        assert_eq!(state.total_attempts, 4);

        state.record_attempt(
            OrchestratorError::Timeout("test".to_string()),
            Duration::from_millis(100)
        );

        assert_eq!(state.attempt, 1);
        assert!(state.can_retry());
        assert_eq!(state.total_wait_time, Duration::from_millis(100));
    }

    #[tokio::test]
    async fn test_with_retry_success() {
        let config = RetryConfig::default();
        let counter = AtomicUsize::new(0);

        let result = with_retry(&config, || {
            let count = counter.fetch_add(1, Ordering::SeqCst);
            async move {
                if count < 2 {
                    Err(OrchestratorError::Timeout("test".to_string()))
                } else {
                    Ok("success")
                }
            }
        })
        .await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "success");
        assert!(counter.load(Ordering::SeqCst) >= 3);
    }

    #[tokio::test]
    async fn test_with_retry_exhausted() {
        let config = RetryConfigBuilder::new()
            .max_retries(2)
            .build();

        let counter = AtomicUsize::new(0);

        let result = with_retry(&config, || {
            let count = counter.fetch_add(1, Ordering::SeqCst);
            async move {
                Err::<(), _>(OrchestratorError::Timeout(format!("attempt {}", count)))
            }
        })
        .await;

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), OrchestratorError::max_retries_exceeded(_)));
        assert_eq!(counter.load(Ordering::SeqCst), 3); // Initial + 2 retries
    }

    #[tokio::test]
    async fn test_with_retry_non_retryable_error() {
        let config = RetryConfig::default();
        let counter = AtomicUsize::new(0);

        let result = with_retry(&config, || {
            let count = counter.fetch_add(1, Ordering::SeqCst);
            async move {
                Err::<(), _>(OrchestratorError::config_error("non-retryable".to_string()))
            }
        })
        .await;

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), OrchestratorError::config_error(_)));
        assert_eq!(counter.load(Ordering::SeqCst), 1); // Only one attempt
    }

    #[tokio::test]
    async fn test_with_retry_and_handler() {
        let config = RetryConfig::default();
        let counter = AtomicUsize::new(0);
        let handler_calls = AtomicUsize::new(0);

        let result = with_retry_and_handler(
            &config,
            || {
                let count = counter.fetch_add(1, Ordering::SeqCst);
                async move {
                    if count < 1 {
                        Err(OrchestratorError::Timeout("test".to_string()))
                    } else {
                        Ok("success")
                    }
                }
            },
            |_error, attempt| {
                handler_calls.fetch_add(attempt as usize, Ordering::SeqCst);
            },
        )
        .await;

        assert!(result.is_ok());
        assert!(handler_calls.load(Ordering::SeqCst) > 0);
    }
}
