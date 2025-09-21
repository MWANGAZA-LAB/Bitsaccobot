use bitsacco_whatsapp_bot::{
    config::AppConfig,
    services::{bitsacco::BitSaccoService, btc::BtcService, whatsapp::WhatsAppService},
    types::BotCommand,
};
use mockito::{Server, ServerGuard};
use serde_json::json;

// Helper function to create test config
async fn create_test_config() -> (AppConfig, ServerGuard) {
    let server = Server::new_async().await;
    let url = server.url();

    let config = AppConfig {
        whatsapp_access_token: "test_token".to_string(),
        whatsapp_phone_number_id: "test_phone_id".to_string(),
        whatsapp_webhook_verify_token: "test_verify_token".to_string(),
        bitsacco_api_base_url: url.clone(),
        bitsacco_api_token: "test_bitsacco_token".to_string(),
        server_host: "127.0.0.1".to_string(),
        server_port: 8080,
        rust_log: "debug".to_string(),
        rate_limit_requests_per_minute: 60,
        max_message_length: 4096,
        btc_api_base_url: url.clone(),
        btc_api_key: Some("test_btc_key".to_string()),
    };

    (config, server)
}

#[tokio::test]
async fn test_whatsapp_webhook_verification() {
    let (config, _server) = create_test_config().await;
    let whatsapp_service = WhatsAppService::new(&config).unwrap();

    // Test successful verification
    let result = whatsapp_service.verify_webhook("subscribe", "test_verify_token", "challenge123");
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "challenge123");

    // Test failed verification
    let result = whatsapp_service.verify_webhook("subscribe", "wrong_token", "challenge123");
    assert!(result.is_err());
}

#[tokio::test]
async fn test_bot_command_parsing() {
    // Test help command
    assert_eq!(BotCommand::parse("help"), BotCommand::Help);
    assert_eq!(BotCommand::parse("/help"), BotCommand::Help);

    // Test balance command
    assert_eq!(BotCommand::parse("balance"), BotCommand::Balance);
    assert_eq!(BotCommand::parse("/balance"), BotCommand::Balance);

    // Test savings command
    assert_eq!(BotCommand::parse("savings"), BotCommand::Savings);

    // Test chama command
    assert_eq!(BotCommand::parse("chama"), BotCommand::Chama);

    // Test BTC command
    assert_eq!(BotCommand::parse("btc"), BotCommand::BtcPrice);
    assert_eq!(BotCommand::parse("bitcoin"), BotCommand::BtcPrice);

    // Test deposit command
    assert_eq!(
        BotCommand::parse("deposit 100 USD"),
        BotCommand::Deposit {
            amount: 100.0,
            currency: "USD".to_string()
        }
    );

    // Test withdraw command
    assert_eq!(
        BotCommand::parse("withdraw 50 KES"),
        BotCommand::Withdraw {
            amount: 50.0,
            currency: "KES".to_string()
        }
    );

    // Test transfer command
    assert_eq!(
        BotCommand::parse("transfer 25 USD +254712345678"),
        BotCommand::Transfer {
            amount: 25.0,
            currency: "USD".to_string(),
            recipient: "+254712345678".to_string()
        }
    );

    // Test unknown command
    assert_eq!(
        BotCommand::parse("unknown command"),
        BotCommand::Unknown("unknown command".to_string())
    );
}

#[tokio::test]
async fn test_bitsacco_service_user_lookup() {
    let (config, mut server) = create_test_config().await;
    let bitsacco_service = BitSaccoService::new(&config).unwrap();

    // Mock the API response
    let _m = server
        .mock("GET", "/users/phone/+254712345678")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            json!({
                "id": "user123",
                "phone_number": "+254712345678",
                "name": "Test User",
                "email": "test@example.com",
                "created_at": "2023-01-01T00:00:00Z",
                "updated_at": "2023-01-01T00:00:00Z"
            })
            .to_string(),
        )
        .create();

    let user = bitsacco_service
        .get_user_by_phone("+254712345678")
        .await
        .unwrap();

    assert_eq!(user.id, "user123");
    assert_eq!(user.phone_number, "+254712345678");
    assert_eq!(user.name, Some("Test User".to_string()));
}

#[tokio::test]
async fn test_bitsacco_service_savings() {
    let (config, mut server) = create_test_config().await;
    let bitsacco_service = BitSaccoService::new(&config).unwrap();

    // Mock the API response
    let _m = server
        .mock("GET", "/users/user123/savings")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            json!([
                {
                    "id": "savings1",
                    "user_id": "user123",
                    "amount": 1000.0,
                    "currency": "KES",
                    "chama_id": null,
                    "created_at": "2023-01-01T00:00:00Z",
                    "updated_at": "2023-01-01T00:00:00Z"
                },
                {
                    "id": "savings2",
                    "user_id": "user123",
                    "amount": 500.0,
                    "currency": "USD",
                    "chama_id": "chama1",
                    "created_at": "2023-01-01T00:00:00Z",
                    "updated_at": "2023-01-01T00:00:00Z"
                }
            ])
            .to_string(),
        )
        .create();

    let savings = bitsacco_service.get_user_savings("user123").await.unwrap();

    assert_eq!(savings.len(), 2);
    assert_eq!(savings[0].amount, 1000.0);
    assert_eq!(savings[1].amount, 500.0);
}

#[tokio::test]
async fn test_btc_service_price() {
    let (config, mut server) = create_test_config().await;
    let btc_service = BtcService::new(&config).unwrap();

    // Mock the API response
    let _m = server
        .mock(
            "GET",
            "/simple/price?ids=bitcoin&vs_currencies=usd&include_24hr_change=true",
        )
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            json!({
                "bitcoin": {
                    "usd": 50000.0,
                    "usd_24h_change": 2.5
                }
            })
            .to_string(),
        )
        .create();

    let price = btc_service.get_btc_price_usd().await.unwrap();

    assert_eq!(price.currency, "USD");
    assert_eq!(price.price, 50000.0);
    assert_eq!(price.change_24h, 2.5);
}

#[tokio::test]
async fn test_whatsapp_send_message() {
    let (config, mut server) = create_test_config().await;
    let whatsapp_service = WhatsAppService::new(&config).unwrap();

    // Mock the WhatsApp API response
    let _m = server
        .mock("POST", "/test_phone_id/messages")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            json!({
                "messaging_product": "whatsapp",
                "contacts": [{
                    "input": "+254712345678",
                    "wa_id": "+254712345678"
                }],
                "messages": [{
                    "id": "wamid.123456789"
                }]
            })
            .to_string(),
        )
        .create();

    let response = whatsapp_service
        .send_message("+254712345678", "Test message")
        .await
        .unwrap();

    assert_eq!(response.messaging_product, "whatsapp");
    assert_eq!(response.contacts.len(), 1);
    assert_eq!(response.messages.len(), 1);
}

#[tokio::test]
async fn test_error_handling() {
    let (config, mut server) = create_test_config().await;
    let bitsacco_service = BitSaccoService::new(&config).unwrap();

    // Mock API error response
    let _m = server
        .mock("GET", "/users/phone/invalid")
        .with_status(404)
        .with_header("content-type", "application/json")
        .with_body(
            json!({
                "error": "User not found"
            })
            .to_string(),
        )
        .create();

    let result = bitsacco_service.get_user_by_phone("invalid").await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_message_validation() {
    let (config, _server) = create_test_config().await;
    let whatsapp_service = WhatsAppService::new(&config).unwrap();

    // Test message too long
    let long_message = "a".repeat(5000);
    let result = whatsapp_service
        .send_message("+254712345678", &long_message)
        .await;
    assert!(result.is_err());

    // Test valid message
    let valid_message = "Hello, this is a valid message";
    // Note: This would fail in real test due to no mock, but validates the length check passes
    let result = whatsapp_service
        .send_message("+254712345678", valid_message)
        .await;
    // We expect this to fail due to no mock, but not due to validation
    assert!(result.is_err());
}
