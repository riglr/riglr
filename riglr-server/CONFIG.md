# riglr-server Configuration

riglr-server is designed to be production-ready with pluggable auth and telemetry.

- Authentication: Provide a `SignerFactory` implementation or compose multiple via `CompositeSignerFactory`.
- Logging: Set `RUST_LOG=info` (or more specific) and riglr-server uses `tracing_subscriber`.
- Metrics: You can add a tower layer for Prometheus in Axum or middleware in Actix; the server surface allows adding layers before calling `start_*`.

Example: Axum with rate limiting and timeouts

```rust
let app = Router::new()
    .route("/v1/stream", post(sse_handler::<A>))
    .route("/v1/completion", post(completion_handler::<A>))
    .layer(tower::timeout::TimeoutLayer::new(Duration::from_secs(30)))
    .layer(tower::limit::ConcurrencyLimitLayer::new(512));
```

Auth header

- Authorization: `Bearer TOKEN`
- X-Network: `mainnet` | `devnet` | ... (optional; factory may decide chain)

Security tips

- Always validate tokens server side.
- Restrict RPC URLs for mainnet vs testnet using `RpcConfig`.
- Enable TLS and reverse-proxy best practices in deployment.
