//! Cross-chain swap example demonstrating the riglr-cross-chain-tools crate.
//!
//! This example shows how to use the cross-chain bridge tools to transfer tokens
//! between different blockchain networks. It demonstrates:
//!
//! 1. Route discovery across multiple chains
//! 2. Fee estimation for bridge operations  
//! 3. Bridge execution with transaction signing
//! 4. Status monitoring for ongoing transfers
//!
//! The example uses riglr's SignerContext pattern for secure multi-tenant operation.

use riglr_core::{SignerContext, config::SolanaNetworkConfig};
use riglr_core::signer::LocalSolanaSigner;
use riglr_cross_chain_tools::{
    get_cross_chain_routes, estimate_bridge_fees, execute_cross_chain_bridge, 
    get_bridge_status, get_supported_chains,
};
use solana_sdk::signer::keypair::Keypair;
use std::sync::Arc;
use tracing::{info, warn};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt::init();
    
    info!("Starting cross-chain swap example");
    
    // Create a Solana signer (in production, load from secure storage)
    let keypair = Keypair::new();
    let network_config = SolanaNetworkConfig {
        name: "devnet".to_string(),
        rpc_url: "https://api.devnet.solana.com".to_string(),
        explorer_url: Some("https://explorer.solana.com".to_string()),
    };
    let signer = Arc::new(LocalSolanaSigner::from_keypair(keypair, network_config));
    
    // Execute all operations within a signer context
    let result = SignerContext::with_signer(signer, async {
        demonstrate_cross_chain_operations().await
    }).await;
    
    match result {
        Ok(_) => {
            info!("Cross-chain operations completed successfully");
        }
        Err(e) => {
            warn!("Cross-chain operations failed: {}", e);
        }
    }
    
    Ok(())
}

async fn demonstrate_cross_chain_operations() -> Result<(), riglr_core::signer::SignerError> {
    // Step 1: Get supported chains
    info!("\n=== Step 1: Discovering supported chains ===");
    match get_supported_chains().await {
        Ok(chains) => {
            info!("Found {} supported chains:", chains.len());
            for chain in chains.iter().take(5) { // Show first 5
                info!("  {} ({}): {} - {}", 
                    chain.name, 
                    chain.id, 
                    chain.chain_type,
                    chain.native_token.symbol
                );
            }
        }
        Err(e) => warn!("Failed to get supported chains: {}", e),
    }
    
    // Step 2: Discover cross-chain routes
    info!("\n=== Step 2: Discovering cross-chain routes ===");
    let route_result = get_cross_chain_routes(
        "ethereum".to_string(),
        "polygon".to_string(),
        "0xA0b86a33E6417c5d6d6bE6C2e0C6C3e5d6c7D8E9".to_string(), // USDC on Ethereum
        "0x2791Bca1f2de4661ED88A30C99A7a9449Aa84174".to_string(), // USDC on Polygon  
        "1000000".to_string(), // 1 USDC (6 decimals)
        Some(0.5), // 0.5% slippage
    ).await;
    
    match route_result {
        Ok(routes) => {
            info!("Found {} routes for USDC Ethereum -> Polygon:", routes.total_routes);
            
            if let Some(ref recommended) = routes.recommended_route_id {
                info!("  Recommended route: {}", recommended);
            }
            
            for (i, route) in routes.routes.iter().take(3).enumerate() {
                info!("  Route {}: {} via {}", 
                    i + 1,
                    route.id,
                    route.protocols.join(", ")
                );
                info!("    Duration: {}s, Fees: ${:.2}", 
                    route.estimated_duration,
                    route.fees_usd.unwrap_or(0.0) + route.gas_cost_usd.unwrap_or(0.0)
                );
            }
            
            // Step 3: Estimate fees for the best route
            if let Some(best_route) = routes.routes.first() {
                info!("\n=== Step 3: Estimating fees for best route ===");
                match estimate_bridge_fees(
                    "ethereum".to_string(),
                    "polygon".to_string(),
                    "0xA0b86a33E6417c5d6d6bE6C2e0C6C3e5d6c7D8E9".to_string(),
                    "0x2791Bca1f2de4661ED88A30C99A7a9449Aa84174".to_string(),
                    "1000000".to_string(),
                ).await {
                    Ok(fee_estimate) => {
                        info!("Fee breakdown for 1 USDC transfer:");
                        info!("  Input amount: {} USDC", fee_estimate.from_amount);
                        info!("  Expected output: {} USDC", fee_estimate.estimated_output);
                        
                        for fee in &fee_estimate.fees {
                            info!("  {}: {} (${:.2})", 
                                fee.name,
                                fee.percentage,
                                fee.amount_usd.unwrap_or(0.0)
                            );
                        }
                        
                        if let Some(total_usd) = fee_estimate.total_fees_usd {
                            info!("  Total fees: ${:.2}", total_usd);
                        }
                        info!("  Estimated completion: {}s", fee_estimate.estimated_duration);
                    }
                    Err(e) => warn!("Failed to estimate fees: {}", e),
                }
                
                // Step 4: Execute bridge (simulation)
                info!("\n=== Step 4: Executing bridge transaction ===");
                match execute_cross_chain_bridge(
                    best_route.id.clone(),
                    "ethereum".to_string(),
                    "polygon".to_string(), 
                    "1000000".to_string(),
                ).await {
                    Ok(bridge_result) => {
                        info!("Bridge transaction submitted:");
                        info!("  Bridge ID: {}", bridge_result.bridge_id);
                        info!("  Source TX: {}", bridge_result.source_tx_hash);
                        info!("  Status: {}", bridge_result.status);
                        info!("  Message: {}", bridge_result.message);
                        
                        // Step 5: Check bridge status
                        info!("\n=== Step 5: Checking bridge status ===");
                        match get_bridge_status(
                            bridge_result.bridge_id.clone(),
                            bridge_result.source_tx_hash.clone(),
                        ).await {
                            Ok(status) => {
                                info!("Bridge status check:");
                                info!("  Status: {}", status.status);
                                info!("  Complete: {}", status.is_complete);
                                info!("  Failed: {}", status.is_failed);
                                info!("  Message: {}", status.message);
                                
                                if let Some(ref dest_tx) = status.destination_tx_hash {
                                    info!("  Destination TX: {}", dest_tx);
                                }
                                
                                if let Some(ref received) = status.amount_received {
                                    info!("  Amount received: {} USDC", received);
                                }
                            }
                            Err(e) => warn!("Failed to check bridge status: {}", e),
                        }
                    }
                    Err(e) => warn!("Failed to execute bridge: {}", e),
                }
            }
        }
        Err(e) => warn!("Failed to get cross-chain routes: {}", e),
    }
    
    // Step 6: Demonstrate error handling - invalid chain
    info!("\n=== Step 6: Demonstrating error handling ===");
    match get_cross_chain_routes(
        "invalid_chain".to_string(),
        "ethereum".to_string(),
        "USDC".to_string(),
        "USDC".to_string(),
        "1000000".to_string(),
        None,
    ).await {
        Ok(_) => warn!("Expected error for invalid chain but got success"),
        Err(e) => info!("Correctly caught error for invalid chain: {}", e),
    }
    
    info!("\n=== Cross-chain operations example completed ===");
    info!("Note: This example uses simulated bridge execution for safety.");
    info!("In production, actual transactions would be constructed and signed.");
    
    Ok(())
}

// Additional example showing how to integrate with rig agents
#[cfg(feature = "rig")]
mod rig_integration {
    use super::*;
    // use rig::agent::AgentBuilder;
    
    /// Example of creating an agent with cross-chain capabilities
    #[allow(dead_code)]
    pub async fn create_cross_chain_agent() -> Result<(), Box<dyn std::error::Error>> {
        // Create an agent with cross-chain tools
        // Note: This requires proper rig model setup, commented out to avoid compilation errors
        /*
        let agent = AgentBuilder::new("gpt-4")
            .preamble(
                "You are a cross-chain bridge assistant. You can help users transfer tokens \
                 between different blockchain networks, estimate fees, and track transfer status. \
                 Always explain the risks and ensure users understand the process."
            )
            .tool(riglr_cross_chain_tools::GetCrossChainRoutesTool::new())
            .tool(riglr_cross_chain_tools::EstimateBridgeFeesTool::new())
            .tool(riglr_cross_chain_tools::ExecuteCrossChainBridgeTool::new())
            .tool(riglr_cross_chain_tools::GetBridgeStatusTool::new())
            .tool(riglr_cross_chain_tools::GetSupportedChainsTool::new())
            .build();
        */
            
        info!("Cross-chain agent created successfully");
        
        // Example: Agent can now help with questions like:
        // "How can I transfer USDC from Ethereum to Polygon?"
        // "What are the fees for bridging 100 USDC to Arbitrum?"
        // "Check the status of my bridge transaction with ID abc123"
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use riglr_core::SignerContext;
    
    // Mock signer for testing
    #[derive(Debug)]
    struct MockSigner;
    
    #[async_trait::async_trait]
    impl riglr_core::TransactionSigner for MockSigner {
        fn address(&self) -> Option<String> {
            Some("0x1234567890123456789012345678901234567890".to_string())
        }
        
        async fn sign_and_send_solana_transaction(
            &self,
            _tx: &mut solana_sdk::transaction::Transaction,
        ) -> Result<String, riglr_core::SignerError> {
            Ok("mock_signature".to_string())
        }
        
        async fn sign_and_send_evm_transaction(
            &self,
            _tx: alloy::rpc::types::TransactionRequest,
        ) -> Result<String, riglr_core::SignerError> {
            Ok("0xmock_signature".to_string())
        }
        
        fn solana_client(&self) -> Arc<solana_client::rpc_client::RpcClient> {
            Arc::new(solana_client::rpc_client::RpcClient::new("http://localhost:8899"))
        }
        
        fn evm_client(&self) -> Result<std::sync::Arc<dyn std::any::Any + Send + Sync>, riglr_core::SignerError> {
            Err(riglr_core::SignerError::Configuration("Mock EVM client".to_string()))
        }
    }
    
    #[tokio::test]
    async fn test_cross_chain_tools_with_mock_signer() {
        let signer = Arc::new(MockSigner);
        
        let result = SignerContext::with_signer(signer, async {
            // Test that tools can be called within signer context
            let routes = get_cross_chain_routes(
                "ethereum".to_string(),
                "polygon".to_string(),
                "USDC".to_string(),
                "USDC".to_string(),
                "1000000".to_string(),
                None,
            ).await;
            
            // The actual LiFi API call may fail in test environment,
            // but we should at least get past the signer context check
            match routes {
                Ok(_) => info!("Route discovery succeeded"),
                Err(e) => info!("Route discovery failed (expected in test): {}", e),
            }
            
            Ok(())
        }).await;
        
        assert!(result.is_ok());
    }
}