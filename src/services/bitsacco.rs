use crate::{
    config::AppConfig,
    error::{AppError, Result},
    types::{
        BitSaccoBtcBalance, BitSaccoChama, BitSaccoSavings, BitSaccoTransaction, BitSaccoUser,
    },
};
use reqwest::Client;
use serde_json::json;
use tracing::{error, info, warn};

#[derive(Debug, Clone)]
pub struct BitSaccoService {
    client: Client,
    base_url: String,
    api_token: String,
}

impl BitSaccoService {
    pub fn new(config: &AppConfig) -> Result<Self> {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .map_err(|e| AppError::Internal(format!("Failed to create HTTP client: {}", e)))?;

        Ok(Self {
            client,
            base_url: config.bitsacco_api_base_url.clone(),
            api_token: config.bitsacco_api_token.clone(),
        })
    }

    async fn make_request<T>(&self, endpoint: &str) -> Result<T>
    where
        T: serde::de::DeserializeOwned,
    {
        let url = format!("{}/{}", self.base_url, endpoint);
        
        info!("Making request to BitSacco API: {}", endpoint);

        let response = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.api_token))
            .header("Content-Type", "application/json")
            .send()
            .await
            .map_err(|e| AppError::BitSacco(format!("Request failed: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            
            error!("BitSacco API error: {} - {}", status, error_text);
            return Err(AppError::BitSacco(format!(
                "API error {}: {}",
                status, error_text
            )));
        }

        let data: T = response
            .json()
            .await
            .map_err(|e| AppError::BitSacco(format!("Failed to parse response: {}", e)))?;

        Ok(data)
    }

    async fn make_post_request<T, U>(&self, endpoint: &str, payload: &T) -> Result<U>
    where
        T: serde::Serialize,
        U: serde::de::DeserializeOwned,
    {
        let url = format!("{}/{}", self.base_url, endpoint);
        
        info!("Making POST request to BitSacco API: {}", endpoint);

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_token))
            .header("Content-Type", "application/json")
            .json(payload)
            .send()
            .await
            .map_err(|e| AppError::BitSacco(format!("Request failed: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            
            error!("BitSacco API error: {} - {}", status, error_text);
            return Err(AppError::BitSacco(format!(
                "API error {}: {}",
                status, error_text
            )));
        }

        let data: U = response
            .json()
            .await
            .map_err(|e| AppError::BitSacco(format!("Failed to parse response: {}", e)))?;

        Ok(data)
    }

    pub async fn get_user_by_phone(&self, phone_number: &str) -> Result<BitSaccoUser> {
        let endpoint = format!("users/phone/{}", phone_number);
        self.make_request(&endpoint).await
    }

    pub async fn get_user_savings(&self, user_id: &str) -> Result<Vec<BitSaccoSavings>> {
        let endpoint = format!("users/{}/savings", user_id);
        self.make_request(&endpoint).await
    }

    pub async fn get_user_chamas(&self, user_id: &str) -> Result<Vec<BitSaccoChama>> {
        let endpoint = format!("users/{}/chamas", user_id);
        self.make_request(&endpoint).await
    }

    pub async fn get_user_btc_balance(&self, user_id: &str) -> Result<BitSaccoBtcBalance> {
        let endpoint = format!("users/{}/btc-balance", user_id);
        self.make_request(&endpoint).await
    }

    pub async fn get_user_transactions(&self, user_id: &str) -> Result<Vec<BitSaccoTransaction>> {
        let endpoint = format!("users/{}/transactions", user_id);
        self.make_request(&endpoint).await
    }

    pub async fn create_deposit(
        &self,
        user_id: &str,
        amount: f64,
        currency: &str,
    ) -> Result<BitSaccoTransaction> {
        let payload = json!({
            "user_id": user_id,
            "type": "deposit",
            "amount": amount,
            "currency": currency,
            "status": "pending"
        });

        self.make_post_request("transactions", &payload).await
    }

    pub async fn create_withdrawal(
        &self,
        user_id: &str,
        amount: f64,
        currency: &str,
    ) -> Result<BitSaccoTransaction> {
        let payload = json!({
            "user_id": user_id,
            "type": "withdrawal",
            "amount": amount,
            "currency": currency,
            "status": "pending"
        });

        self.make_post_request("transactions", &payload).await
    }

    pub async fn create_transfer(
        &self,
        user_id: &str,
        amount: f64,
        currency: &str,
        recipient_phone: &str,
    ) -> Result<BitSaccoTransaction> {
        let payload = json!({
            "user_id": user_id,
            "type": "transfer",
            "amount": amount,
            "currency": currency,
            "recipient_phone": recipient_phone,
            "status": "pending"
        });

        self.make_post_request("transactions", &payload).await
    }

    pub async fn get_total_savings(&self, user_id: &str) -> Result<f64> {
        let savings = self.get_user_savings(user_id).await?;
        let total: f64 = savings.iter().map(|s| s.amount).sum();
        Ok(total)
    }

    pub async fn health_check(&self) -> Result<()> {
        let endpoint = "health";
        let response = self
            .client
            .get(&format!("{}/{}", self.base_url, endpoint))
            .header("Authorization", format!("Bearer {}", self.api_token))
            .send()
            .await
            .map_err(|e| AppError::BitSacco(format!("Health check failed: {}", e)))?;

        if response.status().is_success() {
            info!("BitSacco API health check passed");
            Ok(())
        } else {
            warn!("BitSacco API health check failed: {}", response.status());
            Err(AppError::BitSacco("Health check failed".to_string()))
        }
    }
}
