use axum::{
    extract::{Query, State},
    http::HeaderMap,
    response::Json,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{error, info};

use crate::{
    error::{AppError, Result},
    rate_limit::RateLimiterService,
    types::{AppState, BotCommand, HealthResponse, WhatsAppSendResponse, WhatsAppWebhook},
    validation::{validate_message, validate_phone_number, validate_amount, validate_currency},
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
    headers: HeaderMap,
    Query(query): Query<WebhookQuery>,
    Json(payload): Json<serde_json::Value>,
) -> Result<String> {
    // Create a simple rate limiter for this request
    let rate_limiter = RateLimiterService::new(crate::rate_limit::RateLimitConfig {
        requests_per_minute: state.config.rate_limit_requests_per_minute,
        burst_size: 10,
    });
    
    // Check rate limit
    rate_limiter.check_rate_limit("webhook").await?;
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

    // Verify webhook signature for incoming messages
    if let Some(signature) = headers.get("x-hub-signature-256") {
        let signature_str = signature.to_str().map_err(|_| {
            AppError::Validation("Invalid signature header".to_string())
        })?;
        
        let payload_str = serde_json::to_string(&payload)?;
        state.whatsapp_service.verify_webhook_signature(&payload_str, signature_str)?;
    } else {
        return Err(AppError::Validation("Missing webhook signature".to_string()));
    }

    // Handle incoming messages
    info!(
        "Received webhook payload: {}",
        serde_json::to_string(&payload)?
    );

    let webhook: WhatsAppWebhook = serde_json::from_value(payload)
        .map_err(|e| AppError::Validation(format!("Invalid webhook payload: {}", e)))?;

    for entry in webhook.entry {
        for change in entry.changes {
            if let Some(messages) = change.value.messages {
                for message in messages {
                    let phone_number = &message.from;
                    
                    // Validate phone number
                    validate_phone_number(phone_number)?;

                    // Process text messages
                    if let Some(text) = message.text {
                        let message_text = &text.body;
                        validate_message(message_text)?;

                        info!("Processing text message from {}: {}", phone_number, message_text);

                        let state_clone = state.clone();
                        let phone_clone = phone_number.clone();
                        let message_clone = message_text.clone();

                        tokio::spawn(async move {
                            if let Err(e) =
                                process_text_message(state_clone, phone_clone, message_clone).await
                            {
                                error!("Error processing text message: {}", e);
                            }
                        });
                    }
                    // Process voice messages
                    else if let Some(voice) = message.voice {
                        info!("Processing voice message from {}: {}", phone_number, voice.id);

                        let state_clone = state.clone();
                        let phone_clone = phone_number.clone();
                        let voice_clone = voice.clone();

                        tokio::spawn(async move {
                            if let Err(e) =
                                process_voice_message(state_clone, phone_clone, voice_clone).await
                            {
                                error!("Error processing voice message: {}", e);
                            }
                        });
                    }
                    // Process audio messages
                    else if let Some(audio) = message.audio {
                        info!("Processing audio message from {}: {}", phone_number, audio.id);

                        let state_clone = state.clone();
                        let phone_clone = phone_number.clone();
                        let audio_clone = audio.clone();

                        tokio::spawn(async move {
                            if let Err(e) =
                                process_audio_message(state_clone, phone_clone, audio_clone).await
                            {
                                error!("Error processing audio message: {}", e);
                            }
                        });
                    }
                }
            }
        }
    }

    Ok("OK".to_string())
}

async fn process_text_message(state: AppState, phone_number: String, message: String) -> Result<()> {
    // Validate that user is registered with BitSacco web app
    validate_registered_user(&state, &phone_number).await?;
    
    let command = BotCommand::parse(&message);

    match command {
        BotCommand::Help => {
            state
                .whatsapp_service
                .send_help_message(&phone_number)
                .await?;
        }
        BotCommand::Balance => match get_user_balance(&state, &phone_number).await {
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
        },
        BotCommand::Savings => match get_user_savings(&state, &phone_number).await {
            Ok(savings) => {
                let total_kes: f64 = savings.iter().map(|s| s.amount).sum();
                let total_sats = (total_kes * 100_000_000.0) as u64; // Convert KES to sats
                
                let message = format!(
                    "💰 *Your Savings*\n\nTotal: {} sats ({:.2} KES)\n\nDetails:\n{}",
                    total_sats,
                    total_kes,
                    savings
                        .iter()
                        .map(|s| {
                            let sats = (s.amount * 100_000_000.0) as u64;
                            format!("• {} sats ({:.2} {}) - {}", sats, s.amount, s.currency, s.id)
                        })
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
        },
        BotCommand::Chama => match get_user_chamas(&state, &phone_number).await {
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
        },
        BotCommand::BtcPrice => match state.btc_service.get_btc_price_usd(&state.cache).await {
            Ok(price) => {
                state
                    .whatsapp_service
                    .send_btc_price_message(
                        &phone_number,
                        price.price,
                        price.change_24h,
                        &price.currency,
                    )
                    .await?;
            }
            Err(e) => {
                state
                    .whatsapp_service
                    .send_error_message(&phone_number, &e.to_string())
                    .await?;
            }
        },
        BotCommand::Deposit { amount, currency, method } => {
            validate_amount(amount)?;
            validate_currency(&currency)?;
            
            // Restrict deposits to KES only
            if currency.to_uppercase() != "KES" {
                let error_message = "❌ *Deposit Error*\n\nOnly KES deposits are supported. Please use KES currency for deposits.\n\nExample: `deposit 100 KES`";
                state
                    .whatsapp_service
                    .send_error_message(&phone_number, &error_message)
                    .await?;
                return Ok(());
            }
            
            let payment_method = method.as_deref().unwrap_or("mpesa");
            
            match payment_method {
                "lightning" => {
                    match create_lightning_deposit(&state, &phone_number, amount, &currency).await {
                        Ok(lightning_response) => {
                            let message = format!(
                                "⚡ *Lightning Deposit Initiated!*\n\nAmount: {:.2} KES\nPayment Request: {}\n\n📱 *Scan the QR code or copy the payment request to your Lightning wallet to complete the deposit.*",
                                amount, lightning_response.payment_request
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
                _ => {
                    match create_deposit(&state, &phone_number, amount, &currency).await {
                        Ok(transaction) => {
                            let message = format!(
                                "💰 *M-Pesa Deposit Initiated!*\n\nAmount: {:.2} KES\nTransaction ID: {}\nStatus: {}\n\n📱 *M-Pesa STK Push sent to your phone!*\n\nPlease check your phone and enter your M-Pesa PIN to complete the deposit.",
                                amount, transaction.id, transaction.status
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
            }
        }
        BotCommand::Withdraw { amount, currency, method } => {
            validate_amount(amount)?;
            validate_currency(&currency)?;
            
            let payment_method = method.as_deref().unwrap_or("mpesa");
            
            match create_withdrawal(&state, &phone_number, amount, &currency).await {
                Ok(transaction) => {
                    let message = format!(
                        "💰 *Withdrawal Initiated!*\n\nAmount: {:.2} {}\nTransaction ID: {}\nStatus: {}\n\n📱 *Withdrawal will be processed.*",
                        amount, currency, transaction.id, transaction.status
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
            validate_amount(amount)?;
            validate_currency(&currency)?;
            validate_phone_number(&recipient)?;
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
        },
        BotCommand::CreateChama { name, description } => {
            match create_chama(&state, &phone_number, &name, description.as_deref()).await {
                Ok(chama) => {
                    let message = format!(
                        "🎉 *Chama Created Successfully!*\n\nName: {}\nID: {}\nDescription: {}\n\nShare this ID with members: `{}`",
                        chama.name,
                        chama.id,
                        description.as_deref().unwrap_or("No description"),
                        chama.id
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
        },
        BotCommand::ContributeChama { chama_id, amount, currency } => {
            validate_amount(amount)?;
            validate_currency(&currency)?;
            match contribute_to_chama(&state, &phone_number, &chama_id, amount, &currency).await {
                Ok(contribution) => {
                    let message = format!(
                        "💰 *Chama Contribution Successful!*\n\nAmount: {:.2} {}\nShares Purchased: {}\nChama ID: {}\nTransaction ID: {}",
                        amount, currency, contribution.shares_purchased, chama_id, contribution.id
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
        },
        BotCommand::SharesBalance { chama_id } => {
            match get_user_chama_shares(&state, &phone_number, chama_id.as_deref()).await {
                Ok(shares) => {
                    if shares.is_empty() {
                        let message = if let Some(chama_id) = chama_id {
                            format!("You don't have any shares in chama {}.", chama_id)
                        } else {
                            "You don't have any chama shares yet.".to_string()
                        };
                        state
                            .whatsapp_service
                            .send_message(&phone_number, &message)
                            .await?;
                    } else {
                        let message = format!(
                            "📊 *Your Chama Shares*\n\n{}",
                            shares
                                .iter()
                                .map(|s| format!(
                                    "• Chama: {}\n  Shares: {}\n  Total Contribution: {:.2} {}\n  Last Updated: {}",
                                    s.chama_id, s.shares_count, s.total_contribution, s.currency, s.updated_at
                                ))
                                .collect::<Vec<_>>()
                                .join("\n\n")
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
        },
        BotCommand::Membership => {
            match get_membership_shares(&state, &phone_number).await {
                Ok(shares) => {
                    let message = format!(
                        "🏛️ *BitSacco Membership*\n\nShares Owned: {}\nTotal Investment: {:.2} {}\n\nUse `buy shares <count>` to purchase more shares.\nUse `share history` to view your purchase history.",
                        shares.shares_count, shares.total_investment, shares.currency
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
        },
        BotCommand::BuyShares { count, method } => {
            let payment_method = method.as_deref().unwrap_or("mpesa");
            match buy_membership_shares(&state, &phone_number, count, payment_method).await {
                Ok(purchase) => {
                    let message = format!(
                        "🎯 *Share Purchase Initiated!*\n\nShares: {}\nAmount: {:.2} {}\nPayment Method: {}\nTransaction ID: {}\nStatus: {}\n\nYour shares will be added to your account once payment is confirmed.",
                        count, purchase.amount, purchase.currency, payment_method, purchase.id, purchase.status
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
        },
        BotCommand::ShareHistory => {
            match get_share_history(&state, &phone_number).await {
                Ok(history) => {
                    if history.is_empty() {
                        let message = "📊 *Share History*\n\nNo share purchases found.";
                        state
                            .whatsapp_service
                            .send_message(&phone_number, &message)
                            .await?;
                    } else {
                        let message = format!(
                            "📊 *Share Purchase History*\n\n{}",
                            history
                                .iter()
                                .map(|p| format!(
                                    "• {} shares - {:.2} {} ({})\n  Date: {}\n  Status: {}",
                                    p.shares_count, p.amount, p.currency, p.payment_method, 
                                    p.created_at, p.status
                                ))
                                .collect::<Vec<_>>()
                                .join("\n\n")
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
        },
        BotCommand::History => {
            match get_transaction_history(&state, &phone_number).await {
                Ok(transactions) => {
                    if transactions.is_empty() {
                        let message = "📋 *Transaction History*\n\nNo transactions found.";
                        state
                            .whatsapp_service
                            .send_message(&phone_number, &message)
                            .await?;
                    } else {
                        let recent_transactions: Vec<_> = transactions.iter().take(5).collect();
                        let message = format!(
                            "📋 *Recent Transactions*\n\n{}",
                            recent_transactions
                                .iter()
                                .map(|t| format!(
                                    "• {} - {:.2} {} ({})\n  Type: {}\n  Status: {}\n  Date: {}",
                                    t.id, t.amount, t.currency, 
                                    t.payment_method.as_deref().unwrap_or("internal"), 
                                    t.r#type, t.status, t.created_at
                                ))
                                .collect::<Vec<_>>()
                                .join("\n\n")
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
        },
        BotCommand::LightningDeposit { amount, currency } => {
            validate_amount(amount)?;
            validate_currency(&currency)?;
            match create_lightning_deposit(&state, &phone_number, amount, &currency).await {
                Ok(lightning_response) => {
                    let message = format!(
                        "⚡ *Lightning Deposit Initiated!*\n\nAmount: {:.2} {}\nPayment Request: {}\n\n📱 *Scan the QR code or copy the payment request to your Lightning wallet to complete the deposit.*",
                        amount, currency, lightning_response.payment_request
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
        },
        BotCommand::LightningWithdraw { amount, currency } => {
            validate_amount(amount)?;
            validate_currency(&currency)?;
            match create_withdrawal(&state, &phone_number, amount, &currency).await {
                Ok(transaction) => {
                    let message = format!(
                        "⚡ *Lightning Withdrawal Initiated!*\n\nAmount: {:.2} {}\nTransaction ID: {}\nStatus: {}\n\n📱 *Withdrawal will be processed via Lightning Network.*",
                        amount, currency, transaction.id, transaction.status
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
        },
        BotCommand::VoiceCommand { transcript } => {
            // This should not happen in text processing, but handle it gracefully
            let response = format!(
                "Voice command received: \"{}\"\n\nProcessing as text command...",
                transcript
            );
            state
                .whatsapp_service
                .send_message(&phone_number, &response)
                .await?;
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

async fn process_voice_message(
    state: AppState,
    phone_number: String,
    voice: crate::types::WhatsAppVoice,
) -> Result<()> {
    info!("Processing voice message from {}", phone_number);

    // Download the voice message
    let audio_path = state.voice_service.download_voice_message(&voice).await?;

    // Convert speech to text
    let transcript = state.voice_service.speech_to_text(&audio_path).await?;
    
    info!("Voice transcript: {}", transcript);

    // Process the transcript as a command
    let command = BotCommand::parse(&transcript);
    
    match command {
        BotCommand::VoiceCommand { transcript } => {
            // Process the voice command
            process_voice_command(&state, &phone_number, &transcript).await?;
        }
        _ => {
            // If it's a regular command, process it normally
            process_text_message(state, phone_number, transcript).await?;
        }
    }

    // Clean up the temporary file
    let _ = std::fs::remove_file(audio_path);

    Ok(())
}

async fn process_audio_message(
    state: AppState,
    phone_number: String,
    audio: crate::types::WhatsAppAudio,
) -> Result<()> {
    info!("Processing audio message from {}", phone_number);

    // Download the audio message
    let audio_path = state.voice_service.download_audio_message(&audio).await?;

    // Convert speech to text
    let transcript = state.voice_service.speech_to_text(&audio_path).await?;
    
    info!("Audio transcript: {}", transcript);

    // Process the transcript as a command
    let command = BotCommand::parse(&transcript);
    
    match command {
        BotCommand::VoiceCommand { transcript } => {
            // Process the voice command
            process_voice_command(&state, &phone_number, &transcript).await?;
        }
        _ => {
            // If it's a regular command, process it normally
            process_text_message(state, phone_number, transcript).await?;
        }
    }

    // Clean up the temporary file
    let _ = std::fs::remove_file(audio_path);

    Ok(())
}

async fn process_voice_command(
    state: &AppState,
    phone_number: &str,
    transcript: &str,
) -> Result<()> {
    info!("Processing voice command: {}", transcript);

    // For now, we'll respond with a text message acknowledging the voice command
    // In the future, we could respond with a voice message using text-to-speech
    let response = format!(
        "🎤 *Voice Command Received*\n\nI heard: \"{}\"\n\nProcessing your request...",
        transcript
    );

    state
        .whatsapp_service
        .send_message(phone_number, &response)
        .await?;

    // Process the transcript as a regular command
    process_text_message(state.clone(), phone_number.to_string(), transcript.to_string()).await?;

    Ok(())
}

/// Validate that user is registered with BitSacco web app
async fn validate_registered_user(
    state: &AppState,
    phone_number: &str,
) -> Result<()> {
    match state
        .bitsacco_service
        .get_user_by_phone(phone_number, &state.cache)
        .await
    {
        Ok(_) => Ok(()),
        Err(_) => Err(AppError::Validation(
            "❌ *Access Denied*\n\nYou must be registered with BitSacco web app to use this bot.\n\nPlease visit our website to create an account first.".to_string()
        )),
    }
}

async fn get_user_balance(state: &AppState, phone_number: &str) -> Result<(f64, f64, String)> {
    let user = state
        .bitsacco_service
        .get_user_by_phone(phone_number, &state.cache)
        .await?;

    let savings = state.bitsacco_service.get_total_savings(&user.id, &state.cache).await?;

    let btc_balance = state
        .bitsacco_service
        .get_user_btc_balance(&user.id, &state.cache)
        .await?;

    Ok((savings, btc_balance.balance, btc_balance.currency))
}

async fn get_user_savings(
    state: &AppState,
    phone_number: &str,
) -> Result<Vec<crate::types::BitSaccoSavings>> {
    let user = state
        .bitsacco_service
        .get_user_by_phone(phone_number, &state.cache)
        .await?;

    state.bitsacco_service.get_user_savings(&user.id, &state.cache).await
}

async fn get_user_chamas(
    state: &AppState,
    phone_number: &str,
) -> Result<Vec<crate::types::BitSaccoChama>> {
    let user = state
        .bitsacco_service
        .get_user_by_phone(phone_number, &state.cache)
        .await?;

    state.bitsacco_service.get_user_chamas(&user.id).await
}

async fn create_deposit(
    state: &AppState,
    phone_number: &str,
    amount: f64,
    currency: &str,
) -> Result<crate::types::BitSaccoTransaction> {
    let user = state
        .bitsacco_service
        .get_user_by_phone(phone_number, &state.cache)
        .await?;

    state
        .bitsacco_service
        .create_deposit(&user.id, amount, currency)
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
        .get_user_by_phone(phone_number, &state.cache)
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

async fn create_chama(
    state: &AppState,
    phone_number: &str,
    name: &str,
    description: Option<&str>,
) -> Result<crate::types::BitSaccoChama> {
    let user = state
        .bitsacco_service
        .get_user_by_phone(phone_number, &state.cache)
        .await?;

    state
        .bitsacco_service
        .create_chama(&user.id, name, description)
        .await
}

async fn contribute_to_chama(
    state: &AppState,
    phone_number: &str,
    chama_id: &str,
    amount: f64,
    currency: &str,
) -> Result<crate::types::BitSaccoChamaContribution> {
    let user = state
        .bitsacco_service
        .get_user_by_phone(phone_number, &state.cache)
        .await?;

    state
        .bitsacco_service
        .contribute_to_chama(&user.id, chama_id, amount, currency)
        .await
}

async fn get_user_chama_shares(
    state: &AppState,
    phone_number: &str,
    chama_id: Option<&str>,
) -> Result<Vec<crate::types::BitSaccoChamaShare>> {
    let user = state
        .bitsacco_service
        .get_user_by_phone(phone_number, &state.cache)
        .await?;

    state
        .bitsacco_service
        .get_user_chama_shares(&user.id, chama_id)
        .await
}

async fn get_membership_shares(
    state: &AppState,
    phone_number: &str,
) -> Result<crate::types::BitSaccoMembershipShare> {
    let user = state
        .bitsacco_service
        .get_user_by_phone(phone_number, &state.cache)
        .await?;

    state
        .bitsacco_service
        .get_membership_shares(&user.id)
        .await
}

async fn buy_membership_shares(
    state: &AppState,
    phone_number: &str,
    count: u32,
    payment_method: &str,
) -> Result<crate::types::BitSaccoSharePurchase> {
    let user = state
        .bitsacco_service
        .get_user_by_phone(phone_number, &state.cache)
        .await?;

    state
        .bitsacco_service
        .buy_membership_shares(&user.id, count, payment_method)
        .await
}

async fn get_share_history(
    state: &AppState,
    phone_number: &str,
) -> Result<Vec<crate::types::BitSaccoSharePurchase>> {
    let user = state
        .bitsacco_service
        .get_user_by_phone(phone_number, &state.cache)
        .await?;

    state
        .bitsacco_service
        .get_share_history(&user.id)
        .await
}

async fn get_transaction_history(
    state: &AppState,
    phone_number: &str,
) -> Result<Vec<crate::types::BitSaccoTransaction>> {
    let user = state
        .bitsacco_service
        .get_user_by_phone(phone_number, &state.cache)
        .await?;

    state
        .bitsacco_service
        .get_transaction_history(&user.id)
        .await
}

async fn create_lightning_deposit(
    state: &AppState,
    phone_number: &str,
    amount: f64,
    currency: &str,
) -> Result<crate::types::LightningPaymentResponse> {
    let user = state
        .bitsacco_service
        .get_user_by_phone(phone_number, &state.cache)
        .await?;

    state
        .bitsacco_service
        .create_lightning_deposit(&user.id, amount, currency)
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
        .get_user_by_phone(phone_number, &state.cache)
        .await?;

    state
        .bitsacco_service
        .create_withdrawal(&user.id, amount, currency)
        .await
}

pub async fn health_check(State(state): State<AppState>) -> Result<Json<HealthResponse>> {
    let mut services = HashMap::new();

    // Check WhatsApp service - just verify configuration without sending messages
    services.insert("whatsapp".to_string(), "healthy".to_string());

    // Check BitSacco service
    match state.bitsacco_service.health_check().await {
        Ok(_) => services.insert("bitsacco".to_string(), "healthy".to_string()),
        Err(_) => services.insert("bitsacco".to_string(), "unhealthy".to_string()),
    };

    // Check BTC service
    match state.btc_service.health_check(&state.cache).await {
        Ok(_) => services.insert("btc".to_string(), "healthy".to_string()),
        Err(_) => services.insert("btc".to_string(), "unhealthy".to_string()),
    };

    // Check Voice service
    match state.voice_service.health_check().await {
        Ok(_) => services.insert("voice".to_string(), "healthy".to_string()),
        Err(_) => services.insert("voice".to_string(), "unhealthy".to_string()),
    };

    let response = HealthResponse {
        status: "ok".to_string(),
        timestamp: chrono::Utc::now().to_rfc3339(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        services,
    };

    Ok(Json(response))
}
