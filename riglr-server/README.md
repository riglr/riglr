# riglr-server

Turnkey, production-ready HTTP server for riglr agents. Provides pre-configured endpoints, auth middleware via SignerFactory, and basic telemetry.

## Endpoints

- POST /v1/stream: SSE stream of agent output
- POST /v1/completion: One-shot completion
- GET  /health: Health check
- GET  /: Info endpoint

## Quick start (Axum)

- Enable default feature (axum). Example:

```rust
use riglr_server::{ServerConfig, start_axum};
use riglr_web_adapters::{CompositeSignerFactory, SignerFactory};
use std::sync::Arc;

#[derive(Clone)]
struct EchoAgent;

#[async_trait::async_trait]
impl riglr_web_adapters::Agent for EchoAgent {
    type Error = std::io::Error;
    async fn prompt(&self, prompt: &str) -> Result<String, Self::Error> { Ok(prompt.to_string()) }
    async fn prompt_stream(&self, _prompt: &str) -> Result<futures_util::stream::BoxStream<'_, Result<String, Self::Error>>, Self::Error> {
        Ok(Box::pin(futures_util::stream::iter(vec![Ok("hello".to_string())])))
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt().with_env_filter("info").init();

    let mut comps = CompositeSignerFactory::new();
    // comps.register_factory("privy".into(), Box::new(PrivyFactory::new()));

    let cfg = ServerConfig::default();
    start_axum(cfg, EchoAgent, Arc::new(comps)).await
}
```

## Features

- axum (default)
- actix
