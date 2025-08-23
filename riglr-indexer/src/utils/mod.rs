//! Utility modules and helper functions

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::time::Duration;
use tokio::time::sleep;

use crate::error::{IndexerError, IndexerResult};

pub mod batch;
pub mod consistent_hash;
pub mod health;
pub mod rate_limiter;

pub use batch::BatchProcessor;
pub use consistent_hash::ConsistentHash;
pub use health::HealthCheck;
pub use rate_limiter::RateLimiter;

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
                    attempt,
                    max_attempts,
                    error,
                    delay
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
    let mut hasher = DefaultHasher::default();
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
        return Err(IndexerError::validation(
            "Event ID too long (max 255 characters)",
        ));
    }

    // Basic validation - must contain only alphanumeric, hyphens, underscores
    if !id
        .chars()
        .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
    {
        return Err(IndexerError::validation(
            "Event ID must contain only alphanumeric characters, hyphens, and underscores",
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
        return Err(IndexerError::validation(
            "Source too long (max 100 characters)",
        ));
    }

    Ok(())
}

/// Validate event type format
pub fn validate_event_type(event_type: &str) -> IndexerResult<()> {
    if event_type.is_empty() {
        return Err(IndexerError::validation("Event type cannot be empty"));
    }

    if event_type.len() > 100 {
        return Err(IndexerError::validation(
            "Event type too long (max 100 characters)",
        ));
    }

    // Event type should follow a specific pattern (lowercase, underscores allowed)
    if !event_type
        .chars()
        .all(|c| c.is_ascii_lowercase() || c == '_' || c.is_ascii_digit())
    {
        return Err(IndexerError::validation(
            "Event type must contain only lowercase letters, underscores, and digits",
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

    // Additional comprehensive tests for 100% coverage

    #[tokio::test]
    async fn test_retry_with_backoff_success_first_attempt() {
        let mut counter = 0;
        let operation = || {
            counter += 1;
            Ok::<i32, &str>(42)
        };

        let result = retry_with_backoff(
            operation,
            3,
            Duration::from_millis(10),
            Duration::from_millis(100),
            2.0,
        )
        .await;

        assert_eq!(result, Ok(42));
        assert_eq!(counter, 1);
    }

    #[tokio::test]
    async fn test_retry_with_backoff_success_after_retries() {
        let mut counter = 0;
        let operation = || {
            counter += 1;
            if counter < 3 {
                Err("Temporary failure")
            } else {
                Ok::<i32, &str>(42)
            }
        };

        let result = retry_with_backoff(
            operation,
            5,
            Duration::from_millis(1),
            Duration::from_millis(10),
            2.0,
        )
        .await;

        assert_eq!(result, Ok(42));
        assert_eq!(counter, 3);
    }

    #[tokio::test]
    async fn test_retry_with_backoff_max_attempts_exceeded() {
        let mut counter = 0;
        let operation = || {
            counter += 1;
            Err::<i32, &str>("Persistent failure")
        };

        let result = retry_with_backoff(
            operation,
            3,
            Duration::from_millis(1),
            Duration::from_millis(10),
            2.0,
        )
        .await;

        assert_eq!(result, Err("Persistent failure"));
        assert_eq!(counter, 3);
    }

    #[tokio::test]
    async fn test_retry_with_backoff_delay_calculation() {
        let mut counter = 0;
        let operation = || {
            counter += 1;
            if counter < 2 {
                Err("Failure")
            } else {
                Ok::<i32, &str>(42)
            }
        };

        let start = std::time::Instant::now();
        let result = retry_with_backoff(
            operation,
            3,
            Duration::from_millis(10),
            Duration::from_millis(50),
            3.0,
        )
        .await;
        let elapsed = start.elapsed();

        assert_eq!(result, Ok(42));
        assert!(elapsed >= Duration::from_millis(10)); // At least one delay occurred
    }

    #[tokio::test]
    async fn test_retry_with_backoff_max_delay_capping() {
        let mut counter = 0;
        let operation = || {
            counter += 1;
            if counter < 3 {
                Err("Failure")
            } else {
                Ok::<i32, &str>(42)
            }
        };

        let result = retry_with_backoff(
            operation,
            5,
            Duration::from_millis(100),
            Duration::from_millis(50), // max_delay < initial_delay * multiplier
            10.0,
        )
        .await;

        assert_eq!(result, Ok(42));
    }

    #[test]
    fn test_hash_string_consistency() {
        let input = "test_string";
        let hash1 = hash_string(input);
        let hash2 = hash_string(input);
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_hash_string_different_inputs() {
        let hash1 = hash_string("input1");
        let hash2 = hash_string("input2");
        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_hash_string_empty() {
        let hash = hash_string("");
        assert!(hash != 0); // Empty string should still produce a hash
    }

    #[test]
    fn test_hash_string_special_characters() {
        let hash1 = hash_string("test@#$%");
        let hash2 = hash_string("test@#$%");
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_generate_id_uniqueness() {
        let id1 = generate_id();
        let id2 = generate_id();
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_generate_id_format() {
        let id = generate_id();
        assert_eq!(id.len(), 36); // UUID v4 format length
        assert!(id.contains('-'));
    }

    #[test]
    fn test_format_bytes_edge_cases() {
        assert_eq!(format_bytes(0), "0 B");
        assert_eq!(format_bytes(1), "1 B");
        assert_eq!(format_bytes(512), "512 B");
        assert_eq!(format_bytes(1023), "1023 B");
        assert_eq!(format_bytes(1024), "1.00 KB");
        assert_eq!(format_bytes(1025), "1.00 KB");
        assert_eq!(format_bytes(1536), "1.50 KB");
        assert_eq!(format_bytes(1024 * 1024), "1.00 MB");
        assert_eq!(format_bytes(1024 * 1024 * 1024), "1.00 GB");
        assert_eq!(format_bytes(1024_u64.pow(4)), "1.00 TB");
        assert_eq!(format_bytes(1024_u64.pow(5)), "1.00 PB");
        assert_eq!(format_bytes(1024_u64.pow(6)), "1024.00 PB"); // Beyond PB
        assert_eq!(
            format_bytes(u64::MAX),
            format!("{:.2} PB", u64::MAX as f64 / 1024_f64.powi(5))
        );
    }

    #[test]
    fn test_format_duration_edge_cases() {
        assert_eq!(format_duration(Duration::from_secs(0)), "0s");
        assert_eq!(format_duration(Duration::from_secs(1)), "1s");
        assert_eq!(format_duration(Duration::from_secs(59)), "59s");
        assert_eq!(format_duration(Duration::from_secs(60)), "1m");
        assert_eq!(format_duration(Duration::from_secs(61)), "1m1s");
        assert_eq!(format_duration(Duration::from_secs(119)), "1m59s");
        assert_eq!(format_duration(Duration::from_secs(120)), "2m");
        assert_eq!(format_duration(Duration::from_secs(3599)), "59m59s");
        assert_eq!(format_duration(Duration::from_secs(3600)), "1h");
        assert_eq!(format_duration(Duration::from_secs(3601)), "1h");
        assert_eq!(format_duration(Duration::from_secs(3660)), "1h1m");
        assert_eq!(format_duration(Duration::from_secs(7199)), "1h59m");
        assert_eq!(format_duration(Duration::from_secs(7200)), "2h");
        assert_eq!(format_duration(Duration::from_secs(86399)), "23h59m");
        assert_eq!(format_duration(Duration::from_secs(86400)), "1d");
        assert_eq!(format_duration(Duration::from_secs(86401)), "1d");
        assert_eq!(format_duration(Duration::from_secs(90000)), "1d1h");
        assert_eq!(format_duration(Duration::from_secs(172799)), "1d23h");
        assert_eq!(format_duration(Duration::from_secs(172800)), "2d");
    }

    #[test]
    fn test_validate_event_id_comprehensive() {
        // Valid cases
        assert!(validate_event_id("a").is_ok());
        assert!(validate_event_id("123").is_ok());
        assert!(validate_event_id("abc123").is_ok());
        assert!(validate_event_id("event-id").is_ok());
        assert!(validate_event_id("event_id").is_ok());
        assert!(validate_event_id("event-id_123").is_ok());
        assert!(validate_event_id("a".repeat(255).as_str()).is_ok());

        // Invalid cases - empty
        let result = validate_event_id("");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("cannot be empty"));

        // Invalid cases - too long
        let result = validate_event_id(&"a".repeat(256));
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("too long"));

        // Invalid cases - invalid characters
        let invalid_chars = [
            "event id", "event@id", "event#id", "event$id", "event%id", "event.id",
        ];
        for invalid in &invalid_chars {
            let result = validate_event_id(invalid);
            assert!(result.is_err());
            assert!(result.unwrap_err().to_string().contains("alphanumeric"));
        }
    }

    #[test]
    fn test_validate_source_comprehensive() {
        // Valid cases
        assert!(validate_source("a").is_ok());
        assert!(validate_source("jupiter").is_ok());
        assert!(validate_source("source with spaces").is_ok());
        assert!(validate_source("source@#$%").is_ok());
        assert!(validate_source(&"a".repeat(100)).is_ok());

        // Invalid cases - empty
        let result = validate_source("");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("cannot be empty"));

        // Invalid cases - too long
        let result = validate_source(&"a".repeat(101));
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("too long"));
    }

    #[test]
    fn test_validate_event_type_comprehensive() {
        // Valid cases
        assert!(validate_event_type("a").is_ok());
        assert!(validate_event_type("swap").is_ok());
        assert!(validate_event_type("token_transfer").is_ok());
        assert!(validate_event_type("swap_v2").is_ok());
        assert!(validate_event_type("event123").is_ok());
        assert!(validate_event_type("123").is_ok());
        assert!(validate_event_type("_").is_ok());
        assert!(validate_event_type(&"a".repeat(100)).is_ok());

        // Invalid cases - empty
        let result = validate_event_type("");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("cannot be empty"));

        // Invalid cases - too long
        let result = validate_event_type(&"a".repeat(101));
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("too long"));

        // Invalid cases - uppercase letters
        let result = validate_event_type("Swap");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("lowercase"));

        // Invalid cases - special characters
        let invalid_chars = [
            "swap-type",
            "swap type",
            "swap@type",
            "swap#type",
            "swap$type",
        ];
        for invalid in &invalid_chars {
            let result = validate_event_type(invalid);
            assert!(result.is_err());
            assert!(result.unwrap_err().to_string().contains("lowercase"));
        }
    }

    #[test]
    fn test_calculate_percentile_comprehensive() {
        // Single value
        let single = vec![5.0];
        assert_eq!(calculate_percentile(&single, 0.0), Some(5.0));
        assert_eq!(calculate_percentile(&single, 50.0), Some(5.0));
        assert_eq!(calculate_percentile(&single, 100.0), Some(5.0));

        // Two values
        let two = vec![1.0, 3.0];
        assert_eq!(calculate_percentile(&two, 0.0), Some(1.0));
        assert_eq!(calculate_percentile(&two, 25.0), Some(1.5));
        assert_eq!(calculate_percentile(&two, 50.0), Some(2.0));
        assert_eq!(calculate_percentile(&two, 75.0), Some(2.5));
        assert_eq!(calculate_percentile(&two, 100.0), Some(3.0));

        // Even number of values
        let even = vec![1.0, 2.0, 3.0, 4.0];
        assert_eq!(calculate_percentile(&even, 25.0), Some(1.75));
        assert_eq!(calculate_percentile(&even, 50.0), Some(2.5));
        assert_eq!(calculate_percentile(&even, 75.0), Some(3.25));

        // Odd number of values
        let odd = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        assert_eq!(calculate_percentile(&odd, 25.0), Some(2.0));
        assert_eq!(calculate_percentile(&odd, 50.0), Some(3.0));
        assert_eq!(calculate_percentile(&odd, 75.0), Some(4.0));

        // Test exact boundary conditions
        assert_eq!(calculate_percentile(&odd, 0.0), Some(1.0));
        assert_eq!(calculate_percentile(&odd, 100.0), Some(5.0));

        // Test invalid percentiles
        assert_eq!(calculate_percentile(&odd, -0.1), None);
        assert_eq!(calculate_percentile(&odd, 100.1), None);

        // Test empty vector
        let empty: Vec<f64> = vec![];
        assert_eq!(calculate_percentile(&empty, 50.0), None);

        // Test fractional percentiles that result in exact indices
        let values = vec![10.0, 20.0, 30.0, 40.0, 50.0];
        assert_eq!(calculate_percentile(&values, 25.0), Some(20.0)); // Index 1.0

        // Test fractional percentiles that require interpolation
        assert_eq!(calculate_percentile(&values, 37.5), Some(25.0)); // Index 1.5, interpolation between 20.0 and 30.0
    }
}
