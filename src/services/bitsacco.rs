use crate::{
    config::AppConfig,
    error::{AppError, Result},
    types::{
        BitSaccoBtcBalance, BitSaccoChama, BitSaccoChamaContribution, BitSaccoChamaShare, 
        BitSaccoSavings, BitSaccoTransaction, BitSaccoUser, MpesaStkPushRequest, MpesaStkPushResponse,
        BitSaccoMembershipShare, BitSaccoSharePurchase, LightningPaymentRequest, LightningPaymentResponse,
        WithdrawalRequest, WithdrawalResponse,
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
            .pool_max_idle_per_host(10)
            .pool_idle_timeout(std::time::Duration::from_secs(90))
            .connect_timeout(std::time::Duration::from_secs(10))
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

    pub async fn get_user_by_phone(&self, phone_number: &str, cache: &crate::cache::AppCache) -> Result<BitSaccoUser> {
        // Try to get from cache first
        if let Some(cached_user) = cache.get_user(phone_number).await {
            tracing::debug!("User found in cache: {}", phone_number);
            return Ok(cached_user);
        }

        // If not in cache, fetch from API
        let endpoint = format!("users/phone/{}", phone_number);
        let user: BitSaccoUser = self.make_request(&endpoint).await?;
        
        // Store in cache
        cache.set_user(phone_number, user.clone()).await;
        tracing::debug!("User cached: {}", phone_number);
        
        Ok(user)
    }

    pub async fn get_user_savings(&self, user_id: &str, cache: &crate::cache::AppCache) -> Result<Vec<BitSaccoSavings>> {
        // Try to get from cache first
        if let Some(cached_savings) = cache.get_savings(user_id).await {
            tracing::debug!("Savings found in cache for user: {}", user_id);
            return Ok(cached_savings);
        }

        // If not in cache, fetch from API
        let endpoint = format!("users/{}/savings", user_id);
        let savings: Vec<BitSaccoSavings> = self.make_request(&endpoint).await?;
        
        // Store in cache
        cache.set_savings(user_id, savings.clone()).await;
        tracing::debug!("Savings cached for user: {}", user_id);
        
        Ok(savings)
    }

    pub async fn get_user_chamas(&self, user_id: &str) -> Result<Vec<BitSaccoChama>> {
        let endpoint = format!("users/{}/chamas", user_id);
        self.make_request(&endpoint).await
    }

    pub async fn get_user_btc_balance(&self, user_id: &str, cache: &crate::cache::AppCache) -> Result<BitSaccoBtcBalance> {
        // Try to get from cache first
        if let Some(cached_balance) = cache.get_btc_balance(user_id).await {
            tracing::debug!("BTC balance found in cache for user: {}", user_id);
            return Ok(cached_balance);
        }

        // If not in cache, fetch from API
        let endpoint = format!("users/{}/btc-balance", user_id);
        let balance: BitSaccoBtcBalance = self.make_request(&endpoint).await?;
        
        // Store in cache
        cache.set_btc_balance(user_id, balance.clone()).await;
        tracing::debug!("BTC balance cached for user: {}", user_id);
        
        Ok(balance)
    }

    #[allow(dead_code)]
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
        // For KES deposits, use M-Pesa STK Push
        if currency.to_uppercase() == "KES" {
            return self.create_mpesa_deposit(user_id, amount).await;
        }

        // For other currencies, use regular deposit
        let payload = json!({
            "user_id": user_id,
            "type": "deposit",
            "amount": amount,
            "currency": currency,
            "status": "pending"
        });

        self.make_post_request("transactions", &payload).await
    }

    /// Create M-Pesa STK Push deposit for KES
    pub async fn create_mpesa_deposit(
        &self,
        user_id: &str,
        amount: f64,
    ) -> Result<BitSaccoTransaction> {
        // First, get user details to get phone number
        let user = self.get_user_by_id(user_id).await?;
        
        // Create M-Pesa STK Push request
        let stk_request = MpesaStkPushRequest {
            phone_number: user.phone_number.clone(),
            amount,
            currency: "KES".to_string(),
            account_reference: format!("BITSACCO_{}", user_id),
            transaction_desc: format!("BitSacco deposit of {} KES", amount),
        };

        // Send STK Push request to BitSacco API
        let stk_response: MpesaStkPushResponse = self.make_post_request("mpesa/stk-push", &stk_request).await?;

        // Create transaction record
        let payload = json!({
            "user_id": user_id,
            "amount": amount,
            "currency": "KES",
            "type": "deposit",
            "status": "pending",
            "payment_method": "mpesa",
            "external_reference": stk_response.checkout_request_id,
            "metadata": {
                "merchant_request_id": stk_response.merchant_request_id,
                "checkout_request_id": stk_response.checkout_request_id,
                "response_code": stk_response.response_code,
                "response_description": stk_response.response_description
            }
        });

        self.make_post_request("transactions", &payload).await
    }

    /// Get user by ID (helper method for M-Pesa integration)
    async fn get_user_by_id(&self, user_id: &str) -> Result<BitSaccoUser> {
        let endpoint = format!("users/{}", user_id);
        self.make_request(&endpoint).await
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

    pub async fn get_total_savings(&self, user_id: &str, cache: &crate::cache::AppCache) -> Result<f64> {
        let savings = self.get_user_savings(user_id, cache).await?;
        let total: f64 = savings.iter().map(|s| s.amount).sum();
        Ok(total)
    }

    pub async fn health_check(&self) -> Result<()> {
        let endpoint = "health";
        let response = self
            .client
            .get(format!("{}/{}", self.base_url, endpoint))
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

    // Chama Management Methods
    pub async fn create_chama(
        &self,
        user_id: &str,
        name: &str,
        description: Option<&str>,
    ) -> Result<BitSaccoChama> {
        let payload = json!({
            "name": name,
            "description": description,
            "created_by": user_id,
            "currency": "USD"
        });

        self.make_post_request("chamas", &payload).await
    }

    pub async fn contribute_to_chama(
        &self,
        user_id: &str,
        chama_id: &str,
        amount: f64,
        currency: &str,
    ) -> Result<BitSaccoChamaContribution> {
        let payload = json!({
            "user_id": user_id,
            "chama_id": chama_id,
            "amount": amount,
            "currency": currency,
            "shares_purchased": (amount / 10.0) as i32, // Assuming 1 share = $10
            "status": "pending"
        });

        self.make_post_request("chama-contributions", &payload).await
    }

    pub async fn get_user_chama_shares(
        &self,
        user_id: &str,
        chama_id: Option<&str>,
    ) -> Result<Vec<BitSaccoChamaShare>> {
        let endpoint = if let Some(chama_id) = chama_id {
            format!("users/{}/chama-shares?chama_id={}", user_id, chama_id)
        } else {
            format!("users/{}/chama-shares", user_id)
        };

        self.make_request(&endpoint).await
    }

    pub async fn get_chama_details(&self, chama_id: &str) -> Result<BitSaccoChama> {
        let endpoint = format!("chamas/{}", chama_id);
        self.make_request(&endpoint).await
    }

    // Membership Shares Methods
    pub async fn get_membership_shares(&self, user_id: &str) -> Result<BitSaccoMembershipShare> {
        let endpoint = format!("users/{}/membership-shares", user_id);
        self.make_request(&endpoint).await
    }

    pub async fn buy_membership_shares(
        &self,
        user_id: &str,
        shares_count: u32,
        payment_method: &str,
    ) -> Result<BitSaccoSharePurchase> {
        let payload = json!({
            "user_id": user_id,
            "shares_count": shares_count,
            "payment_method": payment_method,
            "status": "pending"
        });

        self.make_post_request("membership/buy-shares", &payload).await
    }

    pub async fn get_share_history(&self, user_id: &str) -> Result<Vec<BitSaccoSharePurchase>> {
        let endpoint = format!("users/{}/share-history", user_id);
        self.make_request(&endpoint).await
    }

    // Enhanced Transaction Methods
    pub async fn get_transaction_history(&self, user_id: &str) -> Result<Vec<BitSaccoTransaction>> {
        let endpoint = format!("users/{}/transactions", user_id);
        self.make_request(&endpoint).await
    }

    // Lightning Network Methods
    pub async fn create_lightning_payment(
        &self,
        user_id: &str,
        amount: f64,
        currency: &str,
        description: &str,
    ) -> Result<LightningPaymentResponse> {
        let payload = LightningPaymentRequest {
            amount,
            currency: currency.to_string(),
            description: description.to_string(),
            user_id: user_id.to_string(),
        };

        self.make_post_request("lightning/create-payment", &payload).await
    }

    // Withdrawal Methods
    pub async fn create_withdrawal_enhanced(
        &self,
        user_id: &str,
        amount: f64,
        currency: &str,
        payment_method: &str,
        phone_number: Option<&str>,
    ) -> Result<WithdrawalResponse> {
        let payload = WithdrawalRequest {
            user_id: user_id.to_string(),
            amount,
            currency: currency.to_string(),
            payment_method: payment_method.to_string(),
            phone_number: phone_number.map(|s| s.to_string()),
            description: None,
        };

        self.make_post_request("withdrawals", &payload).await
    }

    // Enhanced Deposit with Lightning Support
    pub async fn create_lightning_deposit(
        &self,
        user_id: &str,
        amount: f64,
        currency: &str,
    ) -> Result<LightningPaymentResponse> {
        let description = format!("BitSacco deposit of {} {}", amount, currency);
        self.create_lightning_payment(user_id, amount, currency, &description).await
    }
}
