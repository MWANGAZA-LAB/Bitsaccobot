use axum::{
    extract::{Query, State},
    response::Json,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{error, info};

use crate::{
    error::{AppError, Result},
    types::{
        AppState, BotCommand, HealthResponse, WhatsAppSendResponse,
        WhatsAppWebhook,
    },
};

#[derive(Debug, Deserialize)]
pub struct WebhookQuery {
    pub hub_mode: Option<String>,
    pub hub_challenge: Option<String>,
    pub hub_verify_token: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SendMessageRequest {
    pub to: String,
    pub message: String,
}

pub async fn handle_webhook(
    State(state): State<AppState>,
    Query(query): Query<WebhookQuery>,
    Json(payload): Json<serde_json::Value>,
) -> Result<String> {
    // Handle webhook verification
    if let (Some(mode), Some(challenge), Some(token)) = (
        &query.hub_mode,
        &query.hub_challenge,
        &query.hub_verify_token,
    ) {
        return state
            .whatsapp_service
            .verify_webhook(mode, token, challenge);
    }

    // Handle incoming messages
    info!("Received webhook payload: {}", serde_json::to_string(&payload)?);

    let webhook: WhatsAppWebhook = serde_json::from_value(payload)
        .map_err(|e| AppError::Validation(format!("Invalid webhook payload: {}", e)))?;

    for entry in webhook.entry {
        for change in entry.changes {
            if let Some(messages) = change.value.messages {
                for message in messages {
                    if let Some(text) = message.text {
                        let phone_number = &message.from;
                        let message_text = &text.body;

                        info!(
                            "Processing message from {}: {}",
                            phone_number, message_text
                        );

                        // Process the message asynchronously
                        let state_clone = state.clone();
                        let phone_clone = phone_number.clone();
                        let message_clone = message_text.clone();

                        tokio::spawn(async move {
                            if let Err(e) = process_message(state_clone, phone_clone, message_clone).await {
                                error!("Error processing message: {}", e);
                            }
                        });
                    }
                }
            }
        }
    }

    Ok("OK".to_string())
}

async fn process_message(state: AppState, phone_number: String, message: String) -> Result<()> {
    let command = BotCommand::parse(&message);

    match command {
        BotCommand::Help => {
            state
                .whatsapp_service
                .send_help_message(&phone_number)
                .await?;
        }
        BotCommand::Balance => {
            match get_user_balance(&state, &phone_number).await {
                Ok((savings, btc_balance, currency)) => {
                    state
                        .whatsapp_service
                        .send_balance_message(&phone_number, savings, btc_balance, &currency)
                        .await?;
                }
                Err(e) => {
                    state
                        .whatsapp_service
                        .send_error_message(&phone_number, &e.to_string())
                        .await?;
                }
            }
        }
        BotCommand::Savings => {
            match get_user_savings(&state, &phone_number).await {
                Ok(savings) => {
                    let message = format!(
                        "💰 *Your Savings*\n\nTotal: {:.2} KES\n\nDetails:\n{}",
                        savings.iter().map(|s| s.amount).sum::<f64>(),
                        savings
                            .iter()
                            .map(|s| format!("• {:.2} {} ({})", s.amount, s.currency, s.id))
                            .collect::<Vec<_>>()
                            .join("\n")
                    );
                    state
                        .whatsapp_service
                        .send_message(&phone_number, &message)
                        .await?;
                }
                Err(e) => {
                    state
                        .whatsapp_service
                        .send_error_message(&phone_number, &e.to_string())
                        .await?;
                }
            }
        }
        BotCommand::Chama => {
            match get_user_chamas(&state, &phone_number).await {
                Ok(chamas) => {
                    if chamas.is_empty() {
                        state
                            .whatsapp_service
                            .send_message(&phone_number, "You are not part of any chama groups yet.")
                            .await?;
                    } else {
                        let message = format!(
                            "👥 *Your Chama Groups*\n\n{}",
                            chamas
                                .iter()
                                .map(|c| format!(
                                    "• {} - {:.2} {} ({} members)",
                                    c.name,
                                    c.total_savings,
                                    c.currency,
                                    c.members.len()
                                ))
                                .collect::<Vec<_>>()
                                .join("\n")
                        );
                        state
                            .whatsapp_service
                            .send_message(&phone_number, &message)
                            .await?;
                    }
                }
                Err(e) => {
                    state
                        .whatsapp_service
                        .send_error_message(&phone_number, &e.to_string())
                        .await?;
                }
            }
        }
        BotCommand::BtcPrice => {
            match state.btc_service.get_btc_price_usd().await {
                Ok(price) => {
                    state
                        .whatsapp_service
                        .send_btc_price_message(&phone_number, price.price, price.change_24h, &price.currency)
                        .await?;
                }
                Err(e) => {
                    state
                        .whatsapp_service
                        .send_error_message(&phone_number, &e.to_string())
                        .await?;
                }
            }
        }
        BotCommand::Deposit { amount, currency } => {
            match create_deposit(&state, &phone_number, amount, &currency).await {
                Ok(transaction) => {
                    let message = format!(
                        "Deposit of {:.2} {} created successfully. Transaction ID: {}",
                        amount, currency, transaction.id
                    );
                    state
                        .whatsapp_service
                        .send_success_message(&phone_number, &message)
                        .await?;
                }
                Err(e) => {
                    state
                        .whatsapp_service
                        .send_error_message(&phone_number, &e.to_string())
                        .await?;
                }
            }
        }
        BotCommand::Withdraw { amount, currency } => {
            match create_withdrawal(&state, &phone_number, amount, &currency).await {
                Ok(transaction) => {
                    let message = format!(
                        "Withdrawal of {:.2} {} created successfully. Transaction ID: {}",
                        amount, currency, transaction.id
                    );
                    state
                        .whatsapp_service
                        .send_success_message(&phone_number, &message)
                        .await?;
                }
                Err(e) => {
                    state
                        .whatsapp_service
                        .send_error_message(&phone_number, &e.to_string())
                        .await?;
                }
            }
        }
        BotCommand::Transfer {
            amount,
            currency,
            recipient,
        } => {
            match create_transfer(&state, &phone_number, amount, &currency, &recipient).await {
                Ok(transaction) => {
                    let message = format!(
                        "Transfer of {:.2} {} to {} created successfully. Transaction ID: {}",
                        amount, currency, recipient, transaction.id
                    );
                    state
                        .whatsapp_service
                        .send_success_message(&phone_number, &message)
                        .await?;
                }
                Err(e) => {
                    state
                        .whatsapp_service
                        .send_error_message(&phone_number, &e.to_string())
                        .await?;
                }
            }
        }
        BotCommand::Unknown(message) => {
            let response = format!(
                "I didn't understand: \"{}\"\n\nSend `help` to see available commands.",
                message
            );
            state
                .whatsapp_service
                .send_message(&phone_number, &response)
                .await?;
        }
    }

    Ok(())
}

async fn get_user_balance(
    state: &AppState,
    phone_number: &str,
) -> Result<(f64, f64, String)> {
    let user = state
        .bitsacco_service
        .get_user_by_phone(phone_number)
        .await?;

    let savings = state
        .bitsacco_service
        .get_total_savings(&user.id)
        .await?;

    let btc_balance = state
        .bitsacco_service
        .get_user_btc_balance(&user.id)
        .await?;

    Ok((savings, btc_balance.balance, btc_balance.currency))
}

async fn get_user_savings(
    state: &AppState,
    phone_number: &str,
) -> Result<Vec<crate::types::BitSaccoSavings>> {
    let user = state
        .bitsacco_service
        .get_user_by_phone(phone_number)
        .await?;

    state
        .bitsacco_service
        .get_user_savings(&user.id)
        .await
}

async fn get_user_chamas(
    state: &AppState,
    phone_number: &str,
) -> Result<Vec<crate::types::BitSaccoChama>> {
    let user = state
        .bitsacco_service
        .get_user_by_phone(phone_number)
        .await?;

    state
        .bitsacco_service
        .get_user_chamas(&user.id)
        .await
}

async fn create_deposit(
    state: &AppState,
    phone_number: &str,
    amount: f64,
    currency: &str,
) -> Result<crate::types::BitSaccoTransaction> {
    let user = state
        .bitsacco_service
        .get_user_by_phone(phone_number)
        .await?;

    state
        .bitsacco_service
        .create_deposit(&user.id, amount, currency)
        .await
}

async fn create_withdrawal(
    state: &AppState,
    phone_number: &str,
    amount: f64,
    currency: &str,
) -> Result<crate::types::BitSaccoTransaction> {
    let user = state
        .bitsacco_service
        .get_user_by_phone(phone_number)
        .await?;

    state
        .bitsacco_service
        .create_withdrawal(&user.id, amount, currency)
        .await
}

async fn create_transfer(
    state: &AppState,
    phone_number: &str,
    amount: f64,
    currency: &str,
    recipient: &str,
) -> Result<crate::types::BitSaccoTransaction> {
    let user = state
        .bitsacco_service
        .get_user_by_phone(phone_number)
        .await?;

    state
        .bitsacco_service
        .create_transfer(&user.id, amount, currency, recipient)
        .await
}

#[axum::debug_handler]
pub async fn send_message(
    State(state): State<AppState>,
    Json(request): Json<SendMessageRequest>,
) -> Result<Json<WhatsAppSendResponse>> {
    let response = state
        .whatsapp_service
        .send_message(&request.to, &request.message)
        .await?;

    Ok(Json(response))
}

pub async fn health_check(State(state): State<AppState>) -> Result<Json<HealthResponse>> {
    let mut services = HashMap::new();

    // Check WhatsApp service
    match state.whatsapp_service.send_message("test", "health check").await {
        Ok(_) => services.insert("whatsapp".to_string(), "healthy".to_string()),
        Err(_) => services.insert("whatsapp".to_string(), "unhealthy".to_string()),
    };

    // Check BitSacco service
    match state.bitsacco_service.health_check().await {
        Ok(_) => services.insert("bitsacco".to_string(), "healthy".to_string()),
        Err(_) => services.insert("bitsacco".to_string(), "unhealthy".to_string()),
    };

    // Check BTC service
    match state.btc_service.health_check().await {
        Ok(_) => services.insert("btc".to_string(), "healthy".to_string()),
        Err(_) => services.insert("btc".to_string(), "unhealthy".to_string()),
    };

    let response = HealthResponse {
        status: "ok".to_string(),
        timestamp: chrono::Utc::now().to_rfc3339(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        services,
    };

    Ok(Json(response))
}
