//! Example demonstrating EVM transfer operations using SignerContext
//!
//! This example shows the correct pattern for write operations using the SignerContext.

use alloy::primitives::Address;
use anyhow::Result;
use riglr_core::{
    provider::ApplicationContext,
    signer::{LocalEvmSigner, SignerContext, SignerError, UnifiedSigner},
};
use riglr_evm_tools::transaction::send_eth;
use std::{str::FromStr, sync::Arc};

// Environment variable constants
const EVM_PRIVATE_KEY: &str = "EVM_PRIVATE_KEY";
const EVM_RPC_URL: &str = "EVM_RPC_URL";
const EVM_CHAIN_ID: &str = "EVM_CHAIN_ID";

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    // Create application context
    let app_context = ApplicationContext::from_env();

    // Create a local EVM signer
    // In production, this would typically come from secure key management
    let private_key =
        std::env::var(EVM_PRIVATE_KEY).expect("EVM_PRIVATE_KEY environment variable must be set");

    let rpc_url =
        std::env::var(EVM_RPC_URL).unwrap_or_else(|_| "https://eth.llamarpc.com".to_string());

    let chain_id = std::env::var(EVM_CHAIN_ID)
        .unwrap_or_else(|_| "1".to_string())
        .parse::<u64>()?;

    // Create the signer and wrap it in Arc for UnifiedSigner trait
    let signer = LocalEvmSigner::new_with_url(private_key, rpc_url, chain_id)?;
    let unified_signer: Arc<dyn UnifiedSigner> = Arc::new(signer);

    // Execute the transfer within a SignerContext
    let result = SignerContext::with_signer(unified_signer, async {
        // Parse recipient address
        let to_address = Address::from_str("0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb5")
            .map_err(|e| SignerError::Configuration(format!("Invalid address: {}", e)))?;

        // Amount to send (0.001 ETH)
        let amount_eth = "0.001";

        println!("Sending {} ETH to {}", amount_eth, to_address);

        // Execute the transfer (send_eth expects: to: String, amount: String, chain_id: Option<u64>, context: &ApplicationContext)
        let tx_hash = send_eth(
            to_address.to_string(),
            amount_eth.to_string(),
            Some(chain_id),
            &app_context,
        )
        .await
        .map_err(|e| SignerError::TransactionFailed(e.to_string()))?;

        println!("✅ Transfer successful!");
        println!("Transaction hash: {}", tx_hash);
        println!("View on Etherscan: https://etherscan.io/tx/{}", tx_hash);

        Ok::<_, SignerError>(tx_hash)
    })
    .await?;

    println!("\n📊 Transaction completed: {}", result);

    Ok(())
}
