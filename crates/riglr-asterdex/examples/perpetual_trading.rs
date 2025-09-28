//! Asterdex Perpetual Trading Example
//!
//! This example demonstrates how to use riglr-asterdex to build
//! a derivatives trading agent using the rig framework with Asterdex Futures v3 API.

use core::error::Error;
use riglr_asterdex::positions::{AsterdexAccountInfo, AsterdexBalance, AsterdexPosition};
use riglr_asterdex::trading::{AsterdexCancelResult, AsterdexOrderResult};
use riglr_asterdex::{
    cancel_asterdex_order, get_asterdex_account_info, get_asterdex_balance,
    get_asterdex_open_orders, get_asterdex_order, get_open_holdings, place_asterdex_order,
    set_asterdex_leverage, CancelOrderParams, OrderParams,
};
use riglr_config::{Config, SolanaNetworkConfig};
use riglr_core::{
    provider::ApplicationContext, signer::error::Error as SignerError, SignerContext,
};
use riglr_solana_tools::LocalSigner;
use solana_sdk::signature::Keypair;
use std::sync::Arc;
use tracing::{info, Level};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Initialize logging
    tracing_subscriber::fmt().with_max_level(Level::INFO).init();

    info!("Starting Asterdex perpetual trading example");

    let signer = create_example_signer();
    run_trading_examples(signer).await?;

    info!("Asterdex trading example completed successfully!");
    Ok(())
}

/// Creates a dummy signer for the example
fn create_example_signer() -> Arc<LocalSigner> {
    // In a real application, you would use a proper signer with actual keys
    // For Asterdex v3, you need to use your actual API wallet private key
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
        demo_balances(&context).await;
        demo_positions(&context).await;
        demo_open_orders(&context).await;
        demo_leverage(&context).await;
        demo_limit_order(&context).await;
        demo_market_order(&context).await;
        demo_cancel_order(&context).await;
        demo_query_order(&context).await;

        Ok::<(), Box<dyn SignerError>>(())
    })
    .await
    .map_err(|e| -> Box<dyn Error> { e })?;
    Ok(())
}

/// Demonstrates getting account information
async fn demo_account_info(context: &ApplicationContext) {
    info!("=== Getting Account Information ===");
    match get_asterdex_account_info(context).await {
        Ok(account) => log_account_details(&account),
        Err(e) => info!("Failed to get account info: {}", e),
    }
}

/// Helper function to log account details
#[allow(clippy::cognitive_complexity)]
fn log_account_details(account: &AsterdexAccountInfo) {
    info!("Can Trade: {}", account.can_trade);
    info!("Total Wallet Balance: {}", account.total_wallet_balance);
    info!("Available Balance: {}", account.available_balance);
    info!("Unrealized P&L: {}", account.total_unrealized_profit);
    info!("Margin Balance: {}", account.total_margin_balance);
    info!("Open Positions: {}", account.open_positions_count);
}

/// Demonstrates getting account balances
async fn demo_balances(context: &ApplicationContext) {
    info!("\n=== Getting Account Balances ===");
    match get_asterdex_balance(context).await {
        Ok(balances) => log_balances(&balances),
        Err(e) => info!("Failed to get balances: {}", e),
    }
}

/// Helper function to log balance details
fn log_balances(balances: &[AsterdexBalance]) {
    if balances.is_empty() {
        info!("No balances found");
        return;
    }

    for balance in balances {
        info!(
            "Asset {}: Balance={}, Available={}, Unrealized PnL={}",
            balance.asset, balance.balance, balance.available_balance, balance.cross_un_pnl
        );
    }
}

/// Demonstrates checking current positions
async fn demo_positions(context: &ApplicationContext) {
    info!("\n=== Getting Current Positions ===");
    let result: Result<Vec<AsterdexPosition>, _> = get_open_holdings(context, None).await;
    match result {
        Ok(positions) => log_positions(&positions),
        Err(e) => info!("Failed to get positions: {}", e),
    }
}

/// Helper function to log position details
fn log_positions(positions: &[AsterdexPosition]) {
    if positions.is_empty() {
        info!("No open positions");
        return;
    }

    for (i, position) in positions.iter().enumerate() {
        let position_num = i.saturating_add(1);
        info!(
            "Position {}: {} {} @ {} (PnL: {}, Mark: {}, Liq: {})",
            position_num,
            position.position_amt,
            position.symbol,
            position.entry_price,
            position.unrealized_profit,
            position.mark_price,
            position.liquidation_price
        );
        info!(
            "  Leverage: {}x, Side: {}, Notional: {}, Margin Type: {}",
            position.leverage, position.position_side, position.notional, position.margin_type
        );
    }
}

/// Demonstrates getting open orders
#[allow(clippy::cognitive_complexity)]
async fn demo_open_orders(context: &ApplicationContext) {
    info!("\n=== Getting Open Orders ===");
    match get_asterdex_open_orders(context, None).await {
        Ok(orders) => {
            if orders.is_empty() {
                info!("No open orders");
            } else {
                for order in orders {
                    info!(
                        "Order {}: {} {} {} @ {} (Status: {})",
                        order.order_id,
                        order.side,
                        order.orig_qty,
                        order.symbol,
                        order.price,
                        order.status
                    );
                }
            }
        }
        Err(e) => info!("Failed to get open orders: {}", e),
    }
}

/// Demonstrates setting leverage
async fn demo_leverage(context: &ApplicationContext) {
    info!("\n=== Setting Leverage ===");
    match set_asterdex_leverage(context, "BTCUSDT".to_string(), 10).await {
        Ok(result) => {
            info!("Leverage set successfully: {:?}", result);
        }
        Err(e) => info!("Failed to set leverage: {}", e),
    }
}

/// Demonstrates placing a limit order
async fn demo_limit_order(context: &ApplicationContext) {
    info!("\n=== Placing Limit Buy Order ===");
    let order_params = create_limit_order_params();
    match place_asterdex_order(context, order_params).await {
        Ok(order) => log_order_success(&order),
        Err(e) => info!("Failed to place order: {}", e),
    }
}

/// Helper function to create limit order parameters
fn create_limit_order_params() -> OrderParams {
    OrderParams {
        symbol: "BTCUSDT".to_string(),
        side: "BUY".to_string(),
        quantity: "0.001".to_string(),
        order_type: "LIMIT".to_string(),
        price: Some("50000.0".to_string()),
        position_side: Some("BOTH".to_string()),
        reduce_only: Some(false),
        time_in_force: Some("GTC".to_string()),
    }
}

/// Demonstrates placing a market order
async fn demo_market_order(context: &ApplicationContext) {
    info!("\n=== Placing Market Sell Order ===");
    let order_params = create_market_order_params();
    match place_asterdex_order(context, order_params).await {
        Ok(order) => log_order_success(&order),
        Err(e) => info!("Failed to place order: {}", e),
    }
}

/// Helper function to create market order parameters
fn create_market_order_params() -> OrderParams {
    OrderParams {
        symbol: "ETHUSDT".to_string(),
        side: "SELL".to_string(),
        quantity: "0.01".to_string(),
        order_type: "MARKET".to_string(),
        price: None,
        position_side: Some("BOTH".to_string()),
        reduce_only: Some(false),
        time_in_force: None,
    }
}

/// Helper function to log successful order placement
#[allow(clippy::cognitive_complexity)]
fn log_order_success(order: &AsterdexOrderResult) {
    info!("Order placed successfully!");
    info!("  Order ID: {}", order.order_id);
    info!("  Symbol: {}", order.symbol);
    info!("  Side: {}", order.side);
    info!("  Quantity: {}", order.quantity);
    info!("  Type: {}", order.order_type);
    if let Some(ref price) = order.price {
        info!("  Price: {}", price);
    }
    info!("  Status: {}", order.status);
    info!("  Message: {}", order.message);
}

/// Demonstrates canceling an order
async fn demo_cancel_order(context: &ApplicationContext) {
    info!("\n=== Canceling Order ===");
    // In a real scenario, you would have a real order ID
    let cancel_params = CancelOrderParams {
        symbol: "BTCUSDT".to_string(),
        order_id: Some(123_456_789),
        orig_client_order_id: None,
    };
    match cancel_asterdex_order(context, cancel_params).await {
        Ok(result) => log_cancel_success(&result),
        Err(e) => info!("Failed to cancel order: {}", e),
    }
}

/// Helper function to log successful order cancellation
#[allow(clippy::cognitive_complexity)]
fn log_cancel_success(result: &AsterdexCancelResult) {
    info!("Order canceled successfully!");
    info!("  Order ID: {}", result.order_id);
    info!("  Symbol: {}", result.symbol);
    info!("  Status: {}", result.status);
    info!("  Message: {}", result.message);
}

/// Demonstrates querying a specific order
#[allow(clippy::cognitive_complexity)]
async fn demo_query_order(context: &ApplicationContext) {
    info!("\n=== Querying Order ===");
    // In a real scenario, you would have a real order ID
    match get_asterdex_order(context, "BTCUSDT".to_string(), 123_456_789).await {
        Ok(order) => {
            info!("Order found!");
            info!("  Order ID: {}", order.order_id);
            info!("  Symbol: {}", order.symbol);
            info!("  Status: {}", order.status);
            info!("  Side: {} {}", order.side, order.position_side);
            info!(
                "  Quantity: {} (Executed: {})",
                order.orig_qty, order.executed_qty
            );
            info!("  Price: {}", order.price);
        }
        Err(e) => info!("Failed to query order: {}", e),
    }
}
