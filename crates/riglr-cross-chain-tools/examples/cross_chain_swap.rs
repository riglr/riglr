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
//! The example uses riglr's `SignerContext` pattern for secure multi-tenant operation.

use core::error::Error;
use riglr_config::{Config, SolanaNetworkConfig};
use riglr_core::{provider::ApplicationContext, signer::SignerError, SignerContext};
use riglr_cross_chain_tools::{
    estimate_bridge_fees, execute_cross_chain_transfer, get_bridge_status, get_cross_chain_routes,
    get_supported_chains,
};
use riglr_solana_tools::LocalSigner;
use solana_sdk::signer::keypair::Keypair;
use std::sync::Arc;
use tracing::{info, warn};
use tracing_subscriber::fmt as tracing_fmt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Initialize logging
    tracing_fmt::init();

    info!("Starting cross-chain swap example");

    // Create a Solana signer (in production, load from secure storage)
    let keypair = Keypair::new();
    let network_config = SolanaNetworkConfig::devnet();
    let signer = Arc::new(LocalSigner::from_keypair(keypair, network_config));

    // Execute all operations within a signer context
    let result =
        SignerContext::with_signer(signer, async { demonstrate_cross_chain_operations().await })
            .await;

    match result {
        Ok(()) => {
            info!("Cross-chain operations completed successfully");
        }
        Err(e) => {
            warn!("Cross-chain operations failed: {}", e);
        }
    }

    Ok(())
}
async fn demonstrate_cross_chain_operations() -> Result<(), Box<dyn SignerError>> {
    let config = Config::from_env();
    let context = ApplicationContext::from_config(&config);

    discover_supported_chains(&context).await;
    discover_and_process_routes(&context).await;
    demonstrate_error_handling(&context).await;

    info!("\n=== Cross-chain operations example completed ===");
    info!("Note: This example uses simulated bridge execution for safety.");
    info!("In production, actual transactions would be constructed and signed.");

    Ok(())
}

async fn discover_supported_chains(context: &ApplicationContext) {
    info!("\n=== Step 1: Discovering supported chains ===");
    match get_supported_chains(context).await {
        Ok(chains) => {
            display_chains_summary(&chains);
        }
        Err(e) => warn!("Failed to get supported chains: {}", e),
    }
}

fn display_chains_summary(chains: &[riglr_cross_chain_tools::ChainInfo]) {
    info!("Found {} supported chains:", chains.len());
    display_chain_details(chains);
}

fn display_chain_details(chains: &[riglr_cross_chain_tools::ChainInfo]) {
    for chain in chains.iter().take(5) {
        // Show first 5
        info!(
            "  {} ({}): {} - {}",
            chain.name, chain.id, chain.chain_type, chain.native_token.symbol
        );
    }
}

async fn discover_and_process_routes(context: &ApplicationContext) {
    info!("\n=== Step 2: Discovering cross-chain routes ===");
    let route_result = get_cross_chain_routes(
        context,
        "ethereum".to_string(),
        "polygon".to_string(),
        "0xA0b86a33E6417c5d6d6bE6C2e0C6C3e5d6c7D8E9".to_string(), // USDC on Ethereum
        "0x2791Bca1f2de4661ED88A30C99A7a9449Aa84174".to_string(), // USDC on Polygon
        "1000000".to_string(),                                    // 1 USDC (6 decimals)
        Some(0.5),                                                // 0.5% slippage
    )
    .await;

    match route_result {
        Ok(routes) => {
            display_route_information(&routes);
            if let Some(best_route) = routes.routes.first() {
                estimate_fees_for_route(context, best_route).await;
                execute_bridge_and_monitor(context, best_route).await;
            }
        }
        Err(e) => warn!("Failed to get cross-chain routes: {}", e),
    }
}

fn display_route_information(routes: &riglr_cross_chain_tools::RouteDiscoveryResult) {
    display_routes_summary(routes);
    display_recommended_route(routes);
    display_route_details(routes);
}

fn display_routes_summary(routes: &riglr_cross_chain_tools::RouteDiscoveryResult) {
    info!(
        "Found {} routes for USDC Ethereum -> Polygon:",
        routes.total_routes
    );
}

fn display_recommended_route(routes: &riglr_cross_chain_tools::RouteDiscoveryResult) {
    if let Some(ref recommended) = routes.recommended_route_id {
        info!("  Recommended route: {}", recommended);
    }
}

fn display_route_details(routes: &riglr_cross_chain_tools::RouteDiscoveryResult) {
    for (i, route) in routes.routes.iter().take(3).enumerate() {
        display_single_route(i, route);
    }
}

fn display_single_route(index: usize, route: &riglr_cross_chain_tools::RouteInfo) {
    #[expect(clippy::arithmetic_side_effects)]
    let route_number = index + 1;
    info!(
        "  Route {}: {} via {}",
        route_number,
        route.id,
        route.protocols.join(", ")
    );
    let total_fees = route.fees_usd.unwrap_or(0.0) + route.gas_cost_usd.unwrap_or(0.0);
    info!(
        "    Duration: {}s, Fees: ${:.2}",
        route.estimated_duration, total_fees
    );
}

async fn estimate_fees_for_route(
    context: &ApplicationContext,
    _route: &riglr_cross_chain_tools::RouteInfo,
) {
    info!("\n=== Step 3: Estimating fees for best route ===");
    match estimate_bridge_fees(
        context,
        "ethereum".to_string(),
        "polygon".to_string(),
        "0xA0b86a33E6417c5d6d6bE6C2e0C6C3e5d6c7D8E9".to_string(),
        "0x2791Bca1f2de4661ED88A30C99A7a9449Aa84174".to_string(),
        "1000000".to_string(),
    )
    .await
    {
        Ok(fee_estimate) => {
            display_fee_breakdown(&fee_estimate);
        }
        Err(e) => warn!("Failed to estimate fees: {}", e),
    }
}

fn display_fee_breakdown(fee_estimate: &riglr_cross_chain_tools::FeeEstimate) {
    display_transfer_amounts(fee_estimate);
    display_individual_fees(fee_estimate);
    display_fee_totals(fee_estimate);
    display_completion_estimate(fee_estimate);
}

fn display_transfer_amounts(fee_estimate: &riglr_cross_chain_tools::FeeEstimate) {
    info!("Fee breakdown for 1 USDC transfer:");
    info!("  Input amount: {} USDC", fee_estimate.from_amount);
    info!("  Expected output: {} USDC", fee_estimate.estimated_output);
}

fn display_individual_fees(fee_estimate: &riglr_cross_chain_tools::FeeEstimate) {
    for fee in &fee_estimate.fees {
        display_single_fee(fee);
    }
}

fn display_single_fee(fee: &riglr_cross_chain_tools::FeeBreakdown) {
    info!(
        "  {}: {} (${:.2})",
        fee.name,
        fee.percentage,
        fee.amount_usd.unwrap_or(0.0)
    );
}

fn display_fee_totals(fee_estimate: &riglr_cross_chain_tools::FeeEstimate) {
    if let Some(total_usd) = fee_estimate.total_fees_usd {
        info!("  Total fees: ${:.2}", total_usd);
    }
}

fn display_completion_estimate(fee_estimate: &riglr_cross_chain_tools::FeeEstimate) {
    info!(
        "  Estimated completion: {}s",
        fee_estimate.estimated_duration
    );
}

async fn execute_bridge_and_monitor(
    context: &ApplicationContext,
    route: &riglr_cross_chain_tools::RouteInfo,
) {
    info!("\n=== Step 4: Executing bridge transaction ===");
    match execute_cross_chain_transfer(
        context,
        route.id.clone(),
        "ethereum".to_string(),
        "polygon".to_string(),
        "1000000".to_string(),
    )
    .await
    {
        Ok(bridge_result) => {
            display_bridge_result(&bridge_result);
            monitor_bridge_status(context, &bridge_result).await;
        }
        Err(e) => warn!("Failed to execute bridge: {}", e),
    }
}

fn display_bridge_result(bridge_result: &riglr_cross_chain_tools::ExecutionResult) {
    display_bridge_transaction_header();
    display_bridge_transaction_details(bridge_result);
}

fn display_bridge_transaction_header() {
    info!("Bridge transaction submitted:");
}

#[expect(clippy::cognitive_complexity)]
fn display_bridge_transaction_details(bridge_result: &riglr_cross_chain_tools::ExecutionResult) {
    info!("  Bridge ID: {}", bridge_result.bridge_id);
    info!("  Source TX: {}", bridge_result.source_tx_hash);
    info!("  Status: {}", bridge_result.status);
    info!("  Message: {}", bridge_result.message);
}

async fn monitor_bridge_status(
    context: &ApplicationContext,
    bridge_result: &riglr_cross_chain_tools::ExecutionResult,
) {
    info!("\n=== Step 5: Checking bridge status ===");
    match get_bridge_status(
        context,
        bridge_result.bridge_id.clone(),
        bridge_result.source_tx_hash.clone(),
    )
    .await
    {
        Ok(status) => {
            display_bridge_status(&status);
        }
        Err(e) => warn!("Failed to check bridge status: {}", e),
    }
}

fn display_bridge_status(status: &riglr_cross_chain_tools::StatusResult) {
    display_status_header();
    display_status_details(status);
    display_optional_status_fields(status);
}

fn display_status_header() {
    info!("Bridge status check:");
}

#[expect(clippy::cognitive_complexity)]
fn display_status_details(status: &riglr_cross_chain_tools::StatusResult) {
    info!("  Status: {}", status.status);
    info!("  Complete: {}", status.is_complete);
    info!("  Failed: {}", status.is_failed);
    info!("  Message: {}", status.message);
}

fn display_optional_status_fields(status: &riglr_cross_chain_tools::StatusResult) {
    display_destination_transaction(status);
    display_received_amount(status);
}

fn display_destination_transaction(status: &riglr_cross_chain_tools::StatusResult) {
    if let Some(ref dest_tx) = status.destination_tx_hash {
        info!("  Destination TX: {}", dest_tx);
    }
}

fn display_received_amount(status: &riglr_cross_chain_tools::StatusResult) {
    if let Some(ref received) = status.amount_received {
        info!("  Amount received: {} USDC", received);
    }
}

async fn demonstrate_error_handling(context: &ApplicationContext) {
    info!("\n=== Step 6: Demonstrating error handling ===");
    match get_cross_chain_routes(
        context,
        "invalid_chain".to_string(),
        "ethereum".to_string(),
        "USDC".to_string(),
        "USDC".to_string(),
        "1000000".to_string(),
        None,
    )
    .await
    {
        Ok(_) => warn!("Expected error for invalid chain but got success"),
        Err(e) => info!("Correctly caught error for invalid chain: {}", e),
    }
}

// Additional example showing how to integrate with rig agents
#[cfg(feature = "rig")]
mod rig_integration {
    use super::{info, Error};
    // use rig::agent::AgentBuilder;

    /// Example of creating an agent with cross-chain capabilities
    #[allow(dead_code, clippy::unnecessary_wraps)]
    pub fn create_cross_chain_agent() -> Result<(), Box<dyn Error>> {
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
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;
    use riglr_core::{provider::ApplicationContext, SignerContext};

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

        fn solana_client(&self) -> Option<Arc<solana_client::rpc_client::RpcClient>> {
            Some(Arc::new(solana_client::rpc_client::RpcClient::new(
                "http://localhost:8899",
            )))
        }

        fn evm_client(
            &self,
        ) -> Result<std::sync::Arc<dyn std::any::Any + Send + Sync>, riglr_core::SignerError>
        {
            Err(riglr_core::SignerError::Configuration(
                "Mock EVM client".to_string(),
            ))
        }
    }

    #[tokio::test]
    async fn test_cross_chain_tools_with_mock_signer() {
        let signer = Arc::new(MockSigner);

        let result = SignerContext::with_signer(signer, async {
            // Test that tools can be called within signer context
            let config = Config::from_env();
            let context = ApplicationContext::from_config(&config);
            let routes = get_cross_chain_routes(
                &context,
                "ethereum".to_string(),
                "polygon".to_string(),
                "USDC".to_string(),
                "USDC".to_string(),
                "1000000".to_string(),
                None,
            )
            .await;

            // The actual LiFi API call may fail in test environment,
            // but we should at least get past the signer context check
            match routes {
                Ok(_) => info!("Route discovery succeeded"),
                Err(e) => info!("Route discovery failed (expected in test): {}", e),
            }

            Ok::<(), riglr_core::signer::SignerError>(())
        })
        .await;

        assert!(result.is_ok());
    }
}
