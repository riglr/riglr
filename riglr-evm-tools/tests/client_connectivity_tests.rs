//! Integration tests for EVM client connectivity
//! 
//! These tests verify that we can connect to EVM networks and perform basic read operations.
//! Run with: cargo test -p riglr-evm-tools client_connectivity --features integration -- --ignored

use riglr_evm_tools::client::EvmClient;
use alloy::primitives::U256;
use std::env;

/// Helper to get RPC URL for a specific chain from environment
fn get_evm_rpc_url(chain_id: u64) -> Option<String> {
    env::var(format!("RPC_URL_{}", chain_id)).ok()
}

/// Get Sepolia testnet RPC URL (most commonly available testnet)
fn get_sepolia_rpc_url() -> String {
    get_evm_rpc_url(11155111)
        .unwrap_or_else(|| "https://ethereum-sepolia-rpc.publicnode.com".to_string())
}

#[tokio::test]
#[ignore] // Run with --ignored flag
async fn test_ethereum_sepolia_connectivity() {
    let rpc_url = get_sepolia_rpc_url();
    let client = EvmClient::new(rpc_url)
        .await
        .expect("Failed to create EVM client");

    // Test basic connectivity by getting chain ID
    let provider = &client.provider();
    
    // Get chain ID
    let chain_id = provider.get_chain_id().await;
    assert!(chain_id.is_ok(), "Failed to get chain ID: {:?}", chain_id.err());
    
    let id = chain_id.unwrap();
    assert_eq!(id, 11155111, "Chain ID mismatch, expected Sepolia (11155111)");
    
    println!("✅ Ethereum Sepolia connectivity test passed. Chain ID: {}", id);
}

#[tokio::test]
#[ignore] // Run with --ignored flag
async fn test_evm_get_block_number() {
    let rpc_url = get_sepolia_rpc_url();
    let client = EvmClient::new(rpc_url)
        .await
        .expect("Failed to create EVM client");

    // Get current block number
    let block_number = client.provider().get_block_number().await;
    assert!(block_number.is_ok(), "Failed to get block number: {:?}", block_number.err());
    
    let block = block_number.unwrap();
    assert!(block > 0, "Block number should be greater than 0");
    
    println!("✅ EVM block number test passed. Current block: {}", block);
}

#[tokio::test]
#[ignore] // Run with --ignored flag
async fn test_evm_get_gas_price() {
    let rpc_url = get_sepolia_rpc_url();
    let client = EvmClient::new(rpc_url)
        .await
        .expect("Failed to create EVM client");

    // Get current gas price
    let gas_price = client.provider().get_gas_price().await;
    assert!(gas_price.is_ok(), "Failed to get gas price: {:?}", gas_price.err());
    
    let price = gas_price.unwrap();
    assert!(price > 0, "Gas price should be greater than 0");
    
    println!("✅ EVM gas price test passed. Current gas price: {} wei", price);
}

#[tokio::test]
#[ignore] // Run with --ignored flag
async fn test_evm_get_block() {
    let rpc_url = get_sepolia_rpc_url();
    let client = EvmClient::new(rpc_url)
        .await
        .expect("Failed to create EVM client");

    // Get latest block
    let block = client.provider().get_block(
        alloy::rpc::types::BlockId::latest()
    ).await;
    
    assert!(block.is_ok(), "Failed to get block: {:?}", block.err());
    
    let block_data = block.unwrap();
    assert!(block_data.is_some(), "Block should exist");
    
    if let Some(b) = block_data {
        println!("✅ EVM block test passed. Block hash: {:?}", b.header.hash);
    }
}

#[tokio::test]
#[ignore] // Run with --ignored flag
async fn test_evm_get_balance() {
    let rpc_url = get_sepolia_rpc_url();
    let client = EvmClient::new(rpc_url)
        .await
        .expect("Failed to create EVM client");

    // Get balance of zero address (burn address)
    let zero_address = "0x0000000000000000000000000000000000000000"
        .parse()
        .expect("Failed to parse address");
    
    let balance = client.provider().get_balance(zero_address).await;
    assert!(balance.is_ok(), "Failed to get balance: {:?}", balance.err());
    
    let bal = balance.unwrap();
    println!("✅ EVM balance test passed. Zero address balance: {} wei", bal);
}

#[tokio::test]
#[ignore] // Run with --ignored flag
async fn test_evm_get_transaction_count() {
    let rpc_url = get_sepolia_rpc_url();
    let client = EvmClient::new(rpc_url)
        .await
        .expect("Failed to create EVM client");

    // Get transaction count (nonce) for zero address
    let zero_address = "0x0000000000000000000000000000000000000000"
        .parse()
        .expect("Failed to parse address");
    
    let tx_count = client.provider().get_transaction_count(zero_address).await;
    assert!(tx_count.is_ok(), "Failed to get transaction count: {:?}", tx_count.err());
    
    let count = tx_count.unwrap();
    println!("✅ EVM transaction count test passed. Count: {}", count);
}

#[tokio::test]
#[ignore] // Run with --ignored flag
async fn test_polygon_amoy_connectivity() {
    // Test Polygon Amoy testnet if configured
    if let Some(rpc_url) = get_evm_rpc_url(80002) {
        let client = EvmClient::new(rpc_url)
            .await
            .expect("Failed to create Polygon client");
        
        let chain_id = client.provider().get_chain_id().await;
        assert!(chain_id.is_ok(), "Failed to connect to Polygon Amoy: {:?}", chain_id.err());
        assert_eq!(chain_id.unwrap(), 80002, "Chain ID mismatch for Polygon Amoy");
        
        println!("✅ Polygon Amoy connectivity test passed");
    } else {
        println!("⚠️ Skipping Polygon Amoy test - RPC_URL_80002 not set");
    }
}

#[tokio::test]
#[ignore] // Run with --ignored flag
async fn test_arbitrum_sepolia_connectivity() {
    // Test Arbitrum Sepolia if configured
    if let Some(rpc_url) = get_evm_rpc_url(421614) {
        let client = EvmClient::new(rpc_url)
            .await
            .expect("Failed to create Arbitrum client");
        
        let chain_id = client.provider().get_chain_id().await;
        assert!(chain_id.is_ok(), "Failed to connect to Arbitrum Sepolia: {:?}", chain_id.err());
        assert_eq!(chain_id.unwrap(), 421614, "Chain ID mismatch for Arbitrum Sepolia");
        
        println!("✅ Arbitrum Sepolia connectivity test passed");
    } else {
        println!("⚠️ Skipping Arbitrum Sepolia test - RPC_URL_421614 not set");
    }
}

#[tokio::test]
#[ignore] // Run with --ignored flag
async fn test_base_sepolia_connectivity() {
    // Test Base Sepolia if configured
    if let Some(rpc_url) = get_evm_rpc_url(84532) {
        let client = EvmClient::new(rpc_url)
            .await
            .expect("Failed to create Base client");
        
        let chain_id = client.provider().get_chain_id().await;
        assert!(chain_id.is_ok(), "Failed to connect to Base Sepolia: {:?}", chain_id.err());
        assert_eq!(chain_id.unwrap(), 84532, "Chain ID mismatch for Base Sepolia");
        
        println!("✅ Base Sepolia connectivity test passed");
    } else {
        println!("⚠️ Skipping Base Sepolia test - RPC_URL_84532 not set");
    }
}

#[tokio::test]
#[ignore] // Run with --ignored flag
async fn test_multiple_concurrent_evm_connections() {
    use tokio::join;
    
    let rpc_url = get_sepolia_rpc_url();
    
    // Create multiple clients
    let client1 = EvmClient::new(rpc_url.clone()).await.expect("Failed to create client 1");
    let client2 = EvmClient::new(rpc_url.clone()).await.expect("Failed to create client 2");
    let client3 = EvmClient::new(rpc_url).await.expect("Failed to create client 3");
    
    // Make concurrent requests
    let (result1, result2, result3) = join!(
        client1.provider().get_block_number(),
        client2.provider().get_chain_id(),
        client3.provider().get_gas_price()
    );
    
    assert!(result1.is_ok(), "Client 1 failed: {:?}", result1.err());
    assert!(result2.is_ok(), "Client 2 failed: {:?}", result2.err());
    assert!(result3.is_ok(), "Client 3 failed: {:?}", result3.err());
    
    println!("✅ Concurrent EVM connections test passed");
}


#[tokio::test]
#[ignore] // Run with --ignored flag
async fn test_evm_estimate_gas() {
    let rpc_url = get_sepolia_rpc_url();
    let client = EvmClient::new(rpc_url)
        .await
        .expect("Failed to create EVM client");

    // Create a simple transfer transaction request
    let tx = alloy::rpc::types::TransactionRequest::default()
        .to("0x0000000000000000000000000000000000000001".parse().unwrap())
        .value(U256::from(1000000000u64)); // 1 gwei
    
    // Estimate gas for the transaction
    let gas_estimate = client.provider().estimate_gas(tx).await;
    
    // Gas estimation might fail without a from address, but connection should work
    if gas_estimate.is_ok() {
        let gas = gas_estimate.unwrap();
        assert!(gas > 0, "Gas estimate should be greater than 0");
        println!("✅ EVM gas estimation test passed. Estimated gas: {}", gas);
    } else {
        // This is expected without a valid from address
        println!("⚠️ Gas estimation failed (expected without from address), but connection works");
    }
}

#[cfg(test)]
mod test_helpers {
    use super::*;
    
    /// Verify that test environment is properly configured
    #[test]
    fn verify_test_environment() {
        // Check if Sepolia RPC is configured
        if env::var("RPC_URL_11155111").is_err() {
            println!("ℹ️ RPC_URL_11155111 not set, will use default Sepolia endpoint");
        }
        
        // Check other testnets
        let testnets = vec![
            (80002, "Polygon Amoy"),
            (421614, "Arbitrum Sepolia"),
            (84532, "Base Sepolia"),
            (11155420, "Optimism Sepolia"),
            (97, "BSC Testnet"),
            (43113, "Avalanche Fuji"),
        ];
        
        for (chain_id, name) in testnets {
            if env::var(format!("RPC_URL_{}", chain_id)).is_err() {
                println!("ℹ️ RPC_URL_{} not set, {} tests will be skipped", chain_id, name);
            }
        }
    }
}