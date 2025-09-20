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
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AppError::Config(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg.to_string()),
            AppError::Http(msg) => (StatusCode::BAD_GATEWAY, msg.to_string()),
            AppError::Json(msg) => (StatusCode::BAD_REQUEST, msg.to_string()),
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
        };

        let body = Json(json!({
            "error": error_message,
            "status": status.as_u16()
        }));

        (status, body).into_response()
    }
}

pub type Result<T> = std::result::Result<T, AppError>;
