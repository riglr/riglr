//! Simple connectivity test for Solana
//! Run with: cargo test -p riglr-solana-tools simple_connectivity -- --ignored

use riglr_solana_tools::client::SolanaClient;

#[test]
#[ignore]
fn test_simple_solana_connectivity() {
    // Create a devnet client
    let client = SolanaClient::devnet();

    // Try to get block height (synchronous call)
    let result = client.rpc_client.get_block_height();

    match result {
        Ok(height) => {
            println!("✅ Connected to Solana devnet. Block height: {}", height);
            assert!(height > 0);
        }
        Err(e) => {
            panic!("❌ Failed to connect to Solana devnet: {:?}", e);
        }
    }
}
