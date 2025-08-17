//! Performance benchmarks for riglr-evm-tools functionality.
//!
//! This module contains comprehensive benchmarks for testing the performance
//! of various EVM operations including client creation across different chains,
//! balance tools, network operations, transaction handling, error processing,
//! serialization operations, and concurrent workloads.

#![allow(missing_docs)]

use alloy::primitives::U256;
use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use riglr_evm_tools::{client::EvmClient, error::EvmToolError};
use serde_json::json;
use std::hint::black_box;
use tokio::runtime::Runtime;

fn client_benchmarks(c: &mut Criterion) {
    let mut group = c.benchmark_group("evm_client");

    group.bench_function("client_creation_mainnet", |b| {
        b.iter(|| {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                let _client = EvmClient::mainnet().await;
            })
        })
    });

    group.bench_function("client_creation_polygon", |b| {
        b.iter(|| {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                let _client = EvmClient::polygon().await;
            })
        })
    });

    group.bench_function("client_creation_arbitrum", |b| {
        b.iter(|| {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                let _client = EvmClient::arbitrum().await;
            })
        })
    });

    group.bench_function("client_creation_optimism", |b| {
        b.iter(|| {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                let _client = EvmClient::optimism().await;
            })
        })
    });

    group.bench_function("client_creation_base", |b| {
        b.iter(|| {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                let _client = EvmClient::base().await;
            })
        })
    });

    group.finish();
}

fn balance_tool_benchmarks(c: &mut Criterion) {
    let mut group = c.benchmark_group("balance_tool");

    group.bench_function("address_validation", |b| {
        let addresses = vec![
            "0x742d35Cc6634C0532925a3b844B9450581d11340",
            "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48",
            "0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045",
        ];

        b.iter(|| {
            for addr in &addresses {
                let _valid = riglr_evm_tools::validate_address(black_box(addr));
            }
        })
    });

    group.bench_function("balance_formatting", |b| {
        let amounts = vec![
            "1000000000000000000",
            "123456789012345678901234567890",
            "0",
            "1",
            "999999999999999999",
        ];

        b.iter(|| {
            for amount in &amounts {
                let wei = amount.parse::<u128>().unwrap();
                let _formatted = format!("{:.6}", wei as f64 / 1e18);
                black_box(_formatted);
            }
        })
    });

    group.bench_function("wei_to_eth_conversion", |b| {
        let wei_amounts = vec![
            U256::from(1000000000000000000u128), // 1 ETH
            U256::from(500000000000000000u128),  // 0.5 ETH
            U256::from(1u128),                   // 1 wei
            U256::from(999999999999999999u128),  // almost 1 ETH
        ];

        b.iter(|| {
            for wei in &wei_amounts {
                let _eth = riglr_evm_tools::wei_to_eth(*wei);
                black_box(_eth);
            }
        })
    });

    group.finish();
}

fn network_tool_benchmarks(c: &mut Criterion) {
    let mut group = c.benchmark_group("network_tool");

    group.bench_function("transaction_hash_validation", |b| {
        let tx_hashes = vec![
            "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef",
            "0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890",
            "0x0000000000000000000000000000000000000000000000000000000000000001",
        ];

        b.iter(|| {
            for hash in &tx_hashes {
                let _valid = hash.starts_with("0x") && hash.len() == 66;
                black_box(_valid);
            }
        })
    });

    group.bench_function("hex_parsing", |b| {
        let hex_values = vec!["0x1234567890abcdef", "0xffffffffffffffff", "0x0", "0x1"];

        b.iter(|| {
            for hex in &hex_values {
                let _parsed = u64::from_str_radix(&hex[2..], 16);
                let _ = black_box(_parsed);
            }
        })
    });

    group.bench_function("json_structure_creation", |b| {
        b.iter(|| {
            let _json = json!({
                "blockNumber": 18000000u64,
                "gasPrice": "30000000000",
                "gasLimit": 21000u64,
                "chain": "ethereum"
            });
            black_box(_json);
        })
    });

    group.finish();
}

fn transaction_tool_benchmarks(c: &mut Criterion) {
    let mut group = c.benchmark_group("transaction_tool");

    group.bench_function("eth_amount_conversion", |b| {
        let amounts = vec![
            "0.1",
            "1.0",
            "123.456789",
            "0.000000000000000001",
            "1000000",
        ];

        b.iter(|| {
            for amount in &amounts {
                if let Ok(eth_amount) = amount.parse::<f64>() {
                    let _wei = riglr_evm_tools::eth_to_wei(eth_amount);
                    black_box(_wei);
                }
            }
        })
    });

    group.bench_function("amount_parsing", |b| {
        let amounts = vec![
            "0.1",
            "1.0",
            "123.456789",
            "0.000000000000000001",
            "1000000",
        ];

        b.iter(|| {
            for amount in &amounts {
                let _parsed: Result<f64, _> = amount.parse();
                let _ = black_box(_parsed);
            }
        })
    });

    group.bench_function("private_key_validation", |b| {
        let keys = vec![
            "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef",
            "0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890",
            "0x0000000000000000000000000000000000000000000000000000000000000001",
        ];

        b.iter(|| {
            for key in &keys {
                let _valid = key.starts_with("0x") && key.len() == 66;
                black_box(_valid);
            }
        })
    });

    group.finish();
}

fn error_handling_benchmarks(c: &mut Criterion) {
    let mut group = c.benchmark_group("error_handling");

    group.bench_function("error_creation_invalid_address", |b| {
        b.iter(|| EvmToolError::InvalidAddress(black_box("invalid_address".to_string())))
    });

    group.bench_function("error_creation_rpc", |b| {
        b.iter(|| EvmToolError::Rpc(black_box("connection failed".to_string())))
    });

    group.bench_function("error_creation_contract", |b| {
        b.iter(|| EvmToolError::Contract(black_box("call reverted".to_string())))
    });

    group.bench_function("error_creation_transaction", |b| {
        b.iter(|| EvmToolError::Transaction(black_box("gas too low".to_string())))
    });

    group.bench_function("error_display", |b| {
        let error = EvmToolError::InvalidAddress("test".to_string());
        b.iter(|| format!("{}", black_box(&error)))
    });

    group.bench_function("error_generic", |b| {
        b.iter(|| EvmToolError::Generic(black_box("generic error".to_string())))
    });

    group.finish();
}

fn serialization_benchmarks(c: &mut Criterion) {
    let mut group = c.benchmark_group("serialization");

    let test_data = vec![
        json!({"address": "0x742d35Cc6634C0532925a3b844B9450581d11340"}),
        json!({"chain": "ethereum", "block_number": 18000000}),
        json!({"amount": "1.5", "gas_price": "30000000000"}),
        json!({"tokens": ["USDC", "USDT", "DAI", "WETH"]}),
    ];

    group.bench_function("json_serialize", |b| {
        b.iter(|| {
            for data in &test_data {
                let _serialized = serde_json::to_string(black_box(data));
            }
        })
    });

    group.bench_function("json_deserialize", |b| {
        let json_strings: Vec<String> = test_data
            .iter()
            .map(|d| serde_json::to_string(d).unwrap())
            .collect();

        b.iter(|| {
            for json_str in &json_strings {
                let _deserialized: serde_json::Value =
                    serde_json::from_str(black_box(json_str)).unwrap();
            }
        })
    });

    group.bench_function("json_pretty_print", |b| {
        b.iter(|| {
            for data in &test_data {
                let _pretty = serde_json::to_string_pretty(black_box(data));
            }
        })
    });

    group.finish();
}

fn throughput_benchmarks(c: &mut Criterion) {
    let mut group = c.benchmark_group("throughput");

    for size in [10, 100, 1000].iter() {
        group.throughput(Throughput::Elements(*size as u64));

        group.bench_with_input(
            BenchmarkId::new("parse_addresses", size),
            size,
            |b, &size| {
                let addresses: Vec<String> = (0..size).map(|i| format!("0x{:040x}", i)).collect();

                b.iter(|| {
                    for addr in &addresses {
                        let _validated = addr.starts_with("0x") && addr.len() == 42;
                        black_box(_validated);
                    }
                })
            },
        );

        group.bench_with_input(
            BenchmarkId::new("format_balances", size),
            size,
            |b, &size| {
                let balances: Vec<u128> = (0..size)
                    .map(|i| (i as u128) * 1000000000000000000)
                    .collect();

                b.iter(|| {
                    for balance in &balances {
                        let _formatted = format!("{:.6} ETH", *balance as f64 / 1e18);
                        black_box(_formatted);
                    }
                })
            },
        );

        group.bench_with_input(
            BenchmarkId::new("validate_tx_hashes", size),
            size,
            |b, &size| {
                let tx_hashes: Vec<String> = (0..size).map(|i| format!("0x{:064x}", i)).collect();

                b.iter(|| {
                    for hash in &tx_hashes {
                        let _validated = hash.starts_with("0x") && hash.len() == 66;
                        black_box(_validated);
                    }
                })
            },
        );
    }

    group.finish();
}

fn concurrent_operations_benchmarks(c: &mut Criterion) {
    let mut group = c.benchmark_group("concurrent_operations");
    let rt = Runtime::new().unwrap();

    for num_tasks in [1, 2, 4, 8].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(num_tasks),
            num_tasks,
            |b, &num_tasks| {
                b.iter(|| {
                    rt.block_on(async {
                        let mut handles = vec![];

                        for _ in 0..num_tasks {
                            let handle = tokio::spawn(async move {
                                for i in 0..100 {
                                    let _addr = format!("0x{:040x}", i);
                                    let _balance = i as u128 * 1000000000000000000;
                                    tokio::task::yield_now().await;
                                }
                            });
                            handles.push(handle);
                        }

                        for handle in handles {
                            handle.await.unwrap();
                        }
                    })
                })
            },
        );
    }

    group.finish();
}

criterion_group!(
    benches,
    client_benchmarks,
    balance_tool_benchmarks,
    network_tool_benchmarks,
    transaction_tool_benchmarks,
    error_handling_benchmarks,
    serialization_benchmarks,
    throughput_benchmarks,
    concurrent_operations_benchmarks,
);

criterion_main!(benches);
