//! Comparison of f64 vs rust_decimal for EVM conversions
//!
//! This test evaluates the trade-offs between using f64 (current) and rust_decimal
//! for Ethereum unit conversions.

use alloy::primitives::U256;
use rust_decimal::prelude::*;
use std::str::FromStr;

// Current f64-based implementation (from conversion.rs)
fn eth_to_wei_f64(eth_amount: f64) -> U256 {
    let wei_str = format!("{:.0}", eth_amount * 1e18);
    U256::from_str(&wei_str).unwrap_or(U256::ZERO)
}

fn wei_to_eth_f64(wei_amount: U256) -> f64 {
    let wei_str = wei_amount.to_string();
    wei_str.parse::<f64>().unwrap_or(0.0) / 1e18
}

// rust_decimal-based implementation
fn eth_to_wei_decimal(eth_amount: Decimal) -> U256 {
    let wei_decimal = eth_amount * Decimal::from_str("1000000000000000000").unwrap();
    let wei_str = wei_decimal.trunc().to_string();
    U256::from_str(&wei_str).unwrap_or(U256::ZERO)
}

fn wei_to_eth_decimal(wei_amount: U256) -> Decimal {
    let wei_decimal = Decimal::from_str(&wei_amount.to_string()).unwrap_or(Decimal::ZERO);
    wei_decimal / Decimal::from_str("1000000000000000000").unwrap()
}

#[test]
fn test_precision_comparison_small_values() {
    // Test with very small values
    let small_eth = 0.000000000000000001; // 1 wei

    // f64 approach
    let wei_f64 = eth_to_wei_f64(small_eth);
    let back_f64 = wei_to_eth_f64(wei_f64);
    let error_f64 = (small_eth - back_f64).abs();

    // Decimal approach
    let small_eth_decimal = Decimal::from_str("0.000000000000000001").unwrap();
    let wei_decimal = eth_to_wei_decimal(small_eth_decimal);
    let back_decimal = wei_to_eth_decimal(wei_decimal);
    let error_decimal = (small_eth_decimal - back_decimal).abs();

    println!("Small value test (1 wei):");
    println!("  Original: {}", small_eth);
    println!("  f64 roundtrip: {} (error: {})", back_f64, error_f64);
    println!(
        "  Decimal roundtrip: {} (error: {})",
        back_decimal, error_decimal
    );

    // Decimal should have perfect precision for 1 wei
    assert_eq!(wei_decimal, U256::from(1u64));
    assert_eq!(back_decimal, small_eth_decimal);
}

#[test]
fn test_precision_comparison_18_decimals() {
    // Test with maximum precision (18 decimal places)
    let precise_eth_str = "0.123456789012345678";

    // f64 approach (loses precision after ~15-16 significant digits)
    let precise_f64 = precise_eth_str.parse::<f64>().unwrap();
    let wei_f64 = eth_to_wei_f64(precise_f64);
    let back_f64 = wei_to_eth_f64(wei_f64);

    // Decimal approach (preserves all 18 decimal places)
    let precise_decimal = Decimal::from_str(precise_eth_str).unwrap();
    let wei_decimal = eth_to_wei_decimal(precise_decimal);
    let back_decimal = wei_to_eth_decimal(wei_decimal);

    println!("\n18 decimal precision test:");
    println!("  Original string: {}", precise_eth_str);
    println!("  f64 value: {:.18}", precise_f64);
    println!("  f64 roundtrip: {:.18}", back_f64);
    println!("  Decimal roundtrip: {}", back_decimal);
    println!("  Wei (f64): {}", wei_f64);
    println!("  Wei (decimal): {}", wei_decimal);

    // Check that decimal preserves exact value
    assert_eq!(wei_decimal, U256::from_str("123456789012345678").unwrap());
}

#[test]
fn test_precision_comparison_large_values() {
    // Test with very large values
    let large_eth = 1_000_000_000.0; // 1 billion ETH

    // f64 approach
    let wei_f64 = eth_to_wei_f64(large_eth);
    let back_f64 = wei_to_eth_f64(wei_f64);
    let error_f64 = (large_eth - back_f64).abs() / large_eth; // Relative error

    // Decimal approach
    let large_eth_decimal = Decimal::from_str("1000000000").unwrap();
    let wei_decimal = eth_to_wei_decimal(large_eth_decimal);
    let back_decimal = wei_to_eth_decimal(wei_decimal);
    let error_decimal = ((large_eth_decimal - back_decimal) / large_eth_decimal).abs();

    println!("\nLarge value test (1 billion ETH):");
    println!("  Original: {}", large_eth);
    println!(
        "  f64 roundtrip: {} (relative error: {:.2e})",
        back_f64, error_f64
    );
    println!(
        "  Decimal roundtrip: {} (relative error: {})",
        back_decimal, error_decimal
    );

    // Both should handle large values reasonably well
    assert!(error_f64 < 1e-10);
    assert_eq!(error_decimal, Decimal::ZERO);
}

#[test]
fn test_gas_price_precision() {
    // Common gas price scenario: 1.5 gwei
    let gwei = 1.5;
    let wei_from_gwei_f64 = gwei * 1e9;
    let wei_from_gwei_decimal =
        Decimal::from_str("1.5").unwrap() * Decimal::from_str("1000000000").unwrap();

    println!("\nGas price precision (1.5 gwei):");
    println!("  f64: {} wei", wei_from_gwei_f64);
    println!("  Decimal: {} wei", wei_from_gwei_decimal);

    assert_eq!(wei_from_gwei_f64, 1_500_000_000.0);
    assert_eq!(
        wei_from_gwei_decimal,
        Decimal::from_str("1500000000").unwrap()
    );
}

#[test]
fn test_defi_precision_requirements() {
    // DeFi scenario: small percentage differences matter
    let amount1 = 1.000000000000000001; // Slightly more than 1 ETH
    let amount2 = 1.000000000000000002; // Slightly more than amount1

    // f64 approach - may not distinguish between these
    let wei1_f64 = eth_to_wei_f64(amount1);
    let wei2_f64 = eth_to_wei_f64(amount2);
    let can_distinguish_f64 = wei1_f64 != wei2_f64;

    // Decimal approach - should distinguish
    let amount1_decimal = Decimal::from_str("1.000000000000000001").unwrap();
    let amount2_decimal = Decimal::from_str("1.000000000000000002").unwrap();
    let wei1_decimal = eth_to_wei_decimal(amount1_decimal);
    let wei2_decimal = eth_to_wei_decimal(amount2_decimal);
    let can_distinguish_decimal = wei1_decimal != wei2_decimal;

    println!("\nDeFi precision test (distinguishing tiny differences):");
    println!("  Amount1: {} ETH", amount1);
    println!("  Amount2: {} ETH", amount2);
    println!(
        "  f64 can distinguish: {} ({} vs {})",
        can_distinguish_f64, wei1_f64, wei2_f64
    );
    println!(
        "  Decimal can distinguish: {} ({} vs {})",
        can_distinguish_decimal, wei1_decimal, wei2_decimal
    );

    // Decimal should be able to distinguish, f64 might not
    assert!(can_distinguish_decimal);
}

#[test]
fn test_edge_cases() {
    // Test edge cases

    // Zero
    assert_eq!(eth_to_wei_f64(0.0), U256::ZERO);
    assert_eq!(eth_to_wei_decimal(Decimal::ZERO), U256::ZERO);

    // Negative (should probably error in real implementation, but testing behavior)
    let neg_wei_f64 = eth_to_wei_f64(-1.0);
    assert_eq!(neg_wei_f64, U256::ZERO); // Current implementation returns ZERO for parse errors

    // Maximum U256 value
    let max_u256 = U256::MAX;
    let max_eth_f64 = wei_to_eth_f64(max_u256);
    let max_eth_decimal = wei_to_eth_decimal(max_u256);
    println!("\nMax U256 as ETH:");
    println!("  f64: {:.2e}", max_eth_f64);
    println!("  Decimal: {}", max_eth_decimal);
}
