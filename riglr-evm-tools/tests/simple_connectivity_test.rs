//! Simple connectivity test for EVM
//! Run with: cargo test -p riglr-evm-tools simple_connectivity -- --ignored

use riglr_evm_tools::client::EvmClient;

#[tokio::test]
#[ignore]
async fn test_simple_evm_connectivity() {
    // Create a Sepolia testnet client
    let rpc_url = "https://ethereum-sepolia-rpc.publicnode.com";
    let client = EvmClient::new(rpc_url.to_string())
        .await
        .expect("Failed to create EVM client");

    // Try to get chain ID (async call)
    let result = client.provider().get_chain_id().await;

    match result {
        Ok(chain_id) => {
            println!("✅ Connected to Ethereum Sepolia. Chain ID: {}", chain_id);
            assert_eq!(chain_id, 11155111);
        }
        Err(e) => {
            panic!("❌ Failed to connect to Ethereum Sepolia: {:?}", e);
        }
    }
}
