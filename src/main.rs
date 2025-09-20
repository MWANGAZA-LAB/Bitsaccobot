use anyhow::Result;
use axum::{
    routing::{get, post},
    Router,
};
use std::net::SocketAddr;
use tower::ServiceBuilder;
use tower_http::{
    cors::CorsLayer,
    trace::TraceLayer,
};
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod config;
mod error;
mod services;
mod types;
mod webhook;

use config::AppConfig;
use services::{bitsacco::BitSaccoService, btc::BtcService, whatsapp::WhatsAppService};
use types::AppState;
use webhook::{handle_webhook, send_message, health_check};

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

    // Initialize services
    let whatsapp_service = WhatsAppService::new(&config)?;
    let bitsacco_service = BitSaccoService::new(&config)?;
    let btc_service = BtcService::new(&config)?;

    let app_state = AppState {
        config,
        whatsapp_service,
        bitsacco_service,
        btc_service,
    };

    // Build application
    let app = Router::new()
        .route("/webhook", post(handle_webhook))
        .route("/send", post(send_message))
        .route("/health", get(health_check))
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
