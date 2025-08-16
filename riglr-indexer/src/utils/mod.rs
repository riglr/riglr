//! Utility modules and helper functions

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::time::Duration;
use tokio::time::sleep;

use crate::error::{IndexerError, IndexerResult};

pub mod batch;
pub mod rate_limiter;
pub mod consistent_hash;
pub mod health;

pub use batch::BatchProcessor;
pub use rate_limiter::RateLimiter;
pub use consistent_hash::ConsistentHash;
pub use health::HealthCheck;

/// Retry helper with exponential backoff
pub async fn retry_with_backoff<F, T, E>(
    mut operation: F,
    max_attempts: u32,
    initial_delay: Duration,
    max_delay: Duration,
    multiplier: f64,
) -> Result<T, E>
where
    F: FnMut() -> Result<T, E>,
    E: std::fmt::Display,
{
    let mut attempt = 0;
    let mut delay = initial_delay;

    loop {
        attempt += 1;
        
        match operation() {
            Ok(result) => return Ok(result),
            Err(error) => {
                if attempt >= max_attempts {
                    return Err(error);
                }
                
                tracing::warn!(
                    "Operation failed (attempt {}/{}): {}. Retrying in {:?}",
                    attempt, max_attempts, error, delay
                );
                
                sleep(delay).await;
                delay = std::cmp::min(
                    Duration::from_millis((delay.as_millis() as f64 * multiplier) as u64),
                    max_delay,
                );
            }
        }
    }
}

/// Simple hash function for consistent hashing
pub fn hash_string(s: &str) -> u64 {
    let mut hasher = DefaultHasher::new();
    s.hash(&mut hasher);
    hasher.finish()
}

/// Format bytes in human-readable format
pub fn format_bytes(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB", "PB"];
    let mut size = bytes as f64;
    let mut unit_index = 0;

    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }

    if unit_index == 0 {
        format!("{} {}", bytes, UNITS[unit_index])
    } else {
        format!("{:.2} {}", size, UNITS[unit_index])
    }
}

/// Format duration in human-readable format
pub fn format_duration(duration: Duration) -> String {
    let seconds = duration.as_secs();
    
    if seconds < 60 {
        format!("{}s", seconds)
    } else if seconds < 3600 {
        let minutes = seconds / 60;
        let remaining_seconds = seconds % 60;
        if remaining_seconds == 0 {
            format!("{}m", minutes)
        } else {
            format!("{}m{}s", minutes, remaining_seconds)
        }
    } else if seconds < 86400 {
        let hours = seconds / 3600;
        let remaining_minutes = (seconds % 3600) / 60;
        if remaining_minutes == 0 {
            format!("{}h", hours)
        } else {
            format!("{}h{}m", hours, remaining_minutes)
        }
    } else {
        let days = seconds / 86400;
        let remaining_hours = (seconds % 86400) / 3600;
        if remaining_hours == 0 {
            format!("{}d", days)
        } else {
            format!("{}d{}h", days, remaining_hours)
        }
    }
}

/// Generate a unique ID
pub fn generate_id() -> String {
    uuid::Uuid::new_v4().to_string()
}

/// Validate event ID format
pub fn validate_event_id(id: &str) -> IndexerResult<()> {
    if id.is_empty() {
        return Err(IndexerError::validation("Event ID cannot be empty"));
    }
    
    if id.len() > 255 {
        return Err(IndexerError::validation("Event ID too long (max 255 characters)"));
    }
    
    // Basic validation - must contain only alphanumeric, hyphens, underscores
    if !id.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_') {
        return Err(IndexerError::validation(
            "Event ID must contain only alphanumeric characters, hyphens, and underscores"
        ));
    }
    
    Ok(())
}

/// Validate source format
pub fn validate_source(source: &str) -> IndexerResult<()> {
    if source.is_empty() {
        return Err(IndexerError::validation("Source cannot be empty"));
    }
    
    if source.len() > 100 {
        return Err(IndexerError::validation("Source too long (max 100 characters)"));
    }
    
    Ok(())
}

/// Validate event type format
pub fn validate_event_type(event_type: &str) -> IndexerResult<()> {
    if event_type.is_empty() {
        return Err(IndexerError::validation("Event type cannot be empty"));
    }
    
    if event_type.len() > 100 {
        return Err(IndexerError::validation("Event type too long (max 100 characters)"));
    }
    
    // Event type should follow a specific pattern (lowercase, underscores allowed)
    if !event_type.chars().all(|c| c.is_ascii_lowercase() || c == '_' || c.is_ascii_digit()) {
        return Err(IndexerError::validation(
            "Event type must contain only lowercase letters, underscores, and digits"
        ));
    }
    
    Ok(())
}

/// Calculate percentile from a sorted vector of values
pub fn calculate_percentile(sorted_values: &[f64], percentile: f64) -> Option<f64> {
    if sorted_values.is_empty() || !(0.0..=100.0).contains(&percentile) {
        return None;
    }
    
    if percentile == 0.0 {
        return Some(sorted_values[0]);
    }
    
    if percentile == 100.0 {
        return Some(sorted_values[sorted_values.len() - 1]);
    }
    
    let index = (percentile / 100.0) * (sorted_values.len() as f64 - 1.0);
    let lower = index.floor() as usize;
    let upper = index.ceil() as usize;
    
    if lower == upper {
        Some(sorted_values[lower])
    } else {
        let weight = index - lower as f64;
        Some(sorted_values[lower] * (1.0 - weight) + sorted_values[upper] * weight)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_bytes() {
        assert_eq!(format_bytes(0), "0 B");
        assert_eq!(format_bytes(1023), "1023 B");
        assert_eq!(format_bytes(1024), "1.00 KB");
        assert_eq!(format_bytes(1024 * 1024), "1.00 MB");
        assert_eq!(format_bytes(1536), "1.50 KB");
    }

    #[test]
    fn test_format_duration() {
        assert_eq!(format_duration(Duration::from_secs(30)), "30s");
        assert_eq!(format_duration(Duration::from_secs(60)), "1m");
        assert_eq!(format_duration(Duration::from_secs(90)), "1m30s");
        assert_eq!(format_duration(Duration::from_secs(3600)), "1h");
        assert_eq!(format_duration(Duration::from_secs(3660)), "1h1m");
        assert_eq!(format_duration(Duration::from_secs(86400)), "1d");
        assert_eq!(format_duration(Duration::from_secs(90000)), "1d1h");
    }

    #[test]
    fn test_validate_event_id() {
        assert!(validate_event_id("valid-id_123").is_ok());
        assert!(validate_event_id("").is_err());
        assert!(validate_event_id("invalid id with spaces").is_err());
        assert!(validate_event_id(&"x".repeat(256)).is_err());
    }

    #[test]
    fn test_validate_source() {
        assert!(validate_source("jupiter").is_ok());
        assert!(validate_source("").is_err());
        assert!(validate_source(&"x".repeat(101)).is_err());
    }

    #[test]
    fn test_validate_event_type() {
        assert!(validate_event_type("swap").is_ok());
        assert!(validate_event_type("token_transfer").is_ok());
        assert!(validate_event_type("swap_v2").is_ok());
        assert!(validate_event_type("").is_err());
        assert!(validate_event_type("Invalid-Type").is_err());
        assert!(validate_event_type("invalid type").is_err());
    }

    #[test]
    fn test_calculate_percentile() {
        let values = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        assert_eq!(calculate_percentile(&values, 0.0), Some(1.0));
        assert_eq!(calculate_percentile(&values, 50.0), Some(3.0));
        assert_eq!(calculate_percentile(&values, 100.0), Some(5.0));
        
        let empty: Vec<f64> = vec![];
        assert_eq!(calculate_percentile(&empty, 50.0), None);
        assert_eq!(calculate_percentile(&values, -1.0), None);
        assert_eq!(calculate_percentile(&values, 101.0), None);
    }
}