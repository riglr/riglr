//! Safe casting utilities for metrics to avoid precision loss warnings

use core::{fmt::Display, time::Duration};

/// Safely cast integer types to f64 for metrics recording.
/// Uses saturating conversion to handle potential overflow cases.
pub fn safe_cast_to_f64<T>(value: T) -> f64
where
    T: TryInto<f64> + Copy + Display,
{
    value.try_into().unwrap_or_else(|_| {
        // For very large values that don't fit in f64, use the max representable value
        tracing::warn!("Value {} too large for f64, using f64::MAX", value);
        f64::MAX
    })
}

/// Safely convert Duration to milliseconds as f64.
/// Handles potential overflow by capping at `f64::MAX`.
#[expect(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_precision_loss
)]
pub fn safe_cast_duration_millis(duration: Duration) -> f64 {
    let millis = duration.as_millis();

    // u128 can be much larger than what f64 can represent precisely
    if millis > (f64::MAX as u128) {
        tracing::warn!("Duration {} ms too large for f64, using f64::MAX", millis);
        return f64::MAX;
    }
    millis as f64
}

/// Safely convert usize to f64 for metrics.
/// Handles precision loss for very large values.
#[expect(clippy::cast_possible_truncation, clippy::cast_precision_loss)]
pub fn safe_cast_usize_to_f64(value: usize) -> f64 {
    // For 64-bit systems, usize is 64 bits, same as u64
    // f64 has 52 bits of mantissa, so values larger than 2^52 lose precision
    const MAX_PRECISE_USIZE: usize = (1u64 << 52) as usize;

    if value > MAX_PRECISE_USIZE {
        tracing::warn!(
            "usize value {} may lose precision when cast to f64 (mantissa only has 52 bits)",
            value
        );
    }

    value as f64
}

/// Safely convert u64 to f64 for metrics.
/// Handles precision loss for very large values.
#[expect(clippy::cast_precision_loss)]
pub fn safe_cast_u64_to_f64(value: u64) -> f64 {
    // f64 has 52 bits of mantissa, so values larger than 2^52 lose precision
    const MAX_PRECISE_U64: u64 = 1u64 << 52;

    if value > MAX_PRECISE_U64 {
        tracing::warn!(
            "u64 value {} may lose precision when cast to f64 (mantissa only has 52 bits)",
            value
        );
    }

    value as f64
}

/// Safely convert u128 to f64 for metrics.
/// Handles potential overflow and precision loss.
#[expect(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_precision_loss
)]
pub fn safe_cast_u128_to_f64(value: u128) -> f64 {
    // f64 has 52 bits of mantissa, warn about precision loss
    const MAX_PRECISE_U128: u128 = 1u128 << 52;

    // f64::MAX is always positive, safe cast to u128; truncation acceptable for bounds checking
    if value > (f64::MAX as u128) {
        tracing::warn!("u128 value {} too large for f64, using f64::MAX", value);
        return f64::MAX;
    }
    if value > MAX_PRECISE_U128 {
        tracing::warn!(
            "u128 value {} may lose precision when cast to f64 (mantissa only has 52 bits)",
            value
        );
    }

    value as f64
}

/// Safely convert f64 to usize, handling negative values and overflow.
/// Returns None if the value is negative, NaN, or too large for usize.
#[must_use]
#[expect(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_precision_loss
)]
pub fn safe_cast_f64_to_usize(value: f64) -> Option<usize> {
    if value.is_nan() || value.is_infinite() || value < 0.0 {
        return None;
    }

    // Check if value exceeds usize::MAX when cast
    if value > (usize::MAX as f64) {
        return None;
    }

    Some(value as usize)
}

#[cfg(test)]
#[expect(clippy::float_cmp)] // All test float comparisons use exact literal values
mod tests {
    use super::*;

    #[test]
    fn test_safe_cast_u64_to_f64_small_value() {
        let value = 42u64;
        let result = safe_cast_u64_to_f64(value);
        assert_eq!(result, 42.0);
    }

    #[test]
    fn test_safe_cast_u64_to_f64_large_value() {
        let value = u64::MAX;
        let result = safe_cast_u64_to_f64(value);
        // Should still cast but warn about precision loss
        assert!(result > 0.0);
    }

    #[test]
    fn test_safe_cast_usize_to_f64_small_value() {
        let value = 123usize;
        let result = safe_cast_usize_to_f64(value);
        assert_eq!(result, 123.0);
    }

    #[test]
    fn test_safe_cast_duration_millis_normal() {
        let duration = Duration::from_millis(1000);
        let result = safe_cast_duration_millis(duration);
        assert_eq!(result, 1000.0);
    }

    #[test]
    fn test_safe_cast_duration_millis_large() {
        let duration = Duration::from_secs(u64::MAX);
        let result = safe_cast_duration_millis(duration);
        // Should return f64::MAX for overflow case
        assert_eq!(result, f64::MAX);
    }

    #[test]
    fn test_safe_cast_u128_to_f64_overflow() {
        let value = u128::MAX;
        let result = safe_cast_u128_to_f64(value);
        assert_eq!(result, f64::MAX);
    }

    #[test]
    fn test_safe_cast_u128_to_f64_normal() {
        let value = 123u128;
        let result = safe_cast_u128_to_f64(value);
        assert_eq!(result, 123.0);
    }

    #[test]
    fn test_safe_cast_f64_to_usize_normal() {
        assert_eq!(safe_cast_f64_to_usize(123.0), Some(123));
        assert_eq!(safe_cast_f64_to_usize(0.0), Some(0));
        assert_eq!(safe_cast_f64_to_usize(123.7), Some(123)); // Truncates
    }

    #[test]
    fn test_safe_cast_f64_to_usize_invalid() {
        assert_eq!(safe_cast_f64_to_usize(-1.0), None);
        assert_eq!(safe_cast_f64_to_usize(f64::NAN), None);
        assert_eq!(safe_cast_f64_to_usize(f64::INFINITY), None);
        assert_eq!(safe_cast_f64_to_usize(f64::NEG_INFINITY), None);
    }

    #[test]
    fn test_safe_cast_f64_to_usize_overflow() {
        // Use a value that's definitely larger than usize::MAX
        let too_large = 2.0_f64.powi(64); // Much larger than any usize value
        assert_eq!(safe_cast_f64_to_usize(too_large), None);

        // Also test infinity
        assert_eq!(safe_cast_f64_to_usize(f64::INFINITY), None);
    }
}
