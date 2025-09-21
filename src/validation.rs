use crate::error::{AppError, Result};
use regex::Regex;
use std::str::FromStr;

/// Validates phone number format (supports international format)
pub fn validate_phone_number(phone: &str) -> Result<()> {
    // Remove all non-digit characters except +
    let cleaned = phone.replace(|c: char| !c.is_ascii_digit() && c != '+', "");
    
    // Check if it starts with + and has 10-15 digits
    let phone_regex = Regex::new(r"^\+[1-9]\d{9,14}$").map_err(|e| {
        AppError::Internal(format!("Failed to compile phone regex: {}", e))
    })?;
    
    if !phone_regex.is_match(&cleaned) {
        return Err(AppError::Validation(format!(
            "Invalid phone number format: {}. Expected format: +1234567890",
            phone
        )));
    }
    
    Ok(())
}

/// Validates currency code (ISO 4217 format)
pub fn validate_currency(currency: &str) -> Result<()> {
    let currency_regex = Regex::new(r"^[A-Z]{3}$").map_err(|e| {
        AppError::Internal(format!("Failed to compile currency regex: {}", e))
    })?;
    
    if !currency_regex.is_match(currency) {
        return Err(AppError::Validation(format!(
            "Invalid currency code: {}. Expected 3-letter ISO code (e.g., USD, KES)",
            currency
        )));
    }
    
    Ok(())
}

/// Validates amount (positive number with reasonable limits)
pub fn validate_amount(amount: f64) -> Result<()> {
    if amount <= 0.0 {
        return Err(AppError::Validation(
            "Amount must be greater than 0".to_string()
        ));
    }
    
    if amount > 1_000_000.0 {
        return Err(AppError::Validation(
            "Amount exceeds maximum limit of 1,000,000".to_string()
        ));
    }
    
    // Check for reasonable decimal places (max 2 for most currencies)
    if (amount * 100.0).fract() != 0.0 {
        return Err(AppError::Validation(
            "Amount cannot have more than 2 decimal places".to_string()
        ));
    }
    
    Ok(())
}

/// Validates message content for security and length
pub fn validate_message(message: &str) -> Result<()> {
    if message.is_empty() {
        return Err(AppError::Validation("Message cannot be empty".to_string()));
    }
    
    if message.len() > 4096 {
        return Err(AppError::Validation(
            "Message exceeds maximum length of 4096 characters".to_string()
        ));
    }
    
    // Check for potential XSS or injection attempts
    let dangerous_patterns = [
        "<script", "javascript:", "data:", "vbscript:", "onload=", "onerror=",
        "eval(", "document.cookie", "window.location", "alert(",
    ];
    
    let message_lower = message.to_lowercase();
    for pattern in &dangerous_patterns {
        if message_lower.contains(pattern) {
            return Err(AppError::Validation(format!(
                "Message contains potentially dangerous content: {}",
                pattern
            )));
        }
    }
    
    Ok(())
}

/// Validates user ID format
pub fn validate_user_id(user_id: &str) -> Result<()> {
    if user_id.is_empty() {
        return Err(AppError::Validation("User ID cannot be empty".to_string()));
    }
    
    if user_id.len() > 100 {
        return Err(AppError::Validation(
            "User ID exceeds maximum length of 100 characters".to_string()
        ));
    }
    
    // Allow alphanumeric, hyphens, and underscores
    let user_id_regex = Regex::new(r"^[a-zA-Z0-9_-]+$").map_err(|e| {
        AppError::Internal(format!("Failed to compile user ID regex: {}", e))
    })?;
    
    if !user_id_regex.is_match(user_id) {
        return Err(AppError::Validation(format!(
            "Invalid user ID format: {}. Only alphanumeric characters, hyphens, and underscores are allowed",
            user_id
        )));
    }
    
    Ok(())
}

/// Sanitizes input by removing potentially dangerous characters
pub fn sanitize_input(input: &str) -> String {
    input
        .chars()
        .filter(|c| c.is_alphanumeric() || c.is_whitespace() || matches!(c, '-' | '_' | '.' | '@' | '+'))
        .collect()
}

/// Validates and parses amount from string
pub fn parse_and_validate_amount(amount_str: &str) -> Result<f64> {
    let amount = f64::from_str(amount_str).map_err(|_| {
        AppError::Validation(format!("Invalid amount format: {}", amount_str))
    })?;
    
    validate_amount(amount)?;
    Ok(amount)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_phone_number() {
        assert!(validate_phone_number("+254712345678").is_ok());
        assert!(validate_phone_number("+1234567890").is_ok());
        assert!(validate_phone_number("254712345678").is_err());
        assert!(validate_phone_number("+254").is_err());
        assert!(validate_phone_number("invalid").is_err());
    }

    #[test]
    fn test_validate_currency() {
        assert!(validate_currency("USD").is_ok());
        assert!(validate_currency("KES").is_ok());
        assert!(validate_currency("usd").is_err());
        assert!(validate_currency("US").is_err());
        assert!(validate_currency("USDD").is_err());
    }

    #[test]
    fn test_validate_amount() {
        assert!(validate_amount(100.0).is_ok());
        assert!(validate_amount(0.01).is_ok());
        assert!(validate_amount(0.0).is_err());
        assert!(validate_amount(-10.0).is_err());
        assert!(validate_amount(1_000_001.0).is_err());
        assert!(validate_amount(100.123).is_err());
    }

    #[test]
    fn test_validate_message() {
        assert!(validate_message("Hello world").is_ok());
        assert!(validate_message("").is_err());
        assert!(validate_message("<script>alert('xss')</script>").is_err());
        assert!(validate_message("javascript:alert('xss')").is_err());
    }
}
