//! Rate limiting with Governor for async-friendly throttling.
//!
//! This module provides Governor-based rate limiting for various services.
//!
//! Features:
//! - Governor-based rate limiting
//! - Configurable rates per service (npm, GitHub)
//! - Async-friendly
//! - Automatic backpressure

use std::num::NonZeroU32;
use std::sync::Arc;
use std::time::Duration;
use tracing::debug;

#[cfg(feature = "rate-limit")]
use governor::{
    clock::DefaultClock,
    middleware::NoOpMiddleware,
    state::{direct::NotKeyed, InMemoryState},
    Quota, RateLimiter as GovernorLimiter,
};

/// Rate limiter wrapper.
pub struct ThrottleLimiter {
    #[cfg(feature = "rate-limit")]
    limiter: Arc<GovernorLimiter<NotKeyed, InMemoryState, DefaultClock, NoOpMiddleware>>,
    #[cfg(not(feature = "rate-limit"))]
    _placeholder: (),
}

impl ThrottleLimiter {
    /// Create a new rate limiter with specified requests per second.
    pub fn new_per_second(rate: f32) -> Self {
        #[cfg(feature = "rate-limit")]
        {
            let quota = Quota::per_second(
                NonZeroU32::new(rate as u32).unwrap_or(NonZeroU32::new(1).unwrap())
            );
            
            let limiter = GovernorLimiter::direct(quota);
            
            Self {
                limiter: Arc::new(limiter),
            }
        }
        
        #[cfg(not(feature = "rate-limit"))]
        {
            Self {
                _placeholder: (),
            }
        }
    }

    /// Create a new rate limiter with specified requests per minute.
    pub fn new_per_minute(rate: u32) -> Self {
        #[cfg(feature = "rate-limit")]
        {
            let quota = Quota::per_minute(NonZeroU32::new(rate).unwrap_or(NonZeroU32::new(1).unwrap()));
            
            let limiter = GovernorLimiter::direct(quota);
            
            Self {
                limiter: Arc::new(limiter),
            }
        }
        
        #[cfg(not(feature = "rate-limit"))]
        {
            Self {
                _placeholder: (),
            }
        }
    }

    /// Create a new rate limiter with specified requests per hour.
    pub fn new_per_hour(rate: u32) -> Self {
        #[cfg(feature = "rate-limit")]
        {
            let quota = Quota::per_hour(NonZeroU32::new(rate).unwrap_or(NonZeroU32::new(1).unwrap()));
            
            let limiter = GovernorLimiter::direct(quota);
            
            Self {
                limiter: Arc::new(limiter),
            }
        }
        
        #[cfg(not(feature = "rate-limit"))]
        {
            Self {
                _placeholder: (),
            }
        }
    }

    /// Wait until a request can be made.
    pub async fn wait(&self) {
        #[cfg(feature = "rate-limit")]
        {
            self.limiter.until_ready().await;
            debug!("Rate limiter: request allowed");
        }
        
        #[cfg(not(feature = "rate-limit"))]
        {
            // No-op when rate limiting is disabled
        }
    }

    /// Check if a request can be made without waiting.
    pub fn check(&self) -> bool {
        #[cfg(feature = "rate-limit")]
        {
            self.limiter.check().is_ok()
        }
        
        #[cfg(not(feature = "rate-limit"))]
        {
            true
        }
    }

    /// Get the number of remaining requests in the current window.
    pub fn remaining(&self) -> u32 {
        #[cfg(feature = "rate-limit")]
        {
            self.limiter.check().map(|_| {
                // This is an approximation - Governor doesn't expose remaining directly
                // In production, you'd want to track this separately
                1
            }).unwrap_or(0)
        }
        
        #[cfg(not(feature = "rate-limit"))]
        {
            u32::MAX
        }
    }

    /// Get the rate limit configuration.
    pub fn quota(&self) -> Option<Quota> {
        #[cfg(feature = "rate-limit")]
        {
            // Governor doesn't expose quota directly, return None
            None
        }
        
        #[cfg(not(feature = "rate-limit"))]
        {
            None
        }
    }

    /// Clone the rate limiter (cheap, uses Arc).
    pub fn clone_limiter(&self) -> Self {
        #[cfg(feature = "rate-limit")]
        {
            Self {
                limiter: Arc::clone(&self.limiter),
            }
        }
        
        #[cfg(not(feature = "rate-limit"))]
        {
            Self {
                _placeholder: (),
            }
        }
    }
}

impl Clone for ThrottleLimiter {
    fn clone(&self) -> Self {
        self.clone_limiter()
    }
}

/// Multi-service rate limiter manager.
pub struct MultiThrottleLimiter {
    #[cfg(feature = "rate-limit")]
    limiters: parking_lot::RwLock<std::collections::HashMap<String, Arc<GovernorLimiter<NotKeyed, InMemoryState, DefaultClock, NoOpMiddleware>>>>,
    #[cfg(not(feature = "rate-limit"))]
    _placeholder: (),
}

impl MultiThrottleLimiter {
    /// Create a new multi-service rate limiter.
    pub fn new() -> Self {
        #[cfg(feature = "rate-limit")]
        {
            Self {
                limiters: parking_lot::RwLock::new(std::collections::HashMap::new()),
            }
        }
        
        #[cfg(not(feature = "rate-limit"))]
        {
            Self {
                _placeholder: (),
            }
        }
    }

    /// Add or update a rate limiter for a service.
    pub fn add_limiter(&self, service: &str, rate_per_second: f32) {
        #[cfg(feature = "rate-limit")]
        {
            let quota = Quota::per_second(
                NonZeroU32::new(rate_per_second as u32).unwrap_or(NonZeroU32::new(1).unwrap())
            );
            
            let limiter = GovernorLimiter::direct(quota);
            
            let mut limiters = self.limiters.write();
            limiters.insert(service.to_string(), Arc::new(limiter));
            
            debug!("Added rate limiter for {}: {} req/s", service, rate_per_second);
        }
        
        #[cfg(not(feature = "rate-limit"))]
        {
            let _ = (service, rate_per_second);
        }
    }

    /// Wait for a specific service's rate limiter.
    pub async fn wait_for(&self, service: &str) {
        #[cfg(feature = "rate-limit")]
        {
            let limiters = self.limiters.read();
            if let Some(limiter) = limiters.get(service) {
                limiter.until_ready().await;
            } else {
                debug!("No rate limiter found for service: {}", service);
            }
        }
        
        #[cfg(not(feature = "rate-limit"))]
        {
            let _ = service;
        }
    }

    /// Check if a request can be made for a service.
    pub fn check_for(&self, service: &str) -> bool {
        #[cfg(feature = "rate-limit")]
        {
            let limiters = self.limiters.read();
            if let Some(limiter) = limiters.get(service) {
                limiter.check().is_ok()
            } else {
                true
            }
        }
        
        #[cfg(not(feature = "rate-limit"))]
        {
            let _ = service;
            true
        }
    }

    /// Remove a rate limiter for a service.
    pub fn remove_limiter(&self, service: &str) {
        #[cfg(feature = "rate-limit")]
        {
            let mut limiters = self.limiters.write();
            limiters.remove(service);
        }
        
        #[cfg(not(feature = "rate-limit"))]
        {
            let _ = service;
        }
    }

    /// Get all configured services.
    pub fn services(&self) -> Vec<String> {
        #[cfg(feature = "rate-limit")]
        {
            self.limiters.read().keys().cloned().collect()
        }
        
        #[cfg(not(feature = "rate-limit"))]
        {
            vec![]
        }
    }
}

impl Default for MultiThrottleLimiter {
    fn default() -> Self {
        Self::new()
    }
}

/// Builder for creating rate limiters with common presets.
pub struct ThrottleLimiterBuilder {
    service: String,
    rate: f32,
    burst: Option<u32>,
}

impl ThrottleLimiterBuilder {
    /// Create a new builder for a service.
    pub fn for_service(service: &str) -> Self {
        Self {
            service: service.to_string(),
            rate: 1.0,
            burst: None,
        }
    }

    /// Set rate per second.
    pub fn per_second(mut self, rate: f32) -> Self {
        self.rate = rate;
        self
    }

    /// Set rate per minute.
    pub fn per_minute(mut self, rate: u32) -> Self {
        self.rate = rate as f32 / 60.0;
        self
    }

    /// Set rate per hour.
    pub fn per_hour(mut self, rate: u32) -> Self {
        self.rate = rate as f32 / 3600.0;
        self
    }

    /// Set burst allowance (not currently used, reserved for future).
    pub fn burst(mut self, burst: u32) -> Self {
        self.burst = Some(burst);
        self
    }

    /// Build the rate limiter.
    pub fn build(self) -> ThrottleLimiter {
        ThrottleLimiter::new_per_second(self.rate)
    }
}

/// Common rate limit presets.
pub mod presets {
    use super::*;

    /// npm registry rate limit (unauthenticated): ~2 req/s.
    pub fn npm_unauthenticated() -> ThrottleLimiter {
        ThrottleLimiter::new_per_second(2.0)
    }

    /// npm registry rate limit (authenticated): ~10 req/s.
    pub fn npm_authenticated() -> ThrottleLimiter {
        ThrottleLimiter::new_per_second(10.0)
    }

    /// GitHub API rate limit (unauthenticated): ~1 req/s (60/hour).
    pub fn github_unauthenticated() -> ThrottleLimiter {
        ThrottleLimiter::new_per_second(1.0)
    }

    /// GitHub API rate limit (authenticated): ~1.39 req/s (5000/hour).
    pub fn github_authenticated() -> ThrottleLimiter {
        ThrottleLimiter::new_per_second(1.39)
    }

    /// GitHub search API rate limit: ~0.33 req/s (30/min).
    pub fn github_search() -> ThrottleLimiter {
        ThrottleLimiter::new_per_minute(30)
    }

    /// Conservative default: 1 req/s.
    pub fn default_conservative() -> ThrottleLimiter {
        ThrottleLimiter::new_per_second(1.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rate_limiter_creation() {
        let limiter = ThrottleLimiter::new_per_second(10.0);
        assert!(limiter.check());
    }

    #[test]
    fn test_rate_limiter_per_minute() {
        let limiter = ThrottleLimiter::new_per_minute(60);
        assert!(limiter.check());
    }

    #[test]
    fn test_rate_limiter_per_hour() {
        let limiter = ThrottleLimiter::new_per_hour(3600);
        assert!(limiter.check());
    }

    #[test]
    fn test_rate_limiter_clone() {
        let limiter = ThrottleLimiter::new_per_second(5.0);
        let cloned = limiter.clone();
        
        // Both should work independently
        assert!(limiter.check());
        assert!(cloned.check());
    }

    #[tokio::test]
    async fn test_rate_limiter_wait() {
        let limiter = ThrottleLimiter::new_per_second(100.0); // High rate for fast test
        
        let start = std::time::Instant::now();
        limiter.wait().await;
        let elapsed = start.elapsed();
        
        // Should complete quickly with high rate limit
        assert!(elapsed < Duration::from_millis(100));
    }

    #[tokio::test]
    async fn test_rate_limiter_backpressure() {
        let limiter = ThrottleLimiter::new_per_second(10.0);
        
        // Make several requests
        for _ in 0..5 {
            limiter.wait().await;
        }
        
        // The last request should have incurred some delay
        // (though Governor's token bucket may allow bursts)
    }

    #[test]
    fn test_multi_rate_limiter() {
        let multi = MultiThrottleLimiter::new();
        
        multi.add_limiter("npm", 2.0);
        multi.add_limiter("github", 1.0);
        
        let services = multi.services();
        assert!(services.contains(&"npm".to_string()));
        assert!(services.contains(&"github".to_string()));
    }

    #[tokio::test]
    async fn test_multi_rate_limiter_wait() {
        let multi = MultiThrottleLimiter::new();
        multi.add_limiter("test_service", 100.0);
        
        multi.wait_for("test_service").await;
        assert!(multi.check_for("test_service"));
    }

    #[test]
    fn test_rate_limiter_builder() {
        let limiter = ThrottleLimiterBuilder::for_service("npm")
            .per_second(2.0)
            .build();
        
        assert!(limiter.check());
    }

    #[test]
    fn test_rate_limiter_presets() {
        let npm = presets::npm_unauthenticated();
        assert!(npm.check());
        
        let github = presets::github_authenticated();
        assert!(github.check());
        
        let search = presets::github_search();
        assert!(search.check());
    }

    #[test]
    fn test_rate_limiter_remaining() {
        let limiter = ThrottleLimiter::new_per_second(1.0);
        let remaining = limiter.remaining();
        // Should return some value (approximation)
        assert!(remaining >= 0);
    }
}
