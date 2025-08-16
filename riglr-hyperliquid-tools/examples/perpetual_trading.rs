//! Hyperliquid Perpetual Trading Example
//!
//! This example demonstrates how to use riglr-hyperliquid-tools to build
//! a derivatives trading agent using the rig framework.

use riglr_core::signer::LocalSolanaSigner;
use riglr_core::{config::SolanaNetworkConfig, signer::SignerError, SignerContext};
use riglr_hyperliquid_tools::{
    cancel_hyperliquid_order, close_hyperliquid_position, get_hyperliquid_account_info,
    get_hyperliquid_portfolio_risk, get_hyperliquid_positions, place_hyperliquid_order,
    set_leverage,
};
use std::sync::Arc;
use tracing::{info, Level};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt().with_max_level(Level::INFO).init();

    info!("Starting Hyperliquid perpetual trading example");

    // Create a dummy signer for the example
    // In a real application, you would use a proper signer with actual keys
    let keypair = solana_sdk::signature::Keypair::new();
    let network_config = SolanaNetworkConfig {
        name: "devnet".to_string(),
        rpc_url: "https://api.devnet.solana.com".to_string(),
        explorer_url: Some("https://explorer.solana.com".to_string()),
    };
    let signer = Arc::new(LocalSolanaSigner::from_keypair(keypair, network_config));

    // Execute trading operations within signer context
    SignerContext::with_signer(signer, async {
        // Example 1: Get account information
        info!("=== Getting Account Information ===");
        match get_hyperliquid_account_info().await {
            Ok(account) => {
                info!("Account: {}", account.user_address);
                info!("Withdrawable: {}", account.withdrawable_balance);
                info!("Margin Used: {}", account.cross_margin_used);
                info!("Positions: {}", account.positions_count);
            }
            Err(e) => info!("Failed to get account info: {}", e),
        }

        // Example 2: Check current positions
        info!("\n=== Getting Current Positions ===");
        match get_hyperliquid_positions().await {
            Ok(positions) => {
                if positions.is_empty() {
                    info!("No open positions");
                } else {
                    for (i, position) in positions.iter().enumerate() {
                        info!(
                            "Position {}: {} {} @ {} (PnL: {})",
                            i + 1,
                            position.size,
                            position.symbol,
                            position.entry_price,
                            position.unrealized_pnl
                        );
                    }
                }
            }
            Err(e) => info!("Failed to get positions: {}", e),
        }

        // Example 3: Set leverage for ETH-PERP
        info!("\n=== Setting Leverage ===");
        match set_leverage("ETH-PERP".to_string(), 10).await {
            Ok(result) => {
                info!("Leverage result: {} - {}", result.status, result.message);
            }
            Err(e) => info!("Failed to set leverage: {}", e),
        }

        // Example 4: Place a limit buy order
        info!("\n=== Placing Limit Buy Order ===");
        match place_hyperliquid_order(
            "ETH-PERP".to_string(),
            "buy".to_string(),
            "0.1".to_string(),
            "limit".to_string(),
            Some("2000.0".to_string()),
            Some(false),
            Some("gtc".to_string()),
        )
        .await
        {
            Ok(order) => {
                info!("Order placed successfully!");
                info!("  Symbol: {}", order.symbol);
                info!("  Side: {}", order.side);
                info!("  Size: {}", order.size);
                info!("  Price: {}", order.price.unwrap_or_default());
                info!("  Order ID: {}", order.order_id.unwrap_or_default());
                info!("  Status: {}", order.status);
            }
            Err(e) => info!("Failed to place order: {}", e),
        }

        // Example 5: Place a market sell order
        info!("\n=== Placing Market Sell Order ===");
        match place_hyperliquid_order(
            "BTC-PERP".to_string(),
            "sell".to_string(),
            "0.01".to_string(),
            "market".to_string(),
            None,
            Some(false),
            None,
        )
        .await
        {
            Ok(order) => {
                info!(
                    "Market order placed: {} {} {}",
                    order.side, order.size, order.symbol
                );
                info!("  Status: {}", order.status);

                // Example 6: Cancel the order (if it's not immediately filled)
                if let Some(order_id) = order.order_id {
                    info!("\n=== Canceling Order ===");
                    match cancel_hyperliquid_order(order.symbol.clone(), order_id).await {
                        Ok(cancel_result) => {
                            info!(
                                "Cancel result: {} - {}",
                                cancel_result.status, cancel_result.message
                            );
                        }
                        Err(e) => info!("Failed to cancel order: {}", e),
                    }
                }
            }
            Err(e) => info!("Failed to place market order: {}", e),
        }

        // Example 7: Get portfolio risk metrics
        info!("\n=== Portfolio Risk Analysis ===");
        match get_hyperliquid_portfolio_risk().await {
            Ok(risk) => {
                info!("Portfolio Risk Metrics:");
                info!("  Total Positions: {}", risk.total_positions);
                info!("  Position Value: {}", risk.total_position_value);
                info!("  Unrealized PnL: {}", risk.total_unrealized_pnl);
                info!(
                    "  Margin Utilization: {:.2}%",
                    risk.margin_utilization_percent
                );
                info!("  Max Leverage: {}x", risk.max_leverage);
                info!("  Positions at Risk: {}", risk.positions_at_risk);
                info!("  Risk Level: {}", risk.risk_level);
            }
            Err(e) => info!("Failed to get risk metrics: {}", e),
        }

        // Example 8: Close a position (if any exists)
        info!("\n=== Closing Position (if exists) ===");
        match close_hyperliquid_position("ETH-PERP".to_string(), None).await {
            Ok(close_result) => {
                info!("Position close order placed:");
                info!("  Symbol: {}", close_result.symbol);
                info!("  Size: {}", close_result.closed_size);
                info!("  Side: {}", close_result.order_side);
                info!("  Status: {}", close_result.status);
            }
            Err(e) => info!("No position to close or failed: {}", e),
        }

        Ok::<(), SignerError>(())
    })
    .await?;

    info!("Hyperliquid trading example completed successfully!");
    Ok(())
}

// Example of how to use with rig::agent::AgentBuilder
// Note: This example requires the rig crate to be added as a dependency
// Commented out due to compilation issues - this is just for demonstration
/*
#[allow(dead_code)]
mod rig_example {
    use super::*;
    use rig::agent::AgentBuilder;

    pub async fn create_trading_agent() -> Result<(), Box<dyn std::error::Error>> {
        let agent = AgentBuilder::new("gpt-4")
            .preamble("You are a derivatives trading assistant specialized in Hyperliquid perpetual futures. You can help users manage positions, place orders, and analyze risk.")
            .tool(place_hyperliquid_order)
            .tool(cancel_hyperliquid_order)
            .tool(get_hyperliquid_positions)
            .tool(get_hyperliquid_account_info)
            .tool(close_hyperliquid_position)
            .tool(set_leverage)
            .tool(get_hyperliquid_portfolio_risk)
            .build();

        // The agent can now be used with natural language prompts
        // Example: "Place a 0.1 ETH long at $2000 with 10x leverage"
        // The agent would use the appropriate tools to execute the trade

        Ok(())
    }
}
*/
