//! Example demonstrating multiple authentication providers

use core::error::Error;
use riglr_auth::config::ProviderConfig;
use riglr_auth::provider::{AuthenticationData, SignerFactory};
use riglr_auth::{AuthProvider, MagicConfig, PrivyConfig, Web3AuthConfig};
use riglr_core::signer::UnifiedSigner;
use std::collections::HashMap;
use std::sync::Arc;
use tracing_subscriber::fmt;

/// Simple composite factory for demonstration
#[derive(Default)]
struct CompositeSignerFactory {
    factories: HashMap<String, Arc<dyn SignerFactory>>,
}

impl CompositeSignerFactory {
    async fn create_signer(
        &self,
        auth_data: AuthenticationData,
    ) -> Result<Box<dyn UnifiedSigner>, Box<dyn Error + Send + Sync>> {
        let factory = self
            .factories
            .get(&auth_data.auth_type)
            .ok_or_else(|| format!("Unsupported auth type: {}", auth_data.auth_type))?;

        factory.create_signer(auth_data).await
    }
    fn get_registered_auth_types(&self) -> Vec<String> {
        self.factories.keys().cloned().collect()
    }
    fn new() -> Self {
        Self::default()
    }

    fn register_factory(&mut self, auth_type: String, factory: Box<dyn SignerFactory>) {
        self.factories.insert(auth_type, Arc::from(factory));
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Initialize tracing
    fmt::init();

    println!("🔐 Setting up multi-provider authentication...\n");

    // Create composite factory
    let mut factory = CompositeSignerFactory::new();

    // Register Privy provider if configured
    match PrivyConfig::from_env() {
        Ok(config) => {
            let provider = AuthProvider::privy(config);
            factory.register_factory(provider.auth_type(), Box::new(provider));
            println!("✅ Privy provider registered");
        }
        Err(e) => {
            println!("⚠️  Privy provider not configured: {e}");
        }
    }

    // Register Web3Auth provider if configured
    match Web3AuthConfig::from_env() {
        Ok(config) => {
            let provider = AuthProvider::web3auth(config);
            factory.register_factory(provider.auth_type(), Box::new(provider));
            println!("✅ Web3Auth provider registered");
        }
        Err(e) => {
            println!("⚠️  Web3Auth provider not configured: {e}");
        }
    }

    // Register Magic provider if configured
    match MagicConfig::from_env() {
        Ok(config) => {
            let provider = AuthProvider::magic(config);
            factory.register_factory(provider.auth_type(), Box::new(provider));
            println!("✅ Magic.link provider registered");
        }
        Err(e) => {
            println!("⚠️  Magic.link provider not configured: {e}");
        }
    }

    // Display registered providers
    let registered_types = factory.get_registered_auth_types();
    println!("\n📋 Registered authentication types: {registered_types:?}");

    // Example: Create a signer using one of the providers
    if registered_types.contains(&"privy".to_string()) {
        println!("\n🔄 Testing Privy authentication...");

        let mut credentials = HashMap::new();
        credentials.insert("token".to_string(), "example_token".to_string());

        let auth_data = AuthenticationData {
            auth_type: "privy".to_string(),
            credentials,
            network: "mainnet".to_string(),
        };

        match factory.create_signer(auth_data).await {
            Ok(_signer) => {
                println!("✅ Successfully created signer with Privy");
            }
            Err(e) => {
                println!("❌ Failed to create signer: {e}");
            }
        }
    }

    println!("\n🎉 Multi-provider authentication setup complete!");

    Ok(())
}
