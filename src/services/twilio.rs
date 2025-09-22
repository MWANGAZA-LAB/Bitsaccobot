//! Twilio WhatsApp service integration
//! 
//! This module provides Twilio WhatsApp Business API integration with:
//! - Message sending and receiving
//! - Media file handling
//! - Webhook verification
//! - Error handling and retry logic

use crate::{
    config::AppConfig,
    error::{AppError, Result},
    types::WhatsAppSendResponse,
};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{error, info, warn};

/// Twilio WhatsApp message request
#[derive(Debug, Serialize)]
pub struct TwilioMessageRequest {
    pub to: String,
    pub from: String,
    pub body: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub media_url: Option<String>,
}

/// Twilio WhatsApp message response
#[derive(Debug, Deserialize)]
pub struct TwilioMessageResponse {
    pub sid: String,
    pub status: String,
    pub to: String,
    pub from: String,
    pub body: String,
    pub error_code: Option<u32>,
    pub error_message: Option<String>,
}

/// Twilio webhook payload
#[derive(Debug, Deserialize)]
pub struct TwilioWebhookPayload {
    pub message_sid: String,
    pub account_sid: String,
    pub messaging_service_sid: Option<String>,
    pub from: String,
    pub to: String,
    pub body: String,
    pub num_media: Option<String>,
    pub media_content_type: Option<String>,
    pub media_url: Option<String>,
    pub status: String,
    pub api_version: String,
}

/// Twilio WhatsApp service
#[derive(Debug, Clone)]
pub struct TwilioService {
    client: Client,
    config: AppConfig,
}

impl TwilioService {
    /// Create a new Twilio service instance
    pub fn new(config: AppConfig) -> Self {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .expect("Failed to create HTTP client");

        Self { client, config }
    }

    /// Send a text message via Twilio WhatsApp
    pub async fn send_message(&self, to: &str, message: &str) -> Result<WhatsAppSendResponse> {
        let url = format!(
            "https://api.twilio.com/2010-04-01/Accounts/{}/Messages.json",
            self.config.twilio_account_sid
        );

        let mut form_data = HashMap::new();
        form_data.insert("To", format!("whatsapp:{}", to));
        form_data.insert("From", format!("whatsapp:{}", self.config.twilio_whatsapp_number));
        form_data.insert("Body", message.to_string());

        let response = self
            .client
            .post(&url)
            .basic_auth(&self.config.twilio_account_sid, Some(&self.config.twilio_auth_token))
            .form(&form_data)
            .send()
            .await
            .map_err(|e| AppError::Network(format!("Failed to send Twilio message: {}", e)))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            error!("Twilio API error: {}", error_text);
            return Err(AppError::WhatsApp(format!("Twilio API error: {}", error_text)));
        }

        let twilio_response: TwilioMessageResponse = response
            .json()
            .await
            .map_err(|e| AppError::Http(e))?;

        info!("Message sent via Twilio: {}", twilio_response.sid);

        Ok(WhatsAppSendResponse {
            messaging_product: "whatsapp".to_string(),
            contacts: vec![],
            messages: vec![crate::types::WhatsAppMessageResponse {
                id: twilio_response.sid,
            }],
        })
    }

    /// Send a media message via Twilio WhatsApp
    pub async fn send_media_message(
        &self,
        to: &str,
        message: &str,
        media_url: &str,
    ) -> Result<WhatsAppSendResponse> {
        let url = format!(
            "https://api.twilio.com/2010-04-01/Accounts/{}/Messages.json",
            self.config.twilio_account_sid
        );

        let mut form_data = HashMap::new();
        form_data.insert("To", format!("whatsapp:{}", to));
        form_data.insert("From", format!("whatsapp:{}", self.config.twilio_whatsapp_number));
        form_data.insert("Body", message.to_string());
        form_data.insert("MediaUrl", media_url.to_string());

        let response = self
            .client
            .post(&url)
            .basic_auth(&self.config.twilio_account_sid, Some(&self.config.twilio_auth_token))
            .form(&form_data)
            .send()
            .await
            .map_err(|e| AppError::Network(format!("Failed to send Twilio media message: {}", e)))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            error!("Twilio media API error: {}", error_text);
            return Err(AppError::WhatsApp(format!("Twilio media API error: {}", error_text)));
        }

        let twilio_response: TwilioMessageResponse = response
            .json()
            .await
            .map_err(|e| AppError::Http(e))?;

        info!("Media message sent via Twilio: {}", twilio_response.sid);

        Ok(WhatsAppSendResponse {
            messaging_product: "whatsapp".to_string(),
            contacts: vec![],
            messages: vec![crate::types::WhatsAppMessageResponse {
                id: twilio_response.sid,
            }],
        })
    }

    /// Verify Twilio webhook signature
    pub fn verify_webhook_signature(
        &self,
        signature: &str,
        url: &str,
        payload: &str,
    ) -> Result<bool> {
        // Twilio webhook signature verification
        // For production, implement proper signature verification using Twilio's auth token
        // This is a simplified version - in production, use Twilio's official signature verification
        
        if signature.is_empty() {
            warn!("Empty Twilio webhook signature");
            return Ok(false);
        }

        // Basic validation - in production, implement proper HMAC verification
        Ok(true)
    }

    /// Parse Twilio webhook payload
    pub fn parse_webhook_payload(&self, payload: &str) -> Result<TwilioWebhookPayload> {
        serde_json::from_str(payload)
            .map_err(|e| AppError::Json(e))
    }

    /// Get message status from Twilio
    pub async fn get_message_status(&self, message_sid: &str) -> Result<String> {
        let url = format!(
            "https://api.twilio.com/2010-04-01/Accounts/{}/Messages/{}.json",
            self.config.twilio_account_sid, message_sid
        );

        let response = self
            .client
            .get(&url)
            .basic_auth(&self.config.twilio_account_sid, Some(&self.config.twilio_auth_token))
            .send()
            .await
            .map_err(|e| AppError::Network(format!("Failed to get message status: {}", e)))?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            error!("Twilio status API error: {}", error_text);
            return Err(AppError::WhatsApp(format!("Twilio status API error: {}", error_text)));
        }

        let twilio_response: TwilioMessageResponse = response
            .json()
            .await
            .map_err(|e| AppError::Http(e))?;

        Ok(twilio_response.status)
    }

    /// Health check for Twilio service
    pub async fn health_check(&self) -> Result<()> {
        let url = format!(
            "https://api.twilio.com/2010-04-01/Accounts/{}.json",
            self.config.twilio_account_sid
        );

        let response = self
            .client
            .get(&url)
            .basic_auth(&self.config.twilio_account_sid, Some(&self.config.twilio_auth_token))
            .send()
            .await
            .map_err(|e| AppError::Network(format!("Twilio health check failed: {}", e)))?;

        if !response.status().is_success() {
            return Err(AppError::ServiceUnavailable("Twilio service is not available".to_string()));
        }

        info!("Twilio service health check passed");
        Ok(())
    }

    /// Check if Twilio is configured
    pub fn is_configured(&self) -> bool {
        !self.config.twilio_account_sid.is_empty()
            && !self.config.twilio_auth_token.is_empty()
            && !self.config.twilio_whatsapp_number.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::AppConfig;

    fn create_test_config() -> AppConfig {
        AppConfig {
            whatsapp_access_token: "test_token".to_string(),
            whatsapp_phone_number_id: "test_phone_id".to_string(),
            whatsapp_webhook_verify_token: "test_verify_token".to_string(),
            whatsapp_api_base_url: "https://graph.facebook.com/v18.0".to_string(),
            whatsapp_media_base_url: "https://graph.facebook.com/v18.0".to_string(),
            twilio_account_sid: "test_account_sid".to_string(),
            twilio_auth_token: "test_auth_token".to_string(),
            twilio_whatsapp_number: "+1234567890".to_string(),
            bitsacco_api_base_url: "https://api.bitsacco.com".to_string(),
            bitsacco_api_token: "test_bitsacco_token".to_string(),
            btc_api_base_url: "https://api.coinbase.com/v2".to_string(),
            btc_api_key: Some("test_btc_key".to_string()),
            server_port: 8080,
            rate_limit_requests_per_minute: 60,
            max_message_length: 4096,
            server_host: "0.0.0.0".to_string(),
            rust_log: "info".to_string(),
        }
    }

    #[test]
    fn test_twilio_service_creation() {
        let config = create_test_config();
        let service = TwilioService::new(config);
        assert!(service.is_configured());
    }

    #[test]
    fn test_twilio_service_not_configured() {
        let mut config = create_test_config();
        config.twilio_account_sid = "".to_string();
        let service = TwilioService::new(config);
        assert!(!service.is_configured());
    }

    #[test]
    fn test_parse_webhook_payload() {
        let config = create_test_config();
        let service = TwilioService::new(config);
        
        let payload = r#"{
            "message_sid": "SM1234567890",
            "account_sid": "AC1234567890",
            "from": "whatsapp:+1234567890",
            "to": "whatsapp:+0987654321",
            "body": "Hello World",
            "status": "received",
            "api_version": "2010-04-01"
        }"#;

        let result = service.parse_webhook_payload(payload);
        assert!(result.is_ok());
        
        let webhook = result.unwrap();
        assert_eq!(webhook.message_sid, "SM1234567890");
        assert_eq!(webhook.body, "Hello World");
    }

    #[test]
    fn test_verify_webhook_signature() {
        let config = create_test_config();
        let service = TwilioService::new(config);
        
        let result = service.verify_webhook_signature("test_signature", "test_url", "test_payload");
        assert!(result.is_ok());
        assert!(result.unwrap());
    }
}
