use crate::{
    config::AppConfig,
    error::{AppError, Result},
    types::{WhatsAppAudio, WhatsAppVoice},
};
use reqwest::Client;
use serde_json;
use std::path::PathBuf;
use tempfile::NamedTempFile;
use tracing::info;

/// Voice processing service for handling voice messages
#[derive(Debug, Clone)]
pub struct VoiceService {
    client: Client,
    whatsapp_access_token: String,
    media_base_url: String,
    temp_dir: PathBuf,
}

impl VoiceService {
    pub fn new(config: &AppConfig) -> Result<Self> {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(60)) // Longer timeout for media downloads
            .pool_max_idle_per_host(10)
            .pool_idle_timeout(std::time::Duration::from_secs(90))
            .connect_timeout(std::time::Duration::from_secs(10))
            .build()
            .map_err(|e| AppError::Internal(format!("Failed to create HTTP client: {}", e)))?;

        let temp_dir = std::env::temp_dir().join("bitsacco_voice");
        std::fs::create_dir_all(&temp_dir)
            .map_err(|e| AppError::Internal(format!("Failed to create temp directory: {}", e)))?;

        Ok(Self {
            client,
            whatsapp_access_token: config.whatsapp_access_token.clone(),
            media_base_url: config.whatsapp_media_base_url.clone(),
            temp_dir,
        })
    }

    /// Download a voice message from WhatsApp
    pub async fn download_voice_message(&self, voice: &WhatsAppVoice) -> Result<PathBuf> {
        let url = format!("{}/{}", self.media_base_url, voice.id);
        
        info!("Downloading voice message from: {}", url);
        
        let response = self
            .client
            .get(&url)
            .bearer_auth(&self.whatsapp_access_token)
            .send()
            .await
            .map_err(|e| AppError::Internal(format!("Failed to download voice message: {}", e)))?;

        if !response.status().is_success() {
            return Err(AppError::Internal(format!(
                "Failed to download voice message: HTTP {}",
                response.status()
            )));
        }

        let audio_data = response
            .bytes()
            .await
            .map_err(|e| AppError::Internal(format!("Failed to read voice message data: {}", e)))?;

        // Create a temporary file with the correct extension
        let extension = self.get_audio_extension(&voice.mime_type);
        let temp_file = NamedTempFile::new_in(&self.temp_dir)
            .map_err(|e| AppError::Internal(format!("Failed to create temp file: {}", e)))?;

        let file_path = temp_file.path().with_extension(extension);
        std::fs::write(&file_path, &audio_data)
            .map_err(|e| AppError::Internal(format!("Failed to write voice message: {}", e)))?;

        info!("Voice message saved to: {:?}", file_path);
        Ok(file_path)
    }

    /// Download an audio message from WhatsApp
    pub async fn download_audio_message(&self, audio: &WhatsAppAudio) -> Result<PathBuf> {
        let url = format!("{}/{}", self.media_base_url, audio.id);
        
        info!("Downloading audio message from: {}", url);
        
        let response = self
            .client
            .get(&url)
            .bearer_auth(&self.whatsapp_access_token)
            .send()
            .await
            .map_err(|e| AppError::Internal(format!("Failed to download audio message: {}", e)))?;

        if !response.status().is_success() {
            return Err(AppError::Internal(format!(
                "Failed to download audio message: HTTP {}",
                response.status()
            )));
        }

        let audio_data = response
            .bytes()
            .await
            .map_err(|e| AppError::Internal(format!("Failed to read audio message data: {}", e)))?;

        // Create a temporary file with the correct extension
        let extension = self.get_audio_extension(&audio.mime_type);
        let temp_file = NamedTempFile::new_in(&self.temp_dir)
            .map_err(|e| AppError::Internal(format!("Failed to create temp file: {}", e)))?;

        let file_path = temp_file.path().with_extension(extension);
        std::fs::write(&file_path, &audio_data)
            .map_err(|e| AppError::Internal(format!("Failed to write audio message: {}", e)))?;

        info!("Audio message saved to: {:?}", file_path);
        Ok(file_path)
    }

    /// Convert speech to text using a simple approach
    /// Note: In production, integrate with cloud services like Azure Speech, Google Cloud Speech, or AWS Transcribe
    pub async fn speech_to_text(&self, audio_path: &PathBuf) -> Result<String> {
        // For now, we'll implement a placeholder that returns a mock transcript
        // In production, this would integrate with a speech-to-text service
        
        info!("Converting speech to text for file: {:?}", audio_path);
        
        // Check if file exists and has reasonable size
        let metadata = std::fs::metadata(audio_path)
            .map_err(|e| AppError::Internal(format!("Failed to read audio file metadata: {}", e)))?;
        
        if metadata.len() == 0 {
            return Err(AppError::Validation("Empty audio file".to_string()));
        }
        
        if metadata.len() > 16 * 1024 * 1024 { // 16MB limit
            return Err(AppError::Validation("Audio file too large".to_string()));
        }

        // Mock implementation - in production, replace with actual STT service
        let mock_transcript = self.generate_mock_transcript(audio_path).await?;
        
        info!("Speech-to-text result: {}", mock_transcript);
        Ok(mock_transcript)
    }

    /// Generate a mock transcript for testing purposes
    /// In production, this would integrate with OpenAI Whisper, Azure Speech, or Google Cloud Speech
    async fn generate_mock_transcript(&self, audio_path: &PathBuf) -> Result<String> {
        // Check if OpenAI API key is available for real transcription
        if let Ok(api_key) = std::env::var("OPENAI_API_KEY") {
            if !api_key.is_empty() {
                return self.transcribe_with_openai(audio_path).await;
            }
        }
        
        // Fallback to mock implementation for testing
        let _file_name = audio_path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown");
        
        // Simple mock based on file characteristics
        let file_size = std::fs::metadata(audio_path)?.len();
        
        if file_size < 1000 {
            Ok("help".to_string())
        } else if file_size < 5000 {
            Ok("balance".to_string())
        } else if file_size < 10000 {
            Ok("savings".to_string())
        } else {
            Ok("bitcoin price".to_string())
        }
    }
    
    /// Transcribe audio using OpenAI Whisper API
    async fn transcribe_with_openai(&self, audio_path: &PathBuf) -> Result<String> {
        info!("Using OpenAI Whisper API for transcription");
        
        // Read the audio file
        let audio_data = std::fs::read(audio_path)
            .map_err(|e| AppError::Internal(format!("Failed to read audio file: {}", e)))?;
        
        // Create multipart form data for OpenAI Whisper API
        let form = reqwest::multipart::Form::new()
            .text("model", "whisper-1")
            .text("language", "en")
            .text("response_format", "json")
            .part("file", reqwest::multipart::Part::bytes(audio_data)
                .file_name("audio.wav")
                .mime_str("audio/wav")?);
        
        // Make request to OpenAI Whisper API
        let response = self.client
            .post("https://api.openai.com/v1/audio/transcriptions")
            .header("Authorization", format!("Bearer {}", std::env::var("OPENAI_API_KEY").unwrap_or_default()))
            .multipart(form)
            .send()
            .await
            .map_err(|e| AppError::Internal(format!("Failed to send request to Whisper API: {}", e)))?;
        
        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(AppError::Internal(format!("Whisper API error: {}", error_text)));
        }
        
        let result: serde_json::Value = response.json().await
            .map_err(|e| AppError::Internal(format!("Failed to parse Whisper API response: {}", e)))?;
        
        let transcript = result["text"]
            .as_str()
            .ok_or_else(|| AppError::Internal("No transcript in Whisper API response".to_string()))?;
        
        info!("OpenAI Whisper transcription completed: {}", transcript);
        Ok(transcript.to_string())
    }

    /// Convert text to speech and return audio file path
    /// Note: In production, integrate with cloud services like Azure Speech, Google Cloud TTS, or AWS Polly
    pub async fn text_to_speech(&self, text: &str) -> Result<PathBuf> {
        info!("Converting text to speech: {}", text);
        
        // Check if OpenAI API key is available for real TTS
        if let Ok(api_key) = std::env::var("OPENAI_API_KEY") {
            if !api_key.is_empty() {
                return self.synthesize_with_openai(text).await;
            }
        }
        
        // Fallback to mock implementation for testing
        let temp_file = NamedTempFile::new_in(&self.temp_dir)
            .map_err(|e| AppError::Internal(format!("Failed to create temp file: {}", e)))?;

        let file_path = temp_file.path().with_extension("wav");
        
        // Create a simple WAV file with silence (placeholder)
        // Duration based on text length (roughly 150 words per minute)
        let word_count = text.split_whitespace().count();
        let duration_ms = (word_count as u32 * 400).max(1000); // 400ms per word, minimum 1 second
        self.create_silence_wav(&file_path, duration_ms)?;
        
        info!("Text-to-speech audio saved to: {:?}", file_path);
        Ok(file_path)
    }

    /// Synthesize speech using OpenAI TTS API
    async fn synthesize_with_openai(&self, text: &str) -> Result<PathBuf> {
        info!("Using OpenAI TTS API for speech synthesis");
        
        // Create request body for OpenAI TTS API
        let request_body = serde_json::json!({
            "model": "tts-1",
            "input": text,
            "voice": "alloy",
            "response_format": "wav"
        });
        
        // Make request to OpenAI TTS API
        let response = self.client
            .post("https://api.openai.com/v1/audio/speech")
            .header("Authorization", format!("Bearer {}", std::env::var("OPENAI_API_KEY").unwrap_or_default()))
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await
            .map_err(|e| AppError::Internal(format!("Failed to send request to TTS API: {}", e)))?;
        
        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(AppError::Internal(format!("TTS API error: {}", error_text)));
        }
        
        // Get audio data
        let audio_data = response.bytes().await
            .map_err(|e| AppError::Internal(format!("Failed to read TTS response: {}", e)))?;
        
        // Save to temporary file
        let temp_file = NamedTempFile::new_in(&self.temp_dir)
            .map_err(|e| AppError::Internal(format!("Failed to create temp file: {}", e)))?;

        let file_path = temp_file.path().with_extension("wav");
        
        std::fs::write(&file_path, audio_data)
            .map_err(|e| AppError::Internal(format!("Failed to write TTS audio file: {}", e)))?;
        
        info!("OpenAI TTS audio saved to: {:?}", file_path);
        Ok(file_path.to_path_buf())
    }

    /// Create a simple WAV file with silence (placeholder implementation)
    fn create_silence_wav(&self, path: &PathBuf, duration_ms: u32) -> Result<()> {
        use std::fs::File;
        use std::io::Write;
        
        let sample_rate: u32 = 16000;
        let channels: u16 = 1;
        let bits_per_sample: u16 = 16;
        let samples = (sample_rate * duration_ms / 1000) as u32;
        let data_size = samples * channels as u32 * (bits_per_sample as u32 / 8);
        let file_size = 44 + data_size; // WAV header is 44 bytes
        
        let mut file = File::create(path)?;
        
        // Write WAV header
        file.write_all(b"RIFF")?;
        file.write_all(&(file_size - 8).to_le_bytes())?;
        file.write_all(b"WAVE")?;
        file.write_all(b"fmt ")?;
        file.write_all(&16u32.to_le_bytes())?; // fmt chunk size
        file.write_all(&1u16.to_le_bytes())?; // PCM format
        file.write_all(&channels.to_le_bytes())?;
        file.write_all(&sample_rate.to_le_bytes())?;
        file.write_all(&(sample_rate * channels as u32 * bits_per_sample as u32 / 8).to_le_bytes())?; // byte rate
        file.write_all(&(channels * bits_per_sample / 8).to_le_bytes())?; // block align
        file.write_all(&bits_per_sample.to_le_bytes())?;
        file.write_all(b"data")?;
        file.write_all(&data_size.to_le_bytes())?;
        
        // Write silence (zeros)
        let silence = vec![0u8; data_size as usize];
        file.write_all(&silence)?;
        
        Ok(())
    }

    /// Get file extension based on MIME type
    pub fn get_audio_extension(&self, mime_type: &str) -> &'static str {
        match mime_type {
            "audio/ogg" => "ogg",
            "audio/mpeg" => "mp3",
            "audio/wav" => "wav",
            "audio/mp4" => "m4a",
            "audio/aac" => "aac",
            "audio/webm" => "webm",
            _ => "bin", // fallback
        }
    }

    /// Clean up temporary files
    pub fn cleanup_temp_files(&self) -> Result<()> {
        if self.temp_dir.exists() {
            std::fs::remove_dir_all(&self.temp_dir)
                .map_err(|e| AppError::Internal(format!("Failed to cleanup temp directory: {}", e)))?;
            std::fs::create_dir_all(&self.temp_dir)
                .map_err(|e| AppError::Internal(format!("Failed to recreate temp directory: {}", e)))?;
        }
        Ok(())
    }

    /// Health check for voice service
    pub async fn health_check(&self) -> Result<()> {
        // Check if temp directory is accessible
        if !self.temp_dir.exists() {
            std::fs::create_dir_all(&self.temp_dir)
                .map_err(|e| AppError::Internal(format!("Failed to create temp directory: {}", e)))?;
        }
        
        // Test creating a temporary file
        let temp_file = NamedTempFile::new_in(&self.temp_dir)
            .map_err(|e| AppError::Internal(format!("Failed to create test temp file: {}", e)))?;
        
        info!("Voice service health check passed");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::AppConfig;
    // tempfile::tempdir removed - using NamedTempFile instead

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
    async fn test_health_check() {
        let config = create_test_config();
        let voice_service = VoiceService::new(&config).unwrap();
        
        let result = voice_service.health_check().await;
        assert!(result.is_ok());
    }
}
