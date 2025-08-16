//! Integration tests for Solana client connectivity
//!
//! These tests verify that we can connect to Solana networks and perform basic read operations.
//! Run with: cargo test -p riglr-solana-tools client_connectivity --features integration -- --ignored

use riglr_solana_tools::client::SolanaClient;
use solana_sdk::commitment_config::CommitmentLevel;
use std::env;

/// Helper to get RPC URL from environment or use default devnet
fn get_solana_rpc_url() -> String {
    env::var("SOLANA_RPC_URL").unwrap_or_else(|_| "https://api.devnet.solana.com".to_string())
}

#[tokio::test]
#[ignore] // Run with --ignored flag
async fn test_solana_devnet_connectivity() {
    let rpc_url = get_solana_rpc_url();
    let client = SolanaClient::with_rpc_url(&rpc_url).with_commitment(CommitmentLevel::Confirmed);

    // Test basic connectivity by getting block height
    let rpc_client = &client.rpc_client;

    // Get current block height
    let block_height = rpc_client.get_block_height();
    assert!(
        block_height.is_ok(),
        "Failed to get block height: {:?}",
        block_height.err()
    );

    let height = block_height.unwrap();
    assert!(height > 0, "Block height should be greater than 0");

    println!(
        "✅ Solana Devnet connectivity test passed. Current block height: {}",
        height
    );
}

#[tokio::test]
#[ignore] // Run with --ignored flag
async fn test_solana_get_version() {
    let rpc_url = get_solana_rpc_url();
    let client = SolanaClient::with_rpc_url(&rpc_url);

    // Get cluster version
    let version = client.rpc_client.get_version();
    assert!(
        version.is_ok(),
        "Failed to get version: {:?}",
        version.err()
    );

    let version_info = version.unwrap();
    assert!(
        !version_info.solana_core.is_empty(),
        "Version string should not be empty"
    );

    println!(
        "✅ Solana version test passed. Version: {}",
        version_info.solana_core
    );
}

#[tokio::test]
#[ignore] // Run with --ignored flag
async fn test_solana_get_genesis_hash() {
    let rpc_url = get_solana_rpc_url();
    let client = SolanaClient::with_rpc_url(&rpc_url);

    // Get genesis hash - unique per network
    let genesis_hash = client.rpc_client.get_genesis_hash();
    assert!(
        genesis_hash.is_ok(),
        "Failed to get genesis hash: {:?}",
        genesis_hash.err()
    );

    let hash = genesis_hash.unwrap();
    println!("✅ Solana genesis hash test passed. Hash: {}", hash);
}

#[tokio::test]
#[ignore] // Run with --ignored flag
async fn test_solana_get_recent_blockhash() {
    let rpc_url = get_solana_rpc_url();
    let client = SolanaClient::with_rpc_url(&rpc_url);

    // Get recent blockhash - needed for transactions
    let blockhash = client.rpc_client.get_latest_blockhash();
    assert!(
        blockhash.is_ok(),
        "Failed to get recent blockhash: {:?}",
        blockhash.err()
    );

    let hash = blockhash.unwrap();
    println!("✅ Solana recent blockhash test passed. Hash: {}", hash);
}

#[tokio::test]
#[ignore] // Run with --ignored flag
async fn test_solana_get_slot() {
    let rpc_url = get_solana_rpc_url();
    let client = SolanaClient::with_rpc_url(&rpc_url);

    // Get current slot
    let slot = client.rpc_client.get_slot();
    assert!(slot.is_ok(), "Failed to get slot: {:?}", slot.err());

    let slot_number = slot.unwrap();
    assert!(slot_number > 0, "Slot should be greater than 0");

    println!("✅ Solana slot test passed. Current slot: {}", slot_number);
}

#[tokio::test]
#[ignore] // Run with --ignored flag
async fn test_solana_health_check() {
    let rpc_url = get_solana_rpc_url();
    let client = SolanaClient::with_rpc_url(&rpc_url);

    // Health check - ensures node is caught up
    let health = client.rpc_client.get_health();

    // Note: Health check might return an error if node is behind, but connection should work
    if health.is_ok() {
        println!("✅ Solana health check passed");
    } else if let Err(e) = health {
        // Check if it's just a health status issue (node behind) vs connection issue
        let error_msg = format!("{:?}", e);
        if error_msg.contains("behind") {
            println!("⚠️ Solana node is behind but connection works");
        } else {
            panic!("Failed to connect to Solana RPC: {:?}", e);
        }
    }
}

#[tokio::test]
#[ignore] // Run with --ignored flag
async fn test_solana_get_minimum_balance_for_rent_exemption() {
    let rpc_url = get_solana_rpc_url();
    let client = SolanaClient::with_rpc_url(&rpc_url);

    // Get minimum balance for rent exemption (important for account creation)
    let min_balance = client.rpc_client.get_minimum_balance_for_rent_exemption(0);
    assert!(
        min_balance.is_ok(),
        "Failed to get minimum balance: {:?}",
        min_balance.err()
    );

    let balance = min_balance.unwrap();
    assert!(balance > 0, "Minimum balance should be greater than 0");

    println!(
        "✅ Solana minimum balance test passed. Min balance: {} lamports",
        balance
    );
}

#[tokio::test]
#[ignore] // Run with --ignored flag
async fn test_solana_testnet_connectivity() {
    // Try testnet if configured
    if let Ok(testnet_url) = env::var("SOLANA_TESTNET_RPC_URL") {
        let client = SolanaClient::with_rpc_url(&testnet_url);

        let block_height = client.rpc_client.get_block_height();
        assert!(
            block_height.is_ok(),
            "Failed to connect to testnet: {:?}",
            block_height.err()
        );

        println!(
            "✅ Solana Testnet connectivity test passed. Block height: {}",
            block_height.unwrap()
        );
    } else {
        println!("⚠️ Skipping testnet test - SOLANA_TESTNET_RPC_URL not set");
    }
}

#[tokio::test]
#[ignore] // Run with --ignored flag
async fn test_multiple_concurrent_connections() {
    use tokio::join;

    let rpc_url = get_solana_rpc_url();

    // Create multiple clients
    let client1 = SolanaClient::with_rpc_url(&rpc_url);
    let client2 = SolanaClient::with_rpc_url(&rpc_url);
    let client3 = SolanaClient::with_rpc_url(&rpc_url);

    // Make concurrent requests - use spawn_blocking for synchronous calls
    let client1_clone = client1.rpc_client.clone();
    let client2_clone = client2.rpc_client.clone();
    let client3_clone = client3.rpc_client.clone();

    let (result1, result2, result3) = join!(
        tokio::task::spawn_blocking(move || client1_clone.get_block_height()),
        tokio::task::spawn_blocking(move || client2_clone.get_version()),
        tokio::task::spawn_blocking(move || client3_clone.get_slot())
    );

    let result1 = result1.unwrap();
    let result2 = result2.unwrap();
    let result3 = result3.unwrap();

    assert!(result1.is_ok(), "Client 1 failed");
    assert!(result2.is_ok(), "Client 2 failed");
    assert!(result3.is_ok(), "Client 3 failed");

    println!("✅ Concurrent connections test passed");
}

#[cfg(test)]
mod test_helpers {
    use super::*;

    /// Verify that test environment is properly configured
    #[test]
    fn verify_test_environment() {
        // Check if we can read environment variables
        if env::var("SOLANA_RPC_URL").is_err() {
            println!("ℹ️ SOLANA_RPC_URL not set, will use default devnet endpoint");
        }

        if env::var("SOLANA_TESTNET_RPC_URL").is_err() {
            println!("ℹ️ SOLANA_TESTNET_RPC_URL not set, testnet tests will be skipped");
        }
    }
}
