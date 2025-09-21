use crate::{
    config::AppConfig,
    error::{AppError, Result},
    types::BtcPrice,
};
use reqwest::Client;
use std::collections::HashMap;
use tracing::{error, info, warn};

#[derive(Debug, Clone)]
pub struct BtcService {
    client: Client,
    base_url: String,
    api_key: Option<String>,
}

impl BtcService {
    pub fn new(config: &AppConfig) -> Result<Self> {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .pool_max_idle_per_host(10)
            .pool_idle_timeout(std::time::Duration::from_secs(90))
            .connect_timeout(std::time::Duration::from_secs(10))
            .build()
            .map_err(|e| AppError::Internal(format!("Failed to create HTTP client: {}", e)))?;

        Ok(Self {
            client,
            base_url: config.btc_api_base_url.clone(),
            api_key: config.btc_api_key.clone(),
        })
    }

    async fn make_request<T>(&self, endpoint: &str) -> Result<T>
    where
        T: serde::de::DeserializeOwned,
    {
        let mut url = format!("{}/{}", self.base_url, endpoint);

        // Add API key if available
        if let Some(api_key) = &self.api_key {
            url = format!("{}?api_key={}", url, api_key);
        }

        info!("Making request to BTC API: {}", endpoint);

        let response = self
            .client
            .get(&url)
            .header("Content-Type", "application/json")
            .send()
            .await
            .map_err(|e| AppError::BtcService(format!("Request failed: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());

            error!("BTC API error: {} - {}", status, error_text);
            return Err(AppError::BtcService(format!(
                "API error {}: {}",
                status, error_text
            )));
        }

        let data: T = response
            .json()
            .await
            .map_err(|e| AppError::BtcService(format!("Failed to parse response: {}", e)))?;

        Ok(data)
    }

    pub async fn get_btc_price(&self, currency: &str, cache: &crate::cache::AppCache) -> Result<BtcPrice> {
        // Try to get from cache first
        if let Some(cached_price) = cache.get_btc_price(currency).await {
            tracing::debug!("BTC price found in cache for currency: {}", currency);
            return Ok(cached_price);
        }

        // If not in cache, fetch from API
        let price = self.get_btc_price_from_coinbase(currency).await?;
        
        // Store in cache
        cache.set_btc_price(currency, price.clone()).await;
        tracing::debug!("BTC price cached for currency: {}", currency);
        
        Ok(price)
    }

    async fn get_btc_price_from_coinbase(&self, currency: &str) -> Result<BtcPrice> {
        // Coinbase API endpoint for BTC price
        let endpoint = format!("prices/BTC-{}/spot", currency.to_uppercase());

        let response: serde_json::Value = self.make_request(&endpoint).await?;

        if let Some(data) = response.get("data") {
            let price_str = data
                .get("amount")
                .and_then(|v| v.as_str())
                .ok_or_else(|| AppError::BtcService("Price not found in response".to_string()))?;

            let price = price_str
                .parse::<f64>()
                .map_err(|_| AppError::BtcService("Invalid price format".to_string()))?;

            // For 24h change, we'll need to make a separate request to get historical data
            // For now, we'll set it to 0.0 and can enhance later
            let change_24h = 0.0;

            Ok(BtcPrice {
                currency: currency.to_uppercase(),
                price,
                change_24h,
                last_updated: chrono::Utc::now().to_rfc3339(),
            })
        } else {
            Err(AppError::BtcService(
                "Bitcoin data not found in response".to_string(),
            ))
        }
    }

    pub async fn get_btc_price_usd(&self, cache: &crate::cache::AppCache) -> Result<BtcPrice> {
        self.get_btc_price("usd", cache).await
    }

    #[allow(dead_code)]
    pub async fn get_btc_price_kes(&self, cache: &crate::cache::AppCache) -> Result<BtcPrice> {
        self.get_btc_price("kes", cache).await
    }

    pub async fn health_check(&self, cache: &crate::cache::AppCache) -> Result<()> {
        // Try to get BTC price as a health check
        match self.get_btc_price_usd(cache).await {
            Ok(_) => {
                info!("BTC service health check passed");
                Ok(())
            }
            Err(e) => {
                warn!("BTC service health check failed: {}", e);
                Err(AppError::BtcService("Health check failed".to_string()))
            }
        }
    }
}
