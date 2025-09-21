use crate::error::{AppError, Result};
use crate::types::{BitSaccoBtcBalance, BitSaccoSavings, BitSaccoUser, BtcPrice};
use moka::future::Cache;
use std::sync::Arc;
use std::time::Duration;

/// Cache configuration
#[derive(Debug, Clone)]
pub struct CacheConfig {
    pub user_cache_ttl: Duration,
    pub btc_price_cache_ttl: Duration,
    pub savings_cache_ttl: Duration,
    pub max_capacity: u64,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            user_cache_ttl: Duration::from_secs(300), // 5 minutes
            btc_price_cache_ttl: Duration::from_secs(60), // 1 minute
            savings_cache_ttl: Duration::from_secs(180), // 3 minutes
            max_capacity: 1000,
        }
    }
}

/// Application cache manager
#[derive(Debug, Clone)]
pub struct AppCache {
    user_cache: Arc<Cache<String, BitSaccoUser>>,
    btc_price_cache: Arc<Cache<String, BtcPrice>>,
    savings_cache: Arc<Cache<String, Vec<BitSaccoSavings>>>,
    btc_balance_cache: Arc<Cache<String, BitSaccoBtcBalance>>,
}

impl AppCache {
    pub fn new(config: CacheConfig) -> Self {
        let user_cache = Arc::new(
            Cache::builder()
                .time_to_live(config.user_cache_ttl)
                .max_capacity(config.max_capacity)
                .build(),
        );

        let btc_price_cache = Arc::new(
            Cache::builder()
                .time_to_live(config.btc_price_cache_ttl)
                .max_capacity(10) // Only need to cache a few currency prices
                .build(),
        );

        let savings_cache = Arc::new(
            Cache::builder()
                .time_to_live(config.savings_cache_ttl)
                .max_capacity(config.max_capacity)
                .build(),
        );

        let btc_balance_cache = Arc::new(
            Cache::builder()
                .time_to_live(config.savings_cache_ttl)
                .max_capacity(config.max_capacity)
                .build(),
        );

        Self {
            user_cache,
            btc_price_cache,
            savings_cache,
            btc_balance_cache,
        }
    }

    /// Get user from cache or return None if not found
    pub async fn get_user(&self, phone_number: &str) -> Option<BitSaccoUser> {
        self.user_cache.get(phone_number).await
    }

    /// Store user in cache
    pub async fn set_user(&self, phone_number: &str, user: BitSaccoUser) {
        self.user_cache.insert(phone_number.to_string(), user).await;
    }

    /// Get BTC price from cache or return None if not found
    pub async fn get_btc_price(&self, currency: &str) -> Option<BtcPrice> {
        self.btc_price_cache.get(currency).await
    }

    /// Store BTC price in cache
    pub async fn set_btc_price(&self, currency: &str, price: BtcPrice) {
        self.btc_price_cache.insert(currency.to_string(), price).await;
    }

    /// Get user savings from cache or return None if not found
    pub async fn get_savings(&self, user_id: &str) -> Option<Vec<BitSaccoSavings>> {
        self.savings_cache.get(user_id).await
    }

    /// Store user savings in cache
    pub async fn set_savings(&self, user_id: &str, savings: Vec<BitSaccoSavings>) {
        self.savings_cache.insert(user_id.to_string(), savings).await;
    }

    /// Get BTC balance from cache or return None if not found
    pub async fn get_btc_balance(&self, user_id: &str) -> Option<BitSaccoBtcBalance> {
        self.btc_balance_cache.get(user_id).await
    }

    /// Store BTC balance in cache
    pub async fn set_btc_balance(&self, user_id: &str, balance: BitSaccoBtcBalance) {
        self.btc_balance_cache.insert(user_id.to_string(), balance).await;
    }

    /// Invalidate user cache entry
    pub async fn invalidate_user(&self, phone_number: &str) {
        self.user_cache.invalidate(phone_number).await;
    }

    /// Invalidate savings cache entry
    pub async fn invalidate_savings(&self, user_id: &str) {
        self.savings_cache.invalidate(user_id).await;
    }

    /// Invalidate BTC balance cache entry
    pub async fn invalidate_btc_balance(&self, user_id: &str) {
        self.btc_balance_cache.invalidate(user_id).await;
    }

    /// Clear all caches
    pub async fn clear_all(&self) {
        self.user_cache.invalidate_all();
        self.btc_price_cache.invalidate_all();
        self.savings_cache.invalidate_all();
        self.btc_balance_cache.invalidate_all();
    }

    /// Get cache statistics
    pub fn get_stats(&self) -> CacheStats {
        CacheStats {
            user_cache_size: self.user_cache.entry_count(),
            btc_price_cache_size: self.btc_price_cache.entry_count(),
            savings_cache_size: self.savings_cache.entry_count(),
            btc_balance_cache_size: self.btc_balance_cache.entry_count(),
        }
    }
}

/// Cache statistics
#[derive(Debug, Clone)]
pub struct CacheStats {
    pub user_cache_size: u64,
    pub btc_price_cache_size: u64,
    pub savings_cache_size: u64,
    pub btc_balance_cache_size: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{BitSaccoUser, BtcPrice};

    #[tokio::test]
    async fn test_user_cache() {
        let cache = AppCache::new(CacheConfig::default());
        
        let user = BitSaccoUser {
            id: "test_user".to_string(),
            phone_number: "+1234567890".to_string(),
            name: Some("Test User".to_string()),
            email: Some("test@example.com".to_string()),
            created_at: chrono::Utc::now().to_rfc3339(),
            updated_at: chrono::Utc::now().to_rfc3339(),
        };

        // Test cache miss
        assert!(cache.get_user("+1234567890").await.is_none());

        // Test cache set and get
        cache.set_user("+1234567890", user.clone()).await;
        let cached_user = cache.get_user("+1234567890").await;
        assert!(cached_user.is_some());
        assert_eq!(cached_user.unwrap().id, user.id);

        // Test cache invalidation
        cache.invalidate_user("+1234567890").await;
        assert!(cache.get_user("+1234567890").await.is_none());
    }

    #[tokio::test]
    async fn test_btc_price_cache() {
        let cache = AppCache::new(CacheConfig::default());
        
        let price = BtcPrice {
            currency: "USD".to_string(),
            price: 50000.0,
            change_24h: 2.5,
            last_updated: chrono::Utc::now().to_rfc3339(),
        };

        // Test cache miss
        assert!(cache.get_btc_price("USD").await.is_none());

        // Test cache set and get
        cache.set_btc_price("USD", price.clone()).await;
        let cached_price = cache.get_btc_price("USD").await;
        assert!(cached_price.is_some());
        assert_eq!(cached_price.unwrap().price, price.price);
    }
}
