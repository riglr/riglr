//! Example demonstrating EVM transfer operations using `SignerContext`
//!
//! This example shows the correct pattern for write operations using the `SignerContext`.
//!
//! ## Security Note
//! This example loads private keys securely from `~/.riglr/keys/evm.key`
//! with fallback to `EVM_PRIVATE_KEY` environment variable for compatibility.

use alloy::primitives::Address;
use anyhow::Result;
use core::str::FromStr;
use riglr_config::Config;
use riglr_core::{
    provider::ApplicationContext,
    signer::{error::Standard, SignerContext, SignerError, UnifiedSigner},
    util::{ensure_key_directory, load_private_key_with_fallback},
};
use riglr_evm_tools::{transaction::send_eth, EvmLocalClient};
use std::env;
use std::sync::Arc;
use tracing_subscriber::fmt;

// Environment variable constants
const EVM_PRIVATE_KEY: &str = "EVM_PRIVATE_KEY";
const EVM_RPC_URL: &str = "EVM_RPC_URL";
const EVM_CHAIN_ID: &str = "EVM_CHAIN_ID";

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    fmt::init();

    // Load configuration from environment
    let config = Config::from_env();

    // Create application context
    let app_context = ApplicationContext::from_config(&config);

    // Load private key securely from file with environment variable fallback
    println!("Loading EVM private key...");
    let key_dir = ensure_key_directory()
        .map_err(|e| anyhow::anyhow!("Failed to create key directory: {}", e))?;
    let key_path = key_dir.join("evm.key");

    let private_key = load_private_key_with_fallback(&key_path, EVM_PRIVATE_KEY).map_err(|_| {
        anyhow::anyhow!(
            "Private key not found. Place it in ~/.riglr/keys/evm.key or set EVM_PRIVATE_KEY"
        )
    })?;

    println!(
        "Private key loaded from: {} (or env var fallback)",
        key_path.display()
    );

    let rpc_url = env::var(EVM_RPC_URL).unwrap_or_else(|_| "https://eth.llamarpc.com".to_string());

    let chain_id = env::var(EVM_CHAIN_ID)
        .unwrap_or_else(|_| "1".to_string())
        .parse::<u64>()?;

    // Create the signer and wrap it in Arc for UnifiedSigner trait
    let signer = EvmLocalClient::new_with_url(&private_key, rpc_url, chain_id)?;
    let unified_signer: Arc<dyn UnifiedSigner> = Arc::new(signer);

    // Execute the transfer within a SignerContext
    let result = SignerContext::with_signer(unified_signer, async {
        // Parse recipient address
        let to_address =
            Address::from_str("0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb5").map_err(|e| {
                Box::new(Standard::Configuration(format!("Invalid address: {e}")))
                    as Box<dyn SignerError>
            })?;

        // Amount to send (0.001 ETH)
        let amount_eth = 0.001;

        println!("Sending {amount_eth} ETH to {to_address}");

        // Execute the transfer (send_eth expects: to: String, amount_eth: f64, chain_id: Option<u64>, context: &ApplicationContext)
        let tx_hash = send_eth(
            to_address.to_string(),
            amount_eth,
            Some(chain_id),
            &app_context,
        )
        .await
        .map_err(|e| Box::new(Standard::SigningFailed(e.to_string())) as Box<dyn SignerError>)?;

        println!("✅ Transfer successful!");
        println!("Transaction hash: {tx_hash}");
        println!("View on Etherscan: https://etherscan.io/tx/{tx_hash}");

        Ok::<_, Box<dyn SignerError>>(tx_hash)
    })
    .await
    .map_err(|e| anyhow::anyhow!("Signer operation failed: {}", e))?;

    println!("\n📊 Transaction completed: {result}");

    Ok(())
}
