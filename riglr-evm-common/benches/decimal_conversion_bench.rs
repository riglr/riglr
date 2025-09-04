//! Benchmarks for EVM decimal conversion methods
//!
//! This benchmark suite compares the performance characteristics of f64-based
//! vs rust_decimal-based implementations for ETH/wei conversions.
//!
//! ## Purpose
//!
//! These benchmarks validate the performance analysis that informed the decision
//! to use f64-based conversions in the production `riglr-evm-common::conversion` module.
//!
//! ## Test Coverage
//!
//! - **Performance**: Raw conversion speed for both approaches
//! - **Precision**: Roundtrip accuracy comparison between approaches
//! - **Scale**: Performance across different value magnitudes
//!
//! ## Usage
//!
//! Run with: `cargo bench --bench decimal_conversion_bench`

#![allow(missing_docs)]

use alloy::primitives::U256;
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rust_decimal::prelude::*;
use std::str::FromStr;

/// Convert ETH to wei using f64 arithmetic (benchmark version)
/// 
/// This function implements the same logic as the production f64-based converter
/// to enable direct performance comparison with the decimal implementation.
fn eth_to_wei_f64(eth_amount: f64) -> U256 {
    let wei_str = format!("{:.0}", eth_amount * 1e18);
    U256::from_str(&wei_str).unwrap_or(U256::ZERO)
}

/// Convert wei to ETH using f64 arithmetic (benchmark version)
///
/// This function implements the same logic as the production f64-based converter
/// to enable direct performance comparison with the decimal implementation.
fn wei_to_eth_f64(wei_amount: U256) -> f64 {
    let wei_str = wei_amount.to_string();
    wei_str.parse::<f64>().unwrap_or(0.0) / 1e18
}

/// Convert ETH to wei using Decimal arithmetic (benchmark version)
///
/// This function provides exact decimal precision for comparison against the f64 approach.
/// Uses rust_decimal crate for arbitrary precision decimal arithmetic.
fn eth_to_wei_decimal(eth_amount: Decimal) -> U256 {
    let wei_decimal = eth_amount * Decimal::from_str("1000000000000000000").unwrap();
    let wei_str = wei_decimal.trunc().to_string();
    U256::from_str(&wei_str).unwrap_or(U256::ZERO)
}

/// Convert wei to ETH using Decimal arithmetic (benchmark version)
///
/// This function provides exact decimal precision for comparison against the f64 approach.
/// Uses rust_decimal crate for arbitrary precision decimal arithmetic.
fn wei_to_eth_decimal(wei_amount: U256) -> Decimal {
    let wei_decimal = Decimal::from_str(&wei_amount.to_string()).unwrap_or(Decimal::ZERO);
    wei_decimal / Decimal::from_str("1000000000000000000").unwrap()
}

/// Benchmark ETH to wei conversion performance
///
/// Compares f64-based vs Decimal-based conversion approaches across various ETH amounts.
/// Tests performance with small, medium, large, and precision-sensitive values.
fn benchmark_eth_to_wei(c: &mut Criterion) {
    let mut group = c.benchmark_group("eth_to_wei");

    // Test with various amounts
    let test_amounts = vec![1.0, 0.123456789, 1000000.0, 0.000000001];

    for amount in test_amounts {
        group.bench_function(format!("f64_{}", amount), |b| {
            b.iter(|| eth_to_wei_f64(black_box(amount)))
        });

        let decimal_amount = Decimal::from_f64(amount).unwrap();
        group.bench_function(format!("decimal_{}", amount), |b| {
            b.iter(|| eth_to_wei_decimal(black_box(decimal_amount)))
        });
    }

    group.finish();
}

/// Benchmark wei to ETH conversion performance
///
/// Compares f64-based vs Decimal-based conversion approaches across various wei amounts.
/// Tests performance with different magnitude U256 values.
fn benchmark_wei_to_eth(c: &mut Criterion) {
    let mut group = c.benchmark_group("wei_to_eth");

    // Test with various amounts
    let test_amounts = vec![
        U256::from_str("1000000000000000000").unwrap(), // 1 ETH
        U256::from_str("123456789012345678").unwrap(),  // 0.123456789012345678 ETH
        U256::from_str("1000000000000000000000000").unwrap(), // 1000000 ETH
        U256::from_str("1000000000").unwrap(),          // 0.000000001 ETH
    ];

    for amount in test_amounts {
        let amount_str = amount.to_string();
        group.bench_function(format!("f64_{}", amount_str.len()), |b| {
            b.iter(|| wei_to_eth_f64(black_box(amount)))
        });

        group.bench_function(format!("decimal_{}", amount_str.len()), |b| {
            b.iter(|| wei_to_eth_decimal(black_box(amount)))
        });
    }

    group.finish();
}

/// Benchmark precision loss in roundtrip conversions
///
/// Measures the computational cost of precision loss analysis by performing
/// roundtrip ETH→wei→ETH conversions and calculating the difference.
/// This demonstrates the precision-performance tradeoff between approaches.
fn benchmark_precision(c: &mut Criterion) {
    let mut group = c.benchmark_group("precision_test");

    // Test precision with difficult numbers
    group.bench_function("f64_precision_loss", |b| {
        b.iter(|| {
            let original = 0.123_456_789_012_345_68;
            let wei = eth_to_wei_f64(black_box(original));
            let back = wei_to_eth_f64(black_box(wei));
            (original - back).abs()
        })
    });

    group.bench_function("decimal_precision_preserved", |b| {
        b.iter(|| {
            let original = Decimal::from_str("0.123456789012345678").unwrap();
            let wei = eth_to_wei_decimal(black_box(original));
            let back = wei_to_eth_decimal(black_box(wei));
            (original - back).abs()
        })
    });

    group.finish();
}

criterion_group!(
    benches,
    benchmark_eth_to_wei,
    benchmark_wei_to_eth,
    benchmark_precision
);
criterion_main!(benches);
