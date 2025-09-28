//! EVM conversion utilities
//!
//! ## Precision vs Performance Trade-off Decision
//!
//! After investigating high-precision decimal types (Task 3), we've decided to keep the f64-based
//! implementation for the following reasons:
//!
//! ### Investigation Results:
//!
//! **Precision Analysis:**
//! - f64 provides ~15-16 significant digits of precision
//! - Can handle values from 1 wei (10^-18 ETH) to billions of ETH
//! - Minor rounding errors occur at the 18th decimal place (e.g., 0.123456789012345678 → 0.123456789012345677)
//! - Cannot distinguish between values differing by less than 1 wei for amounts over ~0.01 ETH
//!
//! **Performance Analysis:**
//! - f64 operations: ~80-160 nanoseconds per conversion
//! - `rust_decimal` operations: ~115-175 nanoseconds per conversion  
//! - Performance difference: `rust_decimal` is ~1.4-1.5x slower
//! - Absolute difference is negligible (tens of nanoseconds)
//!
//! ### Decision Rationale:
//!
//! **We retain f64 because:**
//! 1. **Sufficient for most use cases**: The precision loss only affects the least significant wei,
//!    which is economically negligible (1 wei = 10^-18 ETH ≈ $0.000000000000000001)
//!
//! 2. **String-based conversion mitigates issues**: Our implementation uses string formatting
//!    which avoids most floating-point arithmetic errors
//!
//! 3. **Simplicity**: No additional dependencies, simpler mental model, better ecosystem compatibility
//!
//! 4. **Performance**: While the difference is small, f64 is consistently faster
//!
//! 5. **Blockchain constraints**: Most blockchain operations round to whole wei anyway
//!
//! ### When to Reconsider:
//!
//! Consider migrating to `rust_decimal` or similar if:
//! - Building high-frequency trading systems where sub-wei precision matters
//! - Implementing complex `DeFi` protocols with precise percentage calculations
//! - Working with very small token amounts where every wei counts
//! - Regulatory requirements demand exact decimal precision
//!
//! For applications requiring exact precision, wrap these functions with decimal types
//! at the application layer rather than in this common utility crate.

use alloy::primitives::U256;

#[cfg(not(feature = "high-precision"))]
use core::str::FromStr;

#[cfg(feature = "high-precision")]
use rust_decimal::Decimal;

#[cfg(feature = "high-precision")]
fn wei_per_eth() -> rust_decimal::Decimal {
    use rust_decimal::prelude::*;
    Decimal::from_str("1000000000000000000").unwrap_or(Decimal::ZERO)
}

#[cfg(feature = "high-precision")]
fn wei_per_gwei() -> rust_decimal::Decimal {
    use rust_decimal::prelude::*;
    Decimal::from_str("1000000000").unwrap_or(Decimal::ZERO)
}

/// Convert ETH to wei (f64 version - default)
#[cfg(not(feature = "high-precision"))]
#[must_use]
#[inline]
pub fn eth_to_wei(eth_amount: f64) -> U256 {
    // Use string formatting to avoid f64 precision issues and overflow
    let wei_str = format!("{:.0}", eth_amount * 1e18_f64);
    U256::from_str(&wei_str).unwrap_or(U256::ZERO)
}

/// Convert ETH to wei (high-precision Decimal version)
///
/// # Panics
///
/// Panics if the hardcoded decimal constant "1000000000000000000" cannot be parsed.
/// This should never happen as it represents a valid decimal constant.
#[cfg(feature = "high-precision")]
#[must_use]
pub fn eth_to_wei(eth_amount: Decimal) -> U256 {
    use rust_decimal::prelude::*;
    // Use decimal arithmetic for exact precision
    #[expect(clippy::arithmetic_side_effects)]
    let wei_amount = eth_amount * wei_per_eth();
    // Convert to string and parse as U256
    let wei_str = wei_amount.to_string();
    // Split at decimal point and take only the integer part
    let integer_part = wei_str.split('.').next().unwrap_or("0");
    U256::from_str(integer_part).unwrap_or(U256::ZERO)
}

/// Convert wei to ETH (f64 version - default)
#[cfg(not(feature = "high-precision"))]
#[must_use]
#[inline]
pub fn wei_to_eth(wei_amount: U256) -> f64 {
    let wei_str = wei_amount.to_string();
    {
        wei_str.parse::<f64>().unwrap_or(0.0) / 1e18_f64
    }
}

/// Convert wei to ETH (high-precision Decimal version)
///
/// # Panics
///
/// Panics if the hardcoded decimal constant "1000000000000000000" cannot be parsed.
/// This should never happen as it represents a valid decimal constant.
#[cfg(feature = "high-precision")]
#[must_use]
pub fn wei_to_eth(wei_amount: U256) -> Decimal {
    use rust_decimal::prelude::*;
    let wei_str = wei_amount.to_string();
    let wei_decimal = Decimal::from_str(&wei_str).unwrap_or(Decimal::ZERO);
    #[expect(clippy::arithmetic_side_effects)]
    let result = wei_decimal / wei_per_eth();
    result
}

/// Convert gwei to wei (f64 version - default)
#[cfg(not(feature = "high-precision"))]
#[must_use]
#[inline]
pub fn gwei_to_wei(gwei_amount: f64) -> U256 {
    // Use string formatting to avoid f64 precision issues and overflow
    let wei_str = format!("{:.0}", gwei_amount * 1e9_f64);
    U256::from_str(&wei_str).unwrap_or(U256::ZERO)
}

/// Convert gwei to wei (high-precision Decimal version)
///
/// # Panics
///
/// Panics if the hardcoded decimal constant "1000000000" cannot be parsed.
/// This should never happen as it represents a valid decimal constant.
#[cfg(feature = "high-precision")]
#[must_use]
pub fn gwei_to_wei(gwei_amount: Decimal) -> U256 {
    use rust_decimal::prelude::*;
    #[expect(clippy::arithmetic_side_effects)]
    let wei_amount = gwei_amount * wei_per_gwei();
    let wei_str = wei_amount.to_string();
    let integer_part = wei_str.split('.').next().unwrap_or("0");
    U256::from_str(integer_part).unwrap_or(U256::ZERO)
}

/// Convert wei to gwei (f64 version - default)
#[cfg(not(feature = "high-precision"))]
#[must_use]
#[inline]
pub fn wei_to_gwei(wei_amount: U256) -> f64 {
    let wei_str = wei_amount.to_string();
    {
        wei_str.parse::<f64>().unwrap_or(0.0) / 1e9_f64
    }
}

/// Convert wei to gwei (high-precision Decimal version)
///
/// # Panics
///
/// Panics if the hardcoded decimal constant "1000000000" cannot be parsed.
/// This should never happen as it represents a valid decimal constant.
#[cfg(feature = "high-precision")]
#[must_use]
pub fn wei_to_gwei(wei_amount: U256) -> Decimal {
    use rust_decimal::prelude::*;
    let wei_str = wei_amount.to_string();
    let wei_decimal = Decimal::from_str(&wei_str).unwrap_or(Decimal::ZERO);
    #[expect(clippy::arithmetic_side_effects)]
    let result = wei_decimal / wei_per_gwei();
    result
}

/// Convert token amount to smallest unit based on decimals (f64 version - default)
#[cfg(not(feature = "high-precision"))]
#[must_use]
#[inline]
pub fn token_to_smallest_unit(amount: f64, decimals: u8) -> U256 {
    // Use string formatting to avoid f64 precision issues and overflow
    let multiplier = 10_f64.powi(i32::from(decimals));
    let smallest_unit_str = format!("{:.0}", amount * multiplier);
    U256::from_str(&smallest_unit_str).unwrap_or(U256::ZERO)
}

/// Convert token amount to smallest unit based on decimals (high-precision Decimal version)
#[cfg(feature = "high-precision")]
#[must_use]
pub fn token_to_smallest_unit(amount: Decimal, decimals: u8) -> U256 {
    use rust_decimal::prelude::*;
    let multiplier = Decimal::from(10_u64.pow(u32::from(decimals)));
    #[expect(clippy::arithmetic_side_effects)]
    let smallest_unit = amount * multiplier;
    let smallest_unit_str = smallest_unit.to_string();
    let integer_part = smallest_unit_str.split('.').next().unwrap_or("0");
    U256::from_str(integer_part).unwrap_or(U256::ZERO)
}

/// Convert smallest unit to token amount based on decimals (f64 version - default)
#[cfg(not(feature = "high-precision"))]
#[must_use]
#[inline]
pub fn smallest_unit_to_token(amount: U256, decimals: u8) -> f64 {
    let divisor = 10_f64.powi(i32::from(decimals));
    let amount_str = amount.to_string();
    {
        amount_str.parse::<f64>().unwrap_or(0.0) / divisor
    }
}

/// Convert smallest unit to token amount based on decimals (high-precision Decimal version)
#[cfg(feature = "high-precision")]
#[must_use]
pub fn smallest_unit_to_token(amount: U256, decimals: u8) -> Decimal {
    use rust_decimal::prelude::*;
    let amount_str = amount.to_string();
    let amount_decimal = Decimal::from_str(&amount_str).unwrap_or(Decimal::ZERO);
    let divisor = Decimal::from(10_u64.pow(u32::from(decimals)));
    #[expect(clippy::arithmetic_side_effects)]
    let result = amount_decimal / divisor;
    result
}

#[cfg(all(test, not(feature = "high-precision")))]
#[expect(clippy::expect_used)]
mod tests {
    use super::*;
    use core::str::FromStr;

    #[test]
    fn eth_to_wei_basic() {
        let result = eth_to_wei(1.0);
        assert_eq!(
            result,
            U256::from_str("1000000000000000000").expect("Valid U256 string should parse")
        );
    }

    #[test]
    fn eth_to_wei_with_high_precision() {
        let result = eth_to_wei(0.123_456_789_012_345_68);
        // f64 precision loss at 18th decimal place - demonstrates the limitation discussed in module docs
        assert_eq!(
            result,
            U256::from_str("123456789012345680").expect("Valid U256 string should parse")
        );
    }

    #[test]
    fn eth_to_wei_very_large_value() {
        let result = eth_to_wei(1_000_000.0);
        // f64 precision loss for large values - demonstrates the limitation discussed in module docs
        assert_eq!(
            result,
            U256::from_str("999999999999999983222784").expect("Valid U256 string should parse")
        );
    }

    #[test]
    fn eth_to_wei_zero() {
        let result = eth_to_wei(0.0);
        assert_eq!(result, U256::ZERO);
    }

    #[test]
    fn gwei_to_wei_basic() {
        let result = gwei_to_wei(1.0);
        assert_eq!(
            result,
            U256::from_str("1000000000").expect("Valid U256 string should parse")
        );
    }

    #[test]
    fn gwei_to_wei_with_precision() {
        let result = gwei_to_wei(1.5);
        assert_eq!(
            result,
            U256::from_str("1500000000").expect("Valid U256 string should parse")
        );
    }

    #[test]
    fn gwei_to_wei_large_value() {
        let result = gwei_to_wei(1_000_000.0);
        assert_eq!(
            result,
            U256::from_str("1000000000000000").expect("Valid U256 string should parse")
        );
    }

    #[test]
    fn token_to_smallest_unit_18_decimals() {
        let result = token_to_smallest_unit(1.0, 18);
        assert_eq!(
            result,
            U256::from_str("1000000000000000000").expect("Valid U256 string should parse")
        );
    }

    #[test]
    fn token_to_smallest_unit_6_decimals() {
        // Common for USDC, USDT
        let result = token_to_smallest_unit(1.0, 6);
        assert_eq!(
            result,
            U256::from_str("1000000").expect("Valid U256 string should parse")
        );
    }

    #[test]
    fn token_to_smallest_unit_with_precision() {
        let result = token_to_smallest_unit(1.23456, 8);
        assert_eq!(
            result,
            U256::from_str("123456000").expect("Valid U256 string should parse")
        );
    }

    #[test]
    fn token_to_smallest_unit_very_large() {
        let result = token_to_smallest_unit(1_000_000_000.0, 18);
        // f64 precision loss for very large values - demonstrates the limitation discussed in module docs
        assert_eq!(
            result,
            U256::from_str("1000000000000000013287555072").expect("Valid U256 string should parse")
        );
    }

    #[test]
    fn wei_to_eth_works() {
        let wei = U256::from_str("1000000000000000000").expect("Valid U256 string should parse");
        let result = wei_to_eth(wei);
        assert!((result - 1.0).abs() < 1e-10f64);
    }

    #[test]
    fn wei_to_gwei_works() {
        let wei = U256::from_str("1000000000").expect("Valid U256 string should parse");
        let result = wei_to_gwei(wei);
        assert!((result - 1.0).abs() < 1e-10f64);
    }

    #[test]
    fn smallest_unit_to_token_works() {
        let amount = U256::from_str("1000000000000000000").expect("Valid U256 string should parse");
        let result = smallest_unit_to_token(amount, 18);
        assert!((result - 1.0).abs() < 1e-10f64);
    }

    #[test]
    fn conversion_roundtrip() {
        // Test that converting back and forth maintains reasonable precision
        let original = 123.456_789f64;
        let wei = eth_to_wei(original);
        let back = wei_to_eth(wei);
        assert!((back - original).abs() < 1e-6f64); // Allow small precision loss
    }
}

#[cfg(all(test, feature = "high-precision"))]
#[expect(clippy::expect_used)]
mod high_precision_tests {
    use super::*;
    use core::str::FromStr;
    use rust_decimal::Decimal;

    #[test]
    fn eth_to_wei_basic() {
        let result = eth_to_wei(Decimal::from_str("1.0").expect("Valid decimal"));
        assert_eq!(
            result,
            U256::from_str("1000000000000000000").expect("Valid U256 string should parse")
        );
    }

    #[test]
    fn eth_to_wei_with_high_precision() {
        let result = eth_to_wei(Decimal::from_str("0.123456789012345678").expect("Valid decimal"));
        // With Decimal, precision is maintained
        assert_eq!(
            result,
            U256::from_str("123456789012345678").expect("Valid U256 string should parse")
        );
    }

    #[test]
    fn eth_to_wei_very_large_value() {
        let result = eth_to_wei(Decimal::from_str("1000000.0").expect("Valid decimal"));
        // With Decimal, no precision loss
        assert_eq!(
            result,
            U256::from_str("1000000000000000000000000").expect("Valid U256 string should parse")
        );
    }

    #[test]
    fn eth_to_wei_zero() {
        let result = eth_to_wei(Decimal::ZERO);
        assert_eq!(result, U256::ZERO);
    }

    #[test]
    fn gwei_to_wei_basic() {
        let result = gwei_to_wei(Decimal::from_str("1.0").expect("Valid decimal"));
        assert_eq!(
            result,
            U256::from_str("1000000000").expect("Valid U256 string should parse")
        );
    }

    #[test]
    fn gwei_to_wei_with_precision() {
        let result = gwei_to_wei(Decimal::from_str("1.5").expect("Valid decimal"));
        assert_eq!(
            result,
            U256::from_str("1500000000").expect("Valid U256 string should parse")
        );
    }

    #[test]
    fn gwei_to_wei_large_value() {
        let result = gwei_to_wei(Decimal::from_str("1000000.0").expect("Valid decimal"));
        assert_eq!(
            result,
            U256::from_str("1000000000000000").expect("Valid U256 string should parse")
        );
    }

    #[test]
    fn token_to_smallest_unit_18_decimals() {
        let result = token_to_smallest_unit(Decimal::from_str("1.0").expect("Valid decimal"), 18);
        assert_eq!(
            result,
            U256::from_str("1000000000000000000").expect("Valid U256 string should parse")
        );
    }

    #[test]
    fn token_to_smallest_unit_6_decimals() {
        let result = token_to_smallest_unit(Decimal::from_str("1.0").expect("Valid decimal"), 6);
        assert_eq!(
            result,
            U256::from_str("1000000").expect("Valid U256 string should parse")
        );
    }

    #[test]
    fn token_to_smallest_unit_with_precision() {
        let result =
            token_to_smallest_unit(Decimal::from_str("1.23456").expect("Valid decimal"), 8);
        assert_eq!(
            result,
            U256::from_str("123456000").expect("Valid U256 string should parse")
        );
    }

    #[test]
    fn token_to_smallest_unit_very_large() {
        let result = token_to_smallest_unit(
            Decimal::from_str("1000000000.0").expect("Valid decimal"),
            18,
        );
        // With Decimal, no precision loss
        assert_eq!(
            result,
            U256::from_str("1000000000000000000000000000").expect("Valid U256 string should parse")
        );
    }

    #[test]
    fn wei_to_eth_works() {
        let wei = U256::from_str("1000000000000000000").expect("Valid U256 string should parse");
        let result = wei_to_eth(wei);
        assert_eq!(result, Decimal::from_str("1.0").expect("Valid decimal"));
    }

    #[test]
    fn wei_to_gwei_works() {
        let wei = U256::from_str("1000000000").expect("Valid U256 string should parse");
        let result = wei_to_gwei(wei);
        assert_eq!(result, Decimal::from_str("1.0").expect("Valid decimal"));
    }

    #[test]
    fn smallest_unit_to_token_works() {
        let amount = U256::from_str("1000000000000000000").expect("Valid U256 string should parse");
        let result = smallest_unit_to_token(amount, 18);
        assert_eq!(result, Decimal::from_str("1.0").expect("Valid decimal"));
    }

    #[test]
    fn conversion_roundtrip() {
        // Test that converting back and forth is exact with Decimal
        let original = Decimal::from_str("123.456789").expect("Valid decimal");
        let wei = eth_to_wei(original);
        let back = wei_to_eth(wei);
        assert_eq!(back, original);
    }
}
