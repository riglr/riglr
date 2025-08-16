//! Example demonstrating multiple authentication providers

use riglr_auth::{
    AuthProvider, PrivyConfig, Web3AuthConfig, MagicConfig,
    CompositeSignerFactoryExt,
};
use riglr_auth::config::ProviderConfig;
use riglr_web_adapters::factory::{CompositeSignerFactory, AuthenticationData, SignerFactory};
use riglr_core::config::RpcConfig;
use std::collections::HashMap;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt::init();
    
    println!("🔐 Setting up multi-provider authentication...\n");
    
    // Create composite factory
    let mut factory = CompositeSignerFactory::new();
    
    // Register Privy provider if configured
    match PrivyConfig::from_env() {
        Ok(config) => {
            factory.register_provider(AuthProvider::privy(config));
            println!("✅ Privy provider registered");
        }
        Err(e) => {
            println!("⚠️  Privy provider not configured: {}", e);
        }
    }
    
    // Register Web3Auth provider if configured
    match Web3AuthConfig::from_env() {
        Ok(config) => {
            factory.register_provider(AuthProvider::web3auth(config));
            println!("✅ Web3Auth provider registered");
        }
        Err(e) => {
            println!("⚠️  Web3Auth provider not configured: {}", e);
        }
    }
    
    // Register Magic provider if configured
    match MagicConfig::from_env() {
        Ok(config) => {
            factory.register_provider(AuthProvider::magic(config));
            println!("✅ Magic.link provider registered");
        }
        Err(e) => {
            println!("⚠️  Magic.link provider not configured: {}", e);
        }
    }
    
    // Display registered providers
    let registered_types = factory.get_registered_auth_types();
    println!("\n📋 Registered authentication types: {:?}", registered_types);
    
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
        
        let rpc_config = RpcConfig::default();
        
        match factory.create_signer(auth_data, &rpc_config).await {
            Ok(_signer) => {
                println!("✅ Successfully created signer with Privy");
            }
            Err(e) => {
                println!("❌ Failed to create signer: {}", e);
            }
        }
    }
    
    println!("\n🎉 Multi-provider authentication setup complete!");
    
    Ok(())
}