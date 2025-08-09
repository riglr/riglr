# Riglr Benchmarks

This repository includes comprehensive benchmark tests for all riglr components to ensure optimal performance.

## Running Benchmarks

### Quick Start

Run all benchmarks:
```bash
cargo bench --workspace
```

Run benchmarks for a specific crate:
```bash
cd riglr-core
cargo bench
```

### Using the Benchmark Script

We provide a convenient script to run and manage benchmarks:

```bash
# Run all benchmarks
./scripts/run-benchmarks.sh

# Run benchmarks for a specific crate
./scripts/run-benchmarks.sh riglr-core

# Save benchmark results
./scripts/run-benchmarks.sh --save

# Compare with baseline
./scripts/run-benchmarks.sh --compare benchmark-results/baseline.txt
```

## Benchmark Coverage

Each crate includes benchmarks for:

### riglr-core
- **Job Operations**: Creation, serialization, retry logic
- **Queue Operations**: Enqueue, dequeue, throughput
- **Worker Operations**: Job processing, idempotency checks
- **Concurrent Operations**: Multi-threaded job processing

### riglr-evm-tools
- **Client Operations**: Chain configuration, client creation
- **Balance Operations**: Balance queries, formatting
- **Transaction Operations**: Transaction creation, signing
- **Network Operations**: Block queries, gas estimation

### riglr-solana-tools
- **Pubkey Operations**: Parsing, validation, creation
- **Transaction Operations**: Transaction building, signing
- **Balance Operations**: SOL/SPL token balance queries
- **Network Operations**: Slot/block queries

### riglr-web-tools
- **HTTP Client**: Request creation, header management
- **Search Operations**: Web search, result parsing
- **API Operations**: DexScreener, news, Twitter APIs
- **Serialization**: JSON parsing and formatting

### riglr-graph-memory
- **Document Operations**: Document creation, chunking
- **Graph Operations**: Node/edge creation, traversal
- **Vector Store**: Similarity search, document storage
- **Entity Extraction**: Pattern matching, entity recognition

## CI/CD Integration

Benchmarks are automatically run on:
- Every push to main branch
- Every pull request
- Manual workflow dispatch

The CI workflow includes:
- Individual crate benchmarks
- Benchmark comparison between branches
- Profile-guided optimization checks
- Result artifacts storage

## Interpreting Results

Benchmark results show:
- **Time per iteration**: Lower is better
- **Throughput**: Higher is better for bulk operations
- **Memory usage**: Via system monitoring tools

Example output:
```
test job_creation_benchmarks::new_job ... bench:         245 ns/iter (+/- 12)
test queue_benchmarks::inmemory_enqueue ... bench:         892 ns/iter (+/- 45)
test throughput_benchmarks::process_1000_jobs ... bench:   1,234,567 ns/iter (+/- 50,000)
```

## Performance Goals

Target performance metrics:
- Job creation: < 500ns
- Queue operations: < 1μs
- Worker processing: < 10ms per job
- Vector similarity search: < 100ms for 10k documents

## Optimization Tips

1. **Use release mode**: Always benchmark in release mode
   ```bash
   cargo bench --release
   ```

2. **Isolate benchmarks**: Close other applications to reduce noise

3. **Multiple runs**: Run benchmarks multiple times for consistency

4. **Profile-guided optimization**: Use PGO for production builds
   ```bash
   cargo pgo build
   cargo pgo optimize
   ```

## Contributing

When adding new features:
1. Write corresponding benchmarks
2. Run benchmarks before and after changes
3. Include benchmark results in PR description
4. Ensure no significant performance regressions

## Benchmark Development

To add new benchmarks:

1. Add criterion to dev-dependencies:
   ```toml
   [dev-dependencies]
   criterion = { workspace = true }
   ```

2. Create benchmark file:
   ```rust
   use criterion::{criterion_group, criterion_main, Criterion};
   
   fn my_benchmark(c: &mut Criterion) {
       c.bench_function("operation", |b| {
           b.iter(|| {
               // Code to benchmark
           })
       });
   }
   
   criterion_group!(benches, my_benchmark);
   criterion_main!(benches);
   ```

3. Add benchmark target to Cargo.toml:
   ```toml
   [[bench]]
   name = "my_benchmarks"
   harness = false
   ```

## Resources

- [Criterion.rs Documentation](https://bheisler.github.io/criterion.rs/book/)
- [Rust Performance Book](https://nnethercote.github.io/perf-book/)
- [cargo-flamegraph](https://github.com/flamegraph-rs/flamegraph) for profiling