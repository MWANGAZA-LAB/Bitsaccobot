use crate::{
    config::AppConfig,
    error::{AppError, Result},
    types::{WhatsAppSendRequest, WhatsAppSendResponse, WhatsAppTextContent, WhatsAppAudioContent},
};
use reqwest::Client;
use ring::hmac;
use tracing::{error, info, warn};

#[derive(Debug, Clone)]
pub struct WhatsAppService {
    client: Client,
    access_token: String,
    phone_number_id: String,
    webhook_verify_token: String,
    api_base_url: String,
}

impl WhatsAppService {
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
            access_token: config.whatsapp_access_token.clone(),
            phone_number_id: config.whatsapp_phone_number_id.clone(),
            webhook_verify_token: config.whatsapp_webhook_verify_token.clone(),
            api_base_url: config.whatsapp_api_base_url.clone(),
        })
    }

    pub fn verify_webhook(&self, mode: &str, token: &str, challenge: &str) -> Result<String> {
        if mode == "subscribe" && token == self.webhook_verify_token {
            info!("Webhook verification successful");
            Ok(challenge.to_string())
        } else {
            warn!(
                "Webhook verification failed: mode={}, token={}",
                mode, token
            );
            Err(AppError::Unauthorized)
        }
    }

    pub fn verify_webhook_signature(&self, payload: &str, signature: &str) -> Result<()> {
        // WhatsApp uses HMAC-SHA256 for webhook signature verification
        let key = hmac::Key::new(hmac::HMAC_SHA256, self.webhook_verify_token.as_bytes());
        let expected_signature = hmac::sign(&key, payload.as_bytes());
        let expected_hex = hex::encode(expected_signature.as_ref());
        
        // Remove 'sha256=' prefix if present
        let provided_signature = signature.strip_prefix("sha256=").unwrap_or(signature);
        
        // Use constant-time comparison to prevent timing attacks
        if expected_hex.len() == provided_signature.len() {
            let expected_bytes = expected_hex.as_bytes();
            let provided_bytes = provided_signature.as_bytes();
            
            let mut result = 0u8;
            for (a, b) in expected_bytes.iter().zip(provided_bytes.iter()) {
                result |= a ^ b;
            }
            
            if result == 0 {
                info!("Webhook signature verification successful");
                return Ok(());
            }
        }
        
        warn!("Webhook signature verification failed");
        Err(AppError::Unauthorized)
    }

    pub async fn send_message(&self, to: &str, message: &str) -> Result<WhatsAppSendResponse> {
        if message.len() > 4096 {
            return Err(AppError::Validation("Message too long".to_string()));
        }

        let url = format!("{}/{}/messages", self.api_base_url, self.phone_number_id);

        let request = WhatsAppSendRequest {
            messaging_product: "whatsapp".to_string(),
            to: to.to_string(),
            r#type: "text".to_string(),
            text: Some(WhatsAppTextContent {
                body: message.to_string(),
            }),
            audio: None,
        };

        info!("Sending WhatsApp message to: {}", to);

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.access_token))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| AppError::WhatsApp(format!("Failed to send message: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());

            error!("WhatsApp API error: {} - {}", status, error_text);
            return Err(AppError::WhatsApp(format!(
                "API error {}: {}",
                status, error_text
            )));
        }

        let send_response: WhatsAppSendResponse = response
            .json()
            .await
            .map_err(|e| AppError::WhatsApp(format!("Failed to parse response: {}", e)))?;

        info!(
            "Message sent successfully with ID: {:?}",
            send_response.messages
        );
        Ok(send_response)
    }

    pub async fn send_help_message(&self, to: &str) -> Result<()> {
        let help_text = r#"🤖 *BitSacco WhatsApp Bot Help*

*Available Commands:*
• `help` - Show this help message
• `balance` - Check your savings balance
• `savings` - View your savings details
• `chama` - View your chama groups
• `btc` - Get current Bitcoin price
• `deposit <amount> <currency>` - Make a deposit
• `withdraw <amount> <currency>` - Make a withdrawal
• `transfer <amount> <currency> <phone>` - Transfer to another user

*Voice Commands:*
🎤 You can also send voice messages with commands like:
• "Help" - Get help
• "Balance" - Check balance
• "Bitcoin price" - Get BTC price
• "Deposit 100 dollars" - Make a deposit

*Examples:*
• `deposit 100 USD`
• `withdraw 50 KES`
• `transfer 25 USD +254712345678`

*Security Note:*
All transactions are secure and encrypted. Your data is protected by BitSacco's enterprise-grade security.

Need more help? Visit https://bitsacco.com or contact support."#;

        self.send_message(to, help_text).await?;
        Ok(())
    }

    pub async fn send_balance_message(
        &self,
        to: &str,
        savings_balance: f64,
        btc_balance: f64,
        currency: &str,
    ) -> Result<()> {
        let balance_text = format!(
            r#"💰 *Your BitSacco Balance*

*Savings Balance:* {:.2} {}
*Bitcoin Balance:* {:.8} BTC

*Total Value:* {:.2} {} (approx.)

Last updated: {}"#,
            savings_balance,
            currency,
            btc_balance,
            savings_balance + (btc_balance * 50000.0), // Approximate BTC value
            currency,
            chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC")
        );

        self.send_message(to, &balance_text).await?;
        Ok(())
    }

    pub async fn send_error_message(&self, to: &str, error: &str) -> Result<()> {
        let error_text = format!(
            r#"❌ *Error*

{}

Please try again or contact support if the problem persists.

For help, send `help`"#,
            error
        );

        self.send_message(to, &error_text).await?;
        Ok(())
    }

    pub async fn send_success_message(&self, to: &str, message: &str) -> Result<()> {
        let success_text = format!(
            r#"✅ *Success*

{}

Thank you for using BitSacco!"#,
            message
        );

        self.send_message(to, &success_text).await?;
        Ok(())
    }

    pub async fn send_btc_price_message(
        &self,
        to: &str,
        price: f64,
        change_24h: f64,
        currency: &str,
    ) -> Result<()> {
        let change_emoji = if change_24h >= 0.0 { "📈" } else { "📉" };
        let change_sign = if change_24h >= 0.0 { "+" } else { "" };

        let price_text = format!(
            r#"₿ *Bitcoin Price Update*

*Current Price:* {:.2} {}
*24h Change:* {} {}{:.2}%

*Last Updated:* {}

Data provided by BitSacco API"#,
            price,
            currency,
            change_emoji,
            change_sign,
            change_24h,
            chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC")
        );

        self.send_message(to, &price_text).await?;
        Ok(())
    }

    /// Send a voice message (audio file)
    pub async fn send_voice_message(&self, to: &str, audio_file_path: &str) -> Result<()> {
        // First, upload the audio file to WhatsApp
        let media_id = self.upload_media(audio_file_path).await?;
        
        // Then send the voice message
        let request = WhatsAppSendRequest {
            messaging_product: "whatsapp".to_string(),
            to: to.to_string(),
            r#type: "audio".to_string(),
            text: None,
            audio: Some(WhatsAppAudioContent {
                id: media_id.clone(),
            }),
        };

        let url = format!("{}/{}/messages", self.api_base_url, self.phone_number_id);
        
        info!("Sending voice message to {} with media ID: {}", to, media_id.clone());

        let response = self
            .client
            .post(&url)
            .bearer_auth(&self.access_token)
            .json(&request)
            .send()
            .await
            .map_err(|e| AppError::WhatsApp(format!("Failed to send voice message: {}", e)))?;

        if response.status().is_success() {
            let response_data: WhatsAppSendResponse = response
                .json()
                .await
                .map_err(|e| AppError::WhatsApp(format!("Failed to parse response: {}", e)))?;
            
            info!("Voice message sent successfully: {:?}", response_data);
            Ok(())
        } else {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            error!("Failed to send voice message: status={}, body={}", status, body);
            Err(AppError::WhatsApp(format!(
                "Failed to send voice message: HTTP {} - {}",
                status, body
            )))
        }
    }

    /// Upload media file to WhatsApp and return media ID
    async fn upload_media(&self, file_path: &str) -> Result<String> {
        use std::fs;
        use std::path::Path;
        
        let path = Path::new(file_path);
        if !path.exists() {
            return Err(AppError::Validation(format!("File not found: {}", file_path)));
        }

        let file_data = fs::read(file_path)
            .map_err(|e| AppError::Internal(format!("Failed to read file: {}", e)))?;

        let file_name = path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("audio.wav");

        let form = reqwest::multipart::Form::new()
            .part("file", reqwest::multipart::Part::bytes(file_data)
                .file_name(file_name.to_string())
                .mime_str("audio/wav")?)
            .part("type", reqwest::multipart::Part::text("audio/wav"))
            .part("messaging_product", reqwest::multipart::Part::text("whatsapp"));

        let url = format!("{}/{}/media", self.api_base_url, self.phone_number_id);
        
        info!("Uploading media file: {}", file_path);

        let response = self
            .client
            .post(&url)
            .bearer_auth(&self.access_token)
            .multipart(form)
            .send()
            .await
            .map_err(|e| AppError::WhatsApp(format!("Failed to upload media: {}", e)))?;

        if response.status().is_success() {
            let response_data: serde_json::Value = response
                .json()
                .await
                .map_err(|e| AppError::WhatsApp(format!("Failed to parse upload response: {}", e)))?;
            
            let media_id = response_data["id"]
                .as_str()
                .ok_or_else(|| AppError::WhatsApp("Media ID not found in response".to_string()))?;
            
            info!("Media uploaded successfully with ID: {}", media_id);
            Ok(media_id.to_string())
        } else {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            error!("Failed to upload media: status={}, body={}", status, body);
            Err(AppError::WhatsApp(format!(
                "Failed to upload media: HTTP {} - {}",
                status, body
            )))
        }
    }
}
