use crate::error::{AppError, Result};
use governor::{Quota, RateLimiter};
use std::num::NonZeroU32;
use std::sync::Arc;

/// Rate limiter configuration
#[derive(Debug, Clone)]
pub struct RateLimitConfig {
    pub requests_per_minute: u32,
    pub burst_size: u32,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            requests_per_minute: 60,
            burst_size: 10,
        }
    }
}

/// Rate limiter service
#[derive(Debug, Clone)]
pub struct RateLimiterService {
    limiter: Arc<RateLimiter<governor::state::direct::NotKeyed, governor::state::InMemoryState, governor::clock::DefaultClock>>,
}

impl RateLimiterService {
    pub fn new(config: RateLimitConfig) -> Self {
        let quota = Quota::per_minute(NonZeroU32::new(config.requests_per_minute).unwrap())
            .allow_burst(NonZeroU32::new(config.burst_size).unwrap());
        
        let limiter = RateLimiter::direct(quota);
        
        Self {
            limiter: Arc::new(limiter),
        }
    }

    pub async fn check_rate_limit(&self, identifier: &str) -> Result<()> {
        // For now, we'll use a simple approach
        // In production, you might want to use different rate limits per user/IP
        match self.limiter.check() {
            Ok(_) => Ok(()),
            Err(_) => {
                tracing::warn!("Rate limit exceeded for identifier: {}", identifier);
                Err(AppError::RateLimit)
            }
        }
    }

    pub fn check_rate_limit_sync(&self, identifier: &str) -> Result<()> {
        match self.limiter.check() {
            Ok(_) => Ok(()),
            Err(_) => {
                tracing::warn!("Rate limit exceeded for identifier: {}", identifier);
                Err(AppError::RateLimit)
            }
        }
    }
}

/// Simple rate limiting middleware for Axum
#[derive(Debug, Clone)]
pub struct RateLimitLayer {
    rate_limiter: RateLimiterService,
}

impl RateLimitLayer {
    pub fn new(config: RateLimitConfig) -> Self {
        Self {
            rate_limiter: RateLimiterService::new(config),
        }
    }
}

/// Per-user rate limiter
#[derive(Debug, Clone)]
pub struct UserRateLimiter {
    limiters: Arc<dashmap::DashMap<String, RateLimiterService>>,
    config: RateLimitConfig,
}

impl UserRateLimiter {
    pub fn new(config: RateLimitConfig) -> Self {
        Self {
            limiters: Arc::new(dashmap::DashMap::new()),
            config,
        }
    }

    pub async fn check_user_rate_limit(&self, user_id: &str) -> Result<()> {
        let limiter = self.limiters
            .entry(user_id.to_string())
            .or_insert_with(|| RateLimiterService::new(self.config.clone()));
        
        limiter.check_rate_limit(user_id).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_rate_limiter() {
        let config = RateLimitConfig {
            requests_per_minute: 2,
            burst_size: 2,
        };
        
        let rate_limiter = RateLimiterService::new(config);
        
        // First request should succeed
        assert!(rate_limiter.check_rate_limit("test").await.is_ok());
        
        // Second request should also succeed (within burst)
        assert!(rate_limiter.check_rate_limit("test").await.is_ok());
        
        // Third request should fail (rate limited)
        assert!(rate_limiter.check_rate_limit("test").await.is_err());
    }
}
