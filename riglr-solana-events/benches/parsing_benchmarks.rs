//! Benchmarks for high-performance parsing components
//! 
//! These benchmarks validate that the new parsing infrastructure provides
//! measurable performance improvements over the legacy implementations.

use criterion::{criterion_group, criterion_main, Criterion, BenchmarkId};
use std::hint::black_box;
use riglr_solana_events::prelude::*;
use riglr_solana_events::zero_copy::{BatchEventParser, CustomDeserializer, SIMDPatternMatcher};
use riglr_solana_events::types::EventMetadata;

/// Generate test data for Raydium V4 SwapBaseIn instruction
fn generate_raydium_swap_data() -> Vec<u8> {
    let mut data = vec![0x09]; // SwapBaseIn discriminator
    data.extend_from_slice(&1_000_000u64.to_le_bytes()); // amount_in
    data.extend_from_slice(&950_000u64.to_le_bytes());   // minimum_amount_out
    data
}

/// Generate test data for Jupiter Route instruction
fn generate_jupiter_swap_data() -> Vec<u8> {
    let mut data = vec![229, 23, 203, 151, 122, 227, 173, 42]; // Route discriminator
    data.extend_from_slice(&1_000_000u64.to_le_bytes()); // amount_in
    data.extend_from_slice(&950_000u64.to_le_bytes());   // minimum_amount_out
    data.extend_from_slice(&50u32.to_le_bytes());        // platform_fee_bps
    data
}

/// Generate test data for PumpFun Buy instruction
fn generate_pump_fun_data() -> Vec<u8> {
    let mut data = Vec::new();
    data.extend_from_slice(&0x66063d1201daebeau64.to_le_bytes()); // Buy discriminator
    data.extend_from_slice(&1000u64.to_le_bytes());               // amount
    data.extend_from_slice(&2000u64.to_le_bytes());               // max_price_per_token
    data
}

/// Benchmark single parser performance
fn benchmark_single_parsers(c: &mut Criterion) {
    let mut group = c.benchmark_group("single_parsers");

    // Test data
    let raydium_data = generate_raydium_swap_data();
    let jupiter_data = generate_jupiter_swap_data();
    let pump_fun_data = generate_pump_fun_data();
    
    // Parsers
    let raydium_parser = RaydiumV4ParserFactory::create_zero_copy();
    let jupiter_parser = JupiterParserFactory::create_zero_copy();
    let pump_fun_parser = PumpFunParserFactory::create_zero_copy();
    
    // Metadata
    let metadata = EventMetadata::default();

    group.bench_with_input(
        BenchmarkId::new("raydium_v4_zero_copy", raydium_data.len()),
        &raydium_data,
        |b, data| {
            b.iter(|| {
                let result = black_box(raydium_parser.parse_from_slice(
                    black_box(data), 
                    black_box(metadata.clone())
                ));
                black_box(result)
            })
        },
    );

    group.bench_with_input(
        BenchmarkId::new("jupiter_zero_copy", jupiter_data.len()),
        &jupiter_data,
        |b, data| {
            b.iter(|| {
                let result = black_box(jupiter_parser.parse_from_slice(
                    black_box(data), 
                    black_box(metadata.clone())
                ));
                black_box(result)
            })
        },
    );

    group.bench_with_input(
        BenchmarkId::new("pump_fun_zero_copy", pump_fun_data.len()),
        &pump_fun_data,
        |b, data| {
            b.iter(|| {
                let result = black_box(pump_fun_parser.parse_from_slice(
                    black_box(data), 
                    black_box(metadata.clone())
                ));
                black_box(result)
            })
        },
    );

    group.finish();
}

/// Benchmark batch parsing performance
fn benchmark_batch_parsing(c: &mut Criterion) {
    let mut group = c.benchmark_group("batch_parsing");

    // Create batch parser
    let mut batch_parser = BatchEventParser::new(1000);
    batch_parser.add_parser(RaydiumV4ParserFactory::create_zero_copy());
    batch_parser.add_parser(JupiterParserFactory::create_zero_copy());
    batch_parser.add_parser(PumpFunParserFactory::create_zero_copy());

    // Generate test batches of different sizes
    let batch_sizes = [10, 50, 100, 500];
    
    for &size in &batch_sizes {
        let mut batch_data = Vec::new();
        let mut batch_metadata = Vec::new();
        
        for i in 0..size {
            let data = match i % 3 {
                0 => generate_raydium_swap_data(),
                1 => generate_jupiter_swap_data(),
                _ => generate_pump_fun_data(),
            };
            batch_data.push(data);
            
            let mut metadata = EventMetadata::default();
            metadata.id = format!("event_{}", i);
            batch_metadata.push(metadata);
        }
        
        let batch_refs: Vec<&[u8]> = batch_data.iter().map(|d| d.as_slice()).collect();

        group.bench_with_input(
            BenchmarkId::new("batch_parse", size),
            &size,
            |b, _| {
                b.to_async(tokio::runtime::Runtime::new().unwrap()).iter(|| async {
                    let result = black_box(batch_parser.parse_batch(
                        black_box(&batch_refs), 
                        black_box(batch_metadata.clone())
                    ));
                    black_box(result)
                })
            },
        );
    }

    group.finish();
}

/// Benchmark custom deserializer vs borsh
fn benchmark_deserializers(c: &mut Criterion) {
    let mut group = c.benchmark_group("deserializers");

    let test_data = generate_raydium_swap_data();

    group.bench_function("custom_deserializer", |b| {
        b.iter(|| {
            let mut deserializer = black_box(CustomDeserializer::new(&test_data));
            let discriminator = black_box(deserializer.read_u8().unwrap());
            let amount_in = black_box(deserializer.read_u64_le().unwrap());
            let amount_out = black_box(deserializer.read_u64_le().unwrap());
            black_box((discriminator, amount_in, amount_out))
        })
    });

    // Compare with standard slice operations
    group.bench_function("slice_operations", |b| {
        b.iter(|| {
            let data = black_box(&test_data);
            let discriminator = black_box(data[0]);
            let amount_in = black_box(u64::from_le_bytes([
                data[1], data[2], data[3], data[4],
                data[5], data[6], data[7], data[8],
            ]));
            let amount_out = black_box(u64::from_le_bytes([
                data[9], data[10], data[11], data[12],
                data[13], data[14], data[15], data[16],
            ]));
            black_box((discriminator, amount_in, amount_out))
        })
    });

    group.finish();
}

/// Benchmark SIMD pattern matching
fn benchmark_pattern_matching(c: &mut Criterion) {
    let mut group = c.benchmark_group("pattern_matching");

    let mut matcher = SIMDPatternMatcher::new();
    matcher.add_pattern(vec![0x09], ProtocolType::RaydiumAmmV4);
    matcher.add_pattern(vec![229, 23, 203, 151, 122, 227, 173, 42], ProtocolType::Jupiter);
    
    let test_data_raydium = generate_raydium_swap_data();
    let test_data_jupiter = generate_jupiter_swap_data();
    let test_data_unknown = vec![0xFF, 0xFF, 0xFF, 0xFF];

    group.bench_function("simd_match_raydium", |b| {
        b.iter(|| {
            let result = black_box(matcher.match_discriminator(black_box(&test_data_raydium)));
            black_box(result)
        })
    });

    group.bench_function("simd_match_jupiter", |b| {
        b.iter(|| {
            let result = black_box(matcher.match_discriminator(black_box(&test_data_jupiter)));
            black_box(result)
        })
    });

    group.bench_function("simd_match_unknown", |b| {
        b.iter(|| {
            let result = black_box(matcher.match_discriminator(black_box(&test_data_unknown)));
            black_box(result)
        })
    });

    group.finish();
}

/// Benchmark zero-copy vs owned data
fn benchmark_zero_copy_vs_owned(c: &mut Criterion) {
    let mut group = c.benchmark_group("zero_copy_vs_owned");

    let test_data = generate_raydium_swap_data();
    let metadata = EventMetadata::default();

    group.bench_function("zero_copy_borrowed", |b| {
        b.iter(|| {
            let event = black_box(ZeroCopyEvent::new_borrowed(
                black_box(metadata.clone()),
                black_box(&test_data),
            ));
            black_box(event)
        })
    });

    group.bench_function("zero_copy_owned", |b| {
        b.iter(|| {
            let event = black_box(ZeroCopyEvent::new_owned(
                black_box(metadata.clone()),
                black_box(test_data.clone()),
            ));
            black_box(event)
        })
    });

    group.bench_function("to_owned_conversion", |b| {
        let borrowed_event = ZeroCopyEvent::new_borrowed(metadata.clone(), &test_data);
        b.iter(|| {
            let owned_event = black_box(borrowed_event.to_owned());
            black_box(owned_event)
        })
    });

    group.finish();
}

/// Benchmark memory allocation patterns
fn benchmark_memory_allocation(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_allocation");

    let data_sizes = [100, 1000, 10000, 100000];
    
    for &size in &data_sizes {
        group.bench_with_input(
            BenchmarkId::new("vec_allocation", size),
            &size,
            |b, &size| {
                b.iter(|| {
                    let vec = black_box(vec![0u8; size]);
                    black_box(vec)
                })
            },
        );
        
        group.bench_with_input(
            BenchmarkId::new("vec_with_capacity", size),
            &size,
            |b, &size| {
                b.iter(|| {
                    let mut vec = black_box(Vec::with_capacity(size));
                    vec.resize(size, 0u8);
                    black_box(vec)
                })
            },
        );
    }

    group.finish();
}

criterion_group!(
    benches,
    benchmark_single_parsers,
    benchmark_batch_parsing,
    benchmark_deserializers,
    benchmark_pattern_matching,
    benchmark_zero_copy_vs_owned,
    benchmark_memory_allocation
);
criterion_main!(benches);