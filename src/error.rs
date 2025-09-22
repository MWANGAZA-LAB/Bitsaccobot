use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Configuration error: {0}")]
    Config(#[from] anyhow::Error),

    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),

    #[error("JSON serialization error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("WhatsApp API error: {0}")]
    WhatsApp(String),

    #[error("BitSacco API error: {0}")]
    BitSacco(String),

    #[error("BTC service error: {0}")]
    BtcService(String),

    #[error("Rate limit exceeded")]
    #[allow(dead_code)]
    RateLimit,

    #[error("Unauthorized access")]
    Unauthorized,

    #[error("Internal server error: {0}")]
    Internal(String),

    #[error("User not found")]
    UserNotFound,

    #[error("Insufficient funds")]
    InsufficientFunds,

    #[error("Invalid command: {0}")]
    InvalidCommand(String),

    #[error("Voice processing error: {0}")]
    VoiceProcessing(String),

    #[error("Network error: {0}")]
    Network(String),

    #[error("Authentication failed: {0}")]
    Authentication(String),

    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    #[error("Service unavailable: {0}")]
    ServiceUnavailable(String),

    #[error("Timeout error: {0}")]
    Timeout(String),

    #[error("Data not found: {0}")]
    DataNotFound(String),

    #[error("Invalid input: {0}")]
    InvalidInput(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AppError::Config(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg.to_string()),
            AppError::Http(msg) => (StatusCode::BAD_GATEWAY, msg.to_string()),
            AppError::Json(msg) => (StatusCode::BAD_REQUEST, msg.to_string()),
            AppError::Io(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg.to_string()),
            AppError::Validation(msg) => (StatusCode::BAD_REQUEST, msg),
            AppError::WhatsApp(msg) => (StatusCode::BAD_GATEWAY, msg),
            AppError::BitSacco(msg) => (StatusCode::BAD_GATEWAY, msg),
            AppError::BtcService(msg) => (StatusCode::BAD_GATEWAY, msg),
            AppError::RateLimit => (
                StatusCode::TOO_MANY_REQUESTS,
                "Rate limit exceeded".to_string(),
            ),
            AppError::Unauthorized => (StatusCode::UNAUTHORIZED, "Unauthorized access".to_string()),
            AppError::Internal(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
            AppError::UserNotFound => (StatusCode::NOT_FOUND, "User not found".to_string()),
            AppError::InsufficientFunds => (StatusCode::BAD_REQUEST, "Insufficient funds".to_string()),
            AppError::InvalidCommand(msg) => (StatusCode::BAD_REQUEST, format!("Invalid command: {}", msg)),
            AppError::VoiceProcessing(msg) => (StatusCode::BAD_REQUEST, format!("Voice processing error: {}", msg)),
            AppError::Network(msg) => (StatusCode::BAD_GATEWAY, format!("Network error: {}", msg)),
            AppError::Authentication(msg) => (StatusCode::UNAUTHORIZED, format!("Authentication failed: {}", msg)),
            AppError::PermissionDenied(msg) => (StatusCode::FORBIDDEN, format!("Permission denied: {}", msg)),
            AppError::ServiceUnavailable(msg) => (StatusCode::SERVICE_UNAVAILABLE, format!("Service unavailable: {}", msg)),
            AppError::Timeout(msg) => (StatusCode::REQUEST_TIMEOUT, format!("Timeout: {}", msg)),
            AppError::DataNotFound(msg) => (StatusCode::NOT_FOUND, format!("Data not found: {}", msg)),
            AppError::InvalidInput(msg) => (StatusCode::BAD_REQUEST, format!("Invalid input: {}", msg)),
        };

        let body = Json(json!({
            "error": error_message,
            "status": status.as_u16()
        }));

        (status, body).into_response()
    }
}

impl AppError {
    /// Get user-friendly error message for WhatsApp responses
    pub fn to_user_message(&self) -> String {
        match self {
            AppError::Config(_) => "System configuration error. Please try again later.".to_string(),
            AppError::Http(_) => "Network connection error. Please check your internet and try again.".to_string(),
            AppError::Json(_) => "Data processing error. Please try again.".to_string(),
            AppError::Io(_) => "File system error. Please try again.".to_string(),
            AppError::Internal(_) => "Internal system error. Our team has been notified.".to_string(),
            AppError::Validation(msg) => format!("Invalid input: {}. Please check your message and try again.", msg),
            AppError::WhatsApp(msg) => format!("WhatsApp service error: {}. Please try again later.", msg),
            AppError::BitSacco(msg) => format!("BitSacco service error: {}. Please try again later.", msg),
            AppError::BtcService(msg) => format!("Bitcoin price service error: {}. Please try again later.", msg),
            AppError::RateLimit => "Too many requests. Please wait a moment before trying again.".to_string(),
            AppError::Unauthorized => "Authentication required. Please contact support.".to_string(),
            AppError::UserNotFound => "User account not found. Please register with BitSacco first.".to_string(),
            AppError::InsufficientFunds => "Insufficient funds for this transaction. Please check your balance.".to_string(),
            AppError::InvalidCommand(msg) => format!("Unknown command: {}. Type 'help' to see available commands.", msg),
            AppError::VoiceProcessing(_) => "Voice message processing failed. Please try sending a text message.".to_string(),
            AppError::Network(msg) => format!("Network error: {}. Please check your connection.", msg),
            AppError::Authentication(msg) => format!("Authentication failed: {}. Please contact support.", msg),
            AppError::PermissionDenied(msg) => format!("Permission denied: {}. Please contact support.", msg),
            AppError::ServiceUnavailable(msg) => format!("Service unavailable: {}. Please try again later.", msg),
            AppError::Timeout(msg) => format!("Request timed out: {}. Please try again.", msg),
            AppError::DataNotFound(msg) => format!("Data not found: {}. Please check your input.", msg),
            AppError::InvalidInput(msg) => format!("Invalid input: {}. Please check your message format.", msg),
        }
    }

    /// Get error severity level for monitoring
    pub fn severity(&self) -> ErrorSeverity {
        match self {
            AppError::Config(_) | AppError::Internal(_) => ErrorSeverity::Critical,
            AppError::WhatsApp(_) | AppError::BitSacco(_) | AppError::BtcService(_) => ErrorSeverity::High,
            AppError::Network(_) | AppError::Timeout(_) | AppError::ServiceUnavailable(_) => ErrorSeverity::Medium,
            AppError::Validation(_) | AppError::InvalidCommand(_) | AppError::InvalidInput(_) => ErrorSeverity::Low,
            _ => ErrorSeverity::Medium,
        }
    }

    /// Check if error should trigger an alert
    pub fn should_alert(&self) -> bool {
        matches!(self.severity(), ErrorSeverity::Critical | ErrorSeverity::High)
    }

    /// Get error category for analytics
    pub fn category(&self) -> ErrorCategory {
        match self {
            AppError::Config(_) | AppError::Internal(_) => ErrorCategory::System,
            AppError::Http(_) | AppError::Network(_) | AppError::Timeout(_) => ErrorCategory::Network,
            AppError::WhatsApp(_) | AppError::BitSacco(_) | AppError::BtcService(_) => ErrorCategory::ExternalApi,
            AppError::Validation(_) | AppError::InvalidCommand(_) | AppError::InvalidInput(_) => ErrorCategory::UserInput,
            AppError::UserNotFound | AppError::InsufficientFunds | AppError::PermissionDenied(_) => ErrorCategory::Business,
            AppError::VoiceProcessing(_) => ErrorCategory::Media,
            _ => ErrorCategory::System,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ErrorSeverity {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ErrorCategory {
    System,
    Network,
    ExternalApi,
    UserInput,
    Business,
    Media,
}

pub type Result<T> = std::result::Result<T, AppError>;
