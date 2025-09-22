use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::env;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    // WhatsApp Configuration
    pub whatsapp_access_token: String,
    pub whatsapp_phone_number_id: String,
    pub whatsapp_webhook_verify_token: String,
    pub whatsapp_api_base_url: String,
    pub whatsapp_media_base_url: String,
    
    // Twilio configuration
    pub twilio_account_sid: String,
    pub twilio_auth_token: String,
    pub twilio_whatsapp_number: String,

    // BitSacco API Configuration
    pub bitsacco_api_base_url: String,
    pub bitsacco_api_token: String,

    // Server Configuration
    pub server_host: String,
    pub server_port: u16,
    pub rust_log: String,

    // Security Configuration
    pub rate_limit_requests_per_minute: u32,
    pub max_message_length: usize,

    // BTC Service Configuration
    pub btc_api_base_url: String,
    pub btc_api_key: Option<String>,
}

impl AppConfig {
    pub fn load() -> Result<Self> {
        // Load .env file if it exists
        dotenvy::dotenv().ok();

        let config = AppConfig {
            whatsapp_access_token: env::var("WHATSAPP_ACCESS_TOKEN")
                .context("WHATSAPP_ACCESS_TOKEN must be set")?,
            whatsapp_phone_number_id: env::var("WHATSAPP_PHONE_NUMBER_ID")
                .context("WHATSAPP_PHONE_NUMBER_ID must be set")?,
            whatsapp_webhook_verify_token: env::var("WHATSAPP_WEBHOOK_VERIFY_TOKEN")
                .context("WHATSAPP_WEBHOOK_VERIFY_TOKEN must be set")?,
            whatsapp_api_base_url: env::var("WHATSAPP_API_BASE_URL")
                .unwrap_or_else(|_| "https://graph.facebook.com/v18.0".to_string()),
            whatsapp_media_base_url: env::var("WHATSAPP_MEDIA_BASE_URL")
                .unwrap_or_else(|_| "https://graph.facebook.com/v18.0".to_string()),
            
            // Twilio configuration
            twilio_account_sid: env::var("TWILIO_ACCOUNT_SID")
                .unwrap_or_else(|_| "".to_string()),
            twilio_auth_token: env::var("TWILIO_AUTH_TOKEN")
                .unwrap_or_else(|_| "".to_string()),
            twilio_whatsapp_number: env::var("TWILIO_WHATSAPP_NUMBER")
                .unwrap_or_else(|_| "".to_string()),

            bitsacco_api_base_url: env::var("BITSACCO_API_BASE_URL")
                .unwrap_or_else(|_| "https://api.bitsacco.com".to_string()),
            bitsacco_api_token: env::var("BITSACCO_API_TOKEN")
                .context("BITSACCO_API_TOKEN must be set")?,

            server_host: env::var("SERVER_HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
            server_port: env::var("SERVER_PORT")
                .unwrap_or_else(|_| "8080".to_string())
                .parse()
                .context("Invalid SERVER_PORT")?,
            rust_log: env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string()),

            rate_limit_requests_per_minute: env::var("RATE_LIMIT_REQUESTS_PER_MINUTE")
                .unwrap_or_else(|_| "60".to_string())
                .parse()
                .context("Invalid RATE_LIMIT_REQUESTS_PER_MINUTE")?,
            max_message_length: env::var("MAX_MESSAGE_LENGTH")
                .unwrap_or_else(|_| "4096".to_string())
                .parse()
                .context("Invalid MAX_MESSAGE_LENGTH")?,

            btc_api_base_url: env::var("BTC_API_BASE_URL")
                .unwrap_or_else(|_| "https://api.coingecko.com/api/v3".to_string()),
            btc_api_key: env::var("BTC_API_KEY").ok(),
        };

        // Validate configuration
        config.validate()?;

        Ok(config)
    }

    fn validate(&self) -> Result<()> {
        if self.whatsapp_access_token.is_empty() {
            anyhow::bail!("WhatsApp access token cannot be empty");
        }

        if self.whatsapp_phone_number_id.is_empty() {
            anyhow::bail!("WhatsApp phone number ID cannot be empty");
        }

        if self.whatsapp_webhook_verify_token.is_empty() {
            anyhow::bail!("WhatsApp webhook verify token cannot be empty");
        }

        if self.bitsacco_api_token.is_empty() {
            anyhow::bail!("BitSacco API token cannot be empty");
        }

        if self.rate_limit_requests_per_minute == 0 {
            anyhow::bail!("Rate limit must be greater than 0");
        }

        if self.max_message_length == 0 {
            anyhow::bail!("Max message length must be greater than 0");
        }

        Ok(())
    }
}
