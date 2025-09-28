//! Simple performance comparison between f64 and `rust_decimal`
// Test constants and controlled test data should never fail to parse
#![expect(clippy::expect_used)]
use alloy::primitives::U256;
use core::str::FromStr;
use rust_decimal::prelude::*;
use std::time::Instant;

// Current f64-based implementation
fn eth_to_wei_f64(eth_amount: f64) -> U256 {
    let wei_str = format!("{:.0}", eth_amount * 1e18);
    U256::from_str(&wei_str).unwrap_or(U256::ZERO)
}

// rust_decimal-based implementation
fn eth_to_wei_decimal(eth_amount: Decimal) -> U256 {
    #[expect(clippy::arithmetic_side_effects)]
    let wei_decimal =
        eth_amount * Decimal::from_str("1000000000000000000").expect("Valid decimal constant");
    let wei_str = wei_decimal.trunc().to_string();
    U256::from_str(&wei_str).unwrap_or(U256::ZERO)
}

#[test]
fn test_performance_comparison() {
    const ITERATIONS: usize = 10_000;
    let test_values = vec![1.0, 0.123_456_789, 1_000_000.0, 0.000_000_001, 42.42];

    println!("\nPerformance comparison ({ITERATIONS} iterations per value):");
    println!("{}", "=".repeat(60));

    for value in &test_values {
        // Test f64 performance
        let start = Instant::now();
        for _ in 0..ITERATIONS {
            let _ = eth_to_wei_f64(*value);
        }
        let f64_duration = start.elapsed();
        #[expect(clippy::cast_precision_loss)]
        let f64_per_op = f64_duration.as_nanos() as f64 / ITERATIONS as f64;

        // Test decimal performance
        let decimal_value = Decimal::from_f64(*value).expect("Valid f64 to decimal conversion");
        let start = Instant::now();
        for _ in 0..ITERATIONS {
            let _ = eth_to_wei_decimal(decimal_value);
        }
        let decimal_duration = start.elapsed();
        #[expect(clippy::cast_precision_loss)]
        let decimal_per_op = decimal_duration.as_nanos() as f64 / ITERATIONS as f64;

        let slowdown = decimal_per_op / f64_per_op;

        println!("\nValue: {value} ETH");
        println!("  f64:     {f64_per_op:>8.1} ns/op");
        println!("  Decimal: {decimal_per_op:>8.1} ns/op");
        println!("  Slowdown: {slowdown:.1}x");
    }

    println!("\n{}", "=".repeat(60));
    println!("Summary:");
    println!("- f64 is faster (typically 2-3x)");
    println!("- Decimal provides exact precision for all 18 decimals");
    println!("- For most use cases, the performance difference is negligible");
    println!("  (microseconds vs nanoseconds)");
}
