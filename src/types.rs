use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::{
    config::AppConfig,
    services::{bitsacco::BitSaccoService, btc::BtcService, whatsapp::WhatsAppService},
};

#[derive(Debug, Clone)]
pub struct AppState {
    pub config: AppConfig,
    pub whatsapp_service: WhatsAppService,
    pub bitsacco_service: BitSaccoService,
    pub btc_service: BtcService,
}

// WhatsApp API Types
#[derive(Debug, Deserialize, Serialize)]
pub struct WhatsAppWebhook {
    pub object: String,
    pub entry: Vec<WhatsAppEntry>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct WhatsAppEntry {
    pub id: String,
    pub changes: Vec<WhatsAppChange>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct WhatsAppChange {
    pub value: WhatsAppValue,
    pub field: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct WhatsAppValue {
    pub messaging_product: String,
    pub metadata: WhatsAppMetadata,
    pub contacts: Option<Vec<WhatsAppContact>>,
    pub messages: Option<Vec<WhatsAppMessage>>,
    pub statuses: Option<Vec<WhatsAppStatus>>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct WhatsAppMetadata {
    pub display_phone_number: String,
    pub phone_number_id: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct WhatsAppContact {
    pub profile: WhatsAppProfile,
    pub wa_id: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct WhatsAppProfile {
    pub name: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct WhatsAppMessage {
    pub from: String,
    pub id: String,
    pub timestamp: String,
    pub text: Option<WhatsAppText>,
    pub context: Option<WhatsAppContext>,
    pub r#type: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct WhatsAppText {
    pub body: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct WhatsAppContext {
    pub from: String,
    pub id: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct WhatsAppStatus {
    pub id: String,
    pub status: String,
    pub timestamp: String,
    pub recipient_id: String,
}

// WhatsApp Send Message Types
#[derive(Debug, Serialize)]
pub struct WhatsAppSendRequest {
    pub messaging_product: String,
    pub to: String,
    pub r#type: String,
    pub text: WhatsAppTextContent,
}

#[derive(Debug, Serialize)]
pub struct WhatsAppTextContent {
    pub body: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct WhatsAppSendResponse {
    pub messaging_product: String,
    pub contacts: Vec<WhatsAppContactResponse>,
    pub messages: Vec<WhatsAppMessageResponse>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct WhatsAppContactResponse {
    pub input: String,
    pub wa_id: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct WhatsAppMessageResponse {
    pub id: String,
}

// BitSacco API Types
#[derive(Debug, Deserialize, Serialize)]
pub struct BitSaccoUser {
    pub id: String,
    pub phone_number: String,
    pub name: Option<String>,
    pub email: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct BitSaccoSavings {
    pub id: String,
    pub user_id: String,
    pub amount: f64,
    pub currency: String,
    pub chama_id: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct BitSaccoChama {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub members: Vec<BitSaccoUser>,
    pub total_savings: f64,
    pub currency: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct BitSaccoBtcBalance {
    pub user_id: String,
    pub balance: f64,
    pub currency: String,
    pub last_updated: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct BitSaccoTransaction {
    pub id: String,
    pub user_id: String,
    pub r#type: String, // "deposit", "withdrawal", "transfer"
    pub amount: f64,
    pub currency: String,
    pub status: String, // "pending", "completed", "failed"
    pub created_at: String,
    pub updated_at: String,
}

// BTC Service Types
#[derive(Debug, Deserialize, Serialize)]
pub struct BtcPrice {
    pub currency: String,
    pub price: f64,
    pub change_24h: f64,
    pub last_updated: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct BtcMarketData {
    pub current_price: HashMap<String, f64>,
    pub price_change_percentage_24h: f64,
    pub last_updated: String,
}

// Bot Command Types
#[derive(Debug, Clone, PartialEq)]
pub enum BotCommand {
    Help,
    Balance,
    Savings,
    Chama,
    BtcPrice,
    Deposit { amount: f64, currency: String },
    Withdraw { amount: f64, currency: String },
    Transfer { amount: f64, currency: String, recipient: String },
    Unknown(String),
}

impl BotCommand {
    pub fn parse(message: &str) -> Self {
        let message = message.trim().to_lowercase();
        
        if message == "help" || message == "/help" {
            BotCommand::Help
        } else if message == "balance" || message == "/balance" {
            BotCommand::Balance
        } else if message == "savings" || message == "/savings" {
            BotCommand::Savings
        } else if message == "chama" || message == "/chama" {
            BotCommand::Chama
        } else if message == "btc" || message == "bitcoin" || message == "/btc" {
            BotCommand::BtcPrice
        } else if message.starts_with("deposit ") {
            // Parse deposit command: "deposit 100 USD"
            let parts: Vec<&str> = message.split_whitespace().collect();
            if parts.len() >= 3 {
                if let Ok(amount) = parts[1].parse::<f64>() {
                    return BotCommand::Deposit {
                        amount,
                        currency: parts[2].to_uppercase(),
                    };
                }
            }
            BotCommand::Unknown(message)
        } else if message.starts_with("withdraw ") {
            // Parse withdraw command: "withdraw 50 USD"
            let parts: Vec<&str> = message.split_whitespace().collect();
            if parts.len() >= 3 {
                if let Ok(amount) = parts[1].parse::<f64>() {
                    return BotCommand::Withdraw {
                        amount,
                        currency: parts[2].to_uppercase(),
                    };
                }
            }
            BotCommand::Unknown(message)
        } else if message.starts_with("transfer ") {
            // Parse transfer command: "transfer 25 USD +254712345678"
            let parts: Vec<&str> = message.split_whitespace().collect();
            if parts.len() >= 4 {
                if let Ok(amount) = parts[1].parse::<f64>() {
                    return BotCommand::Transfer {
                        amount,
                        currency: parts[2].to_uppercase(),
                        recipient: parts[3].to_string(),
                    };
                }
            }
            BotCommand::Unknown(message)
        } else {
            BotCommand::Unknown(message)
        }
    }
}

// Health Check Response
#[derive(Debug, Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub timestamp: String,
    pub version: String,
    pub services: HashMap<String, String>,
}
