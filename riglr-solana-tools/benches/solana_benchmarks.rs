use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use std::hint::black_box;
use riglr_solana_tools::{
    client::{SolanaClient, SolanaConfig},
    error::SolanaToolError,
    BalanceResult, TokenBalanceResult, TransactionResult, TransactionStatus,
};
use serde_json::json;
use solana_sdk::{
    pubkey::Pubkey,
    signature::{Keypair, Signature, Signer},
    transaction::Transaction,
    commitment_config::CommitmentLevel,
    bs58,
};
use std::str::FromStr;
use std::time::Duration;
use tokio::runtime::Runtime;

fn pubkey_benchmarks(c: &mut Criterion) {
    let mut group = c.benchmark_group("pubkey_operations");

    group.bench_function("pubkey_from_str", |b| {
        let addr = "11111111111111111111111111111111";
        b.iter(|| Pubkey::from_str(black_box(addr)))
    });

    group.bench_function("pubkey_to_string", |b| {
        let pubkey = Pubkey::new_unique();
        b.iter(|| black_box(pubkey.to_string()))
    });

    group.bench_function("pubkey_new_unique", |b| b.iter(|| Pubkey::new_unique()));

    group.bench_function("pubkey_validation", |b| {
        let valid_keys = vec![
            "11111111111111111111111111111111",
            "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
            "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
        ];

        b.iter(|| {
            for key in &valid_keys {
                let _result = Pubkey::from_str(black_box(key));
            }
        })
    });

    group.finish();
}

fn client_benchmarks(c: &mut Criterion) {
    let mut group = c.benchmark_group("solana_client");

    group.bench_function("client_creation_mainnet", |b| {
        b.iter(|| {
            let config = SolanaConfig {
                rpc_url: "https://api.mainnet-beta.solana.com".to_string(),
                timeout: Duration::from_secs(30),
                commitment: CommitmentLevel::Confirmed,
                skip_preflight: false,
            };
            SolanaClient::new(config)
        })
    });

    group.bench_function("client_creation_devnet", |b| {
        b.iter(|| {
            let config = SolanaConfig {
                rpc_url: "https://api.devnet.solana.com".to_string(),
                timeout: Duration::from_secs(30),
                commitment: CommitmentLevel::Confirmed,
                skip_preflight: false,
            };
            SolanaClient::new(config)
        })
    });

    group.bench_function("client_creation_custom", |b| {
        b.iter(|| {
            let config = SolanaConfig {
                rpc_url: black_box("http://localhost:8899".to_string()),
                timeout: Duration::from_secs(30),
                commitment: CommitmentLevel::Confirmed,
                skip_preflight: false,
            };
            SolanaClient::new(config)
        })
    });

    group.finish();
}

fn balance_benchmarks(c: &mut Criterion) {
    let mut group = c.benchmark_group("balance_operations");

    group.bench_function("balance_result_creation", |b| {
        b.iter(|| {
            BalanceResult {
                address: black_box("11111111111111111111111111111111".to_string()),
                lamports: black_box(1_500_000_000),
                sol: black_box(1.5),
                formatted: black_box("1.5 SOL".to_string()),
            }
        })
    });

    group.bench_function("token_balance_result_creation", |b| {
        b.iter(|| {
            TokenBalanceResult {
                owner_address: black_box("11111111111111111111111111111111".to_string()),
                mint_address: black_box("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string()),
                raw_amount: black_box(1000000000),
                ui_amount: black_box(1000.0),
                decimals: black_box(6),
                formatted: black_box("1,000.0 USDC".to_string()),
            }
        })
    });

    group.bench_function("lamports_to_sol", |b| {
        let lamports_values = vec![1_000_000_000u64, 123_456_789u64, 0u64, 999_999_999_999u64];

        b.iter(|| {
            for lamports in &lamports_values {
                let _sol = *lamports as f64 / 1_000_000_000.0;
                black_box(_sol);
            }
        })
    });

    group.bench_function("sol_to_lamports", |b| {
        let sol_values = vec![1.0, 0.123456789, 1000.0, 0.000000001];

        b.iter(|| {
            for sol in &sol_values {
                let _lamports = (*sol * 1_000_000_000.0) as u64;
                black_box(_lamports);
            }
        })
    });

    group.finish();
}

fn transaction_benchmarks(c: &mut Criterion) {
    let mut group = c.benchmark_group("transaction_operations");

    group.bench_function("transaction_result_creation", |b| {
        b.iter(|| {
            TransactionResult {
                signature: black_box("5VERv8NMvzbJMEkV8xnrLkEaWRtSz9CosKDYjCJjBRnbJLgp8uirBgmQpjKhoR4tjF3ZpRzrFmBV6UjKdiSZkQUW".to_string()),
                from: black_box("11111111111111111111111111111112".to_string()),
                to: black_box("11111111111111111111111111111111".to_string()),
                amount: black_box(100_000_000),
                amount_display: black_box("0.1 SOL".to_string()),
                status: black_box(TransactionStatus::Confirmed),
                memo: black_box(None),
                idempotency_key: black_box(None),
            }
        })
    });

    group.bench_function("json_parsing", |b| {
        b.iter(|| {
            let input = json!({
                "to": "11111111111111111111111111111111",
                "amount": 0.1,
                "network": "mainnet"
            });
            serde_json::to_string(&input).unwrap()
        })
    });

    group.bench_function("keypair_generation", |b| b.iter(|| Keypair::new()));

    group.bench_function("signature_parsing", |b| {
        let sig_str = "5VERv8NMvzbJMEkV8xnrLkEaWRtSz9CosKDYjCJjBRnbJLgp8uirBgmQpjKhoR4tjF3ZpRzrFmBV6UjKdiSZkQUW";
        b.iter(|| {
            Signature::from_str(black_box(sig_str))
        })
    });

    group.bench_function("transaction_creation", |b| {
        let from = Keypair::new();
        let to = Pubkey::new_unique();
        let lamports = 1_000_000_000;

        b.iter(|| {
            let instruction = solana_sdk::system_instruction::transfer(&from.pubkey(), &to, lamports);
            Transaction::new_with_payer(&[instruction], Some(&from.pubkey()))
        })
    });

    group.finish();
}

fn network_benchmarks(c: &mut Criterion) {
    let mut group = c.benchmark_group("network_operations");

    group.bench_function("network_url_creation", |b| {
        b.iter(|| {
            let _mainnet = black_box("https://api.mainnet-beta.solana.com");
            let _devnet = black_box("https://api.devnet.solana.com");
            let _testnet = black_box("https://api.testnet.solana.com");
        })
    });

    group.bench_function("json_rpc_request_format", |b| {
        b.iter(|| {
            let input = json!({
                "jsonrpc": "2.0",
                "id": 1,
                "method": "getAccountInfo",
                "params": ["11111111111111111111111111111111"]
            });
            serde_json::to_string(&input).unwrap()
        })
    });

    group.bench_function("signature_validation", |b| {
        let signatures = vec![
            "5VERv8NMvzbJMEkV8xnrLkEaWRtSz9CosKDYjCJjBRnbJLgp8uirBgmQpjKhoR4tjF3ZpRzrFmBV6UjKdiSZkQUW",
            "2Kzg5s2gJ8xjwk8HkWyZmHQ7aR9fG5LwVtYE3QRhN4vX8cD1P6mS9hB7uT5eK2gQ8wM3nL6vC4sF8hY9jE1dA7pR",
            "3LwBg8kHhR7tK9mQ5rE2sT6nF8pL3gY4vD9uC1oX7jA5hM2dS8fV6tG9yE4wK8bN5oP7qH3nL9mS6eR1dA2gF8kY",
        ];

        b.iter(|| {
            for sig_str in &signatures {
                let _sig = Signature::from_str(black_box(sig_str));
            }
        })
    });

    group.finish();
}

fn error_handling_benchmarks(c: &mut Criterion) {
    let mut group = c.benchmark_group("error_handling");

    group.bench_function("error_invalid_address", |b| {
        b.iter(|| SolanaToolError::InvalidAddress(black_box("invalid".to_string())))
    });

    group.bench_function("error_transaction", |b| {
        b.iter(|| SolanaToolError::Transaction(black_box("insufficient funds".to_string())))
    });

    group.bench_function("error_generic", |b| {
        b.iter(|| SolanaToolError::Generic(black_box("simulation failed".to_string())))
    });

    group.bench_function("error_display", |b| {
        let error = SolanaToolError::InvalidAddress("test".to_string());
        b.iter(|| format!("{}", black_box(&error)))
    });

    group.finish();
}

fn serialization_benchmarks(c: &mut Criterion) {
    let mut group = c.benchmark_group("serialization");

    let test_data = vec![
        json!({"address": "11111111111111111111111111111111"}),
        json!({"amount": 1.5, "network": "mainnet"}),
        json!({"signature": "5VERv8NMvzbJMEkV8xnrLkEaWRtSz9CosKDYjCJjBRnbJLgp8uirBgmQpjKhoR4tjF3ZpRzrFmBV6UjKdiSZkQUW"}),
        json!({"tokens": ["USDC", "USDT", "RAY", "SRM"]}),
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

    group.bench_function("base58_encoding", |b| {
        let bytes = vec![
            vec![1, 2, 3, 4, 5],
            vec![255, 254, 253, 252, 251],
            vec![0; 32],
        ];

        b.iter(|| {
            for byte_vec in &bytes {
                let _encoded = bs58::encode(black_box(byte_vec)).into_string();
            }
        })
    });

    group.bench_function("base58_decoding", |b| {
        let encoded = vec![
            "11111111111111111111111111111111",
            "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
            "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
        ];

        b.iter(|| {
            for enc in &encoded {
                let _decoded = bs58::decode(black_box(enc)).into_vec();
            }
        })
    });

    group.finish();
}

fn throughput_benchmarks(c: &mut Criterion) {
    let mut group = c.benchmark_group("throughput");

    for size in [10, 100, 1000].iter() {
        group.throughput(Throughput::Elements(*size as u64));

        group.bench_with_input(BenchmarkId::new("parse_pubkeys", size), size, |b, &size| {
            let pubkeys: Vec<String> = (0..size)
                .map(|i| {
                    let mut bytes = [0u8; 32];
                    bytes[0] = (i % 256) as u8;
                    bs58::encode(bytes).into_string()
                })
                .collect();

            b.iter(|| {
                for pubkey_str in &pubkeys {
                    let _pubkey = Pubkey::from_str(black_box(pubkey_str));
                }
            })
        });

        group.bench_with_input(
            BenchmarkId::new("format_balances", size),
            size,
            |b, &size| {
                let balances: Vec<u64> = (0..size).map(|i| i as u64 * 1_000_000_000).collect();

                b.iter(|| {
                    for balance in &balances {
                        let _formatted = format!("{:.9} SOL", *balance as f64 / 1e9);
                        black_box(_formatted);
                    }
                })
            },
        );

        group.bench_with_input(
            BenchmarkId::new("validate_signatures", size),
            size,
            |b, &size| {
                let signatures: Vec<String> = (0..size)
                    .map(|i| {
                        let mut bytes = [0u8; 64];
                        bytes[0] = (i % 256) as u8;
                        bs58::encode(bytes).into_string()
                    })
                    .collect();

                b.iter(|| {
                    for sig_str in &signatures {
                        let _sig = Signature::from_str(black_box(sig_str));
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
                                for _ in 0..100 {
                                    let _pubkey = Pubkey::new_unique();
                                    let _keypair = Keypair::new();
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
    pubkey_benchmarks,
    client_benchmarks,
    balance_benchmarks,
    transaction_benchmarks,
    network_benchmarks,
    error_handling_benchmarks,
    serialization_benchmarks,
    throughput_benchmarks,
    concurrent_operations_benchmarks,
);

criterion_main!(benches);
