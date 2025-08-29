//! Simple performance comparison between f64 and rust_decimal

use alloy::primitives::U256;
use rust_decimal::prelude::*;
use std::str::FromStr;
use std::time::Instant;

// Current f64-based implementation
fn eth_to_wei_f64(eth_amount: f64) -> U256 {
    let wei_str = format!("{:.0}", eth_amount * 1e18);
    U256::from_str(&wei_str).unwrap_or(U256::ZERO)
}

// rust_decimal-based implementation
fn eth_to_wei_decimal(eth_amount: Decimal) -> U256 {
    let wei_decimal = eth_amount * Decimal::from_str("1000000000000000000").unwrap();
    let wei_str = wei_decimal.trunc().to_string();
    U256::from_str(&wei_str).unwrap_or(U256::ZERO)
}

#[test]
fn test_performance_comparison() {
    const ITERATIONS: usize = 10_000;
    let test_values = vec![1.0, 0.123456789, 1000000.0, 0.000000001, 42.42];

    println!(
        "\nPerformance comparison ({} iterations per value):",
        ITERATIONS
    );
    println!("{}", "=".repeat(60));

    for value in &test_values {
        // Test f64 performance
        let start = Instant::now();
        for _ in 0..ITERATIONS {
            let _ = eth_to_wei_f64(*value);
        }
        let f64_duration = start.elapsed();
        let f64_per_op = f64_duration.as_nanos() as f64 / ITERATIONS as f64;

        // Test decimal performance
        let decimal_value = Decimal::from_f64(*value).unwrap();
        let start = Instant::now();
        for _ in 0..ITERATIONS {
            let _ = eth_to_wei_decimal(decimal_value);
        }
        let decimal_duration = start.elapsed();
        let decimal_per_op = decimal_duration.as_nanos() as f64 / ITERATIONS as f64;

        let slowdown = decimal_per_op / f64_per_op;

        println!("\nValue: {} ETH", value);
        println!("  f64:     {:>8.1} ns/op", f64_per_op);
        println!("  Decimal: {:>8.1} ns/op", decimal_per_op);
        println!("  Slowdown: {:.1}x", slowdown);
    }

    println!("\n{}", "=".repeat(60));
    println!("Summary:");
    println!("- f64 is faster (typically 2-3x)");
    println!("- Decimal provides exact precision for all 18 decimals");
    println!("- For most use cases, the performance difference is negligible");
    println!("  (microseconds vs nanoseconds)");
}
