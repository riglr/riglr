//! Hyperliquid Perpetual Trading Example
//!
//! This example demonstrates how to use riglr-hyperliquid-tools to build
//! a derivatives trading agent using the rig framework.

use core::error::Error;
use riglr_config::{Config, SolanaNetworkConfig};
use riglr_core::{
    provider::ApplicationContext, signer::error::Error as SignerError, SignerContext,
};
use riglr_hyperliquid_tools::positions::{
    HyperliquidCloseResult, HyperliquidPosition, HyperliquidRiskMetrics,
};
use riglr_hyperliquid_tools::trading::{HyperliquidAccountResult, HyperliquidOrderResult};
use riglr_hyperliquid_tools::{
    cancel_hyperliquid_order, close_hyperliquid_position, get_hyperliquid_account_info,
    get_hyperliquid_portfolio_risk, get_positions, place_hyperliquid_order, set_leverage,
    OrderParams,
};
use riglr_solana_tools::LocalSigner;
use solana_sdk::signature::Keypair;
use std::sync::Arc;
use tracing::{info, Level};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Initialize logging
    tracing_subscriber::fmt().with_max_level(Level::INFO).init();

    info!("Starting Hyperliquid perpetual trading example");

    let signer = create_example_signer();
    run_trading_examples(signer).await?;

    info!("Hyperliquid trading example completed successfully!");
    Ok(())
}

/// Creates a dummy signer for the example
fn create_example_signer() -> Arc<LocalSigner> {
    // In a real application, you would use a proper signer with actual keys
    let keypair = Keypair::new();
    let network_config = SolanaNetworkConfig::devnet();
    Arc::new(LocalSigner::from_keypair(keypair, network_config))
}

/// Runs all trading examples within a signer context
async fn run_trading_examples(signer: Arc<LocalSigner>) -> Result<(), Box<dyn Error>> {
    SignerContext::with_signer(signer, async {
        let config = Config::from_env();
        let context = ApplicationContext::from_config(&config);

        demo_account_info(&context).await;
        demo_positions(&context).await;
        demo_leverage(&context).await;
        demo_limit_order(&context).await;
        demo_market_order(&context).await;
        demo_risk_analysis(&context).await;
        demo_close_position(&context).await;

        Ok::<(), Box<dyn SignerError>>(())
    })
    .await
    .map_err(|e| -> Box<dyn Error> { e })?;
    Ok(())
}

/// Demonstrates getting account information
async fn demo_account_info(context: &ApplicationContext) {
    info!("=== Getting Account Information ===");
    match get_hyperliquid_account_info(context).await {
        Ok(account) => log_account_details(&account),
        Err(e) => info!("Failed to get account info: {}", e),
    }
}

/// Helper function to log account details
#[expect(clippy::cognitive_complexity)]
fn log_account_details(account: &HyperliquidAccountResult) {
    info!("Account: {}", account.user_address);
    info!("Withdrawable: {}", account.withdrawable_balance);
    info!("Margin Used: {}", account.cross_margin_used);
    info!("Positions: {}", account.positions_count);
}

/// Demonstrates checking current positions
async fn demo_positions(context: &ApplicationContext) {
    info!("\n=== Getting Current Positions ===");
    match get_positions(context).await {
        Ok(positions) => log_positions(&positions),
        Err(e) => info!("Failed to get positions: {}", e),
    }
}

/// Helper function to log position details
fn log_positions(positions: &[HyperliquidPosition]) {
    if positions.is_empty() {
        info!("No open positions");
        return;
    }

    for (i, position) in positions.iter().enumerate() {
        let position_num = i.saturating_add(1);
        info!(
            "Position {}: {} {} @ {} (PnL: {})",
            position_num,
            position.size,
            position.symbol,
            position.entry_price,
            position.unrealized_pnl
        );
    }
}

/// Demonstrates setting leverage
async fn demo_leverage(context: &ApplicationContext) {
    info!("\n=== Setting Leverage ===");
    match set_leverage(context, "ETH-PERP".to_string(), 10).await {
        Ok(result) => {
            info!("Leverage result: {} - {}", result.status, result.message);
        }
        Err(e) => info!("Failed to set leverage: {}", e),
    }
}

/// Demonstrates placing a limit order
async fn demo_limit_order(context: &ApplicationContext) {
    info!("\n=== Placing Limit Buy Order ===");
    let order_params = create_limit_order_params();
    match place_hyperliquid_order(context, order_params).await {
        Ok(order) => log_order_success(&order),
        Err(e) => info!("Failed to place order: {}", e),
    }
}

/// Helper function to create limit order parameters
fn create_limit_order_params() -> OrderParams {
    OrderParams {
        symbol: "ETH-PERP".to_string(),
        side: "buy".to_string(),
        quantity: "0.1".to_string(),
        order_type: "limit".to_string(),
        price: Some("2000.0".to_string()),
        reduce_only: Some(false),
        time_in_force: Some("gtc".to_string()),
    }
}

/// Helper function to log successful order placement
#[expect(clippy::cognitive_complexity)]
fn log_order_success(order: &HyperliquidOrderResult) {
    info!("Order placed successfully!");
    info!("  Symbol: {}", order.symbol);
    info!("  Side: {}", order.side);
    info!("  Size: {}", order.size);
    info!("  Price: {}", order.price.clone().unwrap_or_default());
    info!("  Order ID: {}", order.order_id.clone().unwrap_or_default());
    info!("  Status: {}", order.status);
}

/// Demonstrates placing a market order and canceling it
async fn demo_market_order(context: &ApplicationContext) {
    info!("\n=== Placing Market Sell Order ===");
    let market_order_params = create_market_order_params();
    match place_hyperliquid_order(context, market_order_params).await {
        Ok(order) => {
            log_market_order_success(&order);
            attempt_order_cancellation(context, &order).await;
        }
        Err(e) => info!("Failed to place market order: {}", e),
    }
}

/// Helper function to create market order parameters
fn create_market_order_params() -> OrderParams {
    OrderParams {
        symbol: "BTC-PERP".to_string(),
        side: "sell".to_string(),
        quantity: "0.01".to_string(),
        order_type: "market".to_string(),
        price: None,
        reduce_only: Some(false),
        time_in_force: None,
    }
}

/// Helper function to log market order success
fn log_market_order_success(order: &HyperliquidOrderResult) {
    info!(
        "Market order placed: {} {} {}",
        order.side, order.size, order.symbol
    );
    info!("  Status: {}", order.status);
}

/// Helper function to attempt order cancellation
async fn attempt_order_cancellation(context: &ApplicationContext, order: &HyperliquidOrderResult) {
    if let Some(order_id) = order.order_id.as_ref() {
        info!("\n=== Canceling Order ===");
        match cancel_hyperliquid_order(context, order.symbol.clone(), order_id.clone()).await {
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

/// Demonstrates getting portfolio risk metrics
async fn demo_risk_analysis(context: &ApplicationContext) {
    info!("\n=== Portfolio Risk Analysis ===");
    match get_hyperliquid_portfolio_risk(context).await {
        Ok(risk) => log_risk_metrics(&risk),
        Err(e) => info!("Failed to get risk metrics: {}", e),
    }
}

/// Helper function to log risk metrics
#[expect(clippy::cognitive_complexity)]
fn log_risk_metrics(risk: &HyperliquidRiskMetrics) {
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

/// Demonstrates closing a position
async fn demo_close_position(context: &ApplicationContext) {
    info!("\n=== Closing Position (if exists) ===");
    match close_hyperliquid_position(context, "ETH-PERP".to_string(), None).await {
        Ok(close_result) => log_position_close_success(&close_result),
        Err(e) => info!("No position to close or failed: {}", e),
    }
}

/// Helper function to log position close success
#[expect(clippy::cognitive_complexity)]
fn log_position_close_success(close_result: &HyperliquidCloseResult) {
    info!("Position close order placed:");
    info!("  Symbol: {}", close_result.symbol);
    info!("  Size: {}", close_result.closed_size);
    info!("  Side: {}", close_result.order_side);
    info!("  Status: {}", close_result.status);
}

// Example of how to use with rig::agent::AgentBuilder
// Note: This example requires the rig crate to be added as a dependency
// Commented out due to compilation issues - this is just for demonstration
/*
mod rig_example {
    use super::*;
    use rig::agent::AgentBuilder;

    pub async fn create_trading_agent() -> Result<(), Box<dyn std::error::Error>> {
        let agent = AgentBuilder::new("gpt-4")
            .preamble("You are a derivatives trading assistant specialized in Hyperliquid perpetual futures. You can help users manage positions, place orders, and analyze risk.")
            .tool(place_hyperliquid_order)
            .tool(cancel_hyperliquid_order)
            .tool(get_positions)
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
