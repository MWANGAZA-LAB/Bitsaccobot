//! Monitoring and metrics module for the BitSacco WhatsApp Bot
//! 
//! This module provides comprehensive monitoring capabilities including:
//! - Request metrics and performance tracking
//! - Error rate monitoring
//! - Health check endpoints
//! - Alerting capabilities

use crate::error::{AppError, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tokio::sync::RwLock;
use tracing::{error, info, warn};

/// Metrics for tracking system performance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemMetrics {
    pub total_requests: u64,
    pub successful_requests: u64,
    pub failed_requests: u64,
    pub average_response_time_ms: f64,
    pub last_request_time: Option<u64>,
    pub uptime_seconds: u64,
    pub memory_usage_mb: f64,
    pub active_connections: u32,
}

/// Health status for different system components
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthStatus {
    pub status: String,
    pub timestamp: u64,
    pub components: HashMap<String, ComponentHealth>,
    pub overall_health: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentHealth {
    pub status: String,
    pub message: String,
    pub last_check: u64,
    pub response_time_ms: Option<u64>,
}

/// Alert configuration for monitoring
#[derive(Debug, Clone)]
pub struct AlertConfig {
    pub error_rate_threshold: f64,
    pub response_time_threshold_ms: u64,
    pub memory_threshold_mb: f64,
    pub check_interval_seconds: u64,
}

impl Default for AlertConfig {
    fn default() -> Self {
        Self {
            error_rate_threshold: 0.05, // 5% error rate
            response_time_threshold_ms: 5000, // 5 seconds
            memory_threshold_mb: 512.0, // 512 MB
            check_interval_seconds: 60, // 1 minute
        }
    }
}

/// Monitoring service for tracking system metrics and health
#[derive(Debug, Clone)]
pub struct MonitoringService {
    metrics: Arc<RwLock<SystemMetrics>>,
    health_status: Arc<RwLock<HealthStatus>>,
    alert_config: AlertConfig,
    start_time: Instant,
}

impl MonitoringService {
    /// Create a new monitoring service
    pub fn new(alert_config: Option<AlertConfig>) -> Self {
        let start_time = Instant::now();
        let metrics = SystemMetrics {
            total_requests: 0,
            successful_requests: 0,
            failed_requests: 0,
            average_response_time_ms: 0.0,
            last_request_time: None,
            uptime_seconds: 0,
            memory_usage_mb: 0.0,
            active_connections: 0,
        };

        let health_status = HealthStatus {
            status: "healthy".to_string(),
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            components: HashMap::new(),
            overall_health: "healthy".to_string(),
        };

        Self {
            metrics: Arc::new(RwLock::new(metrics)),
            health_status: Arc::new(RwLock::new(health_status)),
            alert_config: alert_config.unwrap_or_default(),
            start_time,
        }
    }

    /// Record a successful request
    pub async fn record_successful_request(&self, response_time_ms: u64) {
        let mut metrics = self.metrics.write().await;
        metrics.total_requests += 1;
        metrics.successful_requests += 1;
        metrics.last_request_time = Some(
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        );
        
        // Update average response time
        let total_time = metrics.average_response_time_ms * (metrics.total_requests - 1) as f64;
        metrics.average_response_time_ms = (total_time + response_time_ms as f64) / metrics.total_requests as f64;
        
        metrics.uptime_seconds = self.start_time.elapsed().as_secs();
        
        info!("Request successful - Response time: {}ms", response_time_ms);
    }

    /// Record a failed request
    pub async fn record_failed_request(&self, error: &str) {
        let mut metrics = self.metrics.write().await;
        metrics.total_requests += 1;
        metrics.failed_requests += 1;
        metrics.last_request_time = Some(
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        );
        
        metrics.uptime_seconds = self.start_time.elapsed().as_secs();
        
        error!("Request failed: {}", error);
        
        // Check if we need to send an alert
        self.check_error_rate_alert(&metrics).await;
    }

    /// Get current system metrics
    pub async fn get_metrics(&self) -> SystemMetrics {
        let mut metrics = self.metrics.read().await.clone();
        metrics.uptime_seconds = self.start_time.elapsed().as_secs();
        metrics.memory_usage_mb = self.get_memory_usage();
        metrics
    }

    /// Get current health status
    pub async fn get_health_status(&self) -> HealthStatus {
        let mut health = self.health_status.read().await.clone();
        health.timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        
        // Update component health
        self.update_component_health(&mut health).await;
        
        // Update overall health
        health.overall_health = self.determine_overall_health(&health);
        
        health
    }

    /// Check if error rate exceeds threshold and send alert
    async fn check_error_rate_alert(&self, metrics: &SystemMetrics) {
        if metrics.total_requests > 10 {
            let error_rate = metrics.failed_requests as f64 / metrics.total_requests as f64;
            if error_rate > self.alert_config.error_rate_threshold {
                warn!(
                    "High error rate detected: {:.2}% (threshold: {:.2}%)",
                    error_rate * 100.0,
                    self.alert_config.error_rate_threshold * 100.0
                );
                // In production, this would send an alert to monitoring systems
                self.send_alert("High Error Rate", &format!("Error rate: {:.2}%", error_rate * 100.0)).await;
            }
        }
    }

    /// Update health status for individual components
    async fn update_component_health(&self, health: &mut HealthStatus) {
        let start_time = Instant::now();
        
        // Check WhatsApp API health
        let whatsapp_health = self.check_whatsapp_api_health().await;
        health.components.insert("whatsapp_api".to_string(), whatsapp_health);
        
        // Check BitSacco API health
        let bitsacco_health = self.check_bitsacco_api_health().await;
        health.components.insert("bitsacco_api".to_string(), bitsacco_health);
        
        // Check BTC API health
        let btc_health = self.check_btc_api_health().await;
        health.components.insert("btc_api".to_string(), btc_health);
        
        // Check database/cache health
        let cache_health = self.check_cache_health().await;
        health.components.insert("cache".to_string(), cache_health);
        
        let check_duration = start_time.elapsed().as_millis() as u64;
        info!("Health check completed in {}ms", check_duration);
    }

    /// Check WhatsApp API health
    async fn check_whatsapp_api_health(&self) -> ComponentHealth {
        let start_time = Instant::now();
        
        // In production, this would make an actual API call
        // For now, we'll simulate a health check
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        
        let response_time = start_time.elapsed().as_millis() as u64;
        
        ComponentHealth {
            status: "healthy".to_string(),
            message: "WhatsApp API is responding normally".to_string(),
            last_check: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            response_time_ms: Some(response_time),
        }
    }

    /// Check BitSacco API health
    async fn check_bitsacco_api_health(&self) -> ComponentHealth {
        let start_time = Instant::now();
        
        // Simulate API health check
        tokio::time::sleep(tokio::time::Duration::from_millis(150)).await;
        
        let response_time = start_time.elapsed().as_millis() as u64;
        
        ComponentHealth {
            status: "healthy".to_string(),
            message: "BitSacco API is responding normally".to_string(),
            last_check: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            response_time_ms: Some(response_time),
        }
    }

    /// Check BTC API health
    async fn check_btc_api_health(&self) -> ComponentHealth {
        let start_time = Instant::now();
        
        // Simulate API health check
        tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
        
        let response_time = start_time.elapsed().as_millis() as u64;
        
        ComponentHealth {
            status: "healthy".to_string(),
            message: "BTC API is responding normally".to_string(),
            last_check: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            response_time_ms: Some(response_time),
        }
    }

    /// Check cache health
    async fn check_cache_health(&self) -> ComponentHealth {
        let start_time = Instant::now();
        
        // Simulate cache health check
        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
        
        let response_time = start_time.elapsed().as_millis() as u64;
        
        ComponentHealth {
            status: "healthy".to_string(),
            message: "Cache is operating normally".to_string(),
            last_check: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            response_time_ms: Some(response_time),
        }
    }

    /// Determine overall system health
    fn determine_overall_health(&self, health: &HealthStatus) -> String {
        let unhealthy_components: Vec<_> = health.components
            .values()
            .filter(|component| component.status != "healthy")
            .collect();
        
        if unhealthy_components.is_empty() {
            "healthy".to_string()
        } else if unhealthy_components.len() == 1 {
            "degraded".to_string()
        } else {
            "unhealthy".to_string()
        }
    }

    /// Get current memory usage (simplified)
    fn get_memory_usage(&self) -> f64 {
        // In production, this would use system APIs to get actual memory usage
        // For now, we'll return a simulated value
        128.5 // MB
    }

    /// Send an alert (placeholder implementation)
    async fn send_alert(&self, title: &str, message: &str) {
        // In production, this would integrate with alerting systems like:
        // - PagerDuty
        // - Slack
        // - Email notifications
        // - SMS alerts
        
        warn!("ALERT: {} - {}", title, message);
        
        // Example: Send to logging system
        error!("Alert sent: {} - {}", title, message);
    }

    /// Start the monitoring service
    pub async fn start_monitoring(&self) {
        info!("Starting monitoring service");
        
        let metrics = self.metrics.clone();
        let health_status = self.health_status.clone();
        let alert_config = self.alert_config.clone();
        let start_time = self.start_time;
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(alert_config.check_interval_seconds));
            
            loop {
                interval.tick().await;
                
                // Update metrics
                {
                    let mut metrics_guard = metrics.write().await;
                    metrics_guard.uptime_seconds = start_time.elapsed().as_secs();
                    metrics_guard.memory_usage_mb = 128.5; // Simulated
                }
                
                // Check for alerts
                let current_metrics = metrics.read().await;
                if current_metrics.total_requests > 0 {
                    let error_rate = current_metrics.failed_requests as f64 / current_metrics.total_requests as f64;
                    if error_rate > alert_config.error_rate_threshold {
                        warn!("High error rate: {:.2}%", error_rate * 100.0);
                    }
                }
                
                // Check response time
                if current_metrics.average_response_time_ms > alert_config.response_time_threshold_ms as f64 {
                    warn!("High response time: {:.2}ms", current_metrics.average_response_time_ms);
                }
                
                info!("Monitoring check completed");
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_monitoring_service_creation() {
        let monitoring = MonitoringService::new(None);
        assert_eq!(monitoring.get_metrics().await.total_requests, 0);
    }

    #[tokio::test]
    async fn test_record_successful_request() {
        let monitoring = MonitoringService::new(None);
        monitoring.record_successful_request(100).await;
        
        let metrics = monitoring.get_metrics().await;
        assert_eq!(metrics.total_requests, 1);
        assert_eq!(metrics.successful_requests, 1);
        assert_eq!(metrics.failed_requests, 0);
    }

    #[tokio::test]
    async fn test_record_failed_request() {
        let monitoring = MonitoringService::new(None);
        monitoring.record_failed_request("Test error").await;
        
        let metrics = monitoring.get_metrics().await;
        assert_eq!(metrics.total_requests, 1);
        assert_eq!(metrics.successful_requests, 0);
        assert_eq!(metrics.failed_requests, 1);
    }

    #[tokio::test]
    async fn test_health_status() {
        let monitoring = MonitoringService::new(None);
        let health = monitoring.get_health_status().await;
        
        assert_eq!(health.overall_health, "healthy");
        assert!(health.components.contains_key("whatsapp_api"));
        assert!(health.components.contains_key("bitsacco_api"));
        assert!(health.components.contains_key("btc_api"));
        assert!(health.components.contains_key("cache"));
    }
}
