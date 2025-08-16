//! Example demonstrating Privy authentication integration

use axum::{http::StatusCode, routing::post, Json, Router};
use riglr_auth::config::ProviderConfig;
use riglr_auth::{AuthProvider, CompositeSignerFactoryExt, PrivyConfig};
use riglr_core::config::RpcConfig;
use riglr_web_adapters::factory::CompositeSignerFactory;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
struct AuthRequest {
    _token: String,
    network: String,
}

#[derive(Debug, Serialize)]
struct AuthResponse {
    success: bool,
    address: Option<String>,
    message: Option<String>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Create Privy configuration from environment
    let privy_config =
        PrivyConfig::from_env().expect("Failed to load Privy configuration from environment");

    // Create composite factory and register Privy provider
    let mut _factory = CompositeSignerFactory::new();
    _factory.register_provider(AuthProvider::privy(privy_config));

    // Create RPC configuration
    let _rpc_config = RpcConfig::from_env().expect("Failed to load RPC configuration");

    // Build Axum router
    let app = Router::new().route("/auth", post(handle_auth));

    // Start server
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;

    println!("🚀 Server running on http://0.0.0.0:3000");
    println!("📝 Endpoints:");
    println!("   POST /auth - Authenticate with Privy token");
    println!("   POST /execute - Execute blockchain operations");

    axum::serve(listener, app).await?;

    Ok(())
}

async fn handle_auth(Json(req): Json<AuthRequest>) -> (StatusCode, Json<AuthResponse>) {
    // This is a simple example endpoint
    // In production, you would validate the token and return user info

    println!("Received auth request for network: {}", req.network);

    (
        StatusCode::OK,
        Json(AuthResponse {
            success: true,
            address: Some("0x...".to_string()),
            message: Some("Authentication successful".to_string()),
        }),
    )
}
