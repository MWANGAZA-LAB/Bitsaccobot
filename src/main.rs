use anyhow::Result;
use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use std::collections::HashMap;
use std::net::SocketAddr;
use tower::ServiceBuilder;
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod cache;
mod config;
mod error;
mod monitoring;
mod services;
mod types;
mod validation;
mod webhook;

use cache::AppCache;
use config::AppConfig;
use error::AppError;
use monitoring::{ComponentHealth, HealthStatus, MonitoringService, SystemMetrics};
use services::{bitsacco::BitSaccoService, btc::BtcService, twilio::TwilioService, voice::VoiceService, whatsapp::WhatsAppService};
use types::AppState;
use webhook::{handle_webhook, health_check, send_message};

/// Get system metrics endpoint
async fn get_metrics(State(state): State<AppState>) -> Result<Json<SystemMetrics>, AppError> {
    // In a real implementation, you would get metrics from the monitoring service
    // For now, we'll return a mock response
    let metrics = SystemMetrics {
        total_requests: 1000,
        successful_requests: 950,
        failed_requests: 50,
        average_response_time_ms: 250.5,
        last_request_time: Some(
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        ),
        uptime_seconds: 3600,
        memory_usage_mb: 128.5,
        active_connections: 5,
    };
    
    Ok(Json(metrics))
}

/// Get detailed health status endpoint
async fn get_detailed_health(State(state): State<AppState>) -> Result<Json<HealthStatus>, AppError> {
    // In a real implementation, you would get health from the monitoring service
    // For now, we'll return a mock response
    let mut components = HashMap::new();
    components.insert("whatsapp_api".to_string(), ComponentHealth {
        status: "healthy".to_string(),
        message: "WhatsApp API is responding normally".to_string(),
        last_check: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs(),
        response_time_ms: Some(150),
    });
    
    let health = HealthStatus {
        status: "healthy".to_string(),
        timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs(),
        components,
        overall_health: "healthy".to_string(),
    };
    
    Ok(Json(health))
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "bitsacco_whatsapp_bot=info,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Load configuration
    let config = AppConfig::load()?;
    info!("Configuration loaded successfully");

    // Initialize cache
    let cache = AppCache::new(cache::CacheConfig::default());

    // Initialize monitoring service
    let monitoring = MonitoringService::new(None);
    monitoring.start_monitoring().await;

    // Initialize services
    let whatsapp_service = WhatsAppService::new(&config)?;
    let bitsacco_service = BitSaccoService::new(&config)?;
    let btc_service = BtcService::new(&config)?;
    let voice_service = VoiceService::new(&config)?;
    let twilio_service = TwilioService::new(config.clone());

    let app_state = AppState {
        config,
        whatsapp_service,
        bitsacco_service,
        btc_service,
        voice_service,
        cache,
        twilio_service,
    };

    // Build application
    let app = Router::new()
        .route("/webhook", post(handle_webhook))
        .route("/send", post(send_message))
        .route("/health", get(health_check))
        .route("/metrics", get(get_metrics))
        .route("/health/detailed", get(get_detailed_health))
        .with_state(app_state)
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(CorsLayer::permissive()),
        );

    // Start server
    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
    info!("Starting server on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
