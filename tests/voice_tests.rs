use bitsacco_whatsapp_bot::{
    config::AppConfig,
    services::voice::VoiceService,
    types::{WhatsAppAudio, WhatsAppVoice},
};
use tempfile::tempdir;

fn create_test_config() -> AppConfig {
    AppConfig {
        whatsapp_access_token: "test_token".to_string(),
        whatsapp_phone_number_id: "test_phone_id".to_string(),
        whatsapp_webhook_verify_token: "test_verify_token".to_string(),
        whatsapp_api_base_url: "https://graph.facebook.com/v18.0".to_string(),
        whatsapp_media_base_url: "https://graph.facebook.com/v18.0".to_string(),
        bitsacco_api_base_url: "https://api.bitsacco.com".to_string(),
        bitsacco_api_token: "test_bitsacco_token".to_string(),
        btc_api_base_url: "https://api.coingecko.com/api/v3".to_string(),
        btc_api_key: None,
        rate_limit_requests_per_minute: 60,
        max_message_length: 4096,
        server_host: "0.0.0.0".to_string(),
        server_port: 8080,
        rust_log: "info".to_string(),
    }
}

#[tokio::test]
async fn test_voice_service_creation() {
    let config = create_test_config();
    let voice_service = VoiceService::new(&config);
    assert!(voice_service.is_ok());
}

#[tokio::test]
async fn test_audio_extension_mapping() {
    let config = create_test_config();
    let voice_service = VoiceService::new(&config).unwrap();
    
    assert_eq!(voice_service.get_audio_extension("audio/ogg"), "ogg");
    assert_eq!(voice_service.get_audio_extension("audio/mpeg"), "mp3");
    assert_eq!(voice_service.get_audio_extension("audio/wav"), "wav");
    assert_eq!(voice_service.get_audio_extension("unknown"), "bin");
}

#[tokio::test]
async fn test_text_to_speech() {
    let config = create_test_config();
    let voice_service = VoiceService::new(&config).unwrap();
    
    let result = voice_service.text_to_speech("Hello, this is a test").await;
    assert!(result.is_ok());
    
    let audio_path = result.unwrap();
    assert!(audio_path.exists());
    assert_eq!(audio_path.extension().unwrap(), "wav");
    
    // Clean up
    let _ = std::fs::remove_file(audio_path);
}

#[tokio::test]
async fn test_speech_to_text_mock() {
    let config = create_test_config();
    let voice_service = VoiceService::new(&config).unwrap();
    
    // Create a temporary audio file
    let temp_dir = tempdir().unwrap();
    let audio_path = temp_dir.path().join("test.wav");
    
    // Create a small test file
    std::fs::write(&audio_path, b"test audio data").unwrap();
    
    let result = voice_service.speech_to_text(&audio_path).await;
    assert!(result.is_ok());
    
    let transcript = result.unwrap();
    assert!(!transcript.is_empty());
}

#[tokio::test]
async fn test_health_check() {
    let config = create_test_config();
    let voice_service = VoiceService::new(&config).unwrap();
    
    let result = voice_service.health_check().await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_voice_message_structures() {
    let voice = WhatsAppVoice {
        id: "test_voice_id".to_string(),
        mime_type: "audio/ogg".to_string(),
        sha256: "test_hash".to_string(),
    };
    
    assert_eq!(voice.id, "test_voice_id");
    assert_eq!(voice.mime_type, "audio/ogg");
    assert_eq!(voice.sha256, "test_hash");
    
    let audio = WhatsAppAudio {
        id: "test_audio_id".to_string(),
        mime_type: "audio/mpeg".to_string(),
        sha256: "test_hash2".to_string(),
    };
    
    assert_eq!(audio.id, "test_audio_id");
    assert_eq!(audio.mime_type, "audio/mpeg");
    assert_eq!(audio.sha256, "test_hash2");
}

#[tokio::test]
async fn test_voice_command_parsing() {
    use bitsacco_whatsapp_bot::types::BotCommand;
    
    // Test voice command parsing
    let command = BotCommand::parse("help");
    assert!(matches!(command, BotCommand::Help));
    
    let command = BotCommand::parse("balance");
    assert!(matches!(command, BotCommand::Balance));
    
    let command = BotCommand::parse("bitcoin");
    assert!(matches!(command, BotCommand::BtcPrice));
    
    let command = BotCommand::parse("bitcoin price");
    assert!(matches!(command, BotCommand::BtcPrice));
    
    let command = BotCommand::parse("deposit 100 usd");
    match command {
        BotCommand::Deposit { amount, currency } => {
            assert_eq!(amount, 100.0);
            assert_eq!(currency, "USD");
        }
        _ => panic!("Expected Deposit command"),
    }
}

#[tokio::test]
async fn test_voice_command_with_transcript() {
    use bitsacco_whatsapp_bot::types::BotCommand;
    
    let command = BotCommand::VoiceCommand {
        transcript: "help me".to_string(),
    };
    
    match command {
        BotCommand::VoiceCommand { transcript } => {
            assert_eq!(transcript, "help me");
        }
        _ => panic!("Expected VoiceCommand"),
    }
}
