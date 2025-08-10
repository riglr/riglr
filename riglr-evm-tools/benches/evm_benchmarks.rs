use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use riglr_evm_tools::{
    balance::{BalanceTool, GetErc20BalanceInput, GetEthBalanceInput},
    client::{ChainConfig, EvmClient},
    error::EvmToolError,
    network::{GetBlockNumberInput, GetGasPriceInput, GetTransactionInput, NetworkTool},
    transaction::{SendErc20Input, SendEthInput, TransactionTool},
};
use serde_json::json;
use std::sync::Arc;
use tokio::runtime::Runtime;

fn client_benchmarks(c: &mut Criterion) {
    let mut group = c.benchmark_group("evm_client");
    let rt = Runtime::new().unwrap();

    group.bench_function("client_creation", |b| {
        b.iter(|| ChainConfig::ethereum_mainnet())
    });

    group.bench_function("chain_config_polygon", |b| {
        b.iter(|| ChainConfig::polygon())
    });

    group.bench_function("chain_config_arbitrum", |b| {
        b.iter(|| ChainConfig::arbitrum_one())
    });

    group.bench_function("chain_config_optimism", |b| {
        b.iter(|| ChainConfig::optimism())
    });

    group.bench_function("chain_config_base", |b| b.iter(|| ChainConfig::base()));

    group.bench_function("custom_chain_config", |b| {
        b.iter(|| {
            ChainConfig::custom(
                black_box(1337),
                black_box("Custom Chain"),
                black_box("http://localhost:8545"),
                black_box("CUSTOM"),
            )
        })
    });

    group.finish();
}

fn balance_tool_benchmarks(c: &mut Criterion) {
    let mut group = c.benchmark_group("balance_tool");
    let rt = Runtime::new().unwrap();

    group.bench_function("eth_balance_input_parsing", |b| {
        b.iter(|| {
            let input = json!({
                "address": "0x742d35Cc6634C0532925a3b844B9450581d11340",
                "chain": "ethereum"
            });
            serde_json::from_value::<GetEthBalanceInput>(black_box(input))
        })
    });

    group.bench_function("erc20_balance_input_parsing", |b| {
        b.iter(|| {
            let input = json!({
                "address": "0x742d35Cc6634C0532925a3b844B9450581d11340",
                "token_address": "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48",
                "chain": "ethereum"
            });
            serde_json::from_value::<GetErc20BalanceInput>(black_box(input))
        })
    });

    group.bench_function("balance_tool_creation", |b| b.iter(|| BalanceTool::new()));

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

    group.finish();
}

fn network_tool_benchmarks(c: &mut Criterion) {
    let mut group = c.benchmark_group("network_tool");

    group.bench_function("block_number_input_parsing", |b| {
        b.iter(|| {
            let input = json!({
                "chain": "ethereum"
            });
            serde_json::from_value::<GetBlockNumberInput>(black_box(input))
        })
    });

    group.bench_function("gas_price_input_parsing", |b| {
        b.iter(|| {
            let input = json!({
                "chain": "ethereum"
            });
            serde_json::from_value::<GetGasPriceInput>(black_box(input))
        })
    });

    group.bench_function("transaction_input_parsing", |b| {
        b.iter(|| {
            let input = json!({
                "tx_hash": "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef",
                "chain": "ethereum"
            });
            serde_json::from_value::<GetTransactionInput>(black_box(input))
        })
    });

    group.bench_function("network_tool_creation", |b| b.iter(|| NetworkTool::new()));

    group.bench_function("hex_parsing", |b| {
        let hex_values = vec!["0x1234567890abcdef", "0xffffffffffffffff", "0x0", "0x1"];

        b.iter(|| {
            for hex in &hex_values {
                let _parsed = u64::from_str_radix(&hex[2..], 16);
                black_box(_parsed);
            }
        })
    });

    group.finish();
}

fn transaction_tool_benchmarks(c: &mut Criterion) {
    let mut group = c.benchmark_group("transaction_tool");

    group.bench_function("send_eth_input_parsing", |b| {
        b.iter(|| {
            let input = json!({
                "to": "0x742d35Cc6634C0532925a3b844B9450581d11340",
                "amount": "0.1",
                "private_key": "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef",
                "chain": "ethereum"
            });
            serde_json::from_value::<SendEthInput>(black_box(input))
        })
    });

    group.bench_function("send_erc20_input_parsing", |b| {
        b.iter(|| {
            let input = json!({
                "token_address": "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48",
                "to": "0x742d35Cc6634C0532925a3b844B9450581d11340",
                "amount": "100",
                "private_key": "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef",
                "chain": "ethereum"
            });
            serde_json::from_value::<SendErc20Input>(black_box(input))
        })
    });

    group.bench_function("transaction_tool_creation", |b| {
        b.iter(|| TransactionTool::new())
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
                black_box(_parsed);
            }
        })
    });

    group.finish();
}

fn error_handling_benchmarks(c: &mut Criterion) {
    let mut group = c.benchmark_group("error_handling");

    group.bench_function("error_creation", |b| {
        b.iter(|| EvmToolError::InvalidAddress(black_box("invalid_address".to_string())))
    });

    group.bench_function("error_chain_not_supported", |b| {
        b.iter(|| EvmToolError::ChainNotSupported(black_box("unknown_chain".to_string())))
    });

    group.bench_function("error_transaction_failed", |b| {
        b.iter(|| EvmToolError::TransactionFailed(black_box("gas too low".to_string())))
    });

    group.bench_function("error_display", |b| {
        let error = EvmToolError::InvalidAddress("test".to_string());
        b.iter(|| format!("{}", black_box(&error)))
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
