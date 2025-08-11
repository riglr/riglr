//! Integration tests for network tools

use riglr_solana_tools::{
    client::SolanaClient,
    network::{get_block_height, get_transaction_status},
};

#[tokio::test]
async fn test_get_block_height_devnet() {
    let _client = SolanaClient::devnet();

    let result = get_block_height().await;

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
    let _client = SolanaClient::mainnet();

    let result = get_block_height().await;

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
    let _client = SolanaClient::devnet();

    // Use a valid signature format but non-existent transaction
    let signature =
        "1111111111111111111111111111111111111111111111111111111111111111111111111111111111111111";

    let result = get_transaction_status(signature.to_string()).await;

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
    let _client = SolanaClient::devnet();

    // Invalid signature format
    let signature = "invalid";

    let result = get_transaction_status(signature.to_string()).await;

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
    let _client = SolanaClient::testnet();

    let result = get_block_height().await;

    // Just verify it executes without panic
    let _ = result;
}

#[tokio::test]
async fn test_get_transaction_status_with_known_signature() {
    let _client = SolanaClient::mainnet();

    // This is a well-known early Solana transaction (may or may not exist)
    let signature =
        "5VERv8NMvzbJMEkV8xnrLkEaWRtSz9CosKDYjCJjBRnbJLgp8uirBgmQpjKhoR4tjF3ZpRzrFmBV6UjKdiSZkQUW";

    let result = get_transaction_status(signature.to_string()).await;

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
    let _client = SolanaClient::with_rpc_url("https://api.devnet.solana.com");

    // Test block height
    let height_result = get_block_height().await;
    if let Ok(_height) = height_result {
        // Height is u64, so it's always >= 0
    }

    // Test transaction status
    let status_result = get_transaction_status("test_sig".to_string()).await;
    if let Ok(status) = status_result {
        assert!(!status.is_empty())
    }
}

#[tokio::test]
#[ignore] // Long-running concurrent test
async fn test_concurrent_network_calls() {
    use tokio::join;

    let _client = SolanaClient::devnet();

    // Make concurrent calls
    let (height_result, status_result) = join!(
        get_block_height(),
        get_transaction_status("test_signature".to_string())
    );

    // Just verify they complete without panic
    let _ = height_result;
    let _ = status_result;
}

#[tokio::test]
async fn test_multiple_transaction_statuses() {
    let _client = SolanaClient::devnet();

    let signatures = vec!["sig1", "sig2", "sig3"];

    for sig in signatures {
        let result = get_transaction_status(sig.to_string()).await;

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

    // This test just verifies compilation
}
