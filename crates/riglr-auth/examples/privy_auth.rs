//! Example demonstrating Privy authentication integration

use axum::{http::StatusCode, routing::post, Json, Router};
use core::error::Error;
use riglr_auth::config::ProviderConfig;
use riglr_auth::provider::SignerFactory;
use riglr_auth::{AuthProvider, PrivyConfig};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::net::TcpListener;
use tracing_subscriber::fmt;

/// Simple composite factory for demonstration
#[derive(Default)]
struct CompositeSignerFactory {
    factories: HashMap<String, Arc<dyn SignerFactory>>,
}

impl CompositeSignerFactory {
    fn new() -> Self {
        Self::default()
    }

    fn register_factory(&mut self, auth_type: String, factory: Box<dyn SignerFactory>) {
        self.factories.insert(auth_type, Arc::from(factory));
    }
}

#[derive(Debug, Deserialize)]
struct AuthRequest {
    _token: String,
    network: String,
}

#[derive(Debug, Serialize)]
struct AuthResponse {
    address: Option<String>,
    message: Option<String>,
    success: bool,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Initialize tracing
    fmt::init();

    // Create Privy configuration from environment
    let privy_config = match PrivyConfig::from_env() {
        Ok(config) => config,
        Err(e) => {
            eprintln!("Failed to load Privy configuration from environment: {e}");
            return Err(e.into());
        }
    };

    // Create composite factory and register Privy provider
    let mut factory = CompositeSignerFactory::new();
    let privy_provider = AuthProvider::privy(privy_config);
    factory.register_factory(privy_provider.auth_type(), Box::new(privy_provider));

    // Build Axum router
    let app = Router::new().route("/auth", post(handle_auth));

    // Start server
    let listener = TcpListener::bind("0.0.0.0:3000").await?;

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
            address: Some("0x...".to_string()),
            message: Some("Authentication successful".to_string()),
            success: true,
        }),
    )
}
