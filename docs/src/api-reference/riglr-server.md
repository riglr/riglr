# riglr-server API Reference

Comprehensive API documentation for the `riglr-server` crate.

## Table of Contents

### Structs

- [`ServerConfig`](#serverconfig)

### Functions (server)

- [`start_actix`](#start_actix)
- [`start_axum`](#start_axum)

## Structs

### ServerConfig

**Source**: `src/server.rs`

**Attributes**:
```rust
#[derive(Clone, Debug)]
```

```rust
pub struct ServerConfig { /// Socket address for the server to bind to pub addr: SocketAddr, /// RPC configuration for blockchain interactions pub rpc: RpcConfig, }
```

Configuration for riglr server instances

---

## Functions (server)

### start_actix

**Source**: `src/server.rs`

**Attributes**:
```rust
#[cfg(feature = "actix")]
```

```rust
pub async fn start_actix<A: Agent + Clone + Send + Sync + 'static>( config: ServerConfig, agent: A, signer_factory: Arc<dyn SignerFactory>, ) -> anyhow::Result<()>
```

Start an Actix server exposing riglr endpoints. Enable with the `actix` feature.

---

### start_axum

**Source**: `src/server.rs`

**Attributes**:
```rust
#[cfg(feature = "axum")]
```

```rust
pub async fn start_axum<A: Agent + Clone + Send + Sync + 'static>( config: ServerConfig, agent: A, signer_factory: Arc<dyn SignerFactory>, ) -> anyhow::Result<()>
```

Start an Axum server exposing riglr endpoints. Enabled with the `axum` feature.

---


---

*This documentation was automatically generated from the source code.*