# riglr-auth

First-class authentication and signer factory implementations for RIGLR, providing official support for popular authentication services.

## Features

- **Privy**: Embedded wallets with social login
- **Web3Auth**: Non-custodial key management with social login  
- **Magic.link**: Email-based authentication with embedded wallets
- Extensible architecture for custom providers
- Built-in caching and token validation
- Multi-tenant support

## Architecture

riglr-auth provides official authentication provider integrations for the riglr ecosystem. It acts as a bridge between external authentication services and the core riglr signing infrastructure.

### Design Principles

- **Provider Abstraction**: Each auth service is wrapped in a unified `SignerFactory` interface
- **Token Validation**: All providers validate authentication tokens before creating signers
- **Chain Support**: Providers return `UnifiedSigner` implementations that support both Solana and EVM
- **Caching**: Built-in token caching to reduce API calls and improve performance
- **Multi-Tenant Ready**: Each request gets its own isolated signer instance

### Integration Flow

1. **Authentication**: User authenticates with their preferred service (Privy, Web3Auth, etc.)
2. **Token Validation**: Provider validates the authentication token with the service
3. **Key Retrieval**: Provider fetches or derives the user's private keys
4. **Signer Creation**: Provider creates appropriate `LocalSolanaSigner` or `LocalEvmSigner`
5. **Request Execution**: Signer executes blockchain operations in isolated context

### Key Components

- **AuthProvider**: Factory for creating provider instances from configuration
- **SignerFactory Trait**: Common interface implemented by all providers
- **CompositeSignerFactory**: Registry that manages multiple providers
- **Provider Configs**: Type-safe configuration for each auth service

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
riglr-auth = { version = "0.3.0", features = ["privy", "web3auth", "magic"] }
```

## Quick Start

### Single Provider Setup

```rust
use riglr_auth::{AuthProvider, PrivyConfig};
use riglr_web_adapters::factory::CompositeSignerFactory;

// Load configuration from environment
let privy_config = PrivyConfig::from_env()?;

// Create provider
let provider = AuthProvider::privy(privy_config);

// Register with composite factory
let mut factory = CompositeSignerFactory::new();
factory.register_provider(provider);
```

### Multiple Providers

```rust
use riglr_auth::{AuthProvider, PrivyConfig, Web3AuthConfig, MagicConfig};

let mut factory = CompositeSignerFactory::new();

// Register multiple providers
if let Ok(config) = PrivyConfig::from_env() {
    factory.register_provider(AuthProvider::privy(config));
}

if let Ok(config) = Web3AuthConfig::from_env() {
    factory.register_provider(AuthProvider::web3auth(config));
}

if let Ok(config) = MagicConfig::from_env() {
    factory.register_provider(AuthProvider::magic(config));
}
```

## Configuration

### Environment Variables

#### Privy

```bash
PRIVY_APP_ID=your_app_id
PRIVY_APP_SECRET=your_app_secret
PRIVY_VERIFICATION_KEY=your_verification_key  # For JWT validation
PRIVY_API_URL=https://api.privy.io           # Optional
PRIVY_AUTH_URL=https://auth.privy.io         # Optional
PRIVY_ENABLE_CACHE=true                       # Optional
```

#### Web3Auth

```bash
WEB3AUTH_CLIENT_ID=your_client_id
WEB3AUTH_VERIFIER=your_verifier
WEB3AUTH_NETWORK=mainnet                      # Optional: mainnet, testnet, cyan, aqua, celeste
WEB3AUTH_API_URL=https://api.openlogin.com   # Optional
```

#### Magic.link

```bash
MAGIC_PUBLISHABLE_KEY=your_publishable_key
MAGIC_SECRET_KEY=your_secret_key
MAGIC_API_URL=https://api.magic.link         # Optional
MAGIC_NETWORK=mainnet                         # Optional: mainnet, testnet
```

### Programmatic Configuration

```rust
// Privy
let privy_config = PrivyConfig::new(app_id, app_secret)
    .with_api_url("https://custom-api.privy.io".to_string())
    .with_cache(true);

// Web3Auth
let web3auth_config = Web3AuthConfig::new(client_id, verifier);

// Magic.link
let magic_config = MagicConfig::new(publishable_key, secret_key);
```

## Web Server Integration

### With Axum

```rust
use riglr_auth::{AuthProvider, PrivyConfig};
use riglr_web_adapters::axum::AxumAdapter;
use axum::Router;

#[tokio::main]
async fn main() {
    // Setup authentication
    let mut factory = CompositeSignerFactory::new();
    let privy_config = PrivyConfig::from_env().unwrap();
    factory.register_provider(AuthProvider::privy(privy_config));
    
    // Create adapter
    let adapter = AxumAdapter::new(Arc::new(factory), Arc::new(rpc_config));
    
    // Build router
    let app = Router::new()
        .route("/execute", adapter.create_handler())
        .layer(adapter.auth_middleware());
    
    // Start server
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
```

### With Actix-web

```rust
use riglr_auth::{AuthProvider, PrivyConfig};
use riglr_web_adapters::actix::ActixAdapter;
use actix_web::{App, HttpServer};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Setup authentication
    let mut factory = CompositeSignerFactory::new();
    let privy_config = PrivyConfig::from_env().unwrap();
    factory.register_provider(AuthProvider::privy(privy_config));
    
    // Create adapter
    let adapter = ActixAdapter::new(Arc::new(factory), Arc::new(rpc_config));
    
    // Start server
    HttpServer::new(move || {
        App::new()
            .configure(|cfg| adapter.configure(cfg))
    })
    .bind("0.0.0.0:3000")?
    .run()
    .await
}
```

## Custom Provider Implementation

```rust
use riglr_auth::provider::AuthenticationProvider;
use async_trait::async_trait;

struct CustomProvider {
    // Your configuration
}

#[async_trait]
impl SignerFactory for CustomProvider {
    async fn create_signer(
        &self,
        auth_data: AuthenticationData,
        config: &RpcConfig,
    ) -> Result<Box<dyn TransactionSigner>, Box<dyn std::error::Error + Send + Sync>> {
        // Validate token
        let token = auth_data.credentials.get("token")
            .ok_or("Missing token")?;
        
        // Create and return signer
        // ...
    }
    
    fn supported_auth_types(&self) -> Vec<String> {
        vec!["custom".to_string()]
    }
}
```

## Examples

See the `examples` directory for complete examples:

- `privy_auth.rs` - Privy authentication with Axum
- `multi_provider.rs` - Multiple providers setup

## Security Considerations

1. **Never log or expose secret keys**
2. **Always validate tokens before creating signers**
3. **Use HTTPS in production**
4. **Implement rate limiting**
5. **Rotate keys regularly**
6. **Use environment variables for sensitive configuration**

## License

MIT OR Apache-2.0
