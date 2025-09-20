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

    pub async fn get_btc_price(&self, currency: &str) -> Result<BtcPrice> {
        // Try to get price from BitSacco API first (if it provides BTC prices)
        // For now, we'll use CoinGecko as fallback
        self.get_btc_price_from_coingecko(currency).await
    }

    async fn get_btc_price_from_coingecko(&self, currency: &str) -> Result<BtcPrice> {
        let endpoint = format!("simple/price?ids=bitcoin&vs_currencies={}&include_24hr_change=true", 
                              currency.to_lowercase());
        
        let response: HashMap<String, serde_json::Value> = self.make_request(&endpoint).await?;
        
        if let Some(bitcoin_data) = response.get("bitcoin") {
            let price = bitcoin_data
                .get(&currency.to_lowercase())
                .and_then(|v| v.as_f64())
                .ok_or_else(|| AppError::BtcService("Price not found in response".to_string()))?;
                
            let change_24h = bitcoin_data
                .get(&format!("{}_24h_change", currency.to_lowercase()))
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0);

            Ok(BtcPrice {
                currency: currency.to_uppercase(),
                price,
                change_24h,
                last_updated: chrono::Utc::now().to_rfc3339(),
            })
        } else {
            Err(AppError::BtcService("Bitcoin data not found in response".to_string()))
        }
    }

    pub async fn get_btc_price_usd(&self) -> Result<BtcPrice> {
        self.get_btc_price("usd").await
    }

    pub async fn get_btc_price_kes(&self) -> Result<BtcPrice> {
        self.get_btc_price("kes").await
    }

    pub async fn health_check(&self) -> Result<()> {
        // Try to get BTC price as a health check
        match self.get_btc_price_usd().await {
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
