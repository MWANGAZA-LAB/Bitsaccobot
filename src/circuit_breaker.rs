use crate::error::{AppError, Result};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;

/// Circuit breaker configuration
#[derive(Debug, Clone)]
pub struct CircuitBreakerConfig {
    pub failure_threshold: u32,
    pub recovery_timeout: Duration,
    pub half_open_max_calls: u32,
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self {
            failure_threshold: 5,
            recovery_timeout: Duration::from_secs(30),
            half_open_max_calls: 3,
        }
    }
}

/// Simple circuit breaker state
#[derive(Debug, Clone, PartialEq)]
pub enum CircuitState {
    Closed,
    Open,
    HalfOpen,
}

/// Simple circuit breaker implementation
#[derive(Debug, Clone)]
pub struct SimpleCircuitBreaker {
    state: CircuitState,
    failure_count: u32,
    failure_threshold: u32,
    last_failure_time: Option<Instant>,
    recovery_timeout: Duration,
}

impl SimpleCircuitBreaker {
    pub fn new(failure_threshold: u32, recovery_timeout: Duration) -> Self {
        Self {
            state: CircuitState::Closed,
            failure_count: 0,
            failure_threshold,
            last_failure_time: None,
            recovery_timeout,
        }
    }

    pub async fn call<F, T>(&mut self, f: F) -> Result<T>
    where
        F: FnOnce() -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<T>> + Send>>,
    {
        match self.state {
            CircuitState::Open => {
                if let Some(last_failure) = self.last_failure_time {
                    if Instant::now().duration_since(last_failure) >= self.recovery_timeout {
                        self.state = CircuitState::HalfOpen;
                    } else {
                        return Err(AppError::Internal("Circuit breaker is open".to_string()));
                    }
                } else {
                    return Err(AppError::Internal("Circuit breaker is open".to_string()));
                }
            }
            CircuitState::HalfOpen => {
                // Allow limited calls in half-open state
            }
            CircuitState::Closed => {
                // Normal operation
            }
        }

        let result = f().await;
        
        match result {
            Ok(value) => {
                // Success - reset circuit breaker
                self.state = CircuitState::Closed;
                self.failure_count = 0;
                self.last_failure_time = None;
                Ok(value)
            }
            Err(e) => {
                // Failure - increment counter
                self.failure_count += 1;
                self.last_failure_time = Some(Instant::now());
                
                if self.failure_count >= self.failure_threshold {
                    self.state = CircuitState::Open;
                }
                
                Err(e)
            }
        }
    }

    pub fn state(&self) -> CircuitState {
        self.state.clone()
    }
}

/// Circuit breaker service for external API calls
#[derive(Debug, Clone)]
pub struct ApiCircuitBreaker {
    whatsapp_breaker: Arc<Mutex<SimpleCircuitBreaker>>,
    bitsacco_breaker: Arc<Mutex<SimpleCircuitBreaker>>,
    btc_breaker: Arc<Mutex<SimpleCircuitBreaker>>,
}

impl ApiCircuitBreaker {
    pub fn new(config: CircuitBreakerConfig) -> Self {
        let whatsapp_breaker = Arc::new(Mutex::new(SimpleCircuitBreaker::new(
            config.failure_threshold,
            config.recovery_timeout,
        )));
        let bitsacco_breaker = Arc::new(Mutex::new(SimpleCircuitBreaker::new(
            config.failure_threshold,
            config.recovery_timeout,
        )));
        let btc_breaker = Arc::new(Mutex::new(SimpleCircuitBreaker::new(
            config.failure_threshold,
            config.recovery_timeout,
        )));

        Self {
            whatsapp_breaker,
            bitsacco_breaker,
            btc_breaker,
        }
    }

    /// Execute a WhatsApp API call with circuit breaker protection
    pub async fn call_whatsapp_api<F, T>(&self, f: F) -> Result<T>
    where
        F: FnOnce() -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<T>> + Send>>,
    {
        let mut breaker = self.whatsapp_breaker.lock().await;
        breaker.call(f).await
    }

    /// Execute a BitSacco API call with circuit breaker protection
    pub async fn call_bitsacco_api<F, T>(&self, f: F) -> Result<T>
    where
        F: FnOnce() -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<T>> + Send>>,
    {
        let mut breaker = self.bitsacco_breaker.lock().await;
        breaker.call(f).await
    }

    /// Execute a BTC API call with circuit breaker protection
    pub async fn call_btc_api<F, T>(&self, f: F) -> Result<T>
    where
        F: FnOnce() -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<T>> + Send>>,
    {
        let mut breaker = self.btc_breaker.lock().await;
        breaker.call(f).await
    }

    /// Get circuit breaker status for monitoring
    pub async fn get_status(&self) -> CircuitBreakerStatus {
        let whatsapp_status = self.whatsapp_breaker.lock().await.state();
        let bitsacco_status = self.bitsacco_breaker.lock().await.state();
        let btc_status = self.btc_breaker.lock().await.state();

        CircuitBreakerStatus {
            whatsapp: whatsapp_status,
            bitsacco: bitsacco_status,
            btc: btc_status,
        }
    }
}

/// Circuit breaker status for monitoring
#[derive(Debug, Clone)]
pub struct CircuitBreakerStatus {
    pub whatsapp: CircuitState,
    pub bitsacco: CircuitState,
    pub btc: CircuitState,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[tokio::test]
    async fn test_circuit_breaker_creation() {
        let config = CircuitBreakerConfig {
            failure_threshold: 3,
            recovery_timeout: Duration::from_secs(10),
            half_open_max_calls: 2,
        };
        
        let breaker = ApiCircuitBreaker::new(config);
        let status = breaker.get_status().await;
        
        // All circuit breakers should start in closed state
        assert_eq!(status.whatsapp, CircuitState::Closed);
        assert_eq!(status.bitsacco, CircuitState::Closed);
        assert_eq!(status.btc, CircuitState::Closed);
    }

    #[tokio::test]
    async fn test_circuit_breaker_success() {
        let breaker = ApiCircuitBreaker::new(CircuitBreakerConfig::default());
        
        let result = breaker.call_whatsapp_api(|| {
            Box::pin(async { Ok("success".to_string()) })
        }).await;
        
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "success");
    }

    #[tokio::test]
    async fn test_circuit_breaker_failure() {
        let config = CircuitBreakerConfig {
            failure_threshold: 2,
            recovery_timeout: Duration::from_secs(1),
            half_open_max_calls: 1,
        };
        
        let breaker = ApiCircuitBreaker::new(config);
        
        // First failure
        let result: Result<String> = breaker.call_whatsapp_api(|| {
            Box::pin(async { Err(AppError::WhatsApp("API error".to_string())) })
        }).await;
        
        assert!(result.is_err());
        
        // Second failure should open the circuit
        let result: Result<String> = breaker.call_whatsapp_api(|| {
            Box::pin(async { Err(AppError::WhatsApp("API error".to_string())) })
        }).await;
        
        assert!(result.is_err());
        
        // Third call should be rejected by circuit breaker
        let result: Result<String> = breaker.call_whatsapp_api(|| {
            Box::pin(async { Ok("should not reach here".to_string()) })
        }).await;
        
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Circuit breaker is open"));
    }
}
