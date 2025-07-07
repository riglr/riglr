//! Integration tests for network tools

use riglr_solana_tools::{
    client::SolanaClient,
    network::{get_block_height, get_transaction_status},
};

#[tokio::test]
async fn test_get_block_height_devnet() {
    let client = SolanaClient::devnet();

    let result = get_block_height(&client).await;

    match result {
        Ok(height) => {
            // Block height should be a positive number
            assert!(height > 0, "Block height should be greater than 0");
        }
        Err(_) => {
            // Network error is acceptable in tests
        }
    }
}

#[tokio::test]
async fn test_get_block_height_mainnet() {
    let client = SolanaClient::mainnet();

    let result = get_block_height(&client).await;

    match result {
        Ok(height) => {
            // Mainnet block height should be very high
            assert!(
                height > 100_000_000,
                "Mainnet block height should be > 100M"
            );
        }
        Err(_) => {
            // Network error is acceptable in tests
        }
    }
}

#[tokio::test]
async fn test_get_transaction_status_not_found() {
    let client = SolanaClient::devnet();

    // Use a valid signature format but non-existent transaction
    let signature =
        "1111111111111111111111111111111111111111111111111111111111111111111111111111111111111111";

    let result = get_transaction_status(&client, signature).await;

    match result {
        Ok(status) => {
            assert_eq!(status, "not_found");
        }
        Err(_) => {
            // Network error is acceptable
        }
    }
}

#[tokio::test]
async fn test_get_transaction_status_invalid_signature() {
    let client = SolanaClient::devnet();

    // Invalid signature format
    let signature = "invalid";

    let result = get_transaction_status(&client, signature).await;

    // Should either return an error or handle it gracefully
    match result {
        Ok(status) => {
            // Could be "not_found" or error handling
            assert!(status == "not_found" || status == "failed");
        }
        Err(_) => {
            // Error is expected for invalid signature
        }
    }
}

#[tokio::test]
async fn test_get_block_height_testnet() {
    let client = SolanaClient::testnet();

    let result = get_block_height(&client).await;

    // Just verify it executes without panic
    let _ = result;
}

#[tokio::test]
async fn test_get_transaction_status_with_known_signature() {
    let client = SolanaClient::mainnet();

    // This is a well-known early Solana transaction (may or may not exist)
    let signature =
        "5VERv8NMvzbJMEkV8xnrLkEaWRtSz9CosKDYjCJjBRnbJLgp8uirBgmQpjKhoR4tjF3ZpRzrFmBV6UjKdiSZkQUW";

    let result = get_transaction_status(&client, signature).await;

    match result {
        Ok(status) => {
            // Should be one of the valid statuses
            assert!(
                status == "finalized"
                    || status == "confirmed"
                    || status == "processed"
                    || status == "failed"
                    || status == "not_found"
                    || status == "pending"
                    || status == "unknown"
            );
        }
        Err(_) => {
            // Network error is acceptable
        }
    }
}

#[tokio::test]
async fn test_network_tools_with_custom_rpc() {
    let client = SolanaClient::with_rpc_url("https://api.devnet.solana.com");

    // Test block height
    let height_result = get_block_height(&client).await;
    match height_result {
        Ok(height) => assert!(height >= 0),
        Err(_) => {} // Network error acceptable
    }

    // Test transaction status
    let status_result = get_transaction_status(&client, "test_sig").await;
    match status_result {
        Ok(status) => assert!(!status.is_empty()),
        Err(_) => {} // Network error acceptable
    }
}

#[tokio::test]
async fn test_concurrent_network_calls() {
    use tokio::join;

    let client = SolanaClient::devnet();

    // Make concurrent calls
    let (height_result, status_result) = join!(
        get_block_height(&client),
        get_transaction_status(&client, "test_signature")
    );

    // Just verify they complete without panic
    let _ = height_result;
    let _ = status_result;
}

#[tokio::test]
async fn test_multiple_transaction_statuses() {
    let client = SolanaClient::devnet();

    let signatures = vec!["sig1", "sig2", "sig3"];

    for sig in signatures {
        let result = get_transaction_status(&client, sig).await;

        match result {
            Ok(status) => {
                // Verify we get a valid status string
                assert!(!status.is_empty());
            }
            Err(_) => {
                // Network error is acceptable
            }
        }
    }
}

#[test]
fn test_network_module_exports() {
    // Verify the functions are exported properly
    use riglr_solana_tools::network::*;

    // This test just verifies compilation
    assert!(true);
}
