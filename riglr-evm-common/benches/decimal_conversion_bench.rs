use alloy::primitives::U256;
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rust_decimal::prelude::*;
use std::str::FromStr;

// Current f64-based implementation
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

fn benchmark_eth_to_wei(c: &mut Criterion) {
    let mut group = c.benchmark_group("eth_to_wei");

    // Test with various amounts
    let test_amounts = vec![1.0, 0.123456789, 1000000.0, 0.000000001];

    for amount in test_amounts {
        group.bench_function(&format!("f64_{}", amount), |b| {
            b.iter(|| eth_to_wei_f64(black_box(amount)))
        });

        let decimal_amount = Decimal::from_f64(amount).unwrap();
        group.bench_function(&format!("decimal_{}", amount), |b| {
            b.iter(|| eth_to_wei_decimal(black_box(decimal_amount)))
        });
    }

    group.finish();
}

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
        group.bench_function(&format!("f64_{}", amount_str.len()), |b| {
            b.iter(|| wei_to_eth_f64(black_box(amount)))
        });

        group.bench_function(&format!("decimal_{}", amount_str.len()), |b| {
            b.iter(|| wei_to_eth_decimal(black_box(amount)))
        });
    }

    group.finish();
}

fn benchmark_precision(c: &mut Criterion) {
    let mut group = c.benchmark_group("precision_test");

    // Test precision with difficult numbers
    group.bench_function("f64_precision_loss", |b| {
        b.iter(|| {
            let original = 0.123456789012345678;
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
